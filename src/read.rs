/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Open file paths that are read-only.

use std::convert::AsRef;
use std::fs;
use std::fmt;
use std::io;
use std::path::Path;
use std::ops::Deref;

use super::PathFile;
use super::open::FileOpen;

/// An open file read only file. Get the associated `PathFile` with with the `path()`
/// method.
///
/// > This type is not serializable.
pub struct FileRead(pub(crate) FileOpen);

impl FileRead {
    /// Open the file with the given `OpenOptions` but always sets `write` to true.
    pub fn read<P: AsRef<Path>>(path: P) -> io::Result<FileRead> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        Ok(FileRead(FileOpen::open(path, options)?))
    }

    /// Shortcut to open the file if the path is already canonicalized.
    pub(crate) fn read_path(path: PathFile) -> io::Result<FileRead> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        Ok(FileRead(FileOpen::open_file(path, options)?))
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
