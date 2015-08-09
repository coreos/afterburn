// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
//138
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct INITCOMMONCONTROLSEX {
    pub dwSize: ::DWORD,
    pub dwICC: ::DWORD,
}
pub type LPINITCOMMONCONTROLSEX = *mut INITCOMMONCONTROLSEX;
pub const ICC_LISTVIEW_CLASSES: ::DWORD = 0x1;
pub const ICC_TREEVIEW_CLASSES: ::DWORD = 0x2;
pub const ICC_BAR_CLASSES: ::DWORD = 0x4;
pub const ICC_TAB_CLASSES: ::DWORD = 0x8;
pub const ICC_UPDOWN_CLASS: ::DWORD = 0x10;
pub const ICC_PROGRESS_CLASS: ::DWORD = 0x20;
pub const ICC_HOTKEY_CLASS: ::DWORD = 0x40;
pub const ICC_ANIMATE_CLASS: ::DWORD = 0x80;
pub const ICC_WIN95_CLASSES: ::DWORD = 0xFF;
pub const ICC_DATE_CLASSES: ::DWORD = 0x100;
pub const ICC_USEREX_CLASSES: ::DWORD = 0x200;
pub const ICC_COOL_CLASSES: ::DWORD = 0x400;
pub const ICC_INTERNET_CLASSES: ::DWORD = 0x800;
pub const ICC_PAGESCROLLER_CLASS: ::DWORD = 0x1000;
pub const ICC_NATIVEFNTCTL_CLASS: ::DWORD = 0x2000;
pub const ICC_STANDARD_CLASSES: ::DWORD = 0x4000;
pub const ICC_LINK_CLASS: ::DWORD = 0x8000;
