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

fn hostname() -> String {
    let hostname = ::std::process::Command::new("hostname")
        .output()
        .expect("could not get hostname")
        .stdout;
    ::std::str::from_utf8(&hostname).unwrap().trim().to_string()
}

#[cfg_attr(windows, test)]
fn cannonicalize_root() {
    expect_err!(r"\");
}

#[cfg_attr(windows, test)]
fn cannonicalize_verbatim() {
    expect_err!(r"\\?\project");
}

#[cfg_attr(windows, test)]
fn cannonicalize_verbatim_unc() {
    let p = format!(r"\\?\{}\share", hostname());
    expect_err!(&p);
}

#[cfg_attr(windows, test)]
fn cannonicalize_verbatim_disk() {
    expect_err!(r"\\?\C:\")
}

#[cfg_attr(windows, test)]
fn cannonicalize_device_ns() {
    expect_err!(r"\\.\com1")
}

#[cfg_attr(windows, test)]
fn cannonicalize_unc() {
    let h = hostname();
    let unc = format!(r"\\{}\share", h);
    let verbatim = format!(r"\\?\{}\share", h);
    let result = Path::new(&unc).canonicalize().unwrap();
    assert_eq!(Path::new(&verbatim), result);
}

#[cfg_attr(windows, test)]
fn cannonicalize_disk() {
    let result = Path::new(r"C:\").canonicalize().unwrap();
    assert_eq!(Path::new(r"\\?\C:\"), result);
}
