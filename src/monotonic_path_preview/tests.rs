extern crate env_logger;
extern crate tempfile;

use super::*;
use std::env;
use std::fs;

#[test]
fn relative_accepts_empty_path() {
    let actual = Relative::new("").expect("Could not parse empty path");

    let expected = path::Path::new("");

    assert_eq!(actual.0, expected);
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::os::windows::fs as winfs;

    fn symlink_dir<P, Q>(src: P, dst: Q)
    where
        P: AsRef<path::Path>,
        Q: AsRef<path::Path>,
    {
        let dst = dst.as_ref();

        winfs::symlink_dir(src, &dst).expect(
            "Could not create symbolic link. \
             Run as Administrator, or on Windows 10 in Developer Mode",
        );
        let link_exists = dst.symlink_metadata().expect(
            "Link creation succeeded, but can't read link? \
             If you're using Wine, see bug 44948",
        );
    }

    #[test]
    fn absolute_path_gets_canonical_prefix() {
        let _ = env_logger::try_init();

        let path = Absolute::new("C:\\foo\\bar")
            .expect("Could not handle an absolute path");

        assert_eq!(path.0, path::Path::new("\\\\?\\C:\\foo\\bar"));
    }

    #[test]
    fn missing_absolute_path_with_double_dot_is_normalized() {
        let _ = env_logger::try_init();

        let path = Absolute::new("C:\\foo\\..\\bar")
            .expect("Could not handle an absolute path");

        assert_eq!(path.0, path::Path::new("\\\\?\\C:\\bar"));
    }

    #[test]
    fn absolute_path_with_single_dot_is_dropped() {
        let _ = env_logger::try_init();

        let path = Absolute::new("C:\\foo\\.\\bar")
            .expect("Could not handle an absolute path");

        assert_eq!(path.0, path::Path::new("\\\\?\\C:\\foo\\bar"));
    }

    #[test]
    fn absolute_path_with_slashes_is_normalized() {
        let _ = env_logger::try_init();

        let path = Absolute::new("C:/foo/bar")
            .expect("Could not handle an absolute path");

        assert_eq!(path.0, path::Path::new("\\\\?\\C:\\foo\\bar"));
    }

    #[test]
    fn relative_path_is_relative_to_current_directory() {
        let _ = env_logger::try_init();

        let path =
            Absolute::new("foo").expect("Could not handle a relative path");

        let current_dir = env::current_dir()
            .expect("Could not read current directory.")
            .canonicalize()
            .expect("Could not canonicalize current directory.");

        let expected = current_dir.join("foo");

        assert_eq!(path.0, expected);
    }

    #[test]
    fn leading_double_dot_in_relative_path() {
        let _ = env_logger::try_init();

        let path =
            Absolute::new("..\\foo").expect("Could not handle a relative path");

        let current_dir = env::current_dir()
            .expect("Could not read current directory.")
            .canonicalize()
            .expect("Could not canonicalize current directory.");

        let expected = current_dir
            .parent()
            .unwrap_or(path::Path::new("/"))
            .join("foo");

        assert_eq!(path.0, expected);
    }

    #[test]
    fn skip_leading_dot_in_relative_path() {
        let _ = env_logger::try_init();

        let path =
            Absolute::new(".\\foo").expect("Could not handle a relative path");

        let current_dir = env::current_dir()
            .expect("Could not read current directory.")
            .canonicalize()
            .expect("Could not canonicalize current directory.");

        let expected = current_dir.join("foo");

        assert_eq!(path.0, expected);
    }

    #[test]
    fn drive_relative_path_is_relative_to_current_drive() {
        let _ = env_logger::try_init();

        let actual = Absolute::new("\\foo")
            .expect("Could not handle a prefix-relative path");

        let current_dir =
            env::current_dir().expect("Could not read current directory.");

        let mut current_drive_root = path::PathBuf::new();
        current_drive_root.push(
            current_dir
                .components()
                .next()
                .expect("Current directory is an empty path?"),
        );
        current_drive_root.push("\\");

        let expected = fs::canonicalize(current_drive_root)
            .expect("Could not canonicalize current drive")
            .join("foo");

        assert_eq!(actual.0, expected);
    }

    /*

    This test can't work without having more than one valid drive letter,
    which we can't possibly guarantee.

    #[test]
    fn prefixed_relative_path_is_relative_to_drive_directory() {
        let _ = env_logger::try_init();

        // Save the current directory on the current drive.
        let original_dir =
            env::current_dir().expect("Could not read current directory.");

        // Remember which drive that's on.
        let original_drive = original_dir
            .components()
            .next()
            .expect("Current directory is an empty path?")
            .as_os_str();

        // Switch to a different drive.
        assert_ne!(original_drive, "A:");
        env::set_current_dir("A:\\")
            .expect("Could not set current directory to A:\\");
    }
    */

    #[test]
    fn do_not_resolve_every_symlink() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        let temp_path = temp.into_path();

        let target_path = temp_path.join("target");
        let link_path = temp_path.join("link");

        // Create an empty file as the target.
        println!("Creating target path: {:?}", target_path);
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&target_path)
            .expect("Could not create target");
        assert_eq!(target_path.exists(), true);

        println!("Creating link path: {:?}", link_path);
        symlink_dir(&target_path, &link_path);

        println!("Absolutizing symlink path");
        let actual =
            Absolute::new(&link_path).expect("Could not handle symlink path");
        println!("Actual: {:?}", actual.0);

        let expected = temp_path
            .canonicalize()
            .expect("Could not canonicalize symlink path")
            .join("link");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn resolve_symlink_before_parentdir_component() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        // apple/ant is a directory that actually exists on disk.
        fs::create_dir_all(temp.path().join("apple/ant"))
            .expect("Could not create Path A");

        // banana/ exists, but banana/bat is just a symlink to apple/ant.
        fs::create_dir_all(temp.path().join("banana"))
            .expect("Could not create Path B");
        symlink_dir("../apple/ant", temp.path().join("banana/bat"));

        let actual = Absolute::new(temp.path().join("banana/bat/../tail"))
            .expect("Could not handle a symlinked path.");

        let expected = temp.path()
            .canonicalize()
            .expect("Could not canonicalize temp path")
            .join("apple\\tail");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn cannot_go_up_from_root_directory() {
        let _ = env_logger::try_init();

        let actual = Absolute::new("C:\\..\\foo")
            .expect("Could not handle going up from the root directory");

        let expected = fs::canonicalize("C:\\")
            .expect("No C:\\ directory?")
            .join("foo");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn resolve_absolute_symlinks() {
        let _ = env_logger::try_init();

        let temp = tempfile::tempdir()
            .expect("Could not make temporary directory.")
            .into_path();

        let link_path = temp.join("link");

        debug!("Creating link at: {:?}", link_path);

        symlink_dir("C:\\does\\not\\exist", &link_path);

        let actual = Absolute::new(link_path.join("..\\bar"))
            .expect("Could not handle symlink with absolute target");

        let expected = fs::canonicalize("C:\\")
            .expect("No C:\\ directory?")
            .join("does\\not\\bar");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn absolute_rejects_dereferencing_cyclic_symlinks() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        symlink_dir("a", temp.path().join("b"));
        symlink_dir("b", temp.path().join("a"));

        let err = Absolute::new(temp.path().join("a/../tail"))
            .expect_err("Cyclic symlinks dereferenced?");

        match err {
            // This is what we expected.
            Error::SymlinkLoops(_) => (),
            // Uh oh.
            e => panic!("Got unexpected error: {:?}", e),
        }
    }

    #[test]
    fn absolute_accepts_navigating_cyclic_symlinks() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        symlink_dir(".", temp.path().join("cur"));

        let actual = Absolute::new(temp.path().join("cur/cur/cur/cur/cur/a"))
            .expect("Could not handle cyclic path.");

        let expected = temp.path()
            .canonicalize()
            .expect("Could not canonicalize temp dir")
            .join("cur\\cur\\cur\\cur\\cur\\a");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn absolute_rejects_invalid_prefix() {
        let _ = env_logger::try_init();

        let err = Absolute::new("\\\\?\\bogus\\path\\")
            .map(|p| {
                panic!("Parsing bogus path should have failed! {:?}", p);
            })
            .err()
            .unwrap();

        match err {
            Error::IoError { err, at } => {
                assert_eq!(err.kind(), io::ErrorKind::NotFound);
                assert_eq!(at, path::Path::new("\\\\?\\bogus\\path"));
            }
            e => panic!("Got unexpected error {:?}", e),
        }
    }

    #[test]
    fn relative_accepts_monotonic_path() {
        let _ = env_logger::try_init();

        let actual = Relative::new("does\\not\\exist")
            .expect("Could not parse monotonic path");

        let expected = path::Path::new("does\\not\\exist");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn relative_normalises_slashes() {
        let _ = env_logger::try_init();

        let actual = Relative::new("does/not/exist")
            .expect("Could not parse path with slashes");

        let expected = path::Path::new("does\\not\\exist");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn relative_normalises_single_dots() {
        let _ = env_logger::try_init();

        let actual = Relative::new("does\\.\\not\\.\\exist")
            .expect("Could not parse path with CurrentDir components");

        let expected = path::Path::new("does\\not\\exist");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn relative_normalises_inner_double_dots() {
        let _ = env_logger::try_init();

        let actual = Relative::new("does\\..\\not\\exist")
            .expect("Could not parse path with ParentDir component");

        let expected = path::Path::new("not\\exist");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn relative_rejects_leading_double_dots() {
        let _ = env_logger::try_init();

        let err = Relative::new("..\\does\\not\\exist")
            .err()
            .expect("Parsing leading '..' should have failed!");

        match err {
            Error::RelativePathEscapesPrefix(p) => {
                assert_eq!(p, path::Path::new("..\\does\\not\\exist"))
            }
            e => panic!("Got unexpected error {:?}", e),
        }
    }

    #[test]
    fn relative_rejects_fully_absolute_path() {
        let _ = env_logger::try_init();

        let err = Relative::new("C:\\absolute\\path")
            .err()
            .expect("Parsing an absolute path should have failed!");

        match err {
            Error::PathIsAbsolute(p) => {
                assert_eq!(p, path::Path::new("C:\\absolute\\path"))
            }
            e => panic!("Got unexpected error {:?}", e),
        }
    }

    #[test]
    fn relative_rejects_prefix_relative_path() {
        let _ = env_logger::try_init();

        let err = Relative::new("\\relative\\path")
            .err()
            .expect("Parsing prefix-relative path should have failed!");

        match err {
            Error::PathIsAbsolute(p) => {
                assert_eq!(p, path::Path::new("\\relative\\path"))
            }
            e => panic!("Got unexpected error {:?}", e),
        }
    }

    #[test]
    fn relative_rejects_drive_relative_path() {
        let _ = env_logger::try_init();

        let err = Relative::new("C:relative\\path")
            .err()
            .expect("Parsing drive-relative path should have failed!");

        match err {
            Error::PathIsAbsolute(p) => {
                assert_eq!(p, path::Path::new("C:relative\\path"))
            }
            e => panic!("Got unexpected error {:?}", e),
        }
    }

    #[test]
    fn relative_rejects_relative_path_escaping_prefix() {
        let _ = env_logger::try_init();

        let err = Relative::new("inside\\..\\..\\outside")
            .err()
            .expect("Parsing escaping path should have failed!");

        match err {
            Error::RelativePathEscapesPrefix(p) => {
                assert_eq!(p, path::Path::new("inside\\..\\..\\outside"))
            }
            e => panic!("Got unexpected error {:?}", e),
        }
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::fs as unixfs;

    #[test]
    fn missing_absolute_path_is_unchanged() {
        let _ = env_logger::try_init();

        let expected = path::Path::new("/some/absolute/path");

        let path = Absolute::new(&expected)
            .expect("Could not handle an absolute path");

        assert_eq!(path.0, expected);
    }

    #[test]
    fn missing_absolute_path_with_double_dot_is_normalized() {
        let _ = env_logger::try_init();

        let actual = Absolute::new("/some/absolute/../path")
            .expect("Could not handle an absolute path");
        let expected = path::Path::new("/some/path");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn existing_path_with_double_dot_is_normalized() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        fs::create_dir_all(temp.path().join("apple"))
            .expect("Could not create apple directory.");
        fs::create_dir_all(temp.path().join("banana"))
            .expect("Could not create apple directory.");

        let actual = Absolute::new(temp.path().join("banana/../apple"))
            .expect("Could not resolve existing directories.");

        let expected = temp.path().join("apple");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn missing_relative_path_is_relative_to_current_directory() {
        let _ = env_logger::try_init();

        let actual = Absolute::new("does/not/exist")
            .expect("Could not handle a missing relative path");

        let expected = env::current_dir()
            .expect("Could not get current directory")
            .join("does/not/exist");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn skip_leading_dots() {
        let _ = env_logger::try_init();

        let actual = Absolute::new("/some/./missing/./path")
            .expect("Could not handle a path with dot components");

        let expected = path::Path::new("/some/missing/path");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn leading_double_dot_in_relative_path() {
        let _ = env_logger::try_init();

        let actual = Absolute::new("../does/not/exist")
            .expect("Could not handle a relative path");

        let expected = env::current_dir()
            .expect("Could not get current directory")
            .parent()
            .unwrap_or(path::Path::new("/"))
            .join("does/not/exist");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn do_not_resolve_every_symlink() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        let target_path = temp.path().join("target");
        let link_path = temp.path().join("link");

        // Create an empty file as the target.
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&target_path)
            .expect("Could not create target");

        unixfs::symlink(&target_path, &link_path)
            .expect("Could not create symbolic link.");

        let actual =
            Absolute::new(&link_path).expect("Could not handle symlink path");

        assert_eq!(actual.0, link_path);
    }

    #[test]
    fn resolve_symlink_before_parentdir_component() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        // apple/ant is a directory that actually exists on disk.
        fs::create_dir_all(temp.path().join("apple/ant"))
            .expect("Could not create Path A");

        // banana/ exists, but banana/bat is just a symlink to apple/ant.
        fs::create_dir_all(temp.path().join("banana"))
            .expect("Could not create Path B");
        unixfs::symlink("../apple/ant", temp.path().join("banana/bat"))
            .expect("Could not create symlink.");

        let actual = Absolute::new(temp.path().join("banana/bat/../tail"))
            .expect("Could not handle a symlinked path.");

        let expected = temp.path().join("apple/tail");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn cannot_go_up_from_root_directory() {
        let _ = env_logger::try_init();

        let actual = Absolute::new("/../foo")
            .expect("Could not handle going up from the root directory");
        let expected = path::Path::new("/foo");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn resolve_absolute_symlinks() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        unixfs::symlink("/does/not/exist", temp.path().join("link"))
            .expect("Could not create symlink");

        let actual = Absolute::new(temp.path().join("link/../bar"))
            .expect("Could not handle symlink with absolute target");

        let expected = path::Path::new("/does/not/bar");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn absolute_rejects_dereferencing_cyclic_symlinks() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        unixfs::symlink("a", temp.path().join("b"))
            .expect("Could not create symlink b->a");
        unixfs::symlink("b", temp.path().join("a"))
            .expect("Could not create symlink a->b");

        let err = Absolute::new(temp.path().join("a/../tail"))
            .expect_err("Cyclic symlinks dereferenced?");

        match err {
            // This is what we expected.
            Error::SymlinkLoops(_) => (),
            // Uh oh.
            e => panic!("Got unexpected error: {:?}", e),
        }
    }

    #[test]
    fn absolute_accepts_navigating_cyclic_symlinks() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        unixfs::symlink(".", temp.path().join("cur"))
            .expect("Could not create symilnk pointing at parent");

        let expected = temp.path().join("cur/cur/cur/cur/cur/a");

        let actual =
            Absolute::new(&expected).expect("Could not handle cyclic path.");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn absolute_reports_permission_error() {
        let _ = env_logger::try_init();

        let temp =
            tempfile::tempdir().expect("Could not make temporary directory.");

        fs::create_dir_all(temp.path().join("dir/secret"))
            .expect("Could not create directory");

        // Make "dir" unreadable.
        use self::unixfs::PermissionsExt;
        fs::set_permissions(
            temp.path().join("dir"),
            fs::Permissions::from_mode(0),
        ).expect("Could not set permissions");

        // Now, trying to stat 'secret' should give permission denied.
        let err = Absolute::new(temp.path().join("dir/secret/.."))
            .map(|p| {
                panic!(
                    "Getting absolute path should have failed: {:?}",
                    p
                );
            })
            .err()
            .unwrap();

        match err {
            Error::IoError { err, at } => {
                assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
                assert_eq!(at, temp.path().join("dir/secret"));
            }
            e => panic!("Got unexpected error: {:?}", e),
        }
    }

    #[test]
    fn relative_accepts_monotonic_path() {
        let actual = Relative::new("does/not/exist")
            .expect("Could not parse monotonic path");

        let expected = path::Path::new("does/not/exist");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn relative_normalises_single_dots() {
        let actual = Relative::new("does/./not/./exist")
            .expect("Could not parse path with CurrentDir components");

        let expected = path::Path::new("does/not/exist");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn relative_normalises_inner_double_dots() {
        let actual = Relative::new("does/../not/exist")
            .expect("Could not parse path with ParentDir component");

        let expected = path::Path::new("not/exist");

        assert_eq!(actual.0, expected);
    }

    #[test]
    fn relative_rejects_leading_double_dots() {
        let err = Relative::new("../does/not/exist")
            .err()
            .expect("Parsing leading '..' should have failed!");

        match err {
            Error::RelativePathEscapesPrefix(p) => {
                assert_eq!(p, path::Path::new("../does/not/exist"))
            }
            e => panic!("Got unexpected error {:?}", e),
        }
    }

    #[test]
    fn relative_rejects_relative_path_escaping_prefix() {
        let err = Relative::new("inside/../../outside")
            .err()
            .expect("Parsing escaping path should have failed!");

        match err {
            Error::RelativePathEscapesPrefix(p) => {
                assert_eq!(p, path::Path::new("inside/../../outside"))
            }
            e => panic!("Got unexpected error {:?}", e),
        }
    }

    #[test]
    fn relative_rejects_fully_absolute_path() {
        let err = Relative::new("/absolute/path")
            .err()
            .expect("Parsing an absolute path should have failed!");

        match err {
            Error::PathIsAbsolute(p) => {
                assert_eq!(p, path::Path::new("/absolute/path"))
            }
            e => panic!("Got unexpected error {:?}", e),
        }
    }
}
