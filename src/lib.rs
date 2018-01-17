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
//!
//! [`PathAbs`](structs.PathAbs.html) adds a much needed type to the rust ecosystem:
//! a path which is guaranteed to exist, is serializable, and is creatable through
//! simple methods:
//! - `Path::new`: ensure the
//! - `Path::create_file`: create the file if it doesn't exist and return the absolute path.
//! - `Path::create_dir`: create the directory if it doesn't exist and return the absolute path.
//! - `Path::create_dir_all`: recursively create the directory and all its parent components and
//!   return the absolute path
//!
//! `PathAbs` is serializable by using the crate [`stfu8`](https://crates.io/crates/stfu8)
//! to encode and decode the potentially non-compliant ASCII.

extern crate serde;
extern crate stfu8;

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;
#[cfg(test)]
extern crate tempdir;

use std::io;
use std::fs;
use std::fmt;
use std::ops::Deref;
use std::convert::AsRef;
use std::path::{Path, PathBuf};

use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

// #[cfg(test)]
// mod tests;

// ------------------------------
// -- EXPORTED TYPES / METHODS

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// An path which is guaranteed to:
/// - Exist (on creation, the file may or may not be deleted later).
/// - Be absolute (cannonicalized). On linux this means it will start with root (`/`) and
///   have no symlinks.
///
/// > Implemented by calling `Path::canonicalize()` under the hood.
pub struct PathAbs(PathBuf);

impl PathAbs {
    /// Instantiate a new `PathAbs`. The file must exist or `io::Error` will be returned.
    ///
    /// # Examples
    /// ```rust
    /// extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    /// let lib = PathAbs::new("src/lib.rs").unwrap();
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        Ok(PathAbs(path.as_ref().canonicalize()?))
    }

    /// Instantiate a new `PathAbs` to a file, creating it first if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example.txt";
    ///
    /// # let _ = ::std::fs::remove_file(example);
    ///
    /// let path = PathAbs::create_file(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathAbs::create_file(example).unwrap();
    /// # }
    /// ```
    pub fn create_file<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)?;
        PathAbs::new(path)
    }

    /// Instantiate a new `PathAbs` to a directory, creating it first if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example";
    ///
    /// # let _ = ::std::fs::remove_dir(example);
    ///
    /// let path = PathAbs::create_dir(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathAbs::create_dir(example).unwrap();
    /// # }
    /// ```
    pub fn create_dir<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        if let Err(err) = fs::create_dir(&path) {
            match err.kind() {
                io::ErrorKind::AlreadyExists => {},
                _ => return Err(err),
            }
        }
        PathAbs::new(path)
    }

    /// Instantiate a new `PathAbs` to a directory, recursively recreating it and all of its parent
    /// components if they are missing.
    ///
    /// # Examples
    /// ```rust
    /// extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example/long/path";
    ///
    /// # let _ = ::std::fs::remove_dir_all("target/example");
    ///
    /// let path = PathAbs::create_dir_all(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathAbs::create_dir_all(example).unwrap();
    /// # }
    /// ```
    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        fs::create_dir_all(&path)?;
        PathAbs::new(path)
    }

    /// For constructing mocked paths during tests. This is effectively the same as a `PathBuf`.
    ///
    /// This is NOT checked for validity so the file may or may not actually exist.
    pub fn mocked<P: AsRef<Path>>(fake_path: P) -> PathAbs {
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

impl Serialize for PathAbs {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> Deserialize<'de> for PathAbs {
    fn deserialize<D>(deserializer: D) -> result::Result<PathAbs, D::Error>
    where
        D: Deserializer<'de>,
    {
        let p = String::deserialize(deserializer)?;
        let p = stfu8::decode(&s)
            .map_err(serde::de::Error::custom)?;
        PathAbs::new(&p).map_err(serde::de::Error::custom)
    }
}
