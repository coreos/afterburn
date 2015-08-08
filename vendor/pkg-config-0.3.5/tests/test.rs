#![feature(std_misc)]

extern crate pkg_config;

use std::env;
use std::sync::{StaticMutex, MUTEX_INIT};
use std::path::PathBuf;

static LOCK: StaticMutex = MUTEX_INIT;

fn reset() {
    for (k, _) in env::vars() {
        if k.contains("PKG_CONFIG") || k.contains("DYNAMIC") ||
           k.contains("STATIC") {
            env::remove_var(&k);
        }
    }
    env::remove_var("TARGET");
    env::remove_var("HOST");
    env::set_var("PKG_CONFIG_PATH", &env::current_dir().unwrap().join("tests"));
}

fn find(name: &str) -> Result<pkg_config::Library, String> {
    pkg_config::find_library(name)
}

#[test]
fn cross_disabled() {
    let _g = LOCK.lock();
    reset();
    env::set_var("TARGET", "foo");
    env::set_var("HOST", "bar");
    find("foo").unwrap_err();
}

#[test]
fn cross_enabled() {
    let _g = LOCK.lock();
    reset();
    env::set_var("TARGET", "foo");
    env::set_var("HOST", "bar");
    env::set_var("PKG_CONFIG_ALLOW_CROSS", "1");
    find("foo").unwrap();
}

#[test]
fn package_disabled() {
    let _g = LOCK.lock();
    reset();
    env::set_var("FOO_NO_PKG_CONFIG", "1");
    find("foo").unwrap_err();
}

#[test]
fn output_ok() {
    let _g = LOCK.lock();
    reset();
    let lib = find("foo").unwrap();
    assert!(lib.libs.contains(&"gcc".to_string()));
    assert!(lib.libs.contains(&"coregrind-amd64-linux".to_string()));
    assert!(lib.link_paths.contains(&PathBuf::from("/usr/lib/valgrind")));
}

#[test]
fn framework() {
    let _g = LOCK.lock();
    reset();
    let lib = find("framework").unwrap();
    assert!(lib.frameworks.contains(&"foo".to_string()));
    assert!(lib.framework_paths.contains(&PathBuf::from("/usr/lib")));
}

#[test]
fn get_variable() {
    let _g = LOCK.lock();
    reset();
    let prefix = pkg_config::Config::get_variable("foo", "prefix").unwrap();
    assert_eq!(prefix, "/usr");
}

#[test]
fn version() {
    let _g = LOCK.lock();
    reset();
    assert_eq!(&find("foo").unwrap().version[..], "3.10.0.SVN");
}
