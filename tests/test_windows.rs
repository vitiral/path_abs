/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Test assumptions about windows
#![cfg_attr(not(windows), allow(dead_code))]
extern crate path_abs;
extern crate tempdir;
#[macro_use]
extern crate pretty_assertions;

use std::path::Path;

macro_rules! expect_err {
    [$s:expr] => {{
        let p = Path::new($s);
        match p.canonicalize() {
            Ok(p) => {
                panic!("Got {:?} when canonicalizing {:?}, expected err", p, $s);
            }
            Err(err) => {
                println!("EXPECTED ERR Canonicalizing {:?} => {}", $s, err);
            }
        }
    }}
}

macro_rules! expect_path {
    [$expect:expr, $s:expr] => {{
        let expect = Path::new($expect);
        let p = Path::new($s);
        match p.canonicalize() {
            Ok(p) => {
                assert_eq!(expect, p);
                println!("EXPECTED OK Canonicalizing {:?} => {:?}", $s, p);
            }
            Err(err) => {
                panic!("Got {:?} when canonicalizing {:?}, expected {:?}", err, $s, $expect);
            }
        }
    }}
}

fn share() -> String {
    if cfg!(windows) {
        let shared = ::std::process::Command::new("wmic")
            .arg("share")
            .arg("get")
            .arg("caption,name,path")
            .output()
            .expect("could not `wmic share`")
            .stdout;
        let out = ::std::str::from_utf8(&shared).unwrap().trim().to_string();
        println!("### SHARED:\n{}\n###", out);
        out
    } else {
        "NONE SHARED".to_string()
    }
}

fn hostname() -> String {
    let hostname = ::std::process::Command::new("hostname")
        .output()
        .expect("could not get hostname")
        .stdout;
    let out = ::std::str::from_utf8(&hostname).unwrap().trim().to_string();
    println!("HOSTNAME: {}", out);
    out
}

#[cfg_attr(windows, test)]
fn canonicalize_root() {
    expect_path!(r"\\?\C:\", r"\");
}

#[cfg_attr(windows, test)]
fn canonicalize_verbatim() {
    println!("CURRENT DIR: {}", ::std::env::current_dir().unwrap().display());
    // TODO:
    // EXPECTED ERR Canonicalizing "\\\\?\\project" => The system cannot find the file specified.
    // (os error 2)
    expect_err!(r"\\?\project");
}

#[cfg_attr(windows, test)]
fn canonicalize_verbatim_unc() {
    // TODO: current result:
    // EXPECTED ERR Canonicalizing "\\\\?\\APPVYR-WIN\\share" => The system cannot find the path
    // specified. (os error 3)

    let _ = share(); // FIXME: just printing for now
    let p = format!(r"\\?\{}\share", hostname());
    expect_err!(&p);
}

#[cfg_attr(windows, test)]
fn canonicalize_verbatim_disk() {
    let with_root = r"\\?\C:\";
    expect_path!(with_root, with_root);
    expect_err!(r"\\?\C:")
}

#[cfg_attr(windows, test)]
fn canonicalize_device_ns() {
    // TODO: EXPECTED ERR Canonicalizing "\\\\.\\com1" => The system cannot find the file
    // specified. (os error 2)
    expect_err!(r"\\.\COM1")
}

#[cfg_attr(windows, test)]
fn canonicalize_unc() {
    // TODO:
    // canonicalize_unc' panicked at 'called `Result::unwrap()` on an `Err` value: Error { repr:
    // Os { code: 67, message: "The network name cannot be found." }
    let h = hostname();
    let unc = format!(r"\\{}\share", h);
    let verbatim = format!(r"\\?\{}\share", h);
    let result = Path::new(&unc).canonicalize().unwrap();
    assert_eq!(Path::new(&verbatim), result);
}

#[cfg_attr(windows, test)]
fn canonicalize_disk() {
    expect_path!(r"\\?\C:\", r"C:\")
}