// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>

// Flags used by the various UIs
pub const SHTDN_REASON_FLAG_COMMENT_REQUIRED: ::DWORD = 0x01000000;
pub const SHTDN_REASON_FLAG_DIRTY_PROBLEM_ID_REQUIRED: ::DWORD = 0x02000000;
pub const SHTDN_REASON_FLAG_CLEAN_UI: ::DWORD = 0x04000000;
pub const SHTDN_REASON_FLAG_DIRTY_UI: ::DWORD = 0x08000000;

// Flags that end up in the event log code.
pub const SHTDN_REASON_FLAG_USER_DEFINED: ::DWORD = 0x40000000;
pub const SHTDN_REASON_FLAG_PLANNED: ::DWORD = 0x80000000;

// Microsoft major reasons.
pub const SHTDN_REASON_MAJOR_OTHER: ::DWORD = 0x00000000;
pub const SHTDN_REASON_MAJOR_NONE: ::DWORD = 0x00000000;
pub const SHTDN_REASON_MAJOR_HARDWARE: ::DWORD = 0x00010000;
pub const SHTDN_REASON_MAJOR_OPERATINGSYSTEM: ::DWORD = 0x00020000;
pub const SHTDN_REASON_MAJOR_SOFTWARE: ::DWORD = 0x00030000;
pub const SHTDN_REASON_MAJOR_APPLICATION: ::DWORD = 0x00040000;
pub const SHTDN_REASON_MAJOR_SYSTEM: ::DWORD = 0x00050000;
pub const SHTDN_REASON_MAJOR_POWER: ::DWORD = 0x00060000;
pub const SHTDN_REASON_MAJOR_LEGACY_API: ::DWORD = 0x00070000;

// Microsoft minor reasons.
pub const SHTDN_REASON_MINOR_OTHER: ::DWORD = 0x00000000;
pub const SHTDN_REASON_MINOR_NONE: ::DWORD = 0x000000ff;
pub const SHTDN_REASON_MINOR_MAINTENANCE: ::DWORD = 0x00000001;
pub const SHTDN_REASON_MINOR_INSTALLATION: ::DWORD = 0x00000002;
pub const SHTDN_REASON_MINOR_UPGRADE: ::DWORD = 0x00000003;
pub const SHTDN_REASON_MINOR_RECONFIG: ::DWORD = 0x00000004;
pub const SHTDN_REASON_MINOR_HUNG: ::DWORD = 0x00000005;
pub const SHTDN_REASON_MINOR_UNSTABLE: ::DWORD = 0x00000006;
pub const SHTDN_REASON_MINOR_DISK: ::DWORD = 0x00000007;
pub const SHTDN_REASON_MINOR_PROCESSOR: ::DWORD = 0x00000008;
pub const SHTDN_REASON_MINOR_NETWORKCARD: ::DWORD = 0x00000009;
pub const SHTDN_REASON_MINOR_POWER_SUPPLY: ::DWORD = 0x0000000a;
pub const SHTDN_REASON_MINOR_CORDUNPLUGGED: ::DWORD = 0x0000000b;
pub const SHTDN_REASON_MINOR_ENVIRONMENT: ::DWORD = 0x0000000c;
pub const SHTDN_REASON_MINOR_HARDWARE_DRIVER: ::DWORD = 0x0000000d;
pub const SHTDN_REASON_MINOR_OTHERDRIVER: ::DWORD = 0x0000000e;
pub const SHTDN_REASON_MINOR_BLUESCREEN: ::DWORD = 0x0000000F;
pub const SHTDN_REASON_MINOR_SERVICEPACK: ::DWORD = 0x00000010;
pub const SHTDN_REASON_MINOR_HOTFIX: ::DWORD = 0x00000011;
pub const SHTDN_REASON_MINOR_SECURITYFIX: ::DWORD = 0x00000012;
pub const SHTDN_REASON_MINOR_SECURITY: ::DWORD = 0x00000013;
pub const SHTDN_REASON_MINOR_NETWORK_CONNECTIVITY: ::DWORD = 0x00000014;
pub const SHTDN_REASON_MINOR_WMI: ::DWORD = 0x00000015;
pub const SHTDN_REASON_MINOR_SERVICEPACK_UNINSTALL: ::DWORD = 0x00000016;
pub const SHTDN_REASON_MINOR_HOTFIX_UNINSTALL: ::DWORD = 0x00000017;
pub const SHTDN_REASON_MINOR_SECURITYFIX_UNINSTALL: ::DWORD = 0x00000018;
pub const SHTDN_REASON_MINOR_MMC: ::DWORD = 0x00000019;
pub const SHTDN_REASON_MINOR_SYSTEMRESTORE: ::DWORD = 0x0000001a;
pub const SHTDN_REASON_MINOR_TERMSRV: ::DWORD = 0x00000020;
pub const SHTDN_REASON_MINOR_DC_PROMOTION: ::DWORD = 0x00000021;
pub const SHTDN_REASON_MINOR_DC_DEMOTION: ::DWORD = 0x00000022;

pub const SHTDN_REASON_UNKNOWN: ::DWORD = SHTDN_REASON_MINOR_NONE;
pub const SHTDN_REASON_LEGACY_API: ::DWORD = 
	(SHTDN_REASON_MAJOR_LEGACY_API | SHTDN_REASON_FLAG_PLANNED);

// This mask cuts out UI flags.
pub const SHTDN_REASON_VALID_BIT_MASK: ::DWORD = 0xc0ffffff;

// Convenience flags.
pub const PCLEANUI: ::DWORD = (SHTDN_REASON_FLAG_PLANNED | SHTDN_REASON_FLAG_CLEAN_UI);
pub const UCLEANUI: ::DWORD = (SHTDN_REASON_FLAG_CLEAN_UI);
pub const PDIRTYUI: ::DWORD = (SHTDN_REASON_FLAG_PLANNED | SHTDN_REASON_FLAG_DIRTY_UI);
pub const UDIRTYUI: ::DWORD = (SHTDN_REASON_FLAG_DIRTY_UI);
//89
