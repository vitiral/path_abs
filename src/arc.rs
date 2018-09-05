/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! `PathArc`: Atomically reference counted path with better errors.

use std::fmt;
use std::fs;
use std::io;
use std::env;
use std::ffi::OsStr;
use std::path::{Component, PrefixComponent};
use std_prelude::*;

use super::{Error, Result};
use abs::PathAbs;
use dir::{ListDir, PathDir};

/// Same as [`std::env::current_dir`] except it uses a more descriptive error message
/// and returns `PathArc`.
///
/// [`std::env::current_dir`]: https://doc.rust-lang.org/beta/std/env/fn.set_current_dir.html
pub fn current_dir(resolving: &PathArc) -> Result<PathArc> {
    let cwd = env::current_dir().map_err(|e| {
        Error::new(
            e,
            "getting current_dir while resolving absolute",
            resolving.clone(),
        )
    })?;
    Ok(PathArc::from(cwd))
}

/// Converts any PrefixComponent into verbatim ("extended-length") form.
fn make_verbatim_prefix(prefix: &PrefixComponent) -> Result<PathBuf> {
    let path_prefix = Path::new(prefix.as_os_str());

    if prefix.kind().is_verbatim() {
        // This prefix already uses the extended-length
        // syntax, so we can use it as-is.
        Ok(path_prefix.to_path_buf())
    } else {
        // This prefix needs canonicalization.
        let res = path_prefix
            .canonicalize()
            .map_err(|e|
                Error::new(e, "canonicalizing", PathArc::new(path_prefix))
            )?;
        Ok(res)
    }
}

/// Pops the last component from path, returning an error for a root path.
fn pop_or_error(path: &mut PathBuf) -> ::std::result::Result<(), io::Error> {
    if path.pop() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, ".. consumed root"))
    }
}

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// A `PathBuf` that is atomically reference counted and reimplements the `PathBuf`
/// methods to display the action and path when there is an error.
///
/// This is the root type of all other `Path*` types in this crate.
///
/// This type is also serializable when the `serialize` feature is enabled.
pub struct PathArc(pub(crate) Arc<PathBuf>);

impl PathArc {
    /// Instantiate a new `PathArc`.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathArc;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let path = PathArc::new("some/path");
    /// let path2 = path.clone(); // cloning is cheap
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> PathArc {
        PathArc::from(path.as_ref().to_path_buf())
    }

    /// Creates an owned PathBuf with path adjoined to self.
    ///
    /// This function is identical to [std::path::PathBuf::join][0] except
    /// it returns `PathArc` instead of `PathBuf`
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.join
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathArc {
        PathArc::from(self.0.join(path))
    }

    /// Creates an owned `PathArc` like self but with the given file name.
    ///
    /// This function is identical to [std::path::PathBuf::with_file_name][0] except
    /// it returns `PathArc` instead of `PathBuf`
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.with_file_name
    pub fn with_file_name<P: AsRef<OsStr>>(&self, file_name: P) -> PathArc {
        PathArc::from(self.0.with_file_name(file_name))
    }

    /// Creates an owned `PathArc` like self but with the given extension.
    ///
    /// This function is identical to [std::path::PathBuf::with_extension][0] except
    /// it returns `PathArc` instead of `PathBuf`
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.with_extension
    pub fn with_extension<P: AsRef<OsStr>>(&self, extension: P) -> PathArc {
        PathArc::from(self.0.with_extension(extension))
    }

    /// Queries the file system to get information about a file, directory, etc.
    ///
    /// This function will traverse symbolic links to query information about the destination file.
    ///
    /// This function is identical to [std::path::Path::metadata][0] except it has error
    /// messages which include the action and the path
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.metadata
    pub fn metadata(&self) -> Result<fs::Metadata> {
        self.0
            .metadata()
            .map_err(|err| Error::new(err, "getting metadata of", self.clone()))
    }

    /// Queries the metadata about a file without following symlinks.
    ///
    /// This function is identical to [std::path::Path::symlink_metadata][0] except it has error
    /// messages which include the action and the path
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.symlink_metadata
    pub fn symlink_metadata(&self) -> Result<fs::Metadata> {
        self.0
            .symlink_metadata()
            .map_err(|err| Error::new(err, "getting symlink_metadata of", self.clone()))
    }

    /// Returns the canonical form of the path with all intermediate components normalized and
    /// symbolic links resolved.
    ///
    /// This function is identical to [std::path::Path::canonicalize][0] except:
    /// - It returns a `PathAbs` object
    /// - It has error messages which include the action and the path
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.canonicalize
    pub fn canonicalize(&self) -> Result<PathAbs> {
        let abs = self.0
            .canonicalize()
            .map_err(|err| Error::new(err, "canonicalizing", self.clone()))?;

        Ok(PathAbs(PathArc::from(abs)))
    }

    /// Reads a symbolic link, returning the file that the link points to.
    ///
    /// This function is identical to [std::path::Path::read_link][0] except:
    /// - It returns a `PathArc` object instead of `PathBuf`
    /// - It has error messages which include the action and the path
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.read_link
    pub fn read_link(&self) -> Result<PathArc> {
        let path = self.0
            .read_link()
            .map_err(|err| Error::new(err, "reading link", self.clone()))?;

        Ok(PathArc::from(path))
    }

    /// Returns an iterator over the entries within a directory.
    ///
    /// This function is a shortcut to `PathDir::list`. It is slightly different
    /// than [std::path::Path::read_dir][0].
    ///
    /// [0]: https://doc.rust-lang.org/std/path/struct.Path.html#method.read_dir
    pub fn read_dir(&self) -> Result<ListDir> {
        let dir = PathDir::new(self)?;
        dir.list()
    }

    /// Return a reference to a basic `std::path::Path`
    pub fn as_path(&self) -> &Path {
        self.as_ref()
    }

    /// Convert the path to an absolute one, this is different from [`canonicalize`] in that it
    /// _preserves_ symlinks and the destination may or may not exist.
    ///
    /// This function will:
    /// - Use [`current_dir`] to resolve relative paths.
    /// - Strip any `.` components (`/a/./c` -> `/a/c`)
    /// - Resolve `..` _semantically_ (not using the file system). So, `a/b/c/../d => a/b/d` will
    ///   _always_ be true regardless of symlinks. If you want symlinks correctly resolved, use
    ///   `canonicalize()` instead.
    ///
    /// > On windows, this will sometimes call `canonicalize()` on the first component to guarantee
    /// > it is the correct canonicalized prefix. For paths starting with root it also has to get
    /// > the [`current_dir`]
    ///
    /// > On linux, the only syscall this will make is to get the [`current_dir`] for relative
    /// > paths.
    ///
    /// [`canonicalize`]: struct.PathAbs.html#method.canonicalize
    /// [`current_dir`]: fn.current_dir.html
    pub fn absolute(&self) -> Result<PathAbs> {
        let mut res = PathBuf::new();

        fn maybe_init_res(res: &mut PathBuf, resolvee: &PathArc) -> Result<()> {
            if !res.as_os_str().is_empty() {
                // res has already been initialized, let's leave it alone.
                return Ok(());
            }

            // res has not been initialized, let's initialize it to the
            // canonicalized current directory.
            let cwd = env::current_dir().map_err(|e| {
                Error::new(
                    e,
                    "getting current_dir while resolving absolute",
                    resolvee.clone(),
                )
            })?;
            *res = cwd.canonicalize().map_err(|e| {
                Error::new(e, "canonicalizing", PathArc::new(&cwd))
            })?;

            Ok(())
        };

        for each in self.components() {
            match each {
                Component::Prefix(p) => {
                    // We don't care what's already in res, we can entirely
                    // replace it..
                    res = make_verbatim_prefix(&p)?;
                }

                Component::RootDir => {
                    if cfg!(windows) {
                        // In an ideal world, we would say
                        //
                        //  res = std::fs::canonicalize(each)?;
                        //
                        // ...to get a properly canonicalized path.
                        // Unfortunately, Windows cannot canonicalize `\` if
                        // the current directory happens to use extended-length
                        // syntax (like `\\?\C:\Windows`), so we'll have to do
                        // it manually: initialize `res` with the current
                        // working directory (whatever it is), and truncate it
                        // to its prefix by pushing `\`.
                        maybe_init_res(&mut res, self)?;
                        res.push(each);
                    } else {
                        // On other platforms, a root path component is always
                        // absolute so we can replace whatever's in res.
                        res = Path::new(&each).to_path_buf();
                    }
                }

                // This does nothing and can be ignored.
                Component::CurDir => (),

                Component::ParentDir => {
                    // A parent component is always relative to some existing
                    // path.
                    maybe_init_res(&mut res, self)?;
                    pop_or_error(&mut res)
                        .map_err(|e| {
                            Error::new(e, "resolving absolute", self.clone())
                        })?;
                }

                Component::Normal(c) => {
                    // A normal component is always relative to some existing
                    // path.
                    maybe_init_res(&mut res, self)?;
                    res.push(c);
                }
            }
        }

        Ok(PathAbs(PathArc(Arc::new(res))))
    }
}

impl fmt::Debug for PathArc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<Path> for PathArc {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl AsRef<PathBuf> for PathArc {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl Borrow<Path> for PathArc {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl Borrow<PathBuf> for PathArc {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl<'a> Borrow<Path> for &'a PathArc {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl<'a> Borrow<PathBuf> for &'a PathArc {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl Deref for PathArc {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

impl From<PathBuf> for PathArc {
    /// Instantiate a new `PathArc` from a `PathBuf`.
    fn from(path: PathBuf) -> PathArc {
        PathArc(Arc::new(path))
    }
}

impl Into<PathBuf> for PathArc {
    /// If there is only one reference to the `PathArc`, returns
    /// the inner `PathBuf`. Otherwise clones the inner `PathBuf`.
    ///
    /// This is useful when you really want a `PathBuf`, especially
    /// when the `PathArc` was only recently created.
    ///
    /// # Examples
    /// ```
    /// # extern crate path_abs;
    /// use path_abs::PathArc;
    /// use std::path::PathBuf;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let base = PathArc::new("base");
    /// let foo: PathBuf = base.join("foo.txt").into();
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    fn into(self) -> PathBuf {
        match Arc::try_unwrap(self.0) {
            Ok(p) => p,
            Err(inner) => inner.as_ref().clone(),
        }
    }
}

#[test]
fn test_prefix_windows() {
    fn f<P: AsRef<Path>>(p: P) -> Result<PathAbs> {
        PathArc::new(p).absolute()
    }
    assert!(f(r"\\?\C:\blah\blah").is_ok());
    assert!(f(r"\blah\blah").is_ok());
    assert!(f(r"C:\blah\blah").is_ok());

    // TODO: this is how to get the hostname, but getting the "share name"
    // seems to be more difficult.
    // let hostname = ::std::process::Command::new("hostname")
    //     .output()
    //     .expect("could not get hostname")
    //     .stdout;
    // let hostname = ::std::str::from_utf8(&hostname).unwrap().trim();

    // assert!(f(format!(r"\\{}\share", hostname)).is_ok());
    // assert!(f(format!(r"\\?\UNC\{}\share", hostname)).is_ok());
}
