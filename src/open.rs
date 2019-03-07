/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Open file paths and associated methods.

use std::fmt;
use std::fs;
use std_prelude::*;

use super::{Error, PathAbs, PathFile, Result};

/// **INTERNAL TYPE: do not use directly.**
///
/// Use `FileRead`, `FileWrite` or `FileEdit` instead.
pub struct FileOpen {
    pub(crate) path: PathFile,
    pub(crate) file: fs::File,
}

impl FileOpen {
    /// Open the file with the given `OpenOptions`.
    pub fn open<P: AsRef<Path>>(path: P, options: fs::OpenOptions) -> Result<FileOpen> {
        let file = options
            .open(&path)
            .map_err(|err| Error::new(err, "opening", path.as_ref().to_path_buf().into()))?;

        let path = PathFile::new(path)?;
        Ok(FileOpen { path: path, file })
    }

    /// Shortcut to open the file if the path is already absolute.
    ///
    /// Typically you should use `PathFile::open` instead (i.e. `file.open(options)` or
    /// `file.read()`).
    pub fn open_abs<P: Into<PathAbs>>(path: P, options: fs::OpenOptions) -> Result<FileOpen> {
        let path = path.into();
        let file = options
            .open(&path)
            .map_err(|err| Error::new(err, "opening", path.clone().into()))?;

        Ok(FileOpen {
            path: PathFile::new_unchecked(path),
            file,
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
    pub fn metadata(&self) -> Result<fs::Metadata> {
        self.file
            .metadata()
            .map_err(|err| Error::new(err, "getting metadata for", self.path.clone().into()))
    }

    /// Creates a new independently owned handle to the underlying file.
    ///
    /// This function is identical to [std::fs::File::try_clone][0] except it has error
    /// messages which include the action and the path and it returns a `FileOpen` object.
    ///
    /// [0]: https://doc.rust-lang.org/std/fs/struct.File.html#method.try_clone
    pub fn try_clone(&self) -> Result<FileOpen> {
        let file = self
            .file
            .try_clone()
            .map_err(|err| Error::new(err, "cloning file handle for", self.path.clone().into()))?;
        Ok(FileOpen {
            file,
            path: self.path.clone(),
        })
    }
}

impl fmt::Debug for FileOpen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Open(")?;
        self.path.fmt(f)?;
        write!(f, ")")
    }
}

impl AsRef<fs::File> for FileOpen {
    fn as_ref(&self) -> &fs::File {
        &self.file
    }
}

impl Borrow<fs::File> for FileOpen {
    fn borrow(&self) -> &fs::File {
        &self.file
    }
}

impl<'a> Borrow<fs::File> for &'a FileOpen {
    fn borrow(&self) -> &fs::File {
        &self.file
    }
}

impl From<FileOpen> for fs::File {
    fn from(orig: FileOpen) -> fs::File {
        orig.file
    }
}
