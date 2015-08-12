#![allow(bad_style)]

pub use self::inner::*;

#[cfg(unix)]
mod inner {
    use libc::{c_int, c_long, c_char, time_t};
    use std::mem;
    use std::io;
    use Tm;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use self::mac::*;
    #[cfg(all(not(target_os = "macos"), not(target_os = "ios")))]
    pub use self::unix::*;

    /// ctime's `tm`
    #[repr(C)]
    struct tm {
        tm_sec: c_int,
        tm_min: c_int,
        tm_hour: c_int,
        tm_mday: c_int,
        tm_mon: c_int,
        tm_year: c_int,
        tm_wday: c_int,
        tm_yday: c_int,
        tm_isdst: c_int,
        tm_gmtoff: c_long,
        tm_zone: *const c_char,
    }

    fn rust_tm_to_tm(rust_tm: &Tm, tm: &mut tm) {
        tm.tm_sec = rust_tm.tm_sec;
        tm.tm_min = rust_tm.tm_min;
        tm.tm_hour = rust_tm.tm_hour;
        tm.tm_mday = rust_tm.tm_mday;
        tm.tm_mon = rust_tm.tm_mon;
        tm.tm_year = rust_tm.tm_year;
        tm.tm_wday = rust_tm.tm_wday;
        tm.tm_yday = rust_tm.tm_yday;
        tm.tm_isdst = rust_tm.tm_isdst;
    }

    fn tm_to_rust_tm(tm: &tm, utcoff: i32, rust_tm: &mut Tm) {
        rust_tm.tm_sec = tm.tm_sec;
        rust_tm.tm_min = tm.tm_min;
        rust_tm.tm_hour = tm.tm_hour;
        rust_tm.tm_mday = tm.tm_mday;
        rust_tm.tm_mon = tm.tm_mon;
        rust_tm.tm_year = tm.tm_year;
        rust_tm.tm_wday = tm.tm_wday;
        rust_tm.tm_yday = tm.tm_yday;
        rust_tm.tm_isdst = tm.tm_isdst;
        rust_tm.tm_utcoff = utcoff;
    }

    type time64_t = i64;

    extern {
        fn gmtime_r(time_p: *const time_t, result: *mut tm) -> *mut tm;
        fn localtime_r(time_p: *const time_t, result: *mut tm) -> *mut tm;
        fn mktime(tm: *const tm) -> time_t;
        #[cfg(not(target_os = "android"))]
        fn timegm(tm: *const tm) -> time_t;
        #[cfg(target_os = "android")]
        fn timegm64(tm: *const tm) -> time64_t;
    }

    pub fn time_to_utc_tm(sec: i64, tm: &mut Tm) {
        unsafe {
            let sec = sec as time_t;
            let mut out = mem::zeroed();
            if gmtime_r(&sec, &mut out).is_null() {
                panic!("gmtime_r failed: {}", io::Error::last_os_error());
            }
            tm_to_rust_tm(&out, 0, tm);
        }
    }

    pub fn time_to_local_tm(sec: i64, tm: &mut Tm) {
        unsafe {
            let sec = sec as time_t;
            let mut out = mem::zeroed();
            if localtime_r(&sec, &mut out).is_null() {
                panic!("localtime_r failed: {}", io::Error::last_os_error());
            }
            tm_to_rust_tm(&out, out.tm_gmtoff as i32, tm);
        }
    }

    pub fn utc_tm_to_time(rust_tm: &Tm) -> i64 {
        #[cfg(target_os = "android")]
        use self::timegm64 as timegm;

        let mut tm = unsafe { mem::zeroed() };
        rust_tm_to_tm(rust_tm, &mut tm);
        unsafe { timegm(&tm) as i64 }
    }

    pub fn local_tm_to_time(rust_tm: &Tm) -> i64 {
        let mut tm = unsafe { mem::zeroed() };
        rust_tm_to_tm(rust_tm, &mut tm);
        unsafe { mktime(&tm) as i64 }
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    mod mac {
        use libc::{timeval, timezone, c_int, mach_timebase_info};
        use std::sync::{Once, ONCE_INIT};
        use std::ops::{Add, Sub};
        use Duration;

        extern {
            fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int;
            fn mach_absolute_time() -> u64;
            fn mach_timebase_info(info: *mut mach_timebase_info) -> c_int;
        }

        fn info() -> &'static mach_timebase_info {
            static mut INFO: mach_timebase_info = mach_timebase_info {
                numer: 0,
                denom: 0,
            };
            static ONCE: Once = ONCE_INIT;

            unsafe {
                ONCE.call_once(|| {
                    mach_timebase_info(&mut INFO);
                });
                &INFO
            }
        }

        pub fn get_time() -> (i64, i32) {
            use std::ptr;
            let mut tv = timeval { tv_sec: 0, tv_usec: 0 };
            unsafe { gettimeofday(&mut tv, ptr::null_mut()); }
            (tv.tv_sec as i64, tv.tv_usec * 1000)
        }

        pub fn get_precise_ns() -> u64 {
            unsafe {
                let time = mach_absolute_time();
                let info = info();
                time * info.numer as u64 / info.denom as u64
            }
        }

        #[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
        pub struct SteadyTime { t: u64 }

        impl SteadyTime {
            pub fn now() -> SteadyTime {
                SteadyTime { t: get_precise_ns() }
            }
        }
        impl Sub for SteadyTime {
            type Output = Duration;
            fn sub(self, other: SteadyTime) -> Duration {
                Duration::nanoseconds(self.t as i64 - other.t as i64)
            }
        }
        impl Sub<Duration> for SteadyTime {
            type Output = SteadyTime;
            fn sub(self, other: Duration) -> SteadyTime {
                self + -other
            }
        }
        impl Add<Duration> for SteadyTime {
            type Output = SteadyTime;
            fn add(self, other: Duration) -> SteadyTime {
                let delta = other.num_nanoseconds().unwrap();
                SteadyTime {
                    t: (self.t as i64 + delta) as u64
                }
            }
        }
    }

    #[cfg(test)]
    pub struct TzReset;

    #[cfg(test)]
    pub fn set_los_angeles_time_zone() -> TzReset {
        use std::env;
        env::set_var("TZ", "America/Los_Angeles");
        ::tzset();
        TzReset
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "ios")))]
    mod unix {
        use std::fmt;
        use std::cmp::Ordering;
        use std::ops::{Add, Sub};
        use libc::{self, c_int, timespec};

        use Duration;

        #[cfg(all(not(target_os = "android"),
                  not(target_os = "bitrig"),
                  not(target_os = "nacl"),
                  not(target_os = "openbsd")))]
        #[link(name = "rt")]
        extern {}

        extern {
            fn clock_gettime(clk_id: c_int, tp: *mut timespec) -> c_int;
        }

        pub fn get_time() -> (i64, i32) {
            let mut tv = libc::timespec { tv_sec: 0, tv_nsec: 0 };
            unsafe { clock_gettime(libc::CLOCK_REALTIME, &mut tv); }
            (tv.tv_sec as i64, tv.tv_nsec as i32)
        }

        pub fn get_precise_ns() -> u64 {
            let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
            unsafe {
                clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
            }
            (ts.tv_sec as u64) * 1000000000 + (ts.tv_nsec as u64)
        }

        #[derive(Copy)]
        pub struct SteadyTime {
            t: libc::timespec,
        }

        impl fmt::Debug for SteadyTime {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "SteadyTime {{ tv_sec: {:?}, tv_nsec: {:?} }}",
                       self.t.tv_sec, self.t.tv_nsec)
            }
        }

        impl Clone for SteadyTime {
            fn clone(&self) -> SteadyTime {
                SteadyTime { t: self.t }
            }
        }

        impl SteadyTime {
            pub fn now() -> SteadyTime {
                let mut t = SteadyTime {
                    t: libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    }
                };
                unsafe {
                    assert_eq!(0, clock_gettime(libc::CLOCK_MONOTONIC, &mut t.t));
                }
                t
            }
        }

        impl Sub for SteadyTime {
            type Output = Duration;
            fn sub(self, other: SteadyTime) -> Duration {
                if self.t.tv_nsec >= other.t.tv_nsec {
                    Duration::seconds(self.t.tv_sec as i64 - other.t.tv_sec as i64) +
                        Duration::nanoseconds(self.t.tv_nsec as i64 - other.t.tv_nsec as i64)
                } else {
                    Duration::seconds(self.t.tv_sec as i64 - 1 - other.t.tv_sec as i64) +
                        Duration::nanoseconds(self.t.tv_nsec as i64 + ::NSEC_PER_SEC as i64 -
                                              other.t.tv_nsec as i64)
                }
            }
        }

        impl Sub<Duration> for SteadyTime {
            type Output = SteadyTime;
            fn sub(self, other: Duration) -> SteadyTime {
                self + -other
            }
        }

        impl Add<Duration> for SteadyTime {
            type Output = SteadyTime;
            fn add(mut self, other: Duration) -> SteadyTime {
                let seconds = other.num_seconds();
                let nanoseconds = other - Duration::seconds(seconds);
                let nanoseconds = nanoseconds.num_nanoseconds().unwrap();
                self.t.tv_sec += seconds as libc::time_t;
                self.t.tv_nsec += nanoseconds as libc::c_long;
                if self.t.tv_nsec >= ::NSEC_PER_SEC as libc::c_long {
                    self.t.tv_nsec -= ::NSEC_PER_SEC as libc::c_long;
                    self.t.tv_sec += 1;
                } else if self.t.tv_nsec < 0 {
                    self.t.tv_sec -= 1;
                    self.t.tv_nsec += ::NSEC_PER_SEC as libc::c_long;
                }
                self
            }
        }

        impl PartialOrd for SteadyTime {
            fn partial_cmp(&self, other: &SteadyTime) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for SteadyTime {
            fn cmp(&self, other: &SteadyTime) -> Ordering {
                match self.t.tv_sec.cmp(&other.t.tv_sec) {
                    Ordering::Equal => self.t.tv_nsec.cmp(&other.t.tv_nsec),
                    ord => ord
                }
            }
        }

        impl PartialEq for SteadyTime {
            fn eq(&self, other: &SteadyTime) -> bool {
                self.t.tv_sec == other.t.tv_sec &&
                    self.t.tv_nsec == other.t.tv_nsec
            }
        }

        impl Eq for SteadyTime {}

    }
}

#[cfg(windows)]
#[allow(non_snake_case)]
mod inner {
    use std::io;
    use std::mem;
    use std::sync::{Once, ONCE_INIT};
    use std::ops::{Add, Sub};
    use {Tm, Duration};

    use kernel32::*;
    use winapi::*;

    fn frequency() -> LARGE_INTEGER {
        static mut FREQUENCY: LARGE_INTEGER = 0;
        static ONCE: Once = ONCE_INIT;

        unsafe {
            ONCE.call_once(|| {
                QueryPerformanceFrequency(&mut FREQUENCY);
            });
            FREQUENCY
        }
    }

    const HECTONANOSECS_IN_SEC: u64 = 10_000_000;
    const HECTONANOSEC_TO_UNIX_EPOCH: u64 = 11_644_473_600 * HECTONANOSECS_IN_SEC;

    fn time_to_file_time(sec: i64) -> FILETIME {
        let t = (sec as u64 * HECTONANOSECS_IN_SEC) + HECTONANOSEC_TO_UNIX_EPOCH;
        FILETIME {
            dwLowDateTime: t as DWORD,
            dwHighDateTime: (t >> 32) as DWORD
        }
    }

    fn file_time_to_nsec(ft: &FILETIME) -> i32 {
        let t = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);
        ((t % HECTONANOSECS_IN_SEC) * 100) as i32
    }

    fn file_time_to_unix_seconds(ft: &FILETIME) -> i64 {
        let t = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);
        ((t - HECTONANOSEC_TO_UNIX_EPOCH) / HECTONANOSECS_IN_SEC) as i64
    }

    fn tm_to_system_time(tm: &Tm) -> SYSTEMTIME {
        let mut sys: SYSTEMTIME = unsafe { mem::zeroed() };
        sys.wSecond = tm.tm_sec as WORD;
        sys.wMinute = tm.tm_min as WORD;
        sys.wHour = tm.tm_hour as WORD;
        sys.wDay = tm.tm_mday as WORD;
        sys.wDayOfWeek = tm.tm_wday as WORD;
        sys.wMonth = (tm.tm_mon + 1) as WORD;
        sys.wYear = (tm.tm_year + 1900) as WORD;
        sys
    }

    fn system_time_to_tm(sys: &SYSTEMTIME, tm: &mut Tm) {
        tm.tm_sec = sys.wSecond as i32;
        tm.tm_min = sys.wMinute as i32;
        tm.tm_hour = sys.wHour as i32;
        tm.tm_mday = sys.wDay as i32;
        tm.tm_wday = sys.wDayOfWeek as i32;
        tm.tm_mon = (sys.wMonth - 1) as i32;
        tm.tm_year = (sys.wYear - 1900) as i32;
        tm.tm_yday = yday(tm.tm_year, tm.tm_mon + 1, tm.tm_mday);

        fn yday(year: i32, month: i32, day: i32) -> i32 {
            let leap = if month > 2 {
                if year % 4 == 0 { 1 } else { 2 }
            } else {
                0
            };
            let july = if month > 7 { 1 } else { 0 };

            (month - 1) * 30 + month / 2 + (day - 1) - leap + july
        }
    }

    macro_rules! call {
        ($name:ident($($arg:expr),*)) => {
            if $name($($arg),*) == 0 {
                panic!(concat!(stringify!($name), " failed with: {}"),
                       io::Error::last_os_error());
            }
        }
    }

    pub fn time_to_utc_tm(sec: i64, tm: &mut Tm) {
        let mut out = unsafe { mem::zeroed() };
        let ft = time_to_file_time(sec);
        unsafe {
            call!(FileTimeToSystemTime(&ft, &mut out));
        }
        system_time_to_tm(&out, tm);
        tm.tm_utcoff = 0;
    }

    pub fn time_to_local_tm(sec: i64, tm: &mut Tm) {
        let ft = time_to_file_time(sec);
        unsafe {
            let mut utc = mem::zeroed();
            let mut local = mem::zeroed();
            call!(FileTimeToSystemTime(&ft, &mut utc));
            call!(SystemTimeToTzSpecificLocalTime(0 as *const _,
                                                  &mut utc, &mut local));
            system_time_to_tm(&local, tm);

            let mut tz = mem::zeroed();
            GetTimeZoneInformation(&mut tz);
            tm.tm_utcoff = -tz.Bias * 60;
        }
    }

    pub fn utc_tm_to_time(tm: &Tm) -> i64 {
        unsafe {
            let mut ft = mem::zeroed();
            let sys_time = tm_to_system_time(tm);
            call!(SystemTimeToFileTime(&sys_time, &mut ft));
            file_time_to_unix_seconds(&ft)
        }
    }

    pub fn local_tm_to_time(tm: &Tm) -> i64 {
        unsafe {
            let mut ft = mem::zeroed();
            let mut utc = mem::zeroed();
            let mut sys_time = tm_to_system_time(tm);
            call!(TzSpecificLocalTimeToSystemTime(0 as *mut _,
                                                  &mut sys_time, &mut utc));
            call!(SystemTimeToFileTime(&utc, &mut ft));
            file_time_to_unix_seconds(&ft)
        }
    }

    pub fn get_time() -> (i64, i32) {
        unsafe {
            let mut ft = mem::zeroed();
            GetSystemTimeAsFileTime(&mut ft);
            (file_time_to_unix_seconds(&ft), file_time_to_nsec(&ft))
        }
    }

    pub fn get_precise_ns() -> u64 {
        let mut ticks = 0;
        unsafe {
            assert!(QueryPerformanceCounter(&mut ticks) == 1);
        }
        mul_div_i64(ticks as i64, 1000000000, frequency() as i64) as u64

    }

    #[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
    pub struct SteadyTime {
        t: LARGE_INTEGER,
    }

    impl SteadyTime {
        pub fn now() -> SteadyTime {
            let mut t = SteadyTime { t: 0 };
            unsafe { QueryPerformanceCounter(&mut t.t); }
            t
        }
    }

    impl Sub for SteadyTime {
        type Output = Duration;
        fn sub(self, other: SteadyTime) -> Duration {
            let diff = self.t as i64 - other.t as i64;
            Duration::nanoseconds(mul_div_i64(diff, 1000000000,
                                              frequency() as i64))
        }
    }

    impl Sub<Duration> for SteadyTime {
        type Output = SteadyTime;
        fn sub(self, other: Duration) -> SteadyTime {
            self + -other
        }
    }

    impl Add<Duration> for SteadyTime {
        type Output = SteadyTime;
        fn add(mut self, other: Duration) -> SteadyTime {
            self.t += (other.num_microseconds().unwrap() * frequency() as i64 /
                       1_000_000) as LARGE_INTEGER;
            self
        }
    }

    #[cfg(test)]
    pub struct TzReset {
        old: TIME_ZONE_INFORMATION,
    }

    #[cfg(test)]
    impl Drop for TzReset {
        fn drop(&mut self) {
            unsafe {
                call!(SetTimeZoneInformation(&self.old));
            }
        }
    }

    #[cfg(test)]
    pub fn set_los_angeles_time_zone() -> TzReset {
        acquire_privileges();

        unsafe {
            let mut tz = mem::zeroed::<TIME_ZONE_INFORMATION>();
            GetTimeZoneInformation(&mut tz);
            let ret = TzReset { old: tz };
            tz.Bias = 60 * 8;
            call!(SetTimeZoneInformation(&tz));
            return ret
        }
    }

    // Ensures that this process has the necessary privileges to set a new time
    // zone, and this is all transcribed from:
    // https://msdn.microsoft.com/en-us/library/windows/desktop/ms724944%28v=vs.85%29.aspx
    #[cfg(test)]
    fn acquire_privileges() {
        use std::sync::{ONCE_INIT, Once};
        use advapi32::*;
        const SE_PRIVILEGE_ENABLED: DWORD = 2;
        static INIT: Once = ONCE_INIT;

        #[repr(C)]
        struct TKP {
            tkp: TOKEN_PRIVILEGES,
            laa: LUID_AND_ATTRIBUTES,
        }

        INIT.call_once(|| unsafe {
            let mut hToken = 0 as *mut _;
            call!(OpenProcessToken(GetCurrentProcess(),
                                   TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                                   &mut hToken));

            let mut tkp = mem::zeroed::<TKP>();
            assert_eq!(tkp.tkp.Privileges.len(), 0);
            let c = ::std::ffi::CString::new("SeTimeZonePrivilege").unwrap();
            call!(LookupPrivilegeValueA(0 as *const _, c.as_ptr(),
                                        &mut tkp.laa.Luid));
            tkp.tkp.PrivilegeCount = 1;
            tkp.laa.Attributes = SE_PRIVILEGE_ENABLED;
            call!(AdjustTokenPrivileges(hToken, FALSE, &mut tkp.tkp, 0,
                                        0 as *mut _, 0 as *mut _));
        });
    }



    // Computes (value*numer)/denom without overflow, as long as both
    // (numer*denom) and the overall result fit into i64 (which is the case
    // for our time conversions).
    fn mul_div_i64(value: i64, numer: i64, denom: i64) -> i64 {
        let q = value / denom;
        let r = value % denom;
        // Decompose value as (value/denom*denom + value%denom),
        // substitute into (value*numer)/denom and simplify.
        // r < denom, so (denom*numer) is the upper bound of (r*numer)
        q * numer + r * numer / denom
    }

    #[test]
    fn test_muldiv() {
        assert_eq!(mul_div_i64( 1_000_000_000_001, 1_000_000_000, 1_000_000),
                   1_000_000_000_001_000);
        assert_eq!(mul_div_i64(-1_000_000_000_001, 1_000_000_000, 1_000_000),
                   -1_000_000_000_001_000);
        assert_eq!(mul_div_i64(-1_000_000_000_001,-1_000_000_000, 1_000_000),
                   1_000_000_000_001_000);
        assert_eq!(mul_div_i64( 1_000_000_000_001, 1_000_000_000,-1_000_000),
                   -1_000_000_000_001_000);
        assert_eq!(mul_div_i64( 1_000_000_000_001,-1_000_000_000,-1_000_000),
                   1_000_000_000_001_000);
    }
}
