/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! The absolute paths have some gotchas that need to be tested.
//!
//! - Using the current working directory
//! - `..` paths that consume the "root"

#[macro_use]
extern crate pretty_assertions;
use tempdir;

use path_abs::*;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

#[test]
fn test_absolute() {
    if cfg!(windows) {
        let result = Path::new(r"\").canonicalize();
        assert!(
            result.is_ok(),
            "Should work before set_current_dir is called: {:?}",
            result
        );
    }
    let tmp = tempdir::TempDir::new("ex").unwrap();
    let tmp = tmp.path();
    let tmp_abs = PathAbs::new(&tmp).unwrap();
    env::set_current_dir(&tmp_abs).unwrap();
    if cfg!(windows) {
        let result = Path::new(r"\").canonicalize();
        assert!(result.is_err());
        println!("Got ERR cananonicalizing root: {}", result.unwrap_err());
    }

    // Create directory like:
    // a/
    // + e/ -> b/c/d
    // + b/
    //   + c/
    //     + d/

    let a = PathDir::create(&tmp.join("a")).unwrap();
    let b = PathDir::create(&a.concat("b").unwrap()).unwrap();
    let c = PathDir::create(&b.concat("c").unwrap()).unwrap();
    let d = PathDir::create(&c.concat("d").unwrap()).unwrap();

    // create symbolic link from a/e -> a/b/c/d
    let e_sym = d.symlink(&a.concat("e").unwrap()).unwrap();
    let ty = e_sym.symlink_metadata().unwrap().file_type();
    assert!(ty.is_symlink(), "{}", e_sym.display());

    assert_ne!(d, e_sym);
    assert_eq!(d, e_sym.canonicalize().unwrap());

    let a_cwd = Path::new("a");
    let b_cwd = a.concat("b").unwrap();
    let c_cwd = b.concat("c").unwrap();
    let d_cwd = c.concat("d").unwrap();
    let e_cwd = a.concat("e").unwrap();

    assert_eq!(a, PathDir::new(&a_cwd).unwrap());
    assert_eq!(b, PathDir::new(&b_cwd).unwrap());
    assert_eq!(c, PathDir::new(&c_cwd).unwrap());
    assert_eq!(d, PathDir::new(&d_cwd).unwrap());
    assert_eq!(e_sym, PathDir::new(&e_cwd).unwrap());

    assert_eq!(b, PathDir::new(c.concat("..").unwrap()).unwrap());
    assert_eq!(
        a,
        PathDir::new(c.concat("..").unwrap().concat("..").unwrap()).unwrap()
    );
    // just create a PathType
    let _ = PathType::new(&e_sym).unwrap();

    let mut root_dots: PathBuf = tmp_abs.clone().into();
    let mut dots = tmp_abs.components().count() - 1;
    if cfg!(windows) {
        // windows has _two_ "roots", prefix _and_ "root".
        dots -= 1;
    }
    for _ in 0..dots {
        root_dots.push("..");
    }
    let root = PathDir::new(root_dots).unwrap();
    if cfg!(windows) {
        assert_eq!(PathDir::new("\\").unwrap(), root);
    } else {
        assert_eq!(PathDir::new("/").unwrap(), root);
    }
    assert!(root.concat("..").is_err());

    if cfg!(windows) {
        // Test that /-separated and \-separated paths can be joined
        let ac1 = a.concat(r"b/c").unwrap();
        assert!(ac1.metadata().is_ok());

        let ac2 = a.concat(r"b\c").unwrap();
        assert!(ac2.metadata().is_ok());
    }
}

/// Check that issue #34 is fixed
///
/// After calling join(), the metadata are accessed to check that the computed path is valid.
#[test]
fn test_forward_and_backward_slashes() {
    let tmp = tempdir::TempDir::new("ex").unwrap();
    let tmp = tmp.path();

    // Create directories:
    // a/
    // + b/
    //   + c/
    let a = PathDir::create(&tmp.join("a")).unwrap();
    let b = PathDir::create(&a.concat("b").unwrap()).unwrap();
    let c = PathDir::create(&b.concat("c").unwrap()).unwrap();

    let a_abs = PathAbs::new(a).unwrap();

    // Join /-separated relative path and check that the metadata are accessible
    let forward_slash = a_abs.concat(r"b/c").unwrap();
    assert!(forward_slash.metadata().is_ok());
    assert_eq!(c, PathDir::new(forward_slash).unwrap());

    // Join \-separated relative path and check that the metadata are accessible
    // The following test only make sense on windows because the \ character isn't illegal in a
    // directory name on linux, so the call to `concat(r"b\c")` would just add the single directory
    // named "b\c"
    if cfg!(windows) {
        let backward_slash = a_abs.concat(r"b\c").unwrap();
        assert!(backward_slash.metadata().is_ok());
        assert_eq!(c, PathDir::new(backward_slash).unwrap());
    }
}

#[test]
fn test_root_parent() {
    let actual = PathAbs::new("/a/../..").expect_err("Can go outside of `/`?");
    assert_eq!(actual.io_error().kind(), io::ErrorKind::NotFound);
    assert_eq!(actual.action(), "resolving absolute");
    assert_eq!(actual.path(), Path::new(r"/a/../.."));
}

#[cfg_attr(windows, test)]
fn _test_root_parent_windows() {
    let actual = PathAbs::new(r"\a\..\..").expect_err(r"Can go outside of \?");
    assert_eq!(actual.io_error().kind(), io::ErrorKind::NotFound);
    assert_eq!(actual.action(), "resolving absolute");
    assert_eq!(actual.path(), Path::new(r"/a/../.."));

    let actual = PathAbs::new(r"C:\a\..\..").expect_err(r"Can go outside of C:\?");
    assert_eq!(actual.io_error().kind(), io::ErrorKind::NotFound);
    assert_eq!(actual.action(), "resolving absolute");
    assert_eq!(actual.path(), Path::new(r"C:\a\..\.."));

    let actual = PathAbs::new(r"\\?\C:\a\..\..").expect_err(r"Can go outside of \\?\C:\?");
    assert_eq!(actual.io_error().kind(), io::ErrorKind::NotFound);
    assert_eq!(actual.action(), "resolving absolute");
    assert_eq!(actual.path(), Path::new(r"\\?\C:\a\..\.."));
}
