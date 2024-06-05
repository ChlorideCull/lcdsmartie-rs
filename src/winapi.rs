use core::ptr;
use windows_sys::{core::PCSTR, Win32::{Foundation::{GetLastError, BOOL, FALSE, TRUE}, Globalization::{MultiByteToWideChar, WideCharToMultiByte, CP_ACP, WC_NO_BEST_FIT_CHARS}}};
use crate::Error as LcdSmartieError;

pub fn utf16_to_code_page(
    code_page: u32,
    wide_string: &[u16],
    out_multi_byte_string: &mut [u8],
) -> Result<usize, LcdSmartieError> {
    if wide_string.len() == 0 {
        // Native API bails on empty strings, but an empty string translates to an empty string :)
        return Ok(0);
    }
    let default_char_mapped: PCSTR = ptr::null();
    let mut default_char_used: BOOL = FALSE;
    let res = unsafe { WideCharToMultiByte(code_page, WC_NO_BEST_FIT_CHARS, wide_string.as_ptr(), wide_string.len().try_into().unwrap(), out_multi_byte_string.as_mut_ptr().cast(), out_multi_byte_string.len().try_into().unwrap(), default_char_mapped, &mut default_char_used) };
    let errorcode = unsafe { GetLastError() };
    if res == 0 && errorcode != 0 {
        return Err(LcdSmartieError::Win32Error(errorcode));
    }
    if default_char_used == TRUE {
        return Err(LcdSmartieError::AnsiConversionError);
    }
    return Ok(res.try_into().unwrap());
}

pub fn utf16_to_ansi(
    wide_string: &[u16],
    out_multi_byte_string: &mut [u8],
) -> Result<usize, LcdSmartieError> {
    utf16_to_code_page(CP_ACP, wide_string, out_multi_byte_string)
}

pub fn code_page_to_utf16(
    code_page: u32,
    multi_byte_string: &[u8],
    out_wide_string: &mut [u16]
) -> Result<usize, LcdSmartieError> {
    let strlen = multi_byte_string.len();
    if strlen == 0 {
        // Native API bails on empty strings, but an empty string translates to an empty string :)
        return Ok(0);
    }
    let res = unsafe { MultiByteToWideChar(code_page, 0, multi_byte_string.as_ptr().cast(), strlen.try_into().unwrap(), out_wide_string.as_mut_ptr(), out_wide_string.len().try_into().unwrap()) };
    let errorcode = unsafe { GetLastError() };
    if res == 0 && errorcode != 0 {
        return Err(LcdSmartieError::Win32Error(errorcode));
    }
    return Ok(res.try_into().unwrap());
}

pub fn ansi_to_utf16(
    multi_byte_string: &[u8],
    out_wide_string: &mut [u16]
) -> Result<usize, LcdSmartieError> {
    code_page_to_utf16(CP_ACP, multi_byte_string, out_wide_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_to_code_page_works_with_simple_string() {
        const TESTSTRING: &str = "test";
        const STRLEN: usize = TESTSTRING.len();
        let mut widedata: [u16; STRLEN+1] = [0; STRLEN+1];
        let mut widedata_used: usize = 0;
        for f in TESTSTRING.encode_utf16() {
            widedata[widedata_used] = f;
            widedata_used += 1;
        }
        let mut outdata: [u8; 256] = [0; 256];

        let res = utf16_to_code_page(1252, &widedata[..widedata_used], &mut outdata);
        assert_eq!(res.is_err(), false);
        assert_eq!(res.unwrap(), 4);
        assert!(outdata[..5] == [0x74, 0x65, 0x73, 0x74, 0x00]);
    }

    #[test]
    fn utf16_to_code_page_errors_on_lossy_conversions() {
        const TESTSTRING: &str = "فلسطين ستتحرر";
        let mut widedata: [u16; 256] = [0; 256];
        let mut widedata_used: usize = 0;
        for f in TESTSTRING.encode_utf16() {
            widedata[widedata_used] = f;
            widedata_used += 1;
        }
        let mut outdata: [u8; 256] = [0; 256];

        let res = utf16_to_code_page(1252, &widedata[..widedata_used], &mut outdata);
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap(), LcdSmartieError::AnsiConversionError);
    }

    #[test]
    fn utf16_to_code_page_works_on_empty_strings() {
        const TESTSTRING: &str = "";
        const STRLEN: usize = TESTSTRING.len();
        let mut widedata: [u16; STRLEN+1] = [0; STRLEN+1];
        let mut widedata_used: usize = 0;
        for f in TESTSTRING.encode_utf16() {
            widedata[widedata_used] = f;
            widedata_used += 1;
        }
        let mut outdata: [u8; 256] = [0; 256];

        let res = utf16_to_code_page(1252, &widedata[..widedata_used], &mut outdata);
        assert_eq!(res.is_err(), false);
        assert_eq!(res.unwrap(), 0);
        assert!(outdata[0] == 0x00);
    }
}
