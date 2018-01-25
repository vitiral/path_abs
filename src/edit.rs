/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Open editable (read+write) file paths and associated methods.

use std::convert::AsRef;
use std::fs;
use std::fmt;
use std::io;
use std::path::Path;
use std::ops::Deref;

use super::PathFile;
use super::open::FileOpen;

/// An open file which can be written to. Get the associated `PathFile` with with the `path()`
/// method.
///
/// > This type is not serializable.
///
/// # Examples
/// ```rust
/// # extern crate path_abs;
/// # extern crate tempdir;
/// use std::io::{Read, Seek, Write, SeekFrom};
/// use path_abs::FileEdit;
///
/// # fn main() {
///
/// let example = "example.txt";
/// # let tmp = tempdir::TempDir::new("ex").unwrap();
/// # let example = &tmp.path().join(example);
///
/// let expected = "foo\nbar";
/// let mut edit = FileEdit::create(example).unwrap();
///
/// let mut s = String::new();
/// edit.write_all(expected.as_bytes()).unwrap();
/// edit.seek(SeekFrom::Start(0)).unwrap();
/// edit.read_to_string(&mut s).unwrap();
///
/// assert_eq!(expected, s);
/// # }
/// ```
pub struct FileEdit(pub(crate) FileOpen);

impl FileEdit {
    /// Open the file with the given `OpenOptions` but always sets `write` to true.
    pub fn open<P: AsRef<Path>>(path: P, mut options: fs::OpenOptions) -> io::Result<FileEdit> {
        options.write(true);
        options.read(true);
        Ok(FileEdit(FileOpen::open(path, options)?))
    }

    /// Shortcut to open the file if the path is already canonicalized.
    pub(crate) fn open_path(path: PathFile, mut options: fs::OpenOptions) -> io::Result<FileEdit> {
        options.write(true);
        options.read(true);
        Ok(FileEdit(FileOpen::open_file(path, options)?))
    }

    /// Open the file in editing mode, truncating it first if it exists and creating it
    /// otherwise.
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<FileEdit> {
        let mut options = fs::OpenOptions::new();
        options.truncate(true);
        options.create(true);
        FileEdit::open(path, options)
    }

    /// Open the file for appending, creating it if it doesn't exist.
    pub fn append<P: AsRef<Path>>(path: P) -> io::Result<FileEdit> {
        let mut options = fs::OpenOptions::new();
        options.append(true);
        options.create(true);
        FileEdit::open(path, options)
    }

    /// Open the file for editing (reading and writing) but do not create it
    /// if it doesn't exist.
    pub fn edit<P: AsRef<Path>>(path: P) -> io::Result<FileEdit> {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        FileEdit::open(path, options)
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

impl fmt::Debug for FileEdit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FileEdit(")?;
        self.path.fmt(f)?;
        write!(f, ")")
    }
}

impl io::Read for FileEdit {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.file.read(buf).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when reading {}", err, self.path().display()),
            )
        })
    }
}


impl io::Write for FileEdit {
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

impl io::Seek for FileEdit {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.0.file.seek(pos).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} seeking {}", err, self.path().display()),
            )
        })
    }
}

impl Deref for FileEdit {
    type Target = FileOpen;

    fn deref(&self) -> &FileOpen {
        &self.0
    }
}
