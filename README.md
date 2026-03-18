
EXPORT PROJECT XML
------------------
This is a post-processor for VeSys(Capital Essentials) XML files, which you get by exporting a whole project into XML.

![Project XML](screenshot2.JPG){width=250}

EXPORT LIBRARY XML
-------------------
The program also needs a `Library.xml` file in the same directory as `*.exe` or same directory as XML of the design. You can export Library XML from Component Manager.


![Library XML](screenshot3.JPG){width=250}

BUILD
-----

This application is written in Rust.

### Installing Rust on Windows

1. Download [rustup-init.exe](https://win.rustup.rs/x86_64) and run it.
2. Press Enter to accept defaults, wait for it to finish.
3. Restart your terminal (or Cursor), then run `rustc --version` to verify.

If `cargo build` fails with linker errors, install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload.

---

```
cargo build
cargo run
```


OPEN PROJECT
------------

Go to File -> Open to load project XML, then you may right click on various harness items and export them into available file formats.

![Library XML](screenshot.JPG){width=250}

