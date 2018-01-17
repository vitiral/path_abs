/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use std::fs;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::ops::Deref;
use std::convert::AsRef;

use super::{PathAbs, PathType};

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// A `PathAbs` that is guaranteed to be a directory, with associated methods.
pub struct PathDir(pub(crate) PathAbs);

impl PathDir {
    /// Instantiate a new `PathDir`. The directory must exist or `io::Error` will be returned.
    ///
    /// Returns `io::ErrorKind::InvalidInput` if the path exists but is not a dir.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathDir;
    ///
    /// # fn main() {
    /// let src = PathDir::new("src").unwrap();
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathDir> {
        let abs = PathAbs::new(path)?;
        PathDir::from_abs(abs)
    }

    /// Consume the `PathAbs` validating that the path is a directory and returning `PathDir`. The
    /// directory must exist or `io::Error` will be returned.
    ///
    /// If the path is actually a file returns `io::ErrorKind::InvalidInput`.
    ///
    /// > This does not call [`Path::cannonicalize()`][1], instead trusting that the input
    /// > already a fully qualified path.
    ///
    /// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathAbs, PathDir};
    ///
    /// # fn main() {
    /// let src_abs = PathAbs::new("src").unwrap();
    /// let src_dir = PathDir::from_abs(src_abs).unwrap();
    /// # }
    /// ```
    pub fn from_abs(abs: PathAbs) -> io::Result<PathDir> {
        if abs.is_dir() {
            Ok(PathDir(abs))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is not a dir",
            ))
        }
    }

    /// Instantiate a new `PathDir` to a directory, creating it first if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathDir;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example";
    ///
    /// # let _ = ::std::fs::remove_dir_all(example);
    ///
    /// let dir = PathDir::create(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathDir::create(example).unwrap();
    /// # }
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<PathDir> {
        if let Err(err) = fs::create_dir(&path) {
            match err.kind() {
                io::ErrorKind::AlreadyExists => {}
                _ => return Err(err),
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
    /// use path_abs::PathDir;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example/long/path";
    ///
    /// # let _ = ::std::fs::remove_dir_all("target/example");
    ///
    /// let path = PathDir::create_all(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathDir::create_all(example).unwrap();
    /// # }
    /// ```
    pub fn create_all<P: AsRef<Path>>(path: P) -> io::Result<PathDir> {
        fs::create_dir_all(&path)?;
        PathDir::new(path)
    }

    /// Join a path onto the `PathDir`, expecting it to exist. Returns the resulting `PathType`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathDir, PathFile};
    ///
    /// # fn main() {
    /// let src = PathDir::new("src").unwrap();
    /// let lib = src.join_abs("lib.rs").unwrap().unwrap_file();
    /// assert!(lib.is_file());
    /// # }
    /// ```
    pub fn join_abs<P: AsRef<Path>>(&self, path: P) -> io::Result<PathType> {
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
    /// use std::collections::HashSet;
    /// use path_abs::{PathDir, PathFile, PathType};
    ///
    /// # fn main() {
    ///
    /// # let _ = ::std::fs::remove_dir_all("target/example");
    ///
    /// let example_dir = PathDir::create("target/example").unwrap();
    /// let foo_dir = PathDir::create("target/example/foo").unwrap();
    /// let bar_file = PathFile::create("target/example/bar.txt").unwrap();
    ///
    /// let mut result = HashSet::new();
    /// for p in example_dir.list().unwrap() {
    ///     result.insert(p.unwrap());
    /// }
    ///
    /// let mut expected = HashSet::new();
    /// expected.insert(PathType::Dir(foo_dir));
    /// expected.insert(PathType::File(bar_file));
    ///
    /// assert_eq!(expected, result);
    /// # }
    pub fn list(&self) -> io::Result<ListDir> {
        Ok(ListDir {
            fsread: fs::read_dir(self)?,
        })
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
    fsread: fs::ReadDir,
}

impl ::std::iter::Iterator for ListDir {
    type Item = io::Result<PathType>;
    fn next(&mut self) -> Option<io::Result<PathType>> {
        let entry = match self.fsread.next() {
            Some(r) => match r {
                Ok(e) => e,
                Err(err) => return Some(Err(err)),
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

#[cfg(test)]
mod tests {
    use tempdir::TempDir;
    use std::collections::HashSet;
    use super::super::{PathDir, PathFile, PathType};

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
        expected.insert(PathType::Dir(foo_dir));
        expected.insert(PathType::File(bar_file));

        assert_eq!(expected, result);
    }
}
