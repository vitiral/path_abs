/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Open file paths and associated methods.

use std::fs;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::ops::Deref;
use std::convert::AsRef;

use super::{PathFile, PathType};

/// A `PathOpen` with an open file handle attached to it.
///
/// Unlike other types in this crate, this type defines `AsRef<File>` and
/// `Deref<File>`, meaning it _looks_ like an open file. You have to use
/// the `path()` method to get access to the associated `PathFile` struct.
///
/// > This type is not serializable.
pub struct PathOpen {
    pub(crate) path: PathFile,
    pub(crate) file: fs::File,
}

impl PathOpen {
    /// Open the file with the given `OpenOptions`.
    pub fn open<P: AsRef<Path>>(path: P, options: fs::OpenOptions) -> io::Result<PathOpen> {
        let file = options.open(&path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when opening {}", err, path.as_ref().display()),
            )
        })?;

        let path = PathFile::new(path)?;
        Ok(PathOpen {
            path: path,
            file: file,
        })
    }

    /// Shortcut to open the file if the path is already canonicalized.
    ///
    /// Typically you should use `File::open` instead (i.e. `file.open(options)` or
    /// `file.read()`).
    pub fn open_file(path_file: PathFile, options: fs::OpenOptions) -> io::Result<PathOpen> {
        let file = options.open(&path_file).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when opening {}", err, path_file.display()),
            )
        })?;

        Ok(PathOpen {
            path: path_file,
            file: file,
        })
    }

    /// Attempts to open a file in read-only mode.
    pub fn read<P: AsRef<Path>>(path: P) -> io::Result<PathOpen> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        PathOpen::open(path, options)
    }

    /// Open the file in write-only mode, truncating it first if it exists and creating it
    /// otherwise.
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<PathOpen> {
        let mut options = fs::OpenOptions::new();
        options.write(true);
        options.truncate(true);
        options.create(true);
        PathOpen::open(path, options)
    }

    /// Open the file for appending, creating it if it doesn't exist.
    pub fn append<P: AsRef<Path>>(path: P) -> io::Result<PathOpen> {
        let mut options = fs::OpenOptions::new();
        options.append(true);
        options.create(true);
        PathOpen::open(path, options)
    }

    /// Get the path associated with the open file.
    pub fn path(&self) -> &PathFile {
        &self.path
    }
}

impl fmt::Debug for PathOpen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Open(")?;
        self.path.fmt(f)?;
        write!(f, ")")
    }
}

impl AsRef<fs::File> for PathOpen {
    fn as_ref(&self) -> &fs::File {
        &self.file
    }
}

impl Deref for PathOpen {
    type Target = fs::File;

    fn deref(&self) -> &fs::File {
        &self.file
    }
}

impl io::Write for PathOpen {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when writing to {}", err, self.path().display()),
            )
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when flushing {}", err, self.path().display()),
            )
        })
    }
}

impl io::Read for PathOpen {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when reading {}", err, self.path().display()),
            )
        })
    }
}
