/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use std::io;
use std_prelude::*;

use super::{Error, Result};
use super::PathAbs;
use file::PathFile;
use dir::PathDir;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", serde(tag = "type", content = "path", rename_all = "lowercase"))]
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An an enum containing either a file or a directory.
///
/// This is used primarily for:
/// - The items returned from `PathDir::list`
/// - Serializing paths of different types.
///
/// > Note: symlinks are not supported because they are
/// > *impossible* for canonicalized paths.
pub enum PathType {
    File(PathFile),
    Dir(PathDir),
}

impl PathType {
    /// Resolves and returns the `PathType` of the given path.
    ///
    /// > If the path exists but is not a file or a directory (i.e. is a symlink), then
    /// > `io::ErrorKind::InvalidInput` is returned.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathType;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let src = PathType::new("src")?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    pub fn new<P: AsRef<Path>>(path: P) -> Result<PathType> {
        let abs = PathAbs::new(&path)?;
        PathType::from_abs(abs)
    }

    /// Consume the `PathAbs` returning the `PathType`.
    pub fn from_abs(abs: PathAbs) -> Result<PathType> {
        let ty = abs.metadata()?.file_type();
        if ty.is_file() {
            Ok(PathType::File(PathFile(abs)))
        } else if ty.is_dir() {
            Ok(PathType::Dir(PathDir(abs)))
        } else {
            Err(Error::new(
                io::Error::new(io::ErrorKind::InvalidInput, "path is not a dir or a file"),
                "resolving",
                abs.into(),
            ))
        }
    }

    /// Unwrap the `PathType` as a `PathFile`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathType;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let lib = PathType::new("src/lib.rs")?.unwrap_file();
    /// # Ok(()) } fn main() { try_main().unwrap() }
    pub fn unwrap_file(self) -> PathFile {
        match self {
            PathType::File(f) => f,
            PathType::Dir(d) => {
                panic!("unwrap_file called on {}, which is not a file", d.display())
            }
        }
    }

    /// Unwrap the `PathType` as a `PathDir`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathType;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let src = PathType::new("src")?.unwrap_dir();
    /// # Ok(()) } fn main() { try_main().unwrap() }
    pub fn unwrap_dir(self) -> PathDir {
        match self {
            PathType::Dir(d) => d,
            PathType::File(f) => panic!(
                "unwrap_dir called on {}, which is not a directory",
                f.display()
            ),
        }
    }

    /// Return whether this variant is `PathType::Dir`.
    pub fn is_dir(&self) -> bool {
        if let PathType::Dir(_) = *self {
            true
        } else {
            false
        }
    }

    /// Return whether this variant is `PathType::File`.
    pub fn is_file(&self) -> bool {
        if let PathType::File(_) = *self {
            true
        } else {
            false
        }
    }

    /// Create a mock file type. *For use in tests only*.
    ///
    /// See the docs for [`PathAbs::mock`](struct.PathAbs.html#method.mock)
    pub fn mock_file<P: AsRef<Path>>(path: P) -> PathType {
        PathType::File(PathFile::mock(path))
    }

    /// Create a mock dir type. *For use in tests only*.
    ///
    /// See the docs for [`PathAbs::mock`](struct.PathAbs.html#method.mock)
    pub fn mock_dir<P: AsRef<Path>>(path: P) -> PathType {
        PathType::Dir(PathDir::mock(path))
    }
}
