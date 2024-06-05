use windows_sys::Win32::Foundation::WIN32_ERROR;

/// The all-in-one error type to represent errors that may occur in the framework.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Error returned when conversion was attempted on a string that cannot be represented in the current ANSI code page.
    AnsiConversionError,
    /// Error returned when there is an error from the Win32 API.
    Win32Error(WIN32_ERROR),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::AnsiConversionError => write!(f, "String contains characters that cannot be represented in the current ANSI code page"),
            Error::Win32Error(e) => write!(f, "Win32 error: {}", e)
        }
    }
}