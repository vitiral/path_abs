/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Absolute serializable path types and associated methods.
//!
//! This library provides the following types:
//! - [`PathAbs`](struct.PathAbs.html): a reference counted absolute (canonicalized) path that is
//!   guaranteed (when created) to exist.
//! - [`PathFile`](struct.PathFile.html): a `PathAbs` that is guaranteed to be a file, with
//!   associated methods.
//! - [`PathDir`](struct.PathDir.html): a `PathAbs` that is guaranteed to be a directory, with
//!   associated methods.
//! - [`PathType`](struct.PathType.html): an enum containing either a file or a directory. Returned
//!   by `PathDir::list`.
//!
//! In addition, all types are serializable through serde (even on windows!) by using the crate
//! [`stfu8`](https://crates.io/crates/stfu8) to encode/decode, allowing ill-formed UTF-16.
//! See that crate for more details on how the resulting encoding can be edited (by hand)
//! even in the case of what *would be* ill-formed UTF-16.
//!
//! Also see the [project repo](https://github.com/vitiral/path_abs) and consider leaving a star!
//!
//! > All types are internally `Arc<PathBuf>` so they are extremely cheap to clone. When working
//! > with paths a reference count is NOT an expensive operation for you!
//!
//! # Examples
//! Recreating `Cargo.init` in `target/example`
//!
//! ```rust
//! # extern crate path_abs;
//! use std::collections::HashSet;
//! use path_abs::{PathAbs, PathDir, PathFile, PathType};
//!
//! # fn main() {
//!
//! let example = "target/example";
//!
//! # let _ = ::std::fs::remove_dir_all(example);
//!
//! // Create your paths
//! let project = PathDir::create_all(example).unwrap();
//! let src = PathDir::create(project.join("src")).unwrap();
//! let lib = PathFile::create(src.join("lib.rs")).unwrap();
//! let cargo = PathFile::create(project.join("Cargo.toml")).unwrap();
//!
//! // Write the templates
//! lib.write_str(r#"
//! #[cfg(test)]
//! mod tests {
//!     #[test]
//!     fn it_works() {
//!         assert_eq!(2 + 2, 4);
//!     }
//! }"#).unwrap();
//!
//! cargo.write_str(r#"
//! [package]
//! name = "example"
//! version = "0.1.0"
//! authors = ["Garrett Berg <googberg@gmail.com>"]
//!
//! [dependencies]
//! "#).unwrap();
//!
//! let mut result = HashSet::new();
//! for p in project.list().unwrap() {
//!     result.insert(p.unwrap());
//! }
//!
//! let mut expected = HashSet::new();
//! expected.insert(PathType::Dir(src));
//! expected.insert(PathType::File(cargo));
//!
//! assert_eq!(expected, result);
//!
//! // Get a file
//! let abs = PathAbs::new("target/example/src/lib.rs").unwrap();
//!
//! // or get the file of known type
//! let file = PathType::new("target/example/src/lib.rs")
//!     .unwrap()
//!     .unwrap_file();
//!
//! // or use `into_file`
//! let file2 = abs.clone().into_file().unwrap();
//!
//! assert!(abs.is_file());
//! assert!(file.is_file());
//! assert!(file2.is_file());
//! # }
//! ```

#[cfg(feature = "serialize")]
extern crate serde;
#[macro_use]
#[cfg(feature = "serialize")]
extern crate serde_derive;
#[cfg(feature = "serialize")]
extern crate stfu8;

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;
#[cfg(test)]
extern crate serde_json;
#[cfg(test)]
extern crate tempdir;

use std::convert::AsRef;
use std::io;
use std::fmt;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::ffi::OsStr;

mod abs;
mod dir;
mod file;
mod open;
#[cfg(feature = "serialize")]
mod ser;
mod ty;

pub use abs::PathAbs;
pub use dir::{PathDir, ListDir};
pub use file::PathFile;
pub use open::PathOpen;
pub use ty::PathType;

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// A `PathBuf` that is atomically reference counted and reimplements the `PathBuf`
/// methods to display the action and path when there is an error.
///
/// This is the root type of all other `Path*` types in this crate.
///
/// This type is also serializable when the `serialize` feature is enabled.
pub struct PathArc(Arc<PathBuf>);

impl PathArc {
    /// Instantiate a new `PathArc`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathArc;
    ///
    /// # fn main() {
    /// let path = PathArc::new("some/path");
    /// let path2 = path.clone(); // cloning is cheap
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> PathArc {
        PathArc::from(path.as_ref().to_path_buf())
    }

    /// Creates an owned PathBuf with path adjoined to self.
    ///
    /// This function is identical to [std::path::PathBuf::join][0] except
    /// it returns `PathArc` instead of `PathBuf`
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.join
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathArc {
        PathArc::from(self.0.join(path))
    }

    /// Creates an owned `PathArc` like self but with the given file name.
    ///
    /// This function is identical to [std::path::PathBuf::with_file_name][0] except
    /// it returns `PathArc` instead of `PathBuf`
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.with_file_name
    pub fn with_file_name<P: AsRef<OsStr>>(&self, file_name: P) -> PathArc {
        PathArc::from(self.0.with_file_name(file_name))
    }

    /// Creates an owned `PathArc` like self but with the given extension.
    ///
    /// This function is identical to [std::path::PathBuf::with_extension][0] except
    /// it returns `PathArc` instead of `PathBuf`
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.with_extension
    pub fn with_extension<P: AsRef<OsStr>>(&self, extension: P) -> PathArc {
        PathArc::from(self.0.with_extension(extension))
    }

    /// Queries the file system to get information about a file, directory, etc.
    ///
    /// This function will traverse symbolic links to query information about the destination file.
    ///
    /// This function is identical to [std::path::Path::metadata][0] except it has error
    /// messages which include the action and the path
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.metadata
    pub fn metadata(&self) -> io::Result<fs::Metadata> {
        self.0.metadata().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when getting metadata of {}", err, self.display()),
            )
        })
    }

    /// Queries the metadata about a file without following symlinks.
    ///
    /// This function is identical to [std::path::Path::symlink_metadata][0] except it has error
    /// messages which include the action and the path
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.symlink_metadata
    pub fn symlink_metadata(&self) -> io::Result<fs::Metadata> {
        self.0.symlink_metadata().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when getting symlink_metadata of {}", err, self.display()),
            )
        })
    }

    /// Returns the canonical form of the path with all intermediate components normalized and
    /// symbolic links resolved.
    ///
    /// > This is identical to `PathAbs::new(path)`.
    ///
    /// This function is identical to [std::path::Path::canonicalize][0] except:
    /// - It returns a `PathAbs` object
    /// - It has error messages which include the action and the path
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.canonicalize
    pub fn canonicalize(&self) -> io::Result<PathAbs> {
        let abs = self.0.canonicalize().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} canonicalizing {}", err, self.display()),
            )
        })?;

        Ok(PathAbs(PathArc::from(abs)))
    }

    /// Reads a symbolic link, returning the file that the link points to.
    ///
    /// This function is identical to [std::path::Path::read_link][0] except:
    /// - It returns a `PathArc` object instead of `PathBuf`
    /// - It has error messages which include the action and the path
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.read_link
    pub fn read_link(&self) -> io::Result<PathArc> {
        let path = self.0.read_link().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} reading link {}", err, self.display()),
            )
        })?;

        Ok(PathArc::from(path))
    }

    /// Returns an iterator over the entries within a directory.
    ///
    /// This function is a shortcut to `PathDir::list`. It is slightly different
    /// than [std::path::Path::read_dir][0].
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.read_dir
    pub fn read_dir(&self) -> io::Result<ListDir> {
        let dir = PathDir::new(self)?;
        dir.list()
    }
}

impl fmt::Debug for PathArc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<PathBuf> for PathArc {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for PathArc {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl Deref for PathArc {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

impl From<PathBuf> for PathArc {
    /// Instantiate a new `PathArc` from a `PathBuf`.
    fn from(path: PathBuf) -> PathArc {
        PathArc(Arc::new(path))
    }
}
