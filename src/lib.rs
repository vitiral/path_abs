/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Ergonomic paths and files in rust.
//!
//! This library aims to provide ergonomic path and file operations to rust with reasonable
//! performance.
//!
//! This includes:
//!
//! - Cleaner _absolute_ paths (which is distinct from canonicalized paths).
//! - Improved error messages, see the **Better Errors** section.
//! - Improved type safety. The types specify that a file/dir _once_ existed and was _once_ a certain
//!   type. Obviously a file/dir can be deleted/changed by another process.
//! - More stringent mutability requirements. See the **Differing Method Signatures** section.
//! - Cheap cloning: all path types are `Arc`, which a cheap operation compared to filesystem
//!   operations and allows more flexibility and ergonomics in the library for relatively low cost.
//!
//! ## Better Errors
//!
//! All errors include the **path** and **action** which caused the error, as well as the unaltered
//! `std::io::Error` message. Errors are convertable into `std::io::Error`, giving almost complete
//! compatibility with existing code.
//!
//! ### `set_len` (i.e. truncate a file):
//!
//! - [`/* */ std::fs::File::set_len(0)`][file_set_len]: `Invalid argument (os error 22)`
//! - [`path_abs::FileWrite::set_len(0)`][path_set_len]: `Invalid argument (os error 22) when setting
//!   len for /path/to/example/foo.txt`
//!
//! > The above error is actually impossible because `FileWrite` is always writeable, and
//! > `FileRead` does not implement `set_len`. However, it is kept for demonstration.
//!
//! ### `read` (open file for reading):
//!
//! - [`/**/ std::fs::File::read(path)`][file_read]: `No such file or directory (os error 2)`
//! - [`path_abs::FileRead::read(path)`][path_read]: `No such file or directory (os error 2) when
//!   opening example/foo.txt`
//!
//! And every other method has similarily improved errors. If a method does not have pretty error
//! messages please open a ticket.
//!
//! [file_set_len]: https://doc.rust-lang.org/std/fs/struct.File.html#method.set_len
//! [file_read]: https://doc.rust-lang.org/std/fs/struct.File.html#method.read
//! [path_set_len]: struct.FileWrite.html#method.set_len
//! [path_read]: struct.FileRead.html#method.read
//!
//!
//! ## Exported Path Types
//!
//! These are the exported Path types. All of them are absolute except for `PathArc`, which
//! is just an `Arc<PathBuf>` with methods that have better error reporting.
//!
//! - [`PathArc`](struct.PathArc.html): a reference counted `PathBuf` with methods reimplemented
//!   with better error messages. Use this for a generic serializable path that may or may
//!   not exist.
//! - [`PathAbs`](struct.PathAbs.html): a reference counted absolute (_not necessarily_
//!   canonicalized) path that is not necessarily guaranteed to exist.
//! - [`PathFile`](struct.PathFile.html): a `PathAbs` that is guaranteed (at instantiation) to
//!   exist and be a file, with associated methods.
//! - [`PathDir`](struct.PathDir.html): a `PathAbs` that is guaranteed (at instantiation) to exist
//!   and be a directory, with associated methods.
//! - [`PathType`](struct.PathType.html): an enum containing either a PathFile or a PathDir.
//!   Returned by [`PathDir::list`][dir_list]
//!
//! In addition, all paths are serializable through serde (even on windows!) by using the crate
//! [`stfu8`](https://crates.io/crates/stfu8) to encode/decode, allowing ill-formed UTF-16. See
//! that crate for more details on how the resulting encoding can be edited (by hand) even in the
//! case of what *would be* ill-formed UTF-16.
//!
//! [dir_list]: struct.PathDir.html#method.list
//!
//!
//! ## Exported File Types
//!
//! All File types provide _type safe_ access to their relevant traits. For instance, you can't
//! `read` with a `FileWrite` and you can't `write` with a `FileRead`.
//!
//! - [`FileRead`](struct.FileRead.html): a read-only file handle with `path()` attached and
//!   improved error messages. Contains only the methods and trait implementations which are
//!   allowed by a read-only file.
//! - [`FileWrite`](struct.FileWrite.html): a write-only file handle with `path()` attached and
//!   improved error messages. Contains only the methods and trait implementations which are
//!   allowed by a write-only file.
//! - [`FileEdit`](struct.FileEdit.html): a read/write file handle with `path()` attached and
//!   improved error messages. Contains methods and trait implements for both readable _and_
//!   writeable files.
//!
//! ### Differing Method Signatures
//!
//! The type signatures of the `File*` types regarding `read`, `write` and other methods is
//! slightly different than `std::fs::File` -- they all take `&mut` instead of `&`. This is to
//! avoid a [common possible footgun](https://github.com/rust-lang/rust/issues/47708).
//!
//! To demonstrate, imagine the following scenario:
//!
//! - You pass your open `&File` to a method, which puts it in a thread. This thread constantly
//!   calls `seek(SeekFrom::Start(10))`
//! - You periodically read from a file expecting new data, but are always getting the same data.
//!
//! Yes, this is actually allowed by the rust compiler since `seek` is implemented for
//! [`&File`](https://doc.rust-lang.org/std/fs/struct.File.html#impl-Seek-1). Technically this is
//! still _memory safe_ since the operating system will handle any contention, however many would
//! argue that it isn't _expected_ that an immutable reference passed to another
//! function can affect the seek position of a file.
//!
//!
//! # Examples
//! Recreating `Cargo.init` in `example/`
//!
//! ```rust
//! # extern crate path_abs;
//! # extern crate tempdir;
//! use std::path::Path;
//! use std::collections::HashSet;
//! use path_abs::{
//!     PathAbs,   // absolute path
//!     PathDir,   // absolute path to a directory
//!     PathFile,  // absolute path to a file
//!     PathType,  // enum of Dir or File
//!     FileRead,  // Open read-only file handler
//!     FileWrite, // Open write-only file handler
//!     FileEdit,  // Open read/write file handler
//! };
//!
//! # fn try_main() -> ::std::io::Result<()> {
//! let example = Path::new("example");
//! # let tmp = tempdir::TempDir::new("ex")?;
//! # let example = &tmp.path().join(example);
//!
//! // Create your paths
//! let project = PathDir::create_all(example)?;
//! let src = PathDir::create(project.join("src"))?;
//! let lib = PathFile::create(src.join("lib.rs"))?;
//! let cargo = PathFile::create(project.join("Cargo.toml"))?;
//!
//! // Write the templates
//! lib.write_str(r#"
//! #[cfg(test)]
//! mod tests {
//!     #[test]
//!     fn it_works() {
//!         assert_eq!(2 + 2, 4);
//!     }
//! }"#)?;
//!
//! cargo.write_str(r#"
//! [package]
//! name = "example"
//! version = "0.1.0"
//! authors = ["Garrett Berg <vitiral@gmail.com>"]
//!
//! [dependencies]
//! "#)?;
//!
//! // Put our result into a HashMap so we can assert it
//! let mut result = HashSet::new();
//! for p in project.list()? {
//!     result.insert(p?);
//! }
//!
//! // Create our expected value
//! let mut expected = HashSet::new();
//! expected.insert(PathType::Dir(src));
//! expected.insert(PathType::File(cargo));
//!
//! assert_eq!(expected, result);
//!
//! // ----------------------------------
//! // Creating types from existing paths
//!
//! // Creating a generic path
//! let lib_path = example.join("src").join("lib.rs");
//! let abs = PathAbs::new(&lib_path)?;
//!
//! // Or a path with a known type
//! let file = PathType::new(&lib_path)
//!     ?
//!     .unwrap_file();
//!
//! // Or use `PathAbs::into_file`
//! let file2 = abs.clone().into_file()?;
//!
//! assert!(abs.is_file());
//! assert!(file.is_file());
//! assert!(file2.is_file());
//!
//! // ----------------------------------
//! // Opening a File
//!
//! // open read-only using the PathFile method
//! let read = file.read()?;
//!
//! // Or use the type directly: open for appending
//! let write = FileWrite::append(&file)?;
//!
//! // Open for read/write editing.
//! let edit = file.edit()?;
//! # Ok(()) } fn main() { try_main().unwrap() }
//! ```

#[cfg(feature = "serialize")]
extern crate serde;
#[macro_use]
#[cfg(feature = "serialize")]
extern crate serde_derive;
extern crate std_prelude;
#[cfg(feature = "serialize")]
extern crate stfu8;

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;
#[cfg(test)]
extern crate regex;
#[cfg(test)]
extern crate serde_json;
#[cfg(test)]
extern crate tempdir;

use std::io;
use std::error;
use std::ffi;
use std::fmt;
use std::fs;
use std::path;
use std_prelude::*;

mod abs;
mod arc;
mod dir;
mod edit;
mod file;
pub mod open;
#[cfg(feature = "serialize")]
mod ser;
mod ty;
mod write;
mod read;

pub use abs::PathAbs;
pub use arc::{current_dir, PathArc};
pub use dir::{ListDir, PathDir};
pub use file::PathFile;
pub use ty::PathType;

pub use edit::FileEdit;
pub use write::FileWrite;
pub use read::FileRead;

pub type Result<T> = ::std::result::Result<T, Error>;

/// An error produced by performing an filesystem operation on a `Path`.
///
/// This error type is a light wrapper around [`std::io::Error`]. In particular, it adds the
/// following information:
///
/// - The action being performed when the error occured
/// - The path associated with the IO error.
///
/// To maintain good ergonomics, this type has a `impl From<Error> for std::io::Error` defined so
/// that you may use an [`io::Result`] with methods in this crate if you don't care about accessing
/// the underlying error data in a structured form (the pretty format will be preserved however).
///
/// [`std::io::Error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html
/// [`io::Result`]: https://doc.rust-lang.org/stable/std/io/type.Result.html
///
/// # Examples
/// ```rust
/// use path_abs::Error as PathError;
/// use path_abs::PathFile;
///
/// /// main function, note that you can use `io::Error`
/// fn try_main() -> Result<(), ::std::io::Error> {
///     let lib = PathFile::new("src/lib.rs")?;
///     Ok(())
/// }
///
/// ```
pub struct Error {
    io_err: io::Error,
    action: String,
    path: PathArc,
}

impl Error {
    /// Create a new error when the path and action are known.
    pub fn new(io_err: io::Error, action: &str, path: PathArc) -> Error {
        Error {
            io_err: io_err,
            action: action.into(),
            path: path,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error<{}>", self)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} when {} {}",
            self.io_err,
            self.action,
            self.path.display()
        )
    }
}

impl Error {
    /// Returns the path associated with this error.
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Returns the `std::io::Error` associated with this errors.
    pub fn io_error(&self) -> &io::Error {
        &self.io_err
    }

    /// Returns the action being performed when this error occured.
    pub fn action(&self) -> &str {
        &self.action
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.io_err.description()
    }

    fn cause(&self) -> Option<&error::Error> {
        Some(&self.io_err)
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(err.io_err.kind(), err)
    }
}

/// Trait for path-like types.
///
/// This trait defines most of the operations one might want to perform on a
/// path-like type. It very closely resembles the interface of the standard
/// library's [`PathBuf`], except that methods that can return an error will
/// return a rich [`path_abs::Error`] rather than a plain [`std::io::Error`].
///
/// Trait methods without documentation should be assumed to be identical to the
/// method of the same name on [`PathBuf`].
///
/// [`PathBuf`]: https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html
/// [`path_abs::Error`]: struct.Error.html
/// [`std::io::Error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html
pub trait PathExt: Sized + Clone + Borrow<PathBuf> + From<PathBuf> + Into<PathArc> {
    /// Borrow this path as `&mut` [`PathBuf`].
    ///
    /// This method exists because `Arc<T>` does not implement `BorrowMut<T>`,
    /// even though it probably could.
    ///
    /// [`PathBuf`]: https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html
    fn as_pathbuf_mut(&mut self) -> &mut PathBuf;

    fn is_absolute(&self) -> bool { Path::is_absolute(self.borrow()) }

    fn is_relative(&self) -> bool { Path::is_relative(self.borrow()) }

    fn has_root(&self) -> bool { Path::has_root(self.borrow()) }

    /// Returns a new value representing the parent of this path (if any).
    ///
    /// This method always returns a new owned value, it does not borrow this
    /// value.
    ///
    /// # Errors
    ///
    /// This method returns an error if this path has no parent, i.e. it
    /// represents a filesystem root, like `/` on Unix or `C:\` on Windows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use path_abs::PathExt;
    ///
    /// fn find_git_repository_root<P: PathExt>(path: P) -> Option<P> {
    ///     while let Ok(path) = path.parent() {
    ///         let maybe_git_dir = path.concat(".git").unwrap();
    ///
    ///         if maybe_git_dir.is_dir() { return Some(path); }
    ///     }
    ///
    ///     None
    /// }
    /// ```
    fn parent(&self) -> Result<Self> {
        let mut res = self.clone();
        res.truncate_to_parent()?;
        Ok(res)
    }

    fn ancestors(&self) -> path::Ancestors { Path::ancestors(self.borrow()) }

    fn file_name(&self) -> Option<&ffi::OsStr> { Path::file_name(self.borrow()) }

    fn starts_with<P: AsRef<Path>>(&self, base: P) -> bool {
        Path::starts_with(self.borrow(), base)
    }

    fn ends_with<P: AsRef<Path>>(&self, base: P) -> bool {
        Path::ends_with(self.borrow(), base)
    }

    fn file_stem(&self) -> Option<&ffi::OsStr> { Path::file_stem(self.borrow()) }

    fn extension(&self) -> Option<&ffi::OsStr> { Path::extension(self.borrow()) }

    /// Returns a new value representing the concatenation of two paths.
    ///
    /// Note that this method represents pure concatenation, not "adjoinment"
    /// like [`PathBuf::join`].
    ///
    /// # Errors
    ///
    /// This method returns an error if the `path` parameter contains a prefix
    /// component (on Windows, like `C:`), a root component (a leading `/`, or
    /// on Windows a leading `\`), or a `..` component.
    ///
    /// # Example
    ///
    /// ```rust
    /// use path_abs::{PathExt, Result};
    ///
    /// fn find_config_file<P: PathExt>(
    ///     search_path: &[P],
    ///     file_name: &str,
    /// ) -> Option<P> {
    ///     for each in search_path.iter() {
    ///         if let Ok(maybe_config) = each.concat(file_name) {
    ///             if maybe_config.is_file() { return Some(maybe_config); }
    ///         }
    ///     }
    //
    ///     None
    /// }
    /// ```
    ///
    /// [`PathBuf::join`]: https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html#method.join
    fn concat<P: AsRef<Path>>(&self, path: P) -> Result<Self> {
        let mut res = self.clone();
        res.append(path)?;
        Ok(res)
    }

    fn with_file_name<S: AsRef<ffi::OsStr>>(&self, file_name: S) -> Self {
        let mut res = self.clone();
        res.set_file_name(file_name);
        res
    }

    fn with_extension<S: AsRef<ffi::OsStr>>(&self, extension: S) -> Self {
        let mut res = self.clone();
        res.set_extension(extension);
        res
    }

    fn components(&self) -> path::Components { Path::components(self.borrow()) }

    fn iter(&self) -> path::Iter { Path::iter(self.borrow()) }

    fn metadata(&self) -> Result<fs::Metadata> {
        Path::metadata(self.borrow())
            .map_err(|err| Error::new(
                err,
                "getting metadata of",
                self.clone().into(),
            ))
    }

    fn symlink_metadata(&self) -> Result<fs::Metadata> {
        Path::symlink_metadata(self.borrow())
            .map_err(|err| Error::new(
                err,
                "getting symlink metadata of",
                self.clone().into(),
            ))
    }

    fn canonicalize(&self) -> Result<Self> {
        Path::canonicalize(self.borrow())
            .map(Into::into)
            .map_err(|err| Error::new(
                err,
                "canonicalizing",
                self.clone().into(),
            ))
    }

    fn read_link(&self) -> Result<Self> {
        Path::read_link(self.borrow())
            .map(Into::into)
            .map_err(|err| Error::new(
                err,
                "reading link target of",
                self.clone().into(),
            ))
    }

    fn exists(&self) -> bool { Path::exists(self.borrow()) }

    fn is_file(&self) -> bool { Path::is_file(self.borrow()) }

    fn is_dir(&self) -> bool { Path::is_dir(self.borrow()) }

    /// Appends `path` to this path.
    ///
    /// Note that this method represents pure concatenation, not "adjoinment"
    /// like [`PathBuf::push`].
    ///
    /// # Errors
    ///
    /// This method returns an error if the `path` parameter contains a prefix
    /// component (on Windows, like `C:`), a root component (a leading `/`, or
    /// on Windows a leading `\`), or a `..` component.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::Path;
    /// use path_abs::PathExt;
    ///
    /// let mut somepath = Path::new("foo").to_path_buf();
    /// somepath.append("bar");
    ///
    /// assert_eq!(somepath.as_path(), Path::new("foo/bar"));
    /// ```
    ///
    /// [`PathBuf::push`]: https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html#method.push
    fn append<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        for each in path.as_ref().components() {
            match each {
                path::Component::Normal(c) => self.as_pathbuf_mut().push(c),
                path::Component::CurDir => (), // "." does nothing
                path::Component::Prefix(_) => return Err(
                    Error::new(
                        io::Error::new(
                            io::ErrorKind::Other,
                            "appending prefix component",
                        ),
                        "appending prefix component",
                        PathArc::new(&path),
                    )
                ),
                path::Component::RootDir => return Err(
                    Error::new(
                        io::Error::new(
                            io::ErrorKind::Other,
                            "appending root component",
                        ),
                        "appending root component",
                        PathArc::new(&path),
                    )
                ),
                path::Component::ParentDir => return Err(
                    Error::new(
                        io::Error::new(
                            io::ErrorKind::Other,
                            "appending .. component",
                        ),
                        "appending .. component",
                        PathArc::new(&path),
                    )
                ),
            }
        }

        Ok(())
    }

    /// Removes the last component of this path.
    ///
    /// # Errors
    ///
    /// This method returns an error if this path has no parent, i.e. it
    /// represents a filesystem root, like `/` on Unix or `C:\` on Windows.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn example() -> Result<(), path_abs::Error> {
    /// use std::path::Path;
    /// use path_abs::PathExt;
    ///
    /// let executable = Path::new("/usr/loca/bin/myapp");
    /// let mut install_path = executable.to_path_buf();
    /// install_path.truncate_to_parent()?;
    ///
    /// assert_eq!(install_path.as_path(), Path::new("/usr/local/bin"));
    /// # Ok(()) }
    fn truncate_to_parent(&mut self) -> Result<()> {
        if self.as_pathbuf_mut().pop() {
            Ok(())
        } else {
            Err(Error::new(
                io::Error::new(io::ErrorKind::NotFound, "path has no parent"),
                "truncating to parent",
                self.clone().into(),
            ))
        }
    }

    /// Removes all components after the root, if any.
    ///
    /// This is mostly useful on Windows, since it preserves the prefix before
    /// the root.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use path_abs::PathExt;
    ///
    /// let mut somepath = Path::new(r"C:\foo\bar").to_path_buf();
    /// somepath.truncate_to_root();
    ///
    /// assert_eq!(somepath.as_path(), Path::new(r"C:\"));
    fn truncate_to_root(&mut self) {
        let mut res = PathBuf::new();
        for component in self.components().take(2) {
            match component {
                // We want to keep prefix and RootDir components of this path
                | path::Component::Prefix(_)
                | path::Component::RootDir
                    => res.push(component),
                // We want to discard all other components.
                _ => break,
            }
        }

        // Clobber ourselves with the new value.
        *self = res.into();
    }

    fn set_file_name<S: AsRef<ffi::OsStr>>(&mut self, file_name: S) {
        self.as_pathbuf_mut().set_file_name(file_name)
    }

    fn set_extension<S: AsRef<ffi::OsStr>>(&mut self, extension: S) -> bool {
        self.as_pathbuf_mut().set_extension(extension)
    }
}

impl PathExt for PathBuf {
    fn as_pathbuf_mut(&mut self) -> &mut PathBuf { self }
}

impl PathExt for Arc<PathBuf> {
    fn as_pathbuf_mut(&mut self) -> &mut PathBuf { Arc::make_mut(self) }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;
    use regex::{self, Regex};

    use super::*;

    macro_rules! assert_match {
        ($re: expr, $err: expr) => {{
            let re = Regex::new(&$re).unwrap();
            let err = $err.to_string();
            assert!(
                re.is_match(&err),
                "\nGot Err         : {:?}\nMatching against: {:?}",
                err.to_string(),
                $re
            );
        }};
    }

    fn escape<P: AsRef<Path>>(path: P) -> String {
        regex::escape(&format!("{}", path.as_ref().display()))
    }

    #[test]
    /// Tests to make sure the error messages look like we expect.
    fn sanity_errors() {
        let tmp_dir = TempDir::new("example").expect("create temp dir");
        let tmp_abs = PathDir::new(tmp_dir.path()).expect("tmp_abs");

        {
            let foo = PathFile::create(tmp_abs.join("foo.txt")).expect("foo.txt");
            foo.clone().remove().unwrap();
            let pat = if cfg!(unix) {
                format!(
                    r"No such file or directory \(os error \d+\) when opening {}",
                    escape(&foo)
                )
            } else {
                format!(
                    r"The system cannot find the file specified. \(os error \d+\) when opening {}",
                    escape(&foo)
                )
            };
            assert_match!(pat, foo.edit().unwrap_err())
        }
    }

    #[cfg(windows)]
    mod windows {
        use super::*;

        #[test]
        fn test_pathext_parent() {
            let p = Path::new(r"C:\foo\bar").to_path_buf();

            let actual = <PathBuf as PathExt>::parent(&p)
                .expect("could not find parent?");
            let expected = Path::new(r"C:\foo").to_path_buf();

            assert_eq!(actual, expected);

            let p = Path::new(r"C:\").to_path_buf();

            let actual = <PathBuf as PathExt>::parent(&p)
                .expect_err("root has a parent?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::NotFound);
            assert_eq!(actual.action(), "truncating to parent");
            assert_eq!(actual.path(), Path::new(r"C:\"));
        }

        #[test]
        fn test_pathext_starts_with() {
            let p = Path::new(r"foo\bar").to_path_buf();

            assert_eq!(
                <PathBuf as PathExt>::starts_with(&p, Path::new("foo")),
                true,
            );
            assert_eq!(
                <PathBuf as PathExt>::starts_with(&p, Path::new("bar")),
                false,
            );
        }

        #[test]
        fn test_pathext_ends_with() {
            let p = Path::new(r"foo\bar").to_path_buf();

            assert_eq!(
                <PathBuf as PathExt>::ends_with(&p, Path::new("foo")),
                false,
            );
            assert_eq!(
                <PathBuf as PathExt>::ends_with(&p, Path::new("bar")),
                true,
            );
        }

        #[test]
        fn test_pathext_concat() {
            let actual = Path::new("foo")
                .to_path_buf()
                .concat(Path::new("bar"))
                .expect("Could not concat paths?");
            let expected = Path::new(r"foo\bar").to_path_buf();

            assert_eq!(actual, expected);

            let actual = Path::new("foo")
                .to_path_buf()
                .concat(Path::new(r"bar\..\baz"))
                .expect_err("Could concat path with ParentDir?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending .. component");
            assert_eq!(actual.path(), Path::new(r"bar\..\baz"));

            let actual = Path::new("foo")
                .to_path_buf()
                .concat(Path::new(r"\windows\system32"))
                .expect_err("Could concat path with RootDir?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending root component");
            assert_eq!(actual.path(), Path::new(r"\windows\system32"));

            let actual = Path::new("foo")
                .to_path_buf()
                .concat(Path::new(r"C:bar"))
                .expect_err("Could concat path with prefix?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending prefix component");
            assert_eq!(actual.path(), Path::new(r"C:bar"));
        }

        #[test]
        fn test_pathext_append() {
            let mut actual = Path::new("foo")
                .to_path_buf();
            actual.append(Path::new("bar"))
                .expect("Could not append paths?");
            let expected = Path::new(r"foo\bar").to_path_buf();

            assert_eq!(actual, expected);

            let actual = Path::new("foo")
                .to_path_buf()
                .append(Path::new(r"bar\..\baz"))
                .expect_err("Could append ParentDir to path?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending .. component");
            assert_eq!(actual.path(), Path::new(r"bar\..\baz"));

            let actual = Path::new("foo")
                .to_path_buf()
                .append(Path::new(r"\windows\system32"))
                .expect_err("Could append RootDir to path?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending root component");
            assert_eq!(actual.path(), Path::new(r"\windows\system32"));

            let actual = Path::new("foo")
                .to_path_buf()
                .append(Path::new(r"C:bar"))
                .expect_err("Could append prefix to path?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending prefix component");
            assert_eq!(actual.path(), Path::new(r"C:bar"));
        }

        #[test]
        fn test_pathext_truncate_to_parent() {
            let mut p = Path::new(r"C:\foo\bar").to_path_buf();
            p.truncate_to_parent()
                .expect("could not find parent?");

            assert_eq!(p.as_path(), Path::new(r"C:\foo"));

            let mut p = Path::new(r"C:\").to_path_buf();
            let actual = p.truncate_to_parent()
                .expect_err("root has a parent?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::NotFound);
            assert_eq!(actual.action(), "truncating to parent");
            assert_eq!(actual.path(), Path::new(r"C:\"));
        }

        #[test]
        fn test_pathext_truncate_to_root() {
            let mut p = Path::new(r"C:\foo\bar").to_path_buf();
            p.truncate_to_root();
            assert_eq!(p.as_path(), Path::new(r"C:\"));

            let mut p = Path::new(r"C:foo").to_path_buf();
            p.truncate_to_root();
            assert_eq!(p.as_path(), Path::new(r"C:"));

            let mut p = Path::new(r"\foo").to_path_buf();
            p.truncate_to_root();
            assert_eq!(p.as_path(), Path::new(r"\"));

            let mut p = Path::new(r"foo").to_path_buf();
            p.truncate_to_root();
            assert_eq!(p.as_path(), Path::new(r""));
        }
    }

    #[cfg(not(windows))]
    mod unix {
        use super::*;

        #[test]
        fn test_pathext_is_absolute() {
            let p = Path::new("/foo/bar").to_path_buf();

            assert_eq!(
                <PathBuf as PathExt>::is_absolute(&p),
                true,
            );
        }

        #[test]
        fn test_pathext_parent() {
            let p = Path::new("/foo/bar").to_path_buf();

            let actual = <PathBuf as PathExt>::parent(&p)
                .expect("could not find parent?");
            let expected = Path::new("/foo").to_path_buf();

            assert_eq!(actual, expected);

            let p = Path::new("/").to_path_buf();

            let actual = <PathBuf as PathExt>::parent(&p)
                .expect_err("root has a parent?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::NotFound);
            assert_eq!(actual.action(), "truncating to parent");
            assert_eq!(actual.path(), Path::new("/"));
        }

        #[test]
        fn test_pathext_starts_with() {
            let p = Path::new("foo/bar").to_path_buf();

            assert_eq!(
                <PathBuf as PathExt>::starts_with(&p, Path::new("foo")),
                true,
            );
            assert_eq!(
                <PathBuf as PathExt>::starts_with(&p, Path::new("bar")),
                false,
            );
        }

        #[test]
        fn test_pathext_ends_with() {
            let p = Path::new("foo/bar").to_path_buf();

            assert_eq!(
                <PathBuf as PathExt>::ends_with(&p, Path::new("foo")),
                false,
            );
            assert_eq!(
                <PathBuf as PathExt>::ends_with(&p, Path::new("bar")),
                true,
            );
        }

        #[test]
        fn test_pathext_concat() {
            let actual = Path::new("foo")
                .to_path_buf()
                .concat(Path::new("bar"))
                .expect("Could not concat paths?");
            let expected = Path::new("foo/bar").to_path_buf();

            assert_eq!(actual, expected);

            let actual = Path::new("foo")
                .to_path_buf()
                .concat(Path::new("bar/../baz"))
                .expect_err("Could concat path with ParentDir?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending .. component");
            assert_eq!(actual.path(), Path::new("bar/../baz"));

            let actual = Path::new("foo")
                .to_path_buf()
                .concat(Path::new("/etc/passwd"))
                .expect_err("Could concat path with RootDir?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending root component");
            assert_eq!(actual.path(), Path::new("/etc/passwd"));
        }

        #[test]
        fn test_pathext_append() {
            let mut actual = Path::new("foo")
                .to_path_buf();
            actual.append(Path::new("bar"))
                .expect("Could not append paths?");
            let expected = Path::new("foo/bar").to_path_buf();

            assert_eq!(actual, expected);

            let actual = Path::new("foo")
                .to_path_buf()
                .append(Path::new("bar/../baz"))
                .expect_err("Could append ParentDir to path?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending .. component");
            assert_eq!(actual.path(), Path::new("bar/../baz"));

            let actual = Path::new("foo")
                .to_path_buf()
                .append(Path::new("/etc/passwd"))
                .expect_err("Could append RootDir to path?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::Other);
            assert_eq!(actual.action(), "appending root component");
            assert_eq!(actual.path(), Path::new("/etc/passwd"));
        }

        #[test]
        fn test_pathext_truncate_to_parent() {
            let mut p = Path::new("/foo/bar").to_path_buf();
            p.truncate_to_parent()
                .expect("could not find parent?");

            assert_eq!(p.as_path(), Path::new("/foo"));

            let mut p = Path::new("/").to_path_buf();
            let actual = p.truncate_to_parent()
                .expect_err("root has a parent?");

            assert_eq!(actual.io_error().kind(), io::ErrorKind::NotFound);
            assert_eq!(actual.action(), "truncating to parent");
            assert_eq!(actual.path(), Path::new("/"));
        }

        #[test]
        fn test_pathext_truncate_to_root() {
            let mut p = Path::new("/foo/bar").to_path_buf();
            p.truncate_to_root();
            assert_eq!(p.as_path(), Path::new("/"));

            let mut p = Path::new("foo/bar").to_path_buf();
            p.truncate_to_root();
            assert_eq!(p.as_path(), Path::new(""));
        }
    }
}
