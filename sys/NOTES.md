
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
