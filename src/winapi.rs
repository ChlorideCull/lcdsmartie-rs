#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
// Here be ugly code

use core::{ffi::c_char, ptr};

pub const FALSE: i32 = 0;
pub const TRUE: i32 = 1;
pub const MB_PRECOMPOSED: u32 = 1;
pub const MB_COMPOSITE: u32 = 2;
pub const MB_USEGLYPHCHARS: u32 = 4;
pub const MB_ERR_INVALID_CHARS: u32 = 8;
pub const CP_INSTALLED: u32 = 1;
pub const CP_SUPPORTED: u32 = 2;
pub const CP_ACP: u32 = 0;
pub const CP_OEMCP: u32 = 1;
pub const CP_MACCP: u32 = 2;
pub const CP_THREAD_ACP: u32 = 3;
pub const CP_SYMBOL: u32 = 42;
pub const CP_UTF7: u32 = 65000;
pub const CP_UTF8: u32 = 65001;
pub const WC_COMPOSITECHECK: u32 = 512;
pub const WC_DISCARDNS: u32 = 16;
pub const WC_SEPCHARS: u32 = 32;
pub const WC_DEFAULTCHAR: u32 = 64;
pub const WC_ERR_INVALID_CHARS: u32 = 128;
pub const WC_NO_BEST_FIT_CHARS: u32 = 1024;
pub type DWORD = ::core::ffi::c_ulong;
pub type BOOL = ::core::ffi::c_int;
pub type LPBOOL = *mut BOOL;
pub type UINT = ::core::ffi::c_uint;
pub type wchar_t = ::core::ffi::c_ushort;
pub type CHAR = ::core::ffi::c_char;
pub type WCHAR = wchar_t;
pub type LPCWCH = *const WCHAR;
pub type LPWSTR = *mut WCHAR;
pub type LPCCH = *const CHAR;
pub type LPSTR = *mut CHAR;
#[link(name = "kernel32")]
extern "C" {
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCCH,
        cbMultiByte: ::core::ffi::c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
}
#[link(name = "kernel32")]
extern "C" {
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWCH,
        cchWideChar: ::core::ffi::c_int,
        lpMultiByteStr: LPSTR,
        cbMultiByte: ::core::ffi::c_int,
        lpDefaultChar: *const CHAR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> ::core::ffi::c_int;
}
#[link(name = "kernel32")]
extern "C" {
    pub fn GetLastError() -> DWORD;
}

pub fn WideCharToMultiByte_Safe(
    code_page: UINT,
    flags: DWORD,
    wide_string: &[u16],
    out_multi_byte_string: &mut [u8],
    default_char: Option<&c_char>,
    out_default_char_used: Option<&mut bool>,
) -> Result<usize, DWORD> {
    if wide_string.len() == 0 {
        // Native API bails on empty strings, but an empty string translates to an empty string :)
        return Ok(0);
    }
    let default_char_mapped: *const CHAR = match default_char {
        Some(c) => c,
        None => ptr::null()
    };
    let mut default_char_used: BOOL = FALSE;
    let res = unsafe { WideCharToMultiByte(code_page, flags, wide_string.as_ptr(), wide_string.len().try_into().unwrap(), out_multi_byte_string.as_mut_ptr().cast(), out_multi_byte_string.len().try_into().unwrap(), default_char_mapped, &mut default_char_used) };
    let errorcode = unsafe { GetLastError() };
    if res == 0 && errorcode != 0 {
        return Err(errorcode);
    }
    if let Some(c) = out_default_char_used {
        *c = default_char_used == TRUE;
    }
    return Ok(res.try_into().unwrap());
}

pub fn MultiByteToWideChar_Safe(
    code_page: UINT,
    flags: DWORD,
    multi_byte_string: &[u8],
    out_wide_string: &mut [u16]
) -> Result<usize, DWORD> {
    let strlen = multi_byte_string.iter().position(|&x| x == 0x00);
    if strlen.is_none() {
        return Err(0x57); // ERROR_INVALID_PARAMETER - input string does not have a null terminator
    }
    let strlen = strlen.unwrap();
    if strlen == 0 {
        // Native API bails on empty strings, but an empty string translates to an empty string :)
        return Ok(0);
    }
    let res = unsafe { MultiByteToWideChar(code_page, flags, multi_byte_string.as_ptr().cast(), strlen.try_into().unwrap(), out_wide_string.as_mut_ptr(), out_wide_string.len().try_into().unwrap()) };
    let errorcode = unsafe { GetLastError() };
    if res == 0 && errorcode != 0 {
        return Err(errorcode);
    }
    return Ok(res.try_into().unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn WideCharToMultiByte_Safe_Basic() {
        const TESTSTRING: &str = "test";
        const STRLEN: usize = TESTSTRING.len();
        let mut widedata: [u16; STRLEN+1] = [0; STRLEN+1];
        let mut widedata_used: usize = 0;
        for f in TESTSTRING.encode_utf16() {
            widedata[widedata_used] = f;
            widedata_used += 1;
        }
        let mut outdata: [u8; 256] = [0; 256];
        let mut converted_lossy: bool = false;

        let res = WideCharToMultiByte_Safe(1252, WC_NO_BEST_FIT_CHARS, &widedata[..widedata_used], &mut outdata, None, Some(&mut converted_lossy));
        assert_eq!(converted_lossy, false);
        assert_eq!(res.is_err(), false);
        assert_eq!(res.unwrap(), 4);
        assert!(outdata[..5] == [0x74, 0x65, 0x73, 0x74, 0x00]);
    }

    #[test]
    fn WideCharToMultiByte_Safe_ProperlyFlagsLossyConversions() {
        const TESTSTRING: &str = "فلسطين ستتحرر";
        let mut widedata: [u16; 256] = [0; 256];
        let mut widedata_used: usize = 0;
        for f in TESTSTRING.encode_utf16() {
            widedata[widedata_used] = f;
            widedata_used += 1;
        }
        let mut outdata: [u8; 256] = [0; 256];
        let mut converted_lossy: bool = false;

        let res = WideCharToMultiByte_Safe(1252, WC_NO_BEST_FIT_CHARS, &widedata[..widedata_used], &mut outdata, None, Some(&mut converted_lossy));
        assert_eq!(converted_lossy, true);
        assert_eq!(res.is_err(), false);
    }

    #[test]
    fn WideCharToMultiByte_Safe_Empty() {
        const TESTSTRING: &str = "";
        const STRLEN: usize = TESTSTRING.len();
        let mut widedata: [u16; STRLEN+1] = [0; STRLEN+1];
        let mut widedata_used: usize = 0;
        for f in TESTSTRING.encode_utf16() {
            widedata[widedata_used] = f;
            widedata_used += 1;
        }
        let mut outdata: [u8; 256] = [0; 256];
        let mut converted_lossy: bool = false;

        let res = WideCharToMultiByte_Safe(1252, WC_NO_BEST_FIT_CHARS, &widedata[..widedata_used], &mut outdata, None, Some(&mut converted_lossy));
        assert_eq!(converted_lossy, false);
        assert_eq!(res.is_err(), false);
        assert_eq!(res.unwrap(), 0);
        assert!(outdata[0] == 0x00);
    }
}
