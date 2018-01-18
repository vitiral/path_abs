/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use stfu8;

#[cfg(unix)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use super::{PathAbs, PathDir, PathFile};

macro_rules! map_err { ($s: expr, $res: expr) => {{
    match $res {
        Ok(v) => Ok(v),
        // FIXME: change degug->display
        Err(err) => Err(serde::de::Error::custom(&format!("{}: {:?}", $s, err))),
    }
}}}

#[derive(Debug)]
// FIXME: flush out this error type
pub enum DeserError {
    Decode(stfu8::DecodeError),
    Filesystem(::std::io::Error),
}

impl PathAbs {
    #[cfg(unix)]
    pub fn to_stfu8(&self) -> String {
        let bytes = self.as_os_str().as_bytes();
        stfu8::encode_u8(bytes)
    }

    #[cfg(windows)]
    pub fn to_stfu8(&self) -> String {
        let wide: Vec<u16> = self.as_os_str().encode_wide().collect();
        stfu8::encode_u16(&wide)
    }

    #[cfg(unix)]
    pub fn from_stfu8(s: &str) -> Result<PathAbs, DeserError> {
        let raw_path = stfu8::decode_u8(&s).map_err(|e| DeserError::Decode(e))?;
        let os_str = OsStr::from_bytes(&raw_path);
        PathAbs::new(os_str).map_err(|e| DeserError::Filesystem(e))
    }

    #[cfg(windows)]
    pub fn from_stfu8(s: &str) -> Result<PathAbs, DeserError> {
        let raw_path = stfu8::decode_u16(&s).map_err(|e| DeserError::Decode(e))?;
        let os_str = OsString::from_wide(&raw_path);
        PathAbs::new(os_str).map_err(|e| DeserError::Filesystem(e))
    }
}

impl Serialize for PathAbs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_stfu8())
    }
}

impl<'de> Deserialize<'de> for PathAbs {
    fn deserialize<D>(deserializer: D) -> Result<PathAbs, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        map_err!(s, PathAbs::from_stfu8(&s))
    }
}

impl Serialize for PathFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PathFile {
    fn deserialize<D>(deserializer: D) -> Result<PathFile, D::Error>
    where
        D: Deserializer<'de>,
    {
        let abs = PathAbs::deserialize(deserializer)?;
        PathFile::from_abs(abs).map_err(serde::de::Error::custom)
    }
}

impl Serialize for PathDir {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PathDir {
    fn deserialize<D>(deserializer: D) -> Result<PathDir, D::Error>
    where
        D: Deserializer<'de>,
    {
        let abs = PathAbs::deserialize(deserializer)?;
        PathDir::from_abs(abs).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{PathDir, PathFile, PathType};

    #[cfg(unix)]
    static SERIALIZED: &str = "[\
                               {\"type\":\"file\",\"path\":\"{0}/foo.txt\"},\
                               {\"type\":\"dir\",\"path\":\"{0}/bar\"},\
                               {\"type\":\"dir\",\"path\":\"{0}/foo/bar\"}\
                               ]";

    #[cfg(windows)]
    static SERIALIZED: &str = "[\
                               {\"type\":\"file\",\"path\":\"{0}\\\\foo.txt\"},\
                               {\"type\":\"dir\",\"path\":\"{0}\\\\bar\"},\
                               {\"type\":\"dir\",\"path\":\"{0}\\\\foo\\\\bar\"}\
                               ]";

    #[test]
    fn sanity_serde() {
        use serde_json;
        use tempdir::TempDir;

        let tmp_dir = TempDir::new("example").expect("create temp dir");
        let tmp_abs = PathDir::new(tmp_dir.path()).expect("tmp_abs");

        let foo = PathFile::create(tmp_abs.join("foo.txt")).expect("foo.txt");
        let bar_dir = PathDir::create(tmp_abs.join("bar")).expect("bar");
        let foo_bar_dir = PathDir::create_all(tmp_abs.join("foo").join("bar")).expect("foo/bar");

        let expected = vec![
            PathType::File(foo),
            PathType::Dir(bar_dir),
            PathType::Dir(foo_bar_dir),
        ];

        let mut expected_str = SERIALIZED.replace(
                "{0}",
                &tmp_abs.to_stfu8(),
            )
            // JSON needs backslashes escaped. Be careful not to invoke BA'AL:
            // https://xkcd.com/1638/)
            .replace(r"\", r"\\");

        println!("### EXPECTED:\n{}", expected_str);
        let result_str = serde_json::to_string(&expected).unwrap();
        println!("### RESULT:\n{}", result_str);
        assert_eq!(expected_str, result_str);

        let result: Vec<PathType> = serde_json::from_str(&result_str).unwrap();
        assert_eq!(expected, result);
    }
}
