/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Paths to Directories and associated methods.
use std::fs;
use std::fmt;
use std::io;
use std_prelude::*;

use super::{Error, Result};
use super::{PathAbs, PathArc, PathType};

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// A `PathAbs` that is guaranteed to be a directory, with associated methods.
pub struct PathDir(pub(crate) PathAbs);

impl PathDir {
    /// Instantiate a new `PathDir`. The directory must exist or `io::Error` will be returned.
    ///
    /// Returns `io::ErrorKind::InvalidInput` if the path exists but is not a directory.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathDir;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let src = PathDir::new("src")?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<PathDir> {
        let abs = PathAbs::new(path)?;
        PathDir::from_abs(abs)
    }

    /// Consume the `PathAbs` validating that the path is a directory and returning `PathDir`. The
    /// directory must exist or `io::Error` will be returned.
    ///
    /// If the path is actually a file returns `io::ErrorKind::InvalidInput`.
    ///
    /// > This does not call [`Path::cannonicalize()`][1], instead trusting that the input is
    /// > already a fully qualified path.
    ///
    /// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathAbs, PathDir};
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let src_abs = PathAbs::new("src")?;
    /// let src_dir = PathDir::from_abs(src_abs)?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn from_abs(abs: PathAbs) -> Result<PathDir> {
        if abs.is_dir() {
            Ok(PathDir(abs))
        } else {
            Err(Error::new(
                io::Error::new(io::ErrorKind::InvalidInput, "path is not a dir"),
                "resolving",
                abs.into(),
            ))
        }
    }

    /// Instantiate a new `PathDir` to a directory, creating the directory if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathDir;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    ///
    /// let dir = PathDir::create(example)?;
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathDir::create(example)?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> Result<PathDir> {
        if let Err(err) = fs::create_dir(&path) {
            match err.kind() {
                io::ErrorKind::AlreadyExists => {}
                _ => return Err(Error::new(err, "creating", PathArc::new(path))),
            }
        }
        PathDir::new(path)
    }

    /// Instantiate a new `PathDir` to a directory, recursively recreating it and all of its parent
    /// components if they are missing.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathDir;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example/long/path";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    ///
    /// let path = PathDir::create_all(example)?;
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathDir::create_all(example)?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn create_all<P: AsRef<Path>>(path: P) -> Result<PathDir> {
        fs::create_dir_all(&path)
            .map_err(|err| Error::new(err, "creating-all", PathArc::new(&path)))?;
        PathDir::new(path)
    }

    /// Join a path onto the `PathDir`, expecting it to exist. Returns the resulting `PathType`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathDir, PathFile};
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let src = PathDir::new("src")?;
    /// let lib = src.join_abs("lib.rs")?.unwrap_file();
    /// assert!(lib.is_file());
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn join_abs<P: AsRef<Path>>(&self, path: P) -> Result<PathType> {
        let joined = self.join(path.as_ref());
        PathType::new(joined)
    }

    /// List the contents of the directory, returning an iterator of `PathType`s.
    ///
    /// > **Warning**: because `PathAbs` is the canonicalized path, symlinks are always resolved.
    /// > This means that if the directory contains a symlink you may get a path from a completely
    /// > _different directory_.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use std::collections::HashSet;
    /// use path_abs::{PathDir, PathFile, PathType};
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join("example");
    ///
    /// let example_dir = PathDir::create(example)?;
    /// let foo_dir = PathDir::create(example_dir.join("foo"))?;
    /// let bar_file = PathFile::create(example_dir.join("bar.txt"))?;
    ///
    /// let mut result = HashSet::new();
    /// for p in example_dir.list()? {
    ///     result.insert(p?);
    /// }
    ///
    /// let mut expected = HashSet::new();
    /// expected.insert(PathType::Dir(foo_dir));
    /// expected.insert(PathType::File(bar_file));
    ///
    /// assert_eq!(expected, result);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    pub fn list(&self) -> Result<ListDir> {
        let fsread =
            fs::read_dir(self).map_err(|err| Error::new(err, "reading dir", self.clone().into()))?;
        Ok(ListDir {
            dir: self.clone(),
            fsread: fsread,
        })
    }

    /// Remove (delete) the _empty_ directory from the filesystem, consuming self.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use std::path::Path;
    /// use path_abs::PathDir;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = Path::new("example/long/path");
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    ///
    /// let dir = PathDir::create_all(example)?;
    /// let parent = dir.parent_dir().unwrap();
    ///
    /// assert!(example.exists());
    /// dir.remove()?;
    /// // assert!(dir.exists());  <--- COMPILE ERROR
    /// assert!(!example.exists());
    /// parent.remove()?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn remove(self) -> Result<()> {
        fs::remove_dir(&self).map_err(|err| Error::new(err, "removing", self.into()))
    }

    /// Remove (delete) the directory, after recursively removing its contents. Use carefully!
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use std::path::Path;
    /// use path_abs::PathDir;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = Path::new("example/long/path");
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    ///
    /// let dir = PathDir::create_all(example)?;
    /// let parent = dir.parent_dir().unwrap();
    ///
    /// assert!(example.exists());
    /// parent.remove_all()?;
    /// assert!(!example.exists());
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn remove_all(self) -> Result<()> {
        fs::remove_dir_all(&self).map_err(|err| Error::new(err, "removing-all", self.into()))
    }

    /// Return a reference to a basic `std::path::Path`
    pub fn as_path(&self) -> &Path {
        self.as_ref()
    }

    /// Create a mock dir type. *For use in tests only*.
    ///
    /// See the docs for [`PathAbs::mock`](struct.PathAbs.html#method.mock)
    pub fn mock<P: AsRef<Path>>(path: P) -> PathDir {
        PathDir(PathAbs::mock(path))
    }
}

/// An iterator over `PathType` objects, returned by `PathDir::list`.
pub struct ListDir {
    // TODO: this should be a reference...?
    // Or is this a good excuse to use Arc under the hood everywhere?
    dir: PathDir,
    fsread: fs::ReadDir,
}

impl ::std::iter::Iterator for ListDir {
    type Item = Result<PathType>;
    fn next(&mut self) -> Option<Result<PathType>> {
        let entry = match self.fsread.next() {
            Some(r) => match r {
                Ok(e) => e,
                Err(err) => {
                    return Some(Err(Error::new(
                        err,
                        "iterating over",
                        self.dir.clone().into(),
                    )))
                }
            },
            None => return None,
        };
        Some(PathType::new(entry.path()))
    }
}

impl fmt::Debug for PathDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<PathAbs> for PathDir {
    fn as_ref(&self) -> &PathAbs {
        &self.0
    }
}

impl AsRef<Path> for PathDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<PathBuf> for PathDir {
    fn as_ref(&self) -> &PathBuf {
        self.0.as_ref()
    }
}

impl Deref for PathDir {
    type Target = PathAbs;

    fn deref(&self) -> &PathAbs {
        &self.0
    }
}

impl Into<PathAbs> for PathDir {
    /// Downgrades the `PathDir` into a `PathAbs`
    ///
    /// # Examples
    /// ```
    /// # extern crate path_abs;
    /// use std::path::PathBuf;
    /// use path_abs::{PathDir, PathAbs};
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let dir = PathDir::new("src")?;
    /// let abs: PathAbs = dir.into();
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    fn into(self) -> PathAbs {
        self.0
    }
}

impl Into<PathArc> for PathDir {
    /// Downgrades the `PathDir` into a `PathArc`
    fn into(self) -> PathArc {
        (self.0).0
    }
}

impl Into<PathBuf> for PathDir {
    /// Downgrades the `PathDir` into a `PathBuf`. Avoids a clone if this is the only reference.
    ///
    /// # Examples
    /// ```
    /// # extern crate path_abs;
    /// use path_abs::PathDir;
    /// use std::path::PathBuf;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let dir = PathDir::new("src")?;
    /// let buf: PathBuf = dir.into();
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    fn into(self) -> PathBuf {
        let arc: PathArc = self.into();
        arc.into()
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;
    use std::collections::HashSet;
    use super::super::{PathAbs, PathDir, PathFile, PathType};

    #[test]
    fn sanity_list() {
        let tmp_dir = TempDir::new("example").expect("create temp dir");
        let tmp_abs = PathDir::new(tmp_dir.path()).unwrap();

        let foo_dir = PathDir::create(tmp_abs.join("foo")).unwrap();
        let bar_file = PathFile::create(tmp_abs.join("bar.txt")).unwrap();

        let mut result = HashSet::new();
        for p in tmp_abs.list().unwrap() {
            result.insert(p.unwrap());
        }

        let mut expected = HashSet::new();
        expected.insert(PathType::Dir(foo_dir.clone()));
        expected.insert(PathType::File(bar_file.clone()));

        assert_eq!(expected, result);

        // just ensure that this compiles
        let _: PathAbs = foo_dir.into();
        let _: PathAbs = bar_file.into();
    }
}
