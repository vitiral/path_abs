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
use std::path::Path;
use std::convert::AsRef;

use super::PathFile;

/// **INTERNAL TYPE: do not use directly.**
///
/// Use `FileRead`, `FileWrite` or `FileEdit` instead.
pub struct FileOpen {
    pub(crate) path: PathFile,
    pub(crate) file: fs::File,
}

impl FileOpen {
    /// Open the file with the given `OpenOptions`.
    pub fn open<P: AsRef<Path>>(path: P, options: fs::OpenOptions) -> io::Result<FileOpen> {
        let file = options.open(&path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when opening {}", err, path.as_ref().display()),
            )
        })?;

        let path = PathFile::new(path)?;
        Ok(FileOpen {
            path: path,
            file: file,
        })
    }

    /// Shortcut to open the file if the path is already canonicalized.
    ///
    /// Typically you should use `PathFile::open` instead (i.e. `file.open(options)` or
    /// `file.read()`).
    pub fn open_file(path_file: PathFile, options: fs::OpenOptions) -> io::Result<FileOpen> {
        let file = options.open(&path_file).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when opening {}", err, path_file.display()),
            )
        })?;

        Ok(FileOpen {
            path: path_file,
            file: file,
        })
    }

    /// Get the path associated with the open file.
    pub fn path(&self) -> &PathFile {
        &self.path
    }

    /// Queries metadata about the underlying file.
    ///
    /// This function is identical to [std::fs::File::metadata][0] except it has error
    /// messages which include the action and the path.
    ///
    /// [0]: https://doc.rust-lang.org/std/fs/struct.File.html#method.metadata
    pub fn metadata(&self) -> io::Result<fs::Metadata> {
        self.file.metadata().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when getting metadata for {}", err, self.path.display()),
            )
        })
    }

    /// Creates a new independently owned handle to the underlying file.
    ///
    /// This function is identical to [std::fs::File::try_clone][0] except it has error
    /// messages which include the action and the path and it returns a `FileOpen` object.
    ///
    /// [0]: https://doc.rust-lang.org/std/fs/struct.File.html#method.try_clone
    pub fn try_clone(&self) -> io::Result<FileOpen> {
        let file = self.file.try_clone().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when cloning {}", err, self.path.display()),
            )
        })?;
        Ok(FileOpen {
            file: file,
            path: self.path.clone(),
        })
    }
}

impl fmt::Debug for FileOpen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Open(")?;
        self.path.fmt(f)?;
        write!(f, ")")
    }
}

impl Into<fs::File> for FileOpen {
    fn into(self) -> fs::File {
        self.file
    }
}
