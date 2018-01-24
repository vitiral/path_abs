/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! The absolute path type, the root type for _most_ `Path*` types in this module
//! (except for `PathArc`).
use std::fmt;
use std::ops::Deref;
use std::io;
use std::convert::AsRef;
use std::path::{Path};

use super::{PathArc, PathFile, PathDir};

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An absolute ([canonicalized][1]) path that is guaranteed (when created) to exist.
///
/// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
pub struct PathAbs(pub(crate) PathArc);

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
        PathAbs::from_arc(PathArc::new(path))
    }

    pub fn from_arc(path: PathArc) -> io::Result<PathAbs> {
        path.canonicalize()
    }

    /// Resolve the `PathAbs` as a `PathFile`. Return an error if it is not a file.
    pub fn into_file(self) -> io::Result<PathFile> {
        PathFile::from_abs(self)
    }

    /// Resolve the `PathAbs` as a `PathDir`. Return an error if it is not a directory.
    pub fn into_dir(self) -> io::Result<PathDir> {
        PathDir::from_abs(self)
    }

    /// Get the parent directory of this path as a `PathDir`.
    ///
    /// > This does not make aditional syscalls, as the parent by definition must be a directory
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
            Some(p) => Some(PathDir(PathAbs(PathArc::new(p)))),
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
        PathAbs(PathArc::new(fake_path))
    }
}

impl fmt::Debug for PathAbs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<PathArc> for PathAbs {
    fn as_ref(&self) -> &PathArc {
        &self.0
    }
}

impl Deref for PathAbs {
    type Target = PathArc;

    fn deref(&self) -> &PathArc {
        &self.0
    }
}
