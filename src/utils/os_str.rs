use std::ffi::{OsStr, OsString};

pub trait OsStrExt {
    /// Like [`str::split_once`].
    ///
    /// Useful while <https://github.com/rust-lang/libs-team/issues/311> is stalled.
    fn split_once(&self, separator: u8) -> Option<(OsString, OsString)>;
}

impl OsStrExt for OsStr {
    fn split_once(&self, separator: u8) -> Option<(OsString, OsString)> {
        if !separator.is_ascii() {
            panic!("split_once only supports ASCII separator");
        }
        #[cfg(any(unix, target_os = "wasi"))]
        {
            use std::os::unix::ffi::OsStrExt;
            let bytes = self.as_bytes();
            let mut iter = bytes.splitn(2, |&b| b == separator);
            let prefix = iter.next().unwrap();
            let suffix = iter.next()?;
            Some((
                OsStr::from_bytes(prefix).to_owned(),
                OsStr::from_bytes(suffix).to_owned(),
            ))
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::{OsStrExt, OsStringExt};
            let wide_chars: Vec<u16> = self.encode_wide().collect();
            let mut iter = wide_chars.splitn(2, |&wc| wc == separator as u16);
            let prefix = iter.next().unwrap();
            let suffix = iter.next()?;
            Some((OsString::from_wide(prefix), OsString::from_wide(suffix)))
        }
        #[cfg(not(any(unix, windows, target_os = "wasi")))]
        {
            // Outside the above platforms, we only support splitting UTF-8
            let input = self.to_str()?;
            let (prefix, suffix) = input.split_once(separator as char)?;
            Some((OsString::from(prefix), OsString::from(suffix)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_once() {
        assert_eq!(
            OsStr::new("=").split_once(b'='),
            Some((OsString::from(""), OsString::from(""))),
        );
        assert_eq!(
            OsStr::new("aa=bb").split_once(b'='),
            Some((OsString::from("aa"), OsString::from("bb"))),
        );
        assert_eq!(OsStr::new("").split_once(b'='), None);
        assert_eq!(OsStr::new("aa").split_once(b'='), None);
    }
}
