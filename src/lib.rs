//! # lcdsmartie-rs
//! 
//! lcdsmartie-rs is a small framework to enable writing plugins for [LCDSmartie](https://github.com/LCD-Smartie/LCDSmartie) in Rust.
//! It is no_std to enable usage on older versions of Windows (down to Windows 2000, theoretically) but requires an alloc implementation.
//! 
//! This also handles various undocumented quirks, including:
//! * Strings in the API being limited to 255 bytes in the default Windows ANSI code page
//! * Undefined lifespans on memory (handled mostly by just reusing a static buffer)
//! 
//! ## Minimum Supported Rust Version (MSRV)
//! Before 1.0, there will be no formal MSRV policy - but it was developed with rustc 1.77, and
//! 1.64 is the lowest version able to compile, according to cargo-msrv (C FFI and CStr in core was stabilized then, which is hardly used - it can be refactored out if there is a need to support older versions)
#![warn(missing_docs)]
#![no_std]
extern crate alloc;

mod winapi;
mod short_string;

use alloc::string::String;
pub use short_string::ShortString;

#[cfg(feature = "std")]
extern crate std;

/// Trait representing the implementation of an LCDSmartie plugin.
/// 
/// # Important
/// LCDSmartie is single threaded, so none of the functions it calls should take long to run,
/// or the UI will freeze for the user.
/// 
/// In addition, please remember that **any panic will cause LCDSmartie to crash.**
pub trait Plugin {
    /// Called by LCDSmartie to initialize the plugin.
    fn new() -> Self;
    
    /// Developer, as shown in the Setup window in LCDSmartie.
    fn developer(&self) -> &'static str;

    /// Version, as shown in the Setup window in LCDSmartie.
    fn version(&self) -> &'static str;
    
    /// The documentation shown in the Setup window as samples. Lines should be separated by CRLF.
    /// Lines can be clicked in the UI to insert them in the current screen.
    fn documentation(&self) -> ShortString;

    /// The plugin defined "minimum refresh interval".
    /// LCDSmartie will call your plugin every `max(user_preferences, minimum_refresh_interval_ms)` milliseconds.
    fn minimum_refresh_interval_ms(&self) -> i32;
    
    /// Function that is called to retrieve data for LCDSmartie through `$dll(<DLL Name>,<fid>,<param1>,<param2>)`.
    /// Errors will be formatted appropriately.
    ///
    /// # Important
    /// LCDSmartie is single threaded, so none of the functions it calls should take long to run,
    /// or the UI will freeze for the user.
    fn function_router(&self, fid: u8, param1: &str, param2: &str) -> Result<ShortString, String>;
}

/// Macro to define a plugin. Takes one parameter - a type that implements the [Plugin] trait.
/// 
/// This macro defines extern functions that LCDSmartie expects, and sets them up to manage the
/// lifecycle of an object with the provided type. It also defines the maximum possible 20 functions,
/// and routes them to the function router.
#[macro_export(local_inner_macros)]
macro_rules! define_plugin {
    ($plug:ty) => {
        use $crate::Plugin;

        static mut PLUGIN: Option<core::cell::RefCell<$plug>> = None;
        static mut OUTPUT_DATA: [u8; 256] = [0; 256];
    
        #[no_mangle]
        pub unsafe extern "stdcall" fn SmartieInit() {
            PLUGIN = Some(core::cell::RefCell::new(<$plug>::new()));
        }
        
        #[no_mangle]
        pub unsafe extern "stdcall" fn SmartieFini() {
            PLUGIN = None;
        }
        
        #[no_mangle]
        pub unsafe extern "stdcall" fn SmartieInfo() -> *const c_char {
            let plug = PLUGIN.as_ref().unwrap().borrow();
            let info = std::format!("Developer: {}\r\nVersion: {}", plug.developer(), plug.version());
            let retval: $crate::ShortString = info.as_str().try_into().unwrap_or("[Err: Failed to convert info line]".try_into().unwrap());
            let retvala = retval.as_arr();
            OUTPUT_DATA[..retvala.len()].copy_from_slice(retvala);
            return OUTPUT_DATA.as_ptr().cast();
        }
        
        #[no_mangle]
        pub unsafe extern "stdcall" fn SmartieDemo() -> *const c_char {
            let retval = PLUGIN.as_ref().unwrap().borrow().documentation();
            let retvala = retval.as_arr();
            OUTPUT_DATA[..retvala.len()].copy_from_slice(retvala);
            return OUTPUT_DATA.as_ptr().cast();
        }
        
        #[no_mangle]
        pub unsafe extern "stdcall" fn GetMinRefreshInterval() -> i32 {
            return PLUGIN.as_ref().unwrap().borrow().minimum_refresh_interval_ms();
        }
        
        function_n!(function1, 1);
        function_n!(function2, 2);
        function_n!(function3, 3);
        function_n!(function4, 4);
        function_n!(function5, 5);
        function_n!(function6, 6);
        function_n!(function7, 7);
        function_n!(function8, 8);
        function_n!(function9, 9);
        function_n!(function10, 10);
        function_n!(function11, 11);
        function_n!(function12, 12);
        function_n!(function13, 13);
        function_n!(function14, 14);
        function_n!(function15, 15);
        function_n!(function16, 16);
        function_n!(function17, 17);
        function_n!(function18, 18);
        function_n!(function19, 19);
        function_n!(function20, 20);
    }
}

/// Internal helper macro to generate function#N exports.
#[doc(hidden)]
#[macro_export]
macro_rules! function_n {
    ($name:ident, $n:literal) => {
        #[no_mangle]
        pub unsafe extern "stdcall" fn $name(param1: *const c_char, param2: *const c_char) -> *const c_char {
            let param1c: $crate::ShortString = core::ffi::CStr::from_ptr(param1).into();
            let param2c: $crate::ShortString = core::ffi::CStr::from_ptr(param2).into();
            let param1s: String = param1c.into();
            let param2s: String = param2c.into();
            let retval = PLUGIN.as_ref().unwrap().borrow().function_router($n, &param1s, &param2s);
            let unwrapped_retval: $crate::ShortString;
            if let Err(e) = retval {
                let error = format!("[Err: {}]", e);
                unwrapped_retval = error.as_str().try_into().unwrap_or("[Err: Failed to display error]".try_into().unwrap());
            } else {
                unwrapped_retval = retval.ok().unwrap();
            }
            let retvala = unwrapped_retval.as_arr();
            OUTPUT_DATA[..retvala.len()].copy_from_slice(retvala);
            return OUTPUT_DATA.as_ptr().cast();
        }
    }
}
