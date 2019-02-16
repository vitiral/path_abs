/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::string::ToString;
use std_prelude::*;
use stfu8;

use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use super::{PathAbs, PathDir, PathFile};

pub struct PathSer(Arc<PathBuf>);

impl AsRef<std::ffi::OsStr> for PathSer {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_ref().as_ref()
    }
}

impl AsRef<Path> for PathSer {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<PathBuf> for PathSer {
    fn as_ref(&self) -> &PathBuf {
        self.0.as_ref()
    }
}

impl Borrow<Path> for PathSer {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl Borrow<PathBuf> for PathSer {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl<'a> Borrow<Path> for &'a PathSer {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl<'a> Borrow<PathBuf> for &'a PathSer {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl From<PathAbs> for PathSer {
    fn from(path: PathAbs) -> PathSer {
        PathSer(path.0.clone())
    }
}

trait ToStfu8 {
    fn to_stfu8(&self) -> String;
}

trait FromStfu8: Sized {
    fn from_stfu8(s: &str) -> Result<Self, stfu8::DecodeError>;
}

impl<T> ToStfu8 for T
where
    T: Borrow<PathBuf>,
{
    #[cfg(unix)]
    fn to_stfu8(&self) -> String {
        let bytes = self.borrow().as_os_str().as_bytes();
        stfu8::encode_u8(bytes)
    }

    #[cfg(windows)]
    fn to_stfu8(&self) -> String {
        let wide: Vec<u16> = self.borrow().as_os_str().encode_wide().collect();
        stfu8::encode_u16(&wide)
    }
}

impl<T> FromStfu8 for T
where
    T: From<PathBuf>,
{
    #[cfg(unix)]
    fn from_stfu8(s: &str) -> Result<T, stfu8::DecodeError> {
        let raw_path = stfu8::decode_u8(s)?;
        let os_str = OsString::from_vec(raw_path);
        let pathbuf: PathBuf = os_str.into();
        Ok(pathbuf.into())
    }

    #[cfg(windows)]
    fn from_stfu8(s: &str) -> Result<T, stfu8::DecodeError> {
        let raw_path = stfu8::decode_u16(&s)?;
        let os_str = OsString::from_wide(&raw_path);
        let pathbuf: PathBuf = os_str.into();
        Ok(pathbuf.into())
    }
}

macro_rules! stfu8_serialize {
    ($name:ident) => {
        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.to_stfu8())
            }
        }
    };
}

stfu8_serialize!(PathSer);
stfu8_serialize!(PathAbs);
stfu8_serialize!(PathFile);
stfu8_serialize!(PathDir);

impl<'de> Deserialize<'de> for PathSer {
    fn deserialize<D>(deserializer: D) -> Result<PathSer, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let path =
            PathBuf::from_stfu8(&s).map_err(|err| serde::de::Error::custom(&err.to_string()))?;
        Ok(PathSer(Arc::new(path)))
    }
}

impl<'de> Deserialize<'de> for PathAbs {
    fn deserialize<D>(deserializer: D) -> Result<PathAbs, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let path =
            PathBuf::from_stfu8(&s).map_err(|err| serde::de::Error::custom(&err.to_string()))?;
        Ok(PathAbs(Arc::new(path)))
    }
}

impl<'de> Deserialize<'de> for PathFile {
    fn deserialize<D>(deserializer: D) -> Result<PathFile, D::Error>
    where
        D: Deserializer<'de>,
    {
        let abs = PathAbs::deserialize(deserializer)?;
        PathFile::try_from(abs).map_err(|err| serde::de::Error::custom(&err.to_string()))
    }
}

impl<'de> Deserialize<'de> for PathDir {
    fn deserialize<D>(deserializer: D) -> Result<PathDir, D::Error>
    where
        D: Deserializer<'de>,
    {
        let abs = PathAbs::deserialize(deserializer)?;
        PathDir::try_from(abs).map_err(|err| serde::de::Error::custom(&err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::super::{PathDir, PathFile, PathOps, PathType};
    use super::*;

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

        let foo = PathFile::create(tmp_abs.concat("foo.txt").unwrap()).expect("foo.txt");
        let bar_dir = PathDir::create(tmp_abs.concat("bar").unwrap()).expect("bar");
        let foo_bar_dir =
            PathDir::create_all(tmp_abs.concat("foo").unwrap().concat("bar").unwrap())
                .expect("foo/bar");

        let expected = vec![
            PathType::File(foo),
            PathType::Dir(bar_dir),
            PathType::Dir(foo_bar_dir),
        ];

        let expected_str = SERIALIZED
            .replace("{0}", &tmp_abs.to_stfu8())
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
