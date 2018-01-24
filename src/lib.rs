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
use std::ops::Deref;
use std::path::{Path, PathBuf, StripPrefixError};
use std::sync::Arc;

mod abs;
mod dir;
mod file;
mod open;
#[cfg(feature = "serialize")]
mod ser;
mod ty;

pub use abs::PathAbs;
pub use dir::PathDir;
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
        PathArc::from_buf(path.as_ref().to_path_buf())
    }

    /// Instantiate a new `PathArc` from a `PathBuf`.
    pub fn from_buf(path: PathBuf) -> PathArc {
        PathArc(Arc::new(path))
    }

    // /// Returns a path that, when joined onto base, yields self.
    // ///
    // /// This function is identical to [std::path::Path::strip_prefix][0] except
    // /// it has error messages which include the action and the path
    // ///
    // /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.strip_prefix
    // pub fn strip_prefix<'a, P>(&'a self, base: &'a P) -> Result<&'a Path, StripPrefixError>
    //     where P: AsRef<Path>
    // {
    //     self.0.strip_prefix(base).map_err(|err| {
    //         io::Error::new(
    //             err.kind(),
    //             format!("{} when stripping prefix of {}", err, self.display()),
    //         )
    //     })
    // }
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
