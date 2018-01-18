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
        Err(err) => Err(serde::de::Error::custom(&format!("{}: {}", $s, err))),
    }
}}}

impl Serialize for PathAbs {
    #[cfg(unix)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.as_os_str().as_bytes();
        let stfu = stfu8::encode_u8(bytes);
        serializer.serialize_str(&stfu)
    }

    #[cfg(windows)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wide: Vec<u16> = self.as_os_str().encode_wide().collect();
        let stfu = stfu8::encode_u16(&wide);
        serializer.serialize_str(&stfu)
    }
}

impl<'de> Deserialize<'de> for PathAbs {
    #[cfg(unix)]
    fn deserialize<D>(deserializer: D) -> Result<PathAbs, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let raw_path = map_err!(s, stfu8::decode_u8(&s))?;
        let os_str = OsStr::from_bytes(&raw_path);
        map_err!(s, PathAbs::new(os_str))
    }

    #[cfg(windows)]
    fn deserialize<D>(deserializer: D) -> Result<PathAbs, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let raw_path = map_err!(s, stfu8::decode_u16(&s))?;
        let os_str = OsString::from_wide(&raw_path);
        map_err!(s, PathAbs::new(os_str))
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
        let tmp_abs = PathDir::new(tmp_dir.path()).unwrap();

        let foo = PathFile::create(tmp_abs.join("foo.txt")).unwrap();
        let bar_dir = PathDir::create(tmp_abs.join("bar")).unwrap();
        let foo_bar_dir = PathDir::create_all(tmp_abs.join("foo/bar")).unwrap();

        let expected = vec![
            PathType::File(foo),
            PathType::Dir(bar_dir),
            PathType::Dir(foo_bar_dir),
        ];

        let expected_str = SERIALIZED.replace(
            "{0}",
            tmp_abs.to_string_lossy().as_ref(),
        );

        println!("### EXPECTED:\n{}", expected_str);
        let result_str = serde_json::to_string(&expected).unwrap();
        println!("### RESULT:\n{}", result_str);
        assert_eq!(expected_str, result_str);

        let result: Vec<PathType> = serde_json::from_str(&result_str).unwrap();
        assert_eq!(expected, result);
    }
}
