/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! The absolute path type, the root type for _most_ `Path*` types in this module
//! (except for `PathArc`).
use std::env;
use std::fmt;
use std::io;
use std::path::{Component, PrefixComponent};
use std_prelude::*;

use super::{PathArc, PathDir, PathFile, Error, Result};

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
/// An absolute (not _necessarily_ [canonicalized][1]) path that may or may not exist.
///
/// [1]: https://doc.rust-lang.org/std/path/struct.Path.html?search=#method.canonicalize
pub struct PathAbs(pub(crate) PathArc);

impl PathAbs {
    /// Construct an absolute path from an arbitrary (absolute or relative) one.
    ///
    /// This is different from [`canonicalize`] in that it _preserves_ symlinks
    /// and the destination may or may not exist.
    ///
    /// This function will:
    /// - Resolve relative paths against the current working directory.
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use path_abs::PathAbs;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let lib = PathAbs::new("src/lib.rs")?;
    ///
    /// assert_eq!(lib.is_absolute(), true);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<PathAbs> {
        let path = PathArc::new(path);
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

        for each in path.components() {
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
                        maybe_init_res(&mut res, &path)?;
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
                    maybe_init_res(&mut res, &path)?;
                    pop_or_error(&mut res)
                        .map_err(|e| {
                            Error::new(e, "resolving absolute", path.clone())
                        })?;
                }

                Component::Normal(c) => {
                    // A normal component is always relative to some existing
                    // path.
                    maybe_init_res(&mut res, &path)?;
                    res.push(c);
                }
            }
        }

        Ok(PathAbs(PathArc(Arc::new(res))))
    }

    /// Resolve the `PathAbs` as a `PathFile`. Return an error if it is not a file.
    pub fn into_file(self) -> Result<PathFile> {
        PathFile::from_abs(self)
    }

    /// Resolve the `PathAbs` as a `PathDir`. Return an error if it is not a directory.
    pub fn into_dir(self) -> Result<PathDir> {
        PathDir::from_abs(self)
    }

    /// Return a reference to a basic `std::path::Path`
    pub fn as_path(&self) -> &Path {
        self.as_ref()
    }

    /// For constructing mocked paths during tests. This is effectively the same as a `PathBuf`.
    ///
    /// This is NOT checked for validity so the file may or may not actually exist and will
    /// NOT be, in any way, an absolute or canonicalized path.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate path_abs;
    /// use path_abs::PathAbs;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// // this file exist
    /// let lib = PathAbs::new("src/lib.rs")?;
    ///
    /// let lib_mocked = PathAbs::mock("src/lib.rs");
    ///
    /// // in this case, the mocked file exists
    /// assert!(lib_mocked.exists());
    ///
    /// // However, it is NOT equivalent to `lib`
    /// assert_ne!(lib, lib_mocked);
    ///
    /// // this file doesn't exist at all
    /// let dne = PathAbs::mock("src/dne.rs");
    /// assert!(!dne.exists());
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn mock<P: AsRef<Path>>(fake_path: P) -> PathAbs {
        PathAbs(PathArc::new(fake_path))
    }
}

impl fmt::Debug for PathAbs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<PathArc> for PathAbs {
    fn as_ref(&self) -> &PathArc {
        &self.0
    }
}

impl AsRef<Path> for PathAbs {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<PathBuf> for PathAbs {
    fn as_ref(&self) -> &PathBuf {
        self.0.as_ref()
    }
}

impl Borrow<PathArc> for PathAbs {
    fn borrow(&self) -> &PathArc {
        self.as_ref()
    }
}

impl Borrow<Path> for PathAbs {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl Borrow<PathBuf> for PathAbs {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl<'a> Borrow<PathArc> for &'a PathAbs {
    fn borrow(&self) -> &PathArc {
        self.as_ref()
    }
}

impl<'a> Borrow<Path> for &'a PathAbs {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl<'a> Borrow<PathBuf> for &'a PathAbs {
    fn borrow(&self) -> &PathBuf {
        self.as_ref()
    }
}

impl Deref for PathAbs {
    type Target = PathArc;

    fn deref(&self) -> &PathArc {
        &self.0
    }
}

impl From<PathAbs> for PathArc {
    fn from(path: PathAbs) -> PathArc {
        path.0
    }
}

impl From<PathAbs> for Arc<PathBuf> {
    fn from(path: PathAbs) -> Arc<PathBuf> {
        let arc: PathArc = path.into();
        arc.0
    }
}

impl From<PathAbs> for PathBuf {
    fn from(path: PathAbs) -> PathBuf {
        let arc: PathArc = path.into();
        arc.into()
    }
}
