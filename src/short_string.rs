use core::ffi::{c_char, CStr};

use alloc::{format, string::{String, ToString}};

use crate::winapi::{self, CP_ACP, WC_NO_BEST_FIT_CHARS};


/// C-style string that enforces the requirements of a Delphi "short string" - namely a limit of 255 characters,
/// and characters being in the current Windows ANSI codepage.
/// 
/// # Examples
/// ```
/// let x: lcdsmartie_rs::ShortString = "In most of the world, I can convert!".try_into()
///     .expect("The user should have Windows-1252 as the default codepage.");
/// ```
/// ```
/// let x: lcdsmartie_rs::ShortString = "これは日本でしか通用しない。".try_into()
///     .expect("The user should live in Japan.");
/// ```
#[repr(transparent)]
pub struct ShortString([u8; 256]);

impl ShortString {
    /// Get a reference to the internal `u8` array.
    pub fn as_arr(&self) -> &[u8] {
        &self.0
    }
}

impl Default for ShortString {
    fn default() -> Self {
        Self([0; 256])
    }
}

impl From<&CStr> for ShortString {
    fn from(value: &CStr) -> Self {
        let mut outdata: [u8; 256] = [0; 256];
        let cdata = value.to_bytes();
        outdata[..cdata.len()].copy_from_slice(cdata);

        return ShortString { 0: outdata };
    }
}

impl TryFrom<&str> for ShortString {
    type Error = String;
    
    fn try_from(data: &str) -> Result<Self, Self::Error> {
        if data.len() >= 255 {
            return Err("String too long".to_string());
        }
        let mut widedata: [u16; 256] = [0; 256];
        let mut widedata_used: usize = 0;
        for f in data.encode_utf16() {
            widedata[widedata_used] = f;
            widedata_used += 1;
        }
        let mut outdata: [u8; 256] = [0; 256];
        let mut converted_lossy: bool = false;
        let res = winapi::WideCharToMultiByte_Safe(CP_ACP, WC_NO_BEST_FIT_CHARS, &widedata[..widedata_used], &mut outdata, None, Some(&mut converted_lossy));
        if let Err(e) = res {
            return Err(format!("Failed to convert to code page - GetLastError {}", e));
        }
        if converted_lossy {
            return Err("String contained characters that cannot be represented in the system default code page".to_string());
        }
        return Ok(Self { 0: outdata });
    }
    
}

impl From<ShortString> for String {
    fn from(value: ShortString) -> Self {
        let mut widedata: [u16; 256] = [0; 256];
        let res = winapi::MultiByteToWideChar_Safe(CP_ACP, 0, &value.0, &mut widedata);
        return String::from_utf16_lossy(&widedata[..res.unwrap()]);
    }
}

impl From<ShortString> for *const c_char {
    fn from(value: ShortString) -> Self {
        value.0.as_ptr().cast()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! roundtrip_test {
        ($str:literal) => {
            let test: ShortString = $str.try_into().unwrap();
            let tests: String = test.into();
            assert_eq!($str, tests);
        };
    }

    #[test]
    fn creating_short_string_from_ascii_string_works() {
        let test: ShortString = "furry".try_into().unwrap();
        assert_eq!(test.0[..6], [0x66, 0x75, 0x72, 0x72, 0x79, 0x00]);
    }

    #[test]
    fn creating_short_string_from_empty_string_works() {
        let test: ShortString = "".try_into().unwrap();
        assert_eq!(test.0[..3], [0x00, 0x00, 0x00]);
    }

    #[test]
    fn creating_short_string_from_number_string_works() {
        let test: ShortString = "0".try_into().unwrap();
        assert_eq!(test.0[..2], [0x30, 0x00]);
    }

    #[test]
    fn short_string_roundtrips() {
        roundtrip_test!("furry");
        roundtrip_test!("");
        roundtrip_test!("0");
    }
    
}