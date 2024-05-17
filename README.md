# lcdsmartie-rs

lcdsmartie-rs is a small framework to enable writing plugins for [LCDSmartie](https://github.com/LCD-Smartie/LCDSmartie) in Rust.
It is no_std to enable usage on older versions of Windows (down to Windows 2000, theoretically) but requires an alloc implementation.
 
This also handles various undocumented quirks, including:
* Strings in the API being limited to 255 bytes in the default Windows ANSI code page
* Undefined lifespans on memory (handled mostly by just reusing a static buffer)
 
## Minimum Supported Rust Version (MSRV)
Before 1.0, there will be no formal MSRV policy - but it was developed with rustc 1.77, and 1.64 is the lowest version able to compile, according to cargo-msrv (C FFI and CStr in core was stabilized then, which is hardly used - it can be refactored out if there is a need to support older versions)