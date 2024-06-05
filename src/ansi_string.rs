use core::{ffi::CStr, fmt::Debug};
use alloc::{ffi::{CString, NulError}, string::String, vec::Vec};
use alloc::vec;

use crate::winapi;


/// C-style string encoded in the current Windows ANSI code page.
/// 
/// # Examples
/// ```
/// let x: lcdsmartie_rs::AnsiString = "In most of the world, I can convert!".try_into()
///     .expect("The user should have Windows-1252 as the default codepage.");
/// ```
/// ```
/// let x: lcdsmartie_rs::AnsiString = "これは日本でしか通用しない。".try_into()
///     .expect("The user should live in Japan.");
/// ```

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct AnsiString(Vec<u8>);

impl AnsiString {
    /// Get a reference to the internal `u8` vector.
    pub fn as_vec(&self) -> &Vec<u8> {
        &self.0
    }

    /// Get a slice, optionally constrained to `max_size`.
    pub fn as_slice(&self, max_size: Option<usize>) -> &[u8] {
        &self.0[0 .. max_size.unwrap_or(usize::MAX).min(self.0.len())]
    }
}

impl Debug for AnsiString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let strlen = self.0.len();
        let pretty: String = self.into();
        f.write_fmt(format_args!("AnsiString[{}; '{}']", strlen, pretty.as_str()))
    }
}

impl Default for AnsiString {
    fn default() -> Self {
        Self(alloc::vec!())
    }
}

impl From<&CStr> for AnsiString {
    fn from(value: &CStr) -> Self {
        return AnsiString { 0: value.to_bytes().to_vec() };
    }
}

impl TryFrom<String> for AnsiString {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<&str> for AnsiString {
    type Error = crate::Error;
    
    fn try_from(data: &str) -> Result<Self, Self::Error> {
        let widedata = data.encode_utf16().collect::<Vec<u16>>();
        let mut outdata: Vec<u8> = vec![0; widedata.len()+1];
        let res = winapi::utf16_to_ansi(widedata.as_slice(), outdata.as_mut_slice());
        if let Err(e) = res {
            return Err(e);
        }
        outdata.truncate(res.unwrap());
        return Ok(Self { 0: outdata });
    }
}

impl From<&AnsiString> for String {
    fn from(value: &AnsiString) -> Self {
        let mut widedata: Vec<u16> = vec![0; value.0.len()];
        let res = winapi::ansi_to_utf16(&value.0, &mut widedata);
        return String::from_utf16_lossy(&widedata[..res.unwrap()]);
    }
}

impl From<AnsiString> for String {
    fn from(value: AnsiString) -> Self {
        (&value).into()
    }
}

impl TryFrom<AnsiString> for CString {
    type Error = NulError;

    fn try_from(value: AnsiString) -> Result<Self, Self::Error> {
        CString::new(value.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! roundtrip_test {
        ($str:literal) => {
            let test: AnsiString = $str.try_into().unwrap();
            let tests: String = test.into();
            assert_eq!($str, tests);
        };
    }

    #[test]
    fn creating_short_string_from_ascii_string_works() {
        let test: AnsiString = "furry".try_into().unwrap();
        assert_eq!(test.0[..5], [0x66, 0x75, 0x72, 0x72, 0x79]);
    }

    #[test]
    fn creating_short_string_from_empty_string_works() {
        let test: AnsiString = "".try_into().unwrap();
        assert_eq!(test.0.len(), 0);
    }

    #[test]
    fn creating_short_string_from_number_string_works() {
        let test: AnsiString = "0".try_into().unwrap();
        assert_eq!(test.0[..1], [0x30]);
    }

    #[test]
    fn short_string_roundtrips() {
        roundtrip_test!("furry");
        roundtrip_test!("");
        roundtrip_test!("0");
    }
    
}