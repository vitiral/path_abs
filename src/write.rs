/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Open write-only file paths and associated methods.

use std::convert::AsRef;
use std::fs;
use std::fmt;
use std::io;
use std::path::Path;
use std::ops::Deref;

use super::PathFile;
use super::open::FileOpen;

/// A write-only file handle with `path()` attached and improved error messages. Contains only the
/// methods and trait implementations which are allowed by a write-only file.
///
/// # Examples
/// ```rust
/// # extern crate path_abs;
/// # extern crate tempdir;
/// use std::io::Write;
/// use path_abs::{PathFile, FileWrite};
///
/// # fn main() {
/// let example = "example.txt";
/// # let tmp = tempdir::TempDir::new("ex").unwrap();
/// # let example = &tmp.path().join(example);
///
/// let expected = "foo\nbar";
/// let mut write = FileWrite::create(example).unwrap();
/// write.write_all(expected.as_bytes()).unwrap();
/// write.flush();
///
/// let file = PathFile::new(example).unwrap();
/// assert_eq!(expected, file.read_string().unwrap());
/// # }
/// ```
pub struct FileWrite(pub(crate) FileOpen);

impl FileWrite {
    /// Open the file with the given `OpenOptions` but always sets `write` to true.
    pub fn open<P: AsRef<Path>>(path: P, mut options: fs::OpenOptions) -> io::Result<FileWrite> {
        options.write(true);
        Ok(FileWrite(FileOpen::open(path, options)?))
    }

    /// Shortcut to open the file if the path is already canonicalized.
    pub(crate) fn open_path(
        path_file: PathFile,
        mut options: fs::OpenOptions,
    ) -> io::Result<FileWrite> {
        options.write(true);
        Ok(FileWrite(FileOpen::open_path(path_file, options)?))
    }

    /// Open the file in write-only mode, truncating it first if it exists and creating it
    /// otherwise.
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<FileWrite> {
        let mut options = fs::OpenOptions::new();
        options.truncate(true);
        options.create(true);
        FileWrite::open(path, options)
    }

    /// Open the file for appending, creating it if it doesn't exist.
    pub fn append<P: AsRef<Path>>(path: P) -> io::Result<FileWrite> {
        let mut options = fs::OpenOptions::new();
        options.append(true);
        options.create(true);
        FileWrite::open(path, options)
    }

    /// Open the file for editing (reading and writing) but do not create it
    /// if it doesn't exist.
    pub fn edit<P: AsRef<Path>>(path: P) -> io::Result<FileWrite> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        FileWrite::open(path, options)
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
                format!(
                    "{} when setting permisions for {}",
                    err,
                    self.path.display()
                ),
            )
        })
    }
}

impl fmt::Debug for FileWrite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FileWrite(")?;
        self.path.fmt(f)?;
        write!(f, ")")
    }
}

impl io::Write for FileWrite {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.file.write(buf).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when writing to {}", err, self.path().display()),
            )
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.file.flush().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when flushing {}", err, self.path().display()),
            )
        })
    }
}

impl io::Seek for FileWrite {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.0.file.seek(pos).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} seeking {}", err, self.path().display()),
            )
        })
    }
}

impl Deref for FileWrite {
    type Target = FileOpen;

    fn deref(&self) -> &FileOpen {
        &self.0
    }
}
