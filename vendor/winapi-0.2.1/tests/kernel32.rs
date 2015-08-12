// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
#![cfg(windows)]
extern crate kernel32;
use kernel32::*;
#[inline(never)] fn bb<T>(_: T) {}
#[test]
fn functions() {
    bb(GetStartupInfoA);
    bb(GetStartupInfoW);
    bb(OpenEventA);
    bb(OpenEventW);
    bb(ResetEvent);
    bb(SetEvent);
    bb(WaitForMultipleObjects);
    bb(WaitForMultipleObjectsEx);
    bb(WaitForSingleObject);
    bb(WaitForSingleObjectEx);
}
