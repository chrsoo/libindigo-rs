//! Server Manager for INDIGO server process lifecycle
//!
//! This module manages the INDIGO server process, including discovery,
//! startup, health monitoring, and graceful shutdown.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Server state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    NotStarted,
    Starting,
    Running,
    Failed,
    ShuttingDown,
    Stopped,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub binary_path: PathBuf,
    pub port: u16,
    pub drivers: Vec<String>,
    pub startup_timeout: Duration,
    pub shutdown_timeout: Duration,
}

/// Manages the INDIGO server process
pub struct ServerManager {
    process: Option<Child>,
    config: ServerConfig,
    state: ServerState,
    output_lines: Arc<Mutex<Vec<String>>>,
}

impl ServerManager {
    /// Create a new server manager with configuration
    pub fn new(config: ServerConfig) -> Self {
        Self {
            process: None,
            config,
            state: ServerState::NotStarted,
            output_lines: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Discover the INDIGO server binary
    ///
    /// Priority order:
    /// 1. Environment variable INDIGO_SERVER_PATH
    /// 2. Built from source in submodule (sys/externals/indigo/build/bin/)
    /// 3. System PATH (which, where commands)
    /// 4. System installation paths (macOS/Linux)
    pub fn discover_binary() -> Result<PathBuf, String> {
        let mut search_log = Vec::new();

        // 1. Check environment variable
        if let Ok(path) = std::env::var("INDIGO_SERVER_PATH") {
            let path_buf = PathBuf::from(&path);
            search_log.push(format!("INDIGO_SERVER_PATH: {}", path));
            if path_buf.exists() {
                eprintln!("[INDIGO] Found server via INDIGO_SERVER_PATH: {}", path);
                return Ok(path_buf);
            }
        }

        // 2. Check built from source in submodule (PRIORITY)
        let submodule_paths = vec![
            "sys/externals/indigo/build/bin/indigo_server",
            "sys/externals/indigo/indigo_server",
        ];

        for path_str in &submodule_paths {
            let path = PathBuf::from(path_str);
            search_log.push(format!("Submodule: {}", path_str));
            if path.exists() {
                eprintln!("[INDIGO] Found server in build directory: {}", path_str);
                return Ok(path);
            }
        }

        // 3. Check system PATH using 'which' (Unix) or 'where' (Windows)
        #[cfg(unix)]
        {
            if let Ok(output) = std::process::Command::new("which")
                .arg("indigo_server")
                .output()
            {
                if output.status.success() {
                    if let Ok(path_str) = String::from_utf8(output.stdout) {
                        let path_str = path_str.trim();
                        if !path_str.is_empty() {
                            let path = PathBuf::from(path_str);
                            if path.exists() {
                                eprintln!("[INDIGO] Found server in PATH: {}", path_str);
                                return Ok(path);
                            }
                        }
                    }
                }
            }
            search_log.push("System PATH (which indigo_server)".to_string());
        }

        #[cfg(windows)]
        {
            if let Ok(output) = std::process::Command::new("where")
                .arg("indigo_server")
                .output()
            {
                if output.status.success() {
                    if let Ok(path_str) = String::from_utf8(output.stdout) {
                        let path_str = path_str.trim();
                        if !path_str.is_empty() {
                            let path = PathBuf::from(path_str);
                            if path.exists() {
                                eprintln!("[INDIGO] Found server in PATH: {}", path_str);
                                return Ok(path);
                            }
                        }
                    }
                }
            }
            search_log.push("System PATH (where indigo_server)".to_string());
        }

        // 4. Check system installation paths (macOS/Linux)
        let system_paths = vec![
            "/usr/local/bin/indigo_server",
            "/usr/bin/indigo_server",
            "/opt/indigo/bin/indigo_server",
        ];

        for path_str in &system_paths {
            let path = PathBuf::from(path_str);
            search_log.push(format!("System: {}", path_str));
            if path.exists() {
                eprintln!("[INDIGO] Found server in system path: {}", path_str);
                return Ok(path);
            }
        }

        // Build detailed error message
        let error_msg = format!(
            "INDIGO server binary not found. Searched:\n  {}\n\n\
             To fix this:\n\
             1. Build INDIGO server: cd sys/externals/indigo && make\n\
             2. Install system-wide: brew install indigo (macOS) or build from source\n\
             3. Set INDIGO_SERVER_PATH environment variable to the binary location",
            search_log.join("\n  ")
        );

        Err(error_msg)
    }

    /// Start the INDIGO server process
    pub fn start(&mut self) -> Result<(), String> {
        if self.state == ServerState::Running {
            return Ok(());
        }

        self.state = ServerState::Starting;

        // Build command
        let mut cmd = Command::new(&self.config.binary_path);

        // Add port argument
        cmd.arg("-p").arg(self.config.port.to_string());

        // Add do-not-fork flag for easier process management
        cmd.arg("-n");

        // Add drivers
        for driver in &self.config.drivers {
            cmd.arg(driver);
        }

        // Redirect output for debugging
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn process
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn server process: {}", e))?;

        // Capture stdout in background thread
        if let Some(stdout) = child.stdout.take() {
            let output_lines = Arc::clone(&self.output_lines);
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let mut lines = output_lines.lock().unwrap();
                        lines.push(line.clone());
                        // Keep only last 1000 lines
                        if lines.len() > 1000 {
                            lines.remove(0);
                        }
                        eprintln!("[INDIGO] {}", line);
                    }
                }
            });
        }

        // Capture stderr in background thread
        if let Some(stderr) = child.stderr.take() {
            let output_lines = Arc::clone(&self.output_lines);
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let mut lines = output_lines.lock().unwrap();
                        lines.push(format!("ERR: {}", line));
                        if lines.len() > 1000 {
                            lines.remove(0);
                        }
                        eprintln!("[INDIGO ERR] {}", line);
                    }
                }
            });
        }

        self.process = Some(child);
        self.state = ServerState::Running;

        Ok(())
    }

    /// Check if server process is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    // Process has exited
                    self.state = ServerState::Stopped;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking status
                    false
                }
            }
        } else {
            false
        }
    }

    /// Get server address
    pub fn address(&self) -> String {
        format!("localhost:{}", self.config.port)
    }

    /// Get server state
    pub fn state(&self) -> ServerState {
        self.state
    }

    /// Get captured server output
    pub fn get_output(&self) -> Vec<String> {
        self.output_lines.lock().unwrap().clone()
    }

    /// Get last N lines of server output
    pub fn tail_output(&self, lines: usize) -> Vec<String> {
        let output = self.output_lines.lock().unwrap();
        let start = if output.len() > lines {
            output.len() - lines
        } else {
            0
        };
        output[start..].to_vec()
    }

    /// Stop the server gracefully
    pub fn stop(&mut self) -> Result<(), String> {
        if self.state == ServerState::Stopped || self.state == ServerState::NotStarted {
            return Ok(());
        }

        self.state = ServerState::ShuttingDown;

        if let Some(mut child) = self.process.take() {
            // Try graceful shutdown first (SIGTERM on Unix)
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let pid = child.id();
                let _ = Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .status();
            }

            // Wait for process to exit with timeout
            let start = Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_status)) => {
                        self.state = ServerState::Stopped;
                        return Ok(());
                    }
                    Ok(None) => {
                        if start.elapsed() > self.config.shutdown_timeout {
                            // Timeout - force kill
                            let _ = child.kill();
                            let _ = child.wait();
                            self.state = ServerState::Stopped;
                            return Err("Server shutdown timeout - forced kill".to_string());
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        return Err(format!("Error waiting for server shutdown: {}", e));
                    }
                }
            }
        }

        self.state = ServerState::Stopped;
        Ok(())
    }

    /// Force kill the server (fallback)
    pub fn kill(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            child
                .kill()
                .map_err(|e| format!("Failed to kill server process: {}", e))?;
            child
                .wait()
                .map_err(|e| format!("Failed to wait for killed process: {}", e))?;
        }

        self.state = ServerState::Stopped;
        Ok(())
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        // Ensure server is stopped when manager is dropped
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_state() {
        let config = ServerConfig {
            binary_path: PathBuf::from("/usr/local/bin/indigo_server"),
            port: 7624,
            drivers: vec!["indigo_ccd_simulator".to_string()],
            startup_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(5),
        };

        let manager = ServerManager::new(config);
        assert_eq!(manager.state(), ServerState::NotStarted);
    }

    #[test]
    fn test_server_address() {
        let config = ServerConfig {
            binary_path: PathBuf::from("/usr/local/bin/indigo_server"),
            port: 7625,
            drivers: vec![],
            startup_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(5),
        };

        let manager = ServerManager::new(config);
        assert_eq!(manager.address(), "localhost:7625");
    }
}
