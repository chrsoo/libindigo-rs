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