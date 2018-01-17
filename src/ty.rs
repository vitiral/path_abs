/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use std::convert::AsRef;
use std::io;
use std::path::Path;

use super::PathAbs;
use file::PathFile;
use dir::PathDir;

#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", content = "path", rename_all = "lowercase")]
/// An enum representing absolute paths of known types.
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
    /// # fn main() {
    /// let src = PathType::new("src").unwrap();
    /// # }
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathType> {
        let abs = PathAbs::new(path)?;
        let ty = abs.metadata()?.file_type();
        if ty.is_file() {
            Ok(PathType::File(PathFile(abs)))
        } else if ty.is_dir() {
            Ok(PathType::Dir(PathDir(abs)))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is not a dir or a file",
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
    /// # fn main() {
    /// let lib = PathType::new("src/lib.rs").unwrap().unwrap_file();
    /// # }
    pub fn unwrap_file(self) -> PathFile {
        if let PathType::File(f) = self {
            f
        } else {
            panic!("unwrap_file called on path that is not a file");
        }
    }

    /// Unwrap the `PathType` as a `PathDir`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathType;
    ///
    /// # fn main() {
    /// let src = PathType::new("src").unwrap().unwrap_dir();
    /// # }
    pub fn unwrap_dir(self) -> PathDir {
        if let PathType::Dir(d) = self {
            d
        } else {
            panic!("unwrap_dir called on path that is not a dir");
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
