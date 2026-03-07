# macOS GTK4 Build Fix

## Problem

The `relm` crate was failing to build on macOS with errors like:

```
The system library `gtk4` required by crate `gdk4-sys` was not found.
The system library `pango` required by crate `pango-sys` was not found.
The system library `cairo` required by crate `cairo-sys` was not found.
```

Even though GTK4 was installed via `brew install gtk4`.

## Root Cause

macOS systems may have multiple `pkg-config` installations:

1. **System pkg-config** at `/usr/local/bin/pkg-config` (older, possibly from MacPorts or manual install)
2. **Homebrew pkg-config** at `/opt/homebrew/bin/pkg-config` (Apple Silicon) or `/usr/local/bin/pkg-config` (Intel)

The issue occurs when:

- GTK4 libraries are installed via Homebrew at `/opt/homebrew/lib/pkgconfig/`
- But the system is using an older `pkg-config` that doesn't search Homebrew paths
- The older `pkg-config` can't find the `.pc` files for GTK4, Pango, and Cairo

## Diagnosis Steps

1. **Check which pkg-config is being used:**

   ```bash
   which pkg-config
   # If this shows /usr/local/bin/pkg-config but you're on Apple Silicon, that's the problem
   ```

2. **Verify Homebrew's pkg-config can find GTK4:**

   ```bash
   /opt/homebrew/bin/pkg-config --modversion gtk4 pango cairo
   # Should show version numbers like: 4.22.0, 1.57.0, 1.18.4
   ```

3. **Check where the .pc files are located:**

   ```bash
   find /opt/homebrew -name "gtk4.pc"
   # Should find: /opt/homebrew/lib/pkgconfig/gtk4.pc
   ```

## Solution

### Option 1: Use the Build Script (Recommended)

The provided `build.sh` script automatically uses the correct pkg-config:

```bash
cd relm
./build.sh
```

### Option 2: Set Environment Variable

Export the `PKG_CONFIG` environment variable before building:

```bash
export PKG_CONFIG=/opt/homebrew/bin/pkg-config
cd relm
cargo build
```

### Option 3: Create Cargo Config (Persistent)

Create or update `.cargo/config.toml` in the relm directory:

```toml
[env]
PKG_CONFIG = "/opt/homebrew/bin/pkg-config"
```

Then build normally:

```bash
cd relm
cargo build
```

### Option 4: Fix System PATH (Global)

Add Homebrew's bin directory to the front of your PATH in `~/.zshrc` or `~/.bash_profile`:

```bash
export PATH="/opt/homebrew/bin:$PATH"
```

Then restart your shell and build:

```bash
cd relm
cargo build
```

## Verification

After applying the fix, the build should proceed past the GTK4 system library checks and compile the Rust dependencies successfully.

You can verify the fix is working by checking the build output - you should see:

```
   Compiling pango-sys v0.20.10
   Compiling cairo-sys-rs v0.20.10
   Compiling gdk4-sys v0.9.6
   ...
```

Without errors about missing system libraries.

## Additional Notes

### Apple Silicon vs Intel

- **Apple Silicon (M1/M2/M3)**: Homebrew installs to `/opt/homebrew`
- **Intel Macs**: Homebrew installs to `/usr/local`

Adjust paths accordingly if you're on an Intel Mac.

### Why This Happens

This is a common issue on macOS when:

1. You've upgraded from an Intel Mac to Apple Silicon
2. You have legacy tools installed in `/usr/local`
3. You've installed tools from multiple package managers (Homebrew, MacPorts, etc.)

The Rust build system uses `pkg-config` to find system libraries, and if it picks up the wrong one, it can't locate Homebrew-installed libraries.

## Related Issues

This fix resolves the GTK4 system library detection issue. However, there may be additional Rust code issues that need to be addressed separately (such as incorrect module imports or missing trait implementations).
