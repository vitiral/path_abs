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
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::ops::Deref;
use std::convert::AsRef;

use super::{FileEdit, FileRead, FileWrite, PathAbs};

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
    /// # fn main() {
    /// let lib = PathFile::new("src/lib.rs").unwrap();
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathFile> {
        let abs = PathAbs::new(path)?;
        PathFile::from_abs(abs)
    }

    /// Consume the `PathAbs` validating that the path is a file and returning `PathFile`. The file
    /// must exist or `io::Error` will be returned.
    ///
    /// If the path is actually a dir returns `io::ErrorKind::InvalidInput`.
    ///
    /// > This does not call [`Path::cannonicalize()`][1], instead trusting that the input is
    /// > already a fully qualified path.
    ///
    /// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::{PathAbs, PathFile};
    ///
    /// # fn main() {
    /// let lib_abs = PathAbs::new("src/lib.rs").unwrap();
    /// let lib_file = PathFile::from_abs(lib_abs).unwrap();
    /// # }
    /// ```
    pub fn from_abs(abs: PathAbs) -> io::Result<PathFile> {
        if abs.is_file() {
            Ok(PathFile(abs))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a file", abs.display()),
            ))
        }
    }

    /// Instantiate a new `PathFile`, creating an empty file if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex").unwrap();
    /// # let example = &tmp.path().join(example);
    ///
    /// # let tmp = tempdir::TempDir::new("ex").unwrap();
    /// # let example = &tmp.path().join(example);
    ///
    /// let file = PathFile::create(example).unwrap();
    ///
    /// // It can be done twice with no effect.
    /// let _ = PathFile::create(example).unwrap();
    /// # }
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<PathFile> {
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|err| {
                io::Error::new(
                    err.kind(),
                    format!("{} when opening {}", err, path.as_ref().display()),
                )
            })?;
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
    /// # fn main() {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex").unwrap();
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected).unwrap();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn read_string(&self) -> io::Result<String> {
        let mut f = self.read()?;
        let mut out = {
            let meta = f.metadata()?;
            String::with_capacity(meta.len() as usize)
        };
        f.read_to_string(&mut out)?;
        Ok(out)
    }

    /// Write the `str` to a file, truncating it first if it exists and creating it otherwise.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex").unwrap();
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected).unwrap();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn write_str(&self, s: &str) -> io::Result<()> {
        let mut options = fs::OpenOptions::new();
        options.create(true);
        options.truncate(true);
        let mut f = FileWrite::open_path(self.clone(), options)?;
        if s.is_empty() {
            return Ok(());
        }
        f.write_all(s.as_bytes())?;
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
    /// # fn main() {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex").unwrap();
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar\nbaz";
    /// file.append_str("foo\nbar").unwrap();
    /// file.append_str("\nbaz").unwrap();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn append_str(&self, s: &str) -> io::Result<()> {
        let mut f = self.append()?;
        if s.is_empty() {
            return Ok(());
        }
        f.write_all(s.as_bytes())?;
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
    /// # fn main() {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex").unwrap();
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected).unwrap();
    ///
    /// let mut read = file.read().unwrap();
    /// let mut s = String::new();
    /// read.read_to_string(&mut s).unwrap();
    /// assert_eq!(expected, s);
    /// # }
    /// ```
    pub fn read(&self) -> io::Result<FileRead> {
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
    /// # fn main() {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex").unwrap();
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar\n";
    /// file.write_str("foo\n").unwrap();
    ///
    /// let mut append = file.append().unwrap();
    /// append.write_all(b"bar\n").unwrap();
    /// append.flush();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn append(&self) -> io::Result<FileWrite> {
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
    /// # fn main() {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex").unwrap();
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar";
    ///
    /// let mut edit = file.edit().unwrap();
    /// let mut s = String::new();
    ///
    /// edit.write_all(expected.as_bytes()).unwrap();
    /// edit.seek(SeekFrom::Start(0)).unwrap();
    /// edit.read_to_string(&mut s).unwrap();
    /// assert_eq!(expected, s);
    /// # }
    /// ```
    pub fn edit(&self) -> io::Result<FileEdit> {
        FileEdit::open_path(self.clone(), fs::OpenOptions::new())
    }

    /// Remove (delete) the file from the filesystem, consuming self.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// # extern crate tempdir;
    /// use path_abs::PathFile;
    /// use std::path::Path;
    ///
    /// # fn main() {
    /// let example = "example.txt";
    /// # let tmp = tempdir::TempDir::new("ex").unwrap();
    /// # let example = &tmp.path().join(example);
    /// let file = PathFile::create(example).unwrap();
    /// assert!(file.exists());
    /// file.remove().unwrap();
    ///
    /// // file.exists() <--- COMPILER ERROR, `file` was consumed
    ///
    /// assert!(!Path::new(example).exists());
    /// # }
    /// ```
    pub fn remove(self) -> io::Result<()> {
        fs::remove_file(&self).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when removing {}", err, self.display()),
            )
        })
    }

    /// Create a mock file type. *For use in tests only*.
    ///
    /// See the docs for [`PathAbs::mock`](struct.PathAbs.html#method.mock)
    pub fn mock<P: AsRef<Path>>(path: P) -> PathFile {
        PathFile(PathAbs::mock(path))
    }
}

impl fmt::Debug for PathFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl Deref for PathFile {
    type Target = PathAbs;

    fn deref(&self) -> &PathAbs {
        &self.0
    }
}
