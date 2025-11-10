use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
};

use crate::{arg_parse_err::ArgParseErr, args::ArgSign, utils::os_str::OsStrExt};

/// Handles `-define` or `+define`
/// (see <https://imagemagick.org/script/command-line-options.php#define>).
pub fn process_definition(
    definitions: &mut HashMap<String, OsString>,
    arg_sign: ArgSign,
    input: &OsStr,
) -> Result<(), ArgParseErr> {
    // `-define` takes an argument in either `key` or `key=value` format.
    // `-define key` is treated like `-define key=`, i.e. the value is an empty string.
    //
    // `+define` takes a `key` argument and removes a definition previously set using `-define`.
    // If `key` is `*`, all existing definitions are removed.
    //
    // Upper case ASCII characters in keys are normalized to lower case.
    //
    // Values are OsString because they can contain file paths.
    // Technically keys should be OsString as well (ImageMagick ignores keys it doesn't understand)
    // but for simplicity we convert to String and return Err if that fails.
    match arg_sign {
        ArgSign::Minus => {
            let (key, value) = input.split_once(b'=').unwrap_or((input.into(), "".into()));
            let key: &str = key.as_os_str().try_into().map_err(ArgParseErr::with_msg)?;
            definitions.insert(key.to_ascii_lowercase(), value.to_owned());
        }
        ArgSign::Plus => {
            if input == "*" {
                definitions.clear();
            } else {
                let key: &str = input.try_into().map_err(ArgParseErr::with_msg)?;
                if definitions.remove(&key.to_ascii_lowercase()).is_none() {
                    return Err(ArgParseErr::with_msg(format!(
                        "attempted to remove a definition that is not set: {key}"
                    )));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set() {
        let mut defs = HashMap::new();
        assert!(process_definition(&mut defs, ArgSign::Minus, OsStr::new("key1")).is_ok());
        assert!(process_definition(&mut defs, ArgSign::Minus, OsStr::new("key2=value2")).is_ok());
        assert!(process_definition(&mut defs, ArgSign::Minus, OsStr::new("KEY3=VALUE3")).is_ok());
        assert_eq!(defs["key1"], "");
        assert_eq!(defs["key2"], "value2");
        assert_eq!(defs["key3"], "VALUE3");
    }

    #[test]
    fn test_override() {
        let mut defs = HashMap::from_iter([("key".into(), "value".into())]);
        assert!(process_definition(&mut defs, ArgSign::Minus, OsStr::new("key=value1")).is_ok());
        assert_eq!(defs["key"], "value1");
    }

    #[test]
    fn test_unset() {
        let mut defs = HashMap::from_iter([
            ("key1".into(), "value1".into()),
            ("key2".into(), "value2".into()),
        ]);
        assert!(process_definition(&mut defs, ArgSign::Plus, OsStr::new("key1")).is_ok());
        assert!(!defs.contains_key("key1"));
        assert!(process_definition(&mut defs, ArgSign::Plus, OsStr::new("KEY2")).is_ok());
        assert!(!defs.contains_key("key2"));
        assert!(process_definition(&mut defs, ArgSign::Plus, OsStr::new("key1")).is_err());
    }

    #[test]
    fn test_clear() {
        let mut defs = HashMap::from_iter([
            ("key1".into(), "value1".into()),
            ("key2".into(), "value2".into()),
        ]);
        assert!(process_definition(&mut defs, ArgSign::Plus, OsStr::new("*")).is_ok());
        assert!(defs.is_empty());
    }
}
