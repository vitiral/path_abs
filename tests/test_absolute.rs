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

extern crate path_abs;
extern crate tempdir;
#[macro_use]
extern crate pretty_assertions;

use path_abs::*;
use std::path::Path;
use std::env;

#[test]
fn test_absolute() {
    let tmp = tempdir::TempDir::new("ex").unwrap();
    let tmp = tmp.path();
    let tmp_abs = PathArc::new(&tmp).canonicalize().unwrap();
    env::set_current_dir(&tmp_abs).unwrap();

    // Create directory like:
    // a/
    // + e/ -> b/c/d
    // + b/
    //   + c/
    //     + d/

    let a = PathDir::create(&tmp.join("a")).unwrap();
    let b = PathDir::create(&a.join("b")).unwrap();
    let c = PathDir::create(&b.join("c")).unwrap();
    let d = PathDir::create(&c.join("d")).unwrap();

    // create symbolic link from a/e -> a/b/c/d
    let e_sym = d.symlink(&a.join("e")).unwrap();
    let ty = e_sym.symlink_metadata().unwrap().file_type();
    assert!(ty.is_symlink(), "{}", e_sym.display());

    assert_ne!(d, e_sym);
    assert_eq!(d, e_sym.canonicalize().unwrap());

    let a_cwd = Path::new("a");
    let b_cwd = a.join("b");
    let c_cwd = b.join("c");
    let d_cwd = c.join("d");
    let e_cwd = a.join("e");

    assert_eq!(a, PathDir::new(&a_cwd).unwrap());
    assert_eq!(b, PathDir::new(&b_cwd).unwrap());
    assert_eq!(c, PathDir::new(&c_cwd).unwrap());
    assert_eq!(d, PathDir::new(&d_cwd).unwrap());
    assert_eq!(e_sym, PathDir::new(&e_cwd).unwrap());

    assert_eq!(b, PathDir::new(c.join("..")).unwrap());
    assert_eq!(a, PathDir::new(c.join("..").join("..")).unwrap());
    // just create a PathType
    let _ = PathType::new(&e_sym).unwrap();

    let mut root_dots = tmp_abs.to_path_buf();
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
    assert!(PathDir::new(root.join("..")).is_err());
}
