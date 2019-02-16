/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Open file paths that are read-only.

use std::fmt;
use std::fs;
use std::io;
use std_prelude::*;

use super::open::FileOpen;
use super::{Error, PathAbs, PathFile, PathInfo, Result};

/// A read-only file handle with `path()` attached and improved error messages. Contains only the
/// methods and trait implementations which are allowed by a read-only file.
///
/// # Examples
/// ```rust
/// # extern crate path_abs;
/// # extern crate tempdir;
/// use std::io::Read;
/// use path_abs::{PathFile, FileRead};
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
/// let mut read = FileRead::open(example)?;
/// let mut s = String::new();
/// read.read_to_string(&mut s)?;
/// assert_eq!(expected, s);
/// # Ok(()) } fn main() { try_main().unwrap() }
/// ```
pub struct FileRead(pub(crate) FileOpen);

impl FileRead {
    /// Open the file as read-only.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<FileRead> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        Ok(FileRead(FileOpen::open(path, options)?))
    }

    /// Shortcut to open the file if the path is already absolute.
    pub(crate) fn open_abs<P: Into<PathAbs>>(path: P) -> Result<FileRead> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        Ok(FileRead(FileOpen::open_abs(path, options)?))
    }

    pub fn path(&self) -> &PathFile {
        &self.0.path
    }

    /// Read what remains of the file to a `String`.
    pub fn read_string(&mut self) -> Result<String> {
        let mut s = String::new();
        self.0
            .file
            .read_to_string(&mut s)
            .map_err(|err| Error::new(err, "reading", self.0.path.clone().into()))?;
        Ok(s)
    }
}

impl fmt::Debug for FileRead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileRead(")?;
        self.0.path.fmt(f)?;
        write!(f, ")")
    }
}

impl io::Read for FileRead {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.file.read(buf).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when reading {}", err, self.path().display()),
            )
        })
    }
}

impl io::Seek for FileRead {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.0.file.seek(pos).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} seeking {}", err, self.path().display()),
            )
        })
    }
}

impl AsRef<FileOpen> for FileRead {
    fn as_ref(&self) -> &FileOpen {
        &self.0
    }
}

impl AsRef<File> for FileRead {
    fn as_ref(&self) -> &File {
        self.0.as_ref()
    }
}

impl Borrow<FileOpen> for FileRead {
    fn borrow(&self) -> &FileOpen {
        &self.0
    }
}

impl Borrow<File> for FileRead {
    fn borrow(&self) -> &File {
        self.0.borrow()
    }
}

impl<'a> Borrow<FileOpen> for &'a FileRead {
    fn borrow(&self) -> &FileOpen {
        &self.0
    }
}

impl<'a> Borrow<File> for &'a FileRead {
    fn borrow(&self) -> &File {
        self.0.borrow()
    }
}

impl From<FileRead> for FileOpen {
    fn from(orig: FileRead) -> FileOpen {
        orig.0
    }
}

impl From<FileRead> for File {
    fn from(orig: FileRead) -> File {
        orig.0.into()
    }
}
