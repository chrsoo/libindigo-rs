
# bindgen

* [Rust bindgen manual](https://rust-lang.github.io/rust-bindgen/introduction.html)

## prerequisits
libclang must be installed, cf. [bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)

For Mac using Homebrew:
```bash
$ brew install llvm
...
To use the bundled libc++ please add the following LDFLAGS:
  LDFLAGS="-L/opt/homebrew/opt/llvm/lib/c++ -Wl,-rpath,/opt/homebrew/opt/llvm/lib/c++"

llvm is keg-only, which means it was not symlinked into /opt/homebrew,
because macOS already provides this software and installing another version in
parallel can cause all kinds of trouble.

If you need to have llvm first in your PATH, run:
  echo 'export PATH="/opt/homebrew/opt/llvm/bin:$PATH"' >> ~/.zshrc

For compilers to find llvm you may need to set:
  export LDFLAGS="-L/opt/homebrew/opt/llvm/lib"
  export CPPFLAGS="-I/opt/homebrew/opt/llvm/include"
```

## VSCode support for bindgen
Include generated bindings in `lib.rs`:
```rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```
To make rust-analyzer happy [the following line](https://github.com/rust-lang/rust-analyzer/issues/5992) needs to be added to VSCode's settings.json:
```
"rust-analyzer.cargo.loadOutDirsFromCheck": true,
```

## compiling on linux

Build on a clean Ubuntu distro running in Docker (optional):
```bash
# TODO save image for reuse
docker run --rm -it --name github-runner ubuntu:22.04
```

Install [INDIGO prerequisits](https://www.indigo-astronomy.org/for-developers.html):
```bash
apt-get update
apt-get install build-essential autoconf autotools-dev libtool cmake libudev-dev libavahi-compat-libdnssd-dev libusb-1.0-0-dev libcurl4-gnutls-dev libgphoto2-dev libz-dev git curl patchelf
```

[Install rust](https://www.rust-lang.org/tools/install):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env" # todo add command for permanently adding to path
```

build [first time] (pthread issue)(https://github.com/indigo-astronomy/indigo/blob/master/README.md#no-pthread_yield):
``` bash
cargo build # building `sys` should fail with a QHY related error
cd sys/externals/indigo/indigo_drivers/ccd_qhy/bin_externals/pthread_yield_compat
make patchlib
cd ~/src/libindigo-rs # change back to the libindigo source dir
cargo build
```
