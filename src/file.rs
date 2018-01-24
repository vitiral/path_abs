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

use super::{PathAbs, PathOpen};

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// a `PathAbs` that is guaranteed to be a file, with associated methods.
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
    /// > This does not call [`Path::cannonicalize()`][1], instead trusting that the input
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

    /// Instantiate a new `PathFile`, creating it first if it doesn't exist.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example.txt";
    ///
    /// # let _ = ::std::fs::remove_file(example);
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
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example.txt";
    /// # let _ = ::std::fs::remove_file(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected).unwrap();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn read_string(&self) -> io::Result<String> {
        let mut f = fs::OpenOptions::new()
            .read(true)
            .open(self)
            .map_err(|err| {
                io::Error::new(
                    err.kind(),
                    format!("{} when opening {}", err, self.display()),
                )
            })?;
        let meta = f.metadata().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when getting metadata for {}", err, self.display()),
            )
        })?;

        let mut out = String::with_capacity(meta.len() as usize);
        f.read_to_string(&mut out).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when reading {}", err, self.display()),
            )
        })?;
        Ok(out)
    }

    /// Write the `str` to a file, truncating it first if it exists and creating it otherwise.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    ///
    /// let example = "target/example.txt";
    /// # let _ = ::std::fs::remove_file(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar";
    /// file.write_str(expected).unwrap();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn write_str(&self, s: &str) -> io::Result<()> {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self)
            .map_err(|err| {
                io::Error::new(
                    err.kind(),
                    format!("{} when opening {}", err, self.display()),
                )
            })?;
        if s.is_empty() {
            return Ok(());
        }
        f.write_all(s.as_bytes()).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when writing to {}", err, self.display()),
            )
        })?;
        f.flush()
    }

    /// Append the `str` to a file, creating it if it doesn't exist.
    ///
    /// If the `str` is empty, this is equivalent to the unix `touch`
    /// except it does NOT update the timestamp
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathFile;
    ///
    /// # fn main() {
    /// let example = "target/example.txt";
    /// # let _ = ::std::fs::remove_file(example);
    /// let file = PathFile::create(example).unwrap();
    ///
    /// let expected = "foo\nbar\nbaz";
    /// file.append_str("foo\nbar").unwrap();
    /// file.append_str("\nbaz").unwrap();
    /// assert_eq!(expected, file.read_string().unwrap());
    /// # }
    /// ```
    pub fn append_str(&self, s: &str) -> io::Result<()> {
        let mut f = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self)
            .map_err(|err| {
                io::Error::new(
                    err.kind(),
                    format!("{} when opening {}", err, self.display()),
                )
            })?;
        if s.is_empty() {
            return Ok(());
        }
        f.write_all(s.as_bytes()).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when appending to {}", err, self.display()),
            )
        })?;
        f.flush()
    }

    /// Open the file with the specified options.
    pub fn open(&self, options: fs::OpenOptions) -> io::Result<PathOpen> {
        PathOpen::open_file(self.clone(), options)
    }

    /// Open the file as read-only.
    pub fn read(&self) -> io::Result<PathOpen> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        PathOpen::open_file(self.clone(), options)
    }

    /// Open the file as writeable. Note that this does NOT truncate the file
    /// OR create it if it doesn't exist.
    pub fn edit(&self) -> io::Result<PathOpen> {
        let mut options = fs::OpenOptions::new();
        options.write(true);
        PathOpen::open_file(self.clone(), options)
    }

    /// Open the file in append mode.
    pub fn append(&self) -> io::Result<PathOpen> {
        let mut options = fs::OpenOptions::new();
        options.append(true);
        PathOpen::open_file(self.clone(), options)
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
