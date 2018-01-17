//! Extensions to stdlib `Path` types, plus the `PathAbs` type.
//!
//! [`PathAbs`](structs.PathAbs.html) adds a much needed type to the rust ecosystem:
//! a path which is guaranteed to exist (at least on creation), is serializable, and has
//! extension methods like `create[file/dir/dir_all]`, `read_*` and `write_*`.
//!
//! In addition, `PathAbs` is serializable through serde (even on windows!) by using the crate
//! [`stfu8`](https://crates.io/crates/stfu8) to encode/decode any ill-formed UTF-16.
//! See that crate for more details on how the resulting encoding can be edited (by hand)
//! even in the case of what *would be* ill-formed UTF-16.

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate stfu8;

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;
#[cfg(test)]
extern crate tempdir;
#[cfg(test)]
extern crate serde_json;

use std::convert::AsRef;
use std::io;
use std::fs;
use std::fmt;
use std::ops::Deref;
use std::path::{Path, PathBuf};

mod dir;
mod file;
mod ser;

pub use file::PathFile;
pub use dir::PathDir;

// #[cfg(test)]
// mod tests;

// ------------------------------
// -- EXPORTED TYPES / METHODS

#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag="type", content="path", rename_all="lowercase")]
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
            Err(io::Error::new(io::ErrorKind::InvalidInput, "path is not a dir or a file"))
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
    pub fn unwrap_file(self) -> Option<PathFile> {
        if let PathType::File(f) = self {
            Some(f)
        } else {
            None
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
    pub fn unwrap_dir(self) -> Option<PathDir> {
        if let PathType::Dir(d) = self {
            Some(d)
        } else {
            None
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
    pub fn mock_file<P: AsRef<Path>>(path: P) -> PathType {
        PathType::File(PathFile::mock(path))
    }

    /// Create a mock dir type. *For use in tests only*.
    pub fn mock_dir<P: AsRef<Path>>(path: P) -> PathType {
        PathType::Dir(PathDir::mock(path))
    }
}

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An path which is guaranteed to:
/// - Exist (on creation, the file may or may not be deleted later).
/// - Be absolute (cannonicalized). On linux this means it will start with root (`/`) and
///   have no symlinks.
///
/// > Implemented by calling [`Path::canonicalize()`][1] under the hood.
///
/// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
pub struct PathAbs(PathBuf);

impl PathAbs {
    /// Instantiate a new `PathAbs`. The path must exist or `io::Error` will be returned.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    /// let lib = PathAbs::new("src/lib.rs").unwrap();
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        Ok(PathAbs(path.as_ref().canonicalize()?))
    }

    pub fn to_file(self) -> io::Result<PathFile> {
        PathFile::from_abs(self)
    }

    pub fn to_dir(self) -> io::Result<PathDir> {
        PathDir::from_abs(self)
    }

    /// Get the parent directory of this path as a `PathDir`.
    ///
    /// > This does not make additinal syscalls, as the parent by definition must be a directory
    /// > and exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathDir, PathFile};
    ///
    /// # fn main() {
    /// let lib = PathFile::new("src/lib.rs").unwrap();
    /// let src = lib.parent_dir().unwrap();
    /// assert_eq!(PathDir::new("src").unwrap(), src);
    /// # }
    /// ```
    pub fn parent_dir(&self) -> Option<PathDir> {
        match self.parent() {
            Some(p) => Some(PathDir(PathAbs(p.to_path_buf()))),
            None => None,
        }
    }

    /// For constructing mocked paths during tests. This is effectively the same as a `PathBuf`.
    ///
    /// This is NOT checked for validity so the file may or may not actually exist and will
    /// NOT be, in any way, an absolute or canonicalized path.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    /// // this file exist
    /// let lib = PathAbs::new("src/lib.rs").unwrap();
    ///
    /// let lib_mocked = PathAbs::mock("src/lib.rs");
    ///
    /// // in this case, the mocked file exists
    /// assert!(lib_mocked.exists());
    ///
    /// // However, it is NOT equivalent to `lib`
    /// assert_ne!(lib, lib_mocked);
    ///
    /// // this file doesn't exist at all
    /// let dne = PathAbs::mock("src/dne.rs");
    /// assert!(!dne.exists());
    /// # }
    /// ```
    pub fn mock<P: AsRef<Path>>(fake_path: P) -> PathAbs {
        PathAbs(fake_path.as_ref().to_path_buf())
    }
}

impl fmt::Debug for PathAbs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<PathBuf> for PathAbs {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for PathAbs {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl Deref for PathAbs {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

