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

    /// Attempts to sync all OS-internal metadata to disk.
    ///
    /// This function will attempt to ensure that all in-core data reaches the filesystem before
    /// returning.
    ///
    /// This function is identical to [std::fs::File::sync_all][0] except it has error
    /// messages which include the action and the path.
    ///
    /// [0]: https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all
    pub fn sync_all(&self) -> io::Result<()> {
        self.file.sync_all().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when syncing {}", err, self.path.display()),
            )
        })
    }

    /// This function is similar to sync_all, except that it may not synchronize file metadata to
    /// the filesystem.
    ///
    /// This function is identical to [std::fs::File::sync_data][0] except it has error
    /// messages which include the action and the path.
    ///
    /// [0]: https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_data
    pub fn sync_data(&self) -> io::Result<()> {
        self.file.sync_data().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when syncing data for {}", err, self.path.display()),
            )
        })
    }

    /// Truncates or extends the underlying file, updating the size of this file to become size.
    ///
    /// This function is identical to [std::fs::File::set_len][0] except:
    ///
    /// - It has error messages which include the action and the path.
    /// - It takes `&mut self` instead of `&self`.
    ///
    /// [0]: https://doc.rust-lang.org/std/fs/struct.File.html#method.set_len
    pub fn set_len(&mut self, size: u64) -> io::Result<()> {
        self.file.set_len(size).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when setting len for {}", err, self.path.display()),
            )
        })
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
    /// messages which include the action and the path and it returns a `PathOpen` object.
    ///
    /// [0]: https://doc.rust-lang.org/std/fs/struct.File.html#method.try_clone
    pub fn try_clone(&self) -> io::Result<PathOpen> {
        let file = self.file.try_clone().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when cloning {}", err, self.path.display()),
            )
        })?;
        Ok(PathOpen {
            file: file,
            path: self.path.clone(),
        })
    }

    /// Changes the permissions on the underlying file.
    ///
    /// This function is identical to [std::fs::File::set_permissions][0] except:
    ///
    /// - It has error messages which include the action and the path.
    /// - It takes `&mut self` instead of `&self`.
    ///
    /// [0]: https://doc.rust-lang.org/std/fs/struct.File.html#method.set_permissions
    pub fn set_permissions(&mut self, perm: fs::Permissions) -> io::Result<()> {
        self.file.set_permissions(perm).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when setting permisions for {}", err, self.path.display()),
            )
        })
    }
}


impl fmt::Debug for PathOpen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Open(")?;
        self.path.fmt(f)?;
        write!(f, ")")
    }
}

impl Into<fs::File> for PathOpen {
    fn into(self) -> fs::File {
        self.file
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
