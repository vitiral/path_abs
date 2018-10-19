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
use std_prelude::*;

use super::{PathArc, PathDir, PathFile, Result};

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An absolute (not _necessarily_ [canonicalized][1]) path that may or may not exist.
///
/// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
pub struct PathAbs(pub(crate) PathArc);

impl PathAbs {
    /// Instantiate a new `PathAbs`. The path must exist or `io::Error` will be returned.
    ///
    /// # Examples
    /// ```rust
    /// use path_abs::PathAbs;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let lib = PathAbs::new("src/lib.rs")?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<PathAbs> {
        let arc = PathArc::new(path);
        arc.absolute()
    }

    /// Resolve the `PathAbs` as a `PathFile`. Return an error if it is not a file.
    pub fn into_file(self) -> Result<PathFile> {
        PathFile::from_abs(self)
    }

    /// Resolve the `PathAbs` as a `PathDir`. Return an error if it is not a directory.
    pub fn into_dir(self) -> Result<PathDir> {
        PathDir::from_abs(self)
    }

    /// Return a reference to a basic `std::path::Path`
    pub fn as_path(&self) -> &Path {
        self.as_ref()
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
    /// # fn try_main() -> ::std::io::Result<()> {
    /// // this file exist
    /// let lib = PathAbs::new("src/lib.rs")?;
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
    /// # Ok(()) } fn main() { try_main().unwrap() }
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

impl AsRef<Path> for PathAbs {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<PathBuf> for PathAbs {
    fn as_ref(&self) -> &PathBuf {
        self.0.as_ref()
    }
}

impl Borrow<PathArc> for PathAbs {
    fn borrow(&self) -> &PathArc {
        self.as_ref()
    }
}

impl Borrow<Path> for PathAbs {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl Borrow<PathBuf> for PathAbs {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl<'a> Borrow<PathArc> for &'a PathAbs {
    fn borrow(&self) -> &PathArc {
        self.as_ref()
    }
}

impl<'a> Borrow<Path> for &'a PathAbs {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl<'a> Borrow<PathBuf> for &'a PathAbs {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl Deref for PathAbs {
    type Target = PathArc;

    fn deref(&self) -> &PathArc {
        &self.0
    }
}

impl Into<PathArc> for PathAbs {
    fn into(self) -> PathArc {
        self.0
    }
}
