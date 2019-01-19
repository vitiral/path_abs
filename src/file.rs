/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use std::ffi;
use std::fmt;
use std::fs;
use std::io;
use std_prelude::*;

use super::{Error, Result};
use super::{FileEdit, FileRead, FileWrite, PathAbs, PathDir, PathInfo, PathOps};

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// a `PathAbs` that was a file at the time of initialization, with associated methods.
pub struct PathFile(pub(crate) PathAbs);

impl PathFile {
    /// Instantiate a new `PathFile`. The file must exist or `io::Error` will be returned.
    ///
    /// Returns `io::ErrorKind::InvalidInput` if the path exists but is not a file.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathFile;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let lib = PathFile::new("src/lib.rs")?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<PathFile> {
        let abs = PathAbs::new(path)?;
        PathFile::from_abs(abs)
    }

    /// Get the parent directory of this file as a `PathDir`.
    ///
    /// > This does not make aditional syscalls, as the parent by definition must be a directory
    /// > and exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathDir, PathFile};
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let lib = PathFile::new("src/lib.rs")?;
    /// let src = lib.parent_dir().unwrap();
    /// assert_eq!(PathDir::new("src")?, src);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn parent_dir(&self) -> Option<PathDir> {
        match self.parent() {
            Ok(path) => Some(PathDir(PathAbs(Arc::new(path.to_path_buf())))),
            Err(_) => None,
        }
    }

    /// Consume the `PathAbs` validating that the path is a file and returning `PathFile`. The file
    /// must exist or `io::Error` will be returned.
    ///
    /// If the path is actually a dir returns `io::ErrorKind::InvalidInput`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathAbs, PathFile};
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let lib_abs = PathAbs::new("src/lib.rs")?;
    /// let lib_file = PathFile::from_abs(lib_abs)?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn from_abs(abs: PathAbs) -> Result<PathFile> {
        if abs.is_file() {
            Ok(PathFile::from_abs_unchecked(abs))
        } else {
            Err(Error::new(
                io::Error::new(io::ErrorKind::InvalidInput, "path is not a file"),
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
    pub fn from_abs_unchecked(abs: PathAbs) -> PathFile {
        PathFile(abs)
    }

    /// Instantiate a new `PathFile`, creating an empty file if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathFile;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    ///
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    ///
    /// let file = PathFile::create(example)?;
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathFile::create(example)?;
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> Result<PathFile> {
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|err| Error::new(err, "opening", path.as_ref().to_path_buf().into()))?;
        PathFile::new(path)
    }

    /// Read the entire contents of the file into a `String`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathFile;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example)?;
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected)?;
    /// assert_eq!(expected, file.read_string()?);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn read_string(&self) -> Result<String> {
        let mut f = self.read()?;
        f.read_string()
    }

    /// Write the `str` to a file, truncating it first if it exists and creating it otherwise.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathFile;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example)?;
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected)?;
    /// assert_eq!(expected, file.read_string()?);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn write_str(&self, s: &str) -> Result<()> {
        let mut options = fs::OpenOptions::new();
        options.create(true);
        options.truncate(true);
        let mut f = FileWrite::open_path(self.clone(), options)?;
        if s.is_empty() {
            return Ok(());
        }
        f.write_str(s)?;
        f.flush()
    }

    /// Append the `str` to a file, creating it if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathFile;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example)?;
    ///
    /// let expected = "foo\nbar\nbaz";
    /// file.append_str("foo\nbar")?;
    /// file.append_str("\nbaz")?;
    /// assert_eq!(expected, file.read_string()?);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn append_str(&self, s: &str) -> Result<()> {
        let mut f = self.append()?;
        if s.is_empty() {
            return Ok(());
        }
        f.write_str(s)?;
        f.flush()
    }

    /// Open the file as read-only.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use std::io::Read;
    /// use path_abs::PathFile;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example)?;
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected)?;
    ///
    /// let mut read = file.read()?;
    /// let mut s = String::new();
    /// read.read_to_string(&mut s)?;
    /// assert_eq!(expected, s);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn read(&self) -> Result<FileRead> {
        FileRead::read_path(self.clone())
    }

    /// Open the file as write-only in append mode.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use std::io::Write;
    /// use path_abs::PathFile;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example)?;
    ///
    /// let expected = "foo\nbar\n";
    /// file.write_str("foo\n")?;
    ///
    /// let mut append = file.append()?;
    /// append.write_all(b"bar\n")?;
    /// append.flush();
    /// assert_eq!(expected, file.read_string()?);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn append(&self) -> Result<FileWrite> {
        let mut options = fs::OpenOptions::new();
        options.append(true);
        FileWrite::open_path(self.clone(), options)
    }

    /// Open the file for editing (reading and writing).
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use std::io::{Read, Seek, Write, SeekFrom};
    /// use path_abs::PathFile;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example)?;
    ///
    /// let expected = "foo\nbar";
    ///
    /// let mut edit = file.edit()?;
    /// let mut s = String::new();
    ///
    /// edit.write_all(expected.as_bytes())?;
    /// edit.seek(SeekFrom::Start(0))?;
    /// edit.read_to_string(&mut s)?;
    /// assert_eq!(expected, s);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn edit(&self) -> Result<FileEdit> {
        FileEdit::open_path(self.clone(), fs::OpenOptions::new())
    }

    /// Copy the file to another location, including permission bits
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathFile;
    /// use std::path::Path;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// let example_bk = "example.txt.bk";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// # let example_bk = &tmp.path().join(example_bk);
    /// let file = PathFile::create(example)?;
    ///
    /// let contents = "This is some contents";
    /// file.write_str(contents);
    /// let file_bk = file.copy(example_bk)?;
    /// assert_eq!(contents, file.read_string()?);
    /// assert_eq!(contents, file_bk.read_string()?);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn copy<P: AsRef<Path>>(&self, path: P) -> Result<PathFile> {
        fs::copy(&self, &path).map_err(|err| {
            Error::new(
                err,
                &format!("copying {} from", path.as_ref().display()),
                self.clone().into(),
            )
        })?;
        Ok(PathFile::new(path)?)
    }

    /// Rename a file, replacing the original file if `to` already exists.
    ///
    /// This will not work if the new name is on a different mount point.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::{PathFile, PathInfo};
    /// use std::path::Path;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// let example_bk = "example.txt.bk";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// # let example_bk = &tmp.path().join(example_bk);
    /// let file = PathFile::create(example)?;
    ///
    /// let contents = "This is some contents";
    /// file.write_str(contents);
    /// let file_bk = file.clone().rename(example_bk)?;
    /// assert!(!file.exists());
    /// assert_eq!(contents, file_bk.read_string()?);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn rename<P: AsRef<Path>>(self, to: P) -> Result<PathFile> {
        fs::rename(&self, &to).map_err(|err| {
            Error::new(
                err,
                &format!("renaming to {} from", to.as_ref().display()),
                self.clone().into(),
            )
        })?;
        Ok(PathFile::new(to)?)
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
    /// use path_abs::PathFile;
    /// use std::path::Path;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// let example_sym = "example.txt.sym";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// # let example_sym = &tmp.path().join(example_sym);
    /// let file = PathFile::create(example)?;
    ///
    /// let contents = "This is some contents";
    /// file.write_str(contents);
    /// let file_sym = file.symlink(example_sym)?;
    ///
    /// // They have a different "absolute path"
    /// assert_ne!(file, file_sym);
    ///
    /// // But they can be canonicalized to the same file.
    /// let file_can = file_sym.canonicalize()?;
    /// assert_eq!(file, file_can);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn symlink<P: AsRef<Path>>(&self, dst: P) -> Result<PathFile> {
        symlink_file(&self, &dst).map_err(|err| {
            Error::new(
                err,
                &format!("linking from {} to", dst.as_ref().display()),
                self.clone().into(),
            )
        })?;
        PathFile::new(dst)
    }

    /// Remove (delete) the file from the filesystem, consuming self.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::{PathFile, PathInfo};
    /// use std::path::Path;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex")?;
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example)?;
    /// assert!(file.exists());
    /// file.remove()?;
    ///
    /// // file.exists() <--- COMPILER ERROR, `file` was consumed
    ///
    /// assert!(!Path::new(example).exists());
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn remove(self) -> Result<()> {
        fs::remove_file(&self).map_err(|err| Error::new(err, "removing", self.into()))
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
    pub fn canonicalize(&self) -> Result<PathFile> {
        Ok(PathFile(self.0.canonicalize()?))
    }

    /// Create a mock file type. *For use in tests only*.
    ///
    /// See the docs for [`PathAbs::mock`](struct.PathAbs.html#method.mock)
    pub fn mock<P: AsRef<Path>>(path: P) -> PathFile {
        PathFile(PathAbs::mock(path))
    }
}

impl fmt::Debug for PathFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<PathAbs> for PathFile {
    fn as_ref(&self) -> &PathAbs {
        &self.0
    }
}

impl AsRef<Path> for PathFile {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<PathBuf> for PathFile {
    fn as_ref(&self) -> &PathBuf {
        self.0.as_ref()
    }
}

impl Borrow<PathAbs> for PathFile {
    fn borrow(&self) -> &PathAbs {
        self.as_ref()
    }
}

impl Borrow<Path> for PathFile {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl Borrow<PathBuf> for PathFile {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl<'a> Borrow<PathAbs> for &'a PathFile {
    fn borrow(&self) -> &PathAbs {
        self.as_ref()
    }
}

impl<'a> Borrow<Path> for &'a PathFile {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl<'a> Borrow<PathBuf> for &'a PathFile {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl Deref for PathFile {
    type Target = PathAbs;

    fn deref(&self) -> &PathAbs {
        &self.0
    }
}

impl From<PathFile> for PathAbs {
    fn from(path: PathFile) -> PathAbs {
        path.0
    }
}

impl From<PathFile> for Arc<PathBuf> {
    fn from(path: PathFile) -> Arc<PathBuf> {
        let abs: PathAbs = path.into();
        abs.into()
    }
}

impl From<PathFile> for PathBuf {
    fn from(path: PathFile) -> PathBuf {
        let abs: PathAbs = path.into();
        abs.into()
    }
}

impl PathOps for PathFile {
    type Output = PathAbs;

    fn concat<P: AsRef<Path>>(&self, path: P) -> Result<Self::Output> {
        Ok(self.0.concat(path)?)
    }

    fn with_file_name<S: AsRef<ffi::OsStr>>(&self, file_name: S) -> Self::Output {
        self.0.with_file_name(file_name)
    }

    fn with_extension<S: AsRef<ffi::OsStr>>(&self, extension: S) -> Self::Output {
        self.0.with_extension(extension)
    }
}

#[cfg(unix)]
fn symlink_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    ::std::os::unix::fs::symlink(src, dst)
}

#[cfg(windows)]
fn symlink_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    ::std::os::windows::fs::symlink_file(src, dst)
}
