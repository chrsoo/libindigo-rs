# GTK RS
1. Install GTK4
```
brew install gtk4 libffi zlib expat proto

```
Note: the `gtk4-rs` instructions only call for installing gtk4 on the system using `brew`, but this did not work for me and the other dependencies had to be installed to make `gtk4-rs` comppile without failure. For more details on the isue, please refer to [The system library `gobject-2.0` required by crate `glib-sys` was not found](https://stackoverflow.com/questions/79044934/the-system-library-gobject-2-0-required-by-crate-glib-sys-was-not-found/79047863#79047863) on Stack Overflow.

2. Find the current GTK4 version
```
$ ls /opt/homebrew/Cellar/gtk4/
4.16.2
```
3. Add the GTK4 dependency
```
cargo add gtk4 --rename gtk --features v4_16
```
4. Run the application
```
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:/opt/homebrew/opt/pkg-config:$PKG_CONFIG_PATH"

#export PATH="/opt/homebrew/bin:$PATH"
PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig" cargo run
```


# Which GUI framework?
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