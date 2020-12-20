/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::string::ToString;
use std_prelude::*;
use stfu8;

use super::{PathMut, PathOps};

use std::ffi::{OsStr, OsString};
#[cfg(target_os = "wasi")]
use std::os::wasi::ffi::{OsStrExt, OsStringExt};
#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use super::{PathAbs, PathDir, PathFile};

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct PathSer(Arc<PathBuf>);

pub trait ToStfu8 {
    fn to_stfu8(&self) -> String;
}

pub trait FromStfu8: Sized {
    fn from_stfu8(s: &str) -> Result<Self, stfu8::DecodeError>;
}

impl PathSer {
    pub fn new<P: Into<Arc<PathBuf>>>(path: P) -> Self {
        PathSer(path.into())
    }

    pub fn as_path(&self) -> &Path {
        self.as_ref()
    }
}

impl fmt::Debug for PathSer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PathMut for PathSer {
    fn append<P: AsRef<Path>>(&mut self, path: P) -> crate::Result<()> {
        self.0.append(path)
    }
    fn pop_up(&mut self) -> crate::Result<()> {
        self.0.pop_up()
    }
    fn truncate_to_root(&mut self) {
        self.0.truncate_to_root()
    }
    fn set_file_name<S: AsRef<OsStr>>(&mut self, file_name: S) {
        self.0.set_file_name(file_name)
    }
    fn set_extension<S: AsRef<OsStr>>(&mut self, extension: S) -> bool {
        self.0.set_extension(extension)
    }
}

impl PathOps for PathSer {
    type Output = PathSer;

    fn concat<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::Output> {
        Ok(PathSer(self.0.concat(path)?))
    }

    fn join<P: AsRef<Path>>(&self, path: P) -> Self::Output {
        let buf = Path::join(self.as_path(), path);
        Self::Output::new(buf)
    }

    fn with_file_name<S: AsRef<OsStr>>(&self, file_name: S) -> Self::Output {
        PathSer(self.0.with_file_name(file_name))
    }

    fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> Self::Output {
        PathSer(self.0.with_extension(extension))
    }
}

impl AsRef<OsStr> for PathSer {
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

impl<P: Into<PathBuf>> From<P> for PathSer {
    fn from(path: P) -> PathSer {
        PathSer::new(path.into())
    }
}

impl From<PathSer> for Arc<PathBuf> {
    fn from(path: PathSer) -> Arc<PathBuf> {
        path.0
    }
}

// impl From<PathAbs> for PathSer {
//     fn from(path: PathAbs) -> PathSer {
//         PathSer(path.0)
//     }
// }

impl<T> ToStfu8 for T
where
    T: Borrow<PathBuf>,
{
    #[cfg(any(target_os = "wasi", unix))]
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
    #[cfg(any(target_os = "wasi", unix))]
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
    use super::super::{PathDir, PathFile, PathInfo, PathMut, PathOps, PathType};
    use super::*;

    #[cfg(any(target_os = "wasi", unix))]
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

        let ser_from_str = PathSer::from("example");
        let ser_from_tmp_abs = PathSer::from(tmp_abs.as_path());

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

    #[test]
    /// Just test that it has all the methods.
    fn sanity_ser() {
        let mut path = PathSer::from("example/path");
        assert_eq!(
            path.join("joined").as_path(),
            Path::new("example/path/joined")
        );
        assert_eq!(path.is_absolute(), false);

        path.append("appended").unwrap();
        assert_eq!(path.as_path(), Path::new("example/path/appended"));
        path.pop_up().unwrap();
        assert_eq!(path.as_path(), Path::new("example/path"));

        assert_eq!(
            path.concat("/concated").unwrap().as_path(),
            Path::new("example/path/concated")
        );
    }
}
