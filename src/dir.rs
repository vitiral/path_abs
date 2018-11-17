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
use super::{PathAbs, PathType, PathInfo, PathOps};

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

    /// Returns the current working directory from the `env` as a `PathDir`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathDir;
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let cwd = PathDir::current_dir()?;
    /// # let env_cwd = ::std::fs::canonicalize(::std::env::current_dir()?)?;
    /// # let cwd_ref: &::std::path::PathBuf = cwd.as_ref();
    /// # assert_eq!(cwd_ref, &env_cwd);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn current_dir() -> Result<PathDir> {
        let dir = ::std::env::current_dir()
            .map_err(|err| Error::new(
                err,
                "getting current_dir",
                Path::new("$CWD").to_path_buf().into(),
            ))?;
        PathDir::new(dir)
    }

    /// Consume the `PathAbs` validating that the path is a directory and returning `PathDir`. The
    /// directory must exist or `io::Error` will be returned.
    ///
    /// If the path is actually a file returns `io::ErrorKind::InvalidInput`.
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
            Ok(PathDir::from_abs_unchecked(abs))
        } else {
            Err(Error::new(
                io::Error::new(io::ErrorKind::InvalidInput, "path is not a dir"),
                "resolving",
                abs.into(),
            ))
        }
    }

    #[inline(always)]
    /// Do the conversion _without checking_.
    ///
    /// This is typically used by external libraries when the type is already known
    /// through some other means (to avoid a syscall).
    pub fn from_abs_unchecked(abs: PathAbs) -> PathDir {
        PathDir(abs)
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
                _ => return Err(Error::new(
                    err,
                    "creating",
                    path.as_ref().to_path_buf().into(),
                )),
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
            .map_err(|err| Error::new(
                err,
                "creating-all",
                path.as_ref().to_path_buf().into(),
            ))?;
        PathDir::new(path)
    }

    /// Join a path onto the `PathDir`, expecting it to exist. Returns the resulting `PathType`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathDir, PathFile, PathInfo};
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let src = PathDir::new("src")?;
    /// let lib = src.join_abs("lib.rs")?.unwrap_file();
    /// assert!(lib.is_file());
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn join_abs<P: AsRef<Path>>(&self, path: P) -> Result<PathType> {
        let joined = self.concat(path.as_ref())?;
        PathType::new(joined)
    }

    /// List the contents of the directory, returning an iterator of `PathType`s.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use std::collections::HashSet;
    /// use path_abs::{PathDir, PathFile, PathType, PathOps};
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join("example");
    ///
    /// let example_dir = PathDir::create(example)?;
    /// let foo_dir = PathDir::create(example_dir.concat("foo")?)?;
    /// let bar_file = PathFile::create(example_dir.concat("bar.txt")?)?;
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

    /// Creates a new symbolic link on the filesystem to the dst.
    ///
    /// This handles platform specific behavior correctly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::{PathDir, PathFile, PathOps};
    /// use std::path::Path;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example";
    /// let example_sym = "example_sym";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// # let example_sym = &tmp.path().join(example_sym);
    /// let dir = PathDir::create(example)?;
    /// let file = PathFile::create(dir.concat("example.txt")?)?;
    ///
    /// let dir_sym = dir.symlink(example_sym)?;
    ///
    /// // They have a different "absolute path"
    /// assert_ne!(dir, dir_sym);
    ///
    /// // But they can be canonicalized to the same file.
    /// let dir_can = dir_sym.canonicalize()?;
    /// assert_eq!(dir, dir_can);
    ///
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn symlink<P: AsRef<Path>>(&self, dst: P) -> Result<PathDir> {
        symlink_dir(&self, &dst).map_err(|err| {
            Error::new(
                err,
                &format!("linking from {} to", dst.as_ref().display()),
                self.clone().into(),
            )
        })?;
        PathDir::new(dst)
    }

    /// Return a reference to a basic `std::path::Path`
    pub fn as_path(&self) -> &Path {
        self.as_ref()
    }

    /// Returns the canonical form of the path with all intermediate components normalized and
    /// symbolic links resolved.
    ///
    /// See [`PathAbs::canonicalize`]
    ///
    /// [`PathAbs::canonicalize`]: struct.PathAbs.html#method.canonicalize
    pub fn canonicalize(&self) -> Result<PathDir> {
        Ok(PathDir(self.0.canonicalize()?))
    }

    /// Get the parent directory of this directory as a `PathDir`.
    ///
    /// > This does not make aditional syscalls, as the parent by definition must be a directory
    /// > and exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathDir;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let src = PathDir::new("src")?;
    /// let proj = src.parent_dir().unwrap();
    /// assert_eq!(PathDir::new("src/..")?, proj);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn parent_dir(&self) -> Option<PathDir> {
        match self.parent() {
            Ok(path) => Some(PathDir(PathAbs(Arc::new(path.to_path_buf())))),
            Err(_) => None,
        }
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

impl Borrow<PathAbs> for PathDir {
    fn borrow(&self) -> &PathAbs {
        self.as_ref()
    }
}

impl Borrow<Path> for PathDir {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl Borrow<PathBuf> for PathDir {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl<'a> Borrow<PathAbs> for &'a PathDir {
    fn borrow(&self) -> &PathAbs {
        self.as_ref()
    }
}

impl<'a> Borrow<Path> for &'a PathDir {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl<'a> Borrow<PathBuf> for &'a PathDir {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl Deref for PathDir {
    type Target = PathAbs;

    fn deref(&self) -> &PathAbs {
        &self.0
    }
}

impl From<PathDir> for PathAbs {
    fn from(path: PathDir) -> PathAbs {
        path.0
    }
}

impl From<PathDir> for Arc<PathBuf> {
    fn from(path: PathDir) -> Arc<PathBuf> {
        let abs: PathAbs = path.into();
        abs.into()
    }
}

impl From<PathDir> for PathBuf {
    fn from(path: PathDir) -> PathBuf {
        let abs: PathAbs = path.into();
        abs.into()
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;
    use std::collections::HashSet;
    use super::super::{PathAbs, PathDir, PathFile, PathType, PathOps};

    #[test]
    fn sanity_list() {
        let tmp_dir = TempDir::new("example").expect("create temp dir");
        let tmp_abs = PathDir::new(tmp_dir.path()).unwrap();

        let foo_path = tmp_abs.concat("foo").expect("path foo");
        let foo_dir = PathDir::create(foo_path).unwrap();

        let bar_path = tmp_abs.concat("bar").expect("path bar");
        let bar_file = PathFile::create(bar_path).unwrap();

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

#[cfg(unix)]
fn symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    ::std::os::unix::fs::symlink(src, dst)
}

#[cfg(windows)]
fn symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    ::std::os::windows::fs::symlink_dir(src, dst)
}
