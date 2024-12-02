# Usecases

## download all new files
Application subscribes to all file creation events and downloads each file to local storage.

# Client interface options
Requirements:
* Active open-source community with stable code base.
* Portable code with support for all major PC platforms (Mac, Linux, Windows).
* Portable and embeddalbe code with support for microcontroller architectures (Pico & Arduino based).

Very few frameworks meet both of the last two requirements. Current focus is on proving the viability of any GUI devrelopment with `libindigo-rs` so selecting a framework for the firs iteration based only on the support for PC platforms.

Tried...

* `slint` promising for embedded development, initial issues related to the dynamic nature of INDIGO, to be revisisted.
* `gtk-rs` complex and too much cumbersome boiler plate, high learning curve.
* `relm` drastically simplifies GTK development on Rust, less boilerplte.

Settled on `relm` for the first tries of implementing a `libindigo-rs` GUI.

[The state of Rust GUI libraries](https://blog.logrocket.com/state-rust-gui-libraries/#comparing-rust-gui-libraries)

| Popularity | (by GitHub stars) | Project users | Compatibility | Production ready |
| ---------- | ----------------- | ------------- | ------------- | ---------------- |
| gtk-rs | 1.5k | 1.4k | Windows, Linux, macOS | No |
| fltk-rs | 1.4k | 1k | Windows, Linux, macOS | **Yes** |
| iced | 21.6k | 2.9k | Windows, macOS, Linux, and Web | No |
| relm | 2.4k | not-in-view | Windows, Linux, macOS | No |
| Azul | 5.7k | not-in-view | Windows, Linux, macOS | **Yes** (alpha release) |
| egui | 18k | 11.3k | Windows, macOS, Linux, and Web | No |
| Tauri | 72.8k | 554 | macOS, Windows, Linux, Android and iOS | **Yes** |
| Slint | 12k | 544 | macOS, Windows, Linux, Android and iOS, Web | **Yes** |
| Druid | 9.2k | 2.1k | macOS, Windows, Linux/BSD and Web | No |
| Yew | 29.3k | 10.7k | Web/WebAssembly | No |
| Xilem | 2.3k|–|Linux | No releases yet |

## One trait with all callback methods

## One trait per callback method