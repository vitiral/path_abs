/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Open file paths that are read-only.

use std::fs;
use std::fmt;
use std::io;
use std_prelude::*;

use super::{Error, Result};
use super::PathFile;
use super::open::FileOpen;

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
/// let mut read = FileRead::read(example)?;
/// let mut s = String::new();
/// read.read_to_string(&mut s)?;
/// assert_eq!(expected, s);
/// # Ok(()) } fn main() { try_main().unwrap() }
/// ```
pub struct FileRead(pub(crate) FileOpen);

impl FileRead {
    /// Open the file as read-only.
    pub fn read<P: AsRef<Path>>(path: P) -> Result<FileRead> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        Ok(FileRead(FileOpen::open(path, options)?))
    }

    /// Shortcut to open the file if the path is already absolute.
    pub(crate) fn read_path(path: PathFile) -> Result<FileRead> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        Ok(FileRead(FileOpen::open_path(path, options)?))
    }

    /// Read what remains of the file to a `String`.
    pub fn read_string(&mut self) -> Result<String> {
        let mut s = String::new();
        self.0
            .file
            .read_to_string(&mut s)
            .map_err(|err| Error::new(err, "reading", self.path.clone().into()))?;
        Ok(s)
    }
}

impl fmt::Debug for FileRead {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FileRead(")?;
        self.path.fmt(f)?;
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

impl Deref for FileRead {
    type Target = FileOpen;

    fn deref(&self) -> &FileOpen {
        &self.0
    }
}
