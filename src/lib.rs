/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */

//! Absolute and serializable paths

extern crate serde;
extern crate stfu8;

use std::io;
use std::fmt;
use std::ops::Deref;
use std::convert::AsRef;
use std::path::{Path, PathBuf};

// ------------------------------
// -- EXPORTED TYPES / METHODS

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An path which is guaranteed to:
/// - exist (on creation)
/// - be absolute
///
/// Implemented by calling `Path::canonicalize()` under the hood.
///
/// # Examples
/// ```rust
/// extern crate path_abs;
/// use path_abs::PathAbs;
///
/// # fn main() {
/// let path = PathAbs::new("src/lib.rs").unwrap();
/// # }
/// ```
pub struct PathAbs(PathBuf);

impl PathAbs {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        Ok(PathAbs(path.as_ref().canonicalize()?))
    }

    /// For constructing fake paths during tests
    ///
    /// This is NOT checked for validity. It may or may not actually be fake.
    pub fn fake<P: AsRef<Path>>(fake_path: P) -> PathAbs {
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
