//! # Mime
//!
//! Mime is now Media Type, technically, but `Mime` is more immediately
//! understandable, so the main type here is `Mime`.
//!
//! ## What is Mime?
//!
//! Example mime string: `text/plain;charset=utf-8`
//!
//! ```rust
//! # #[macro_use] extern crate mime;
//! # fn main() {
//! let plain_text: mime::Mime = "text/plain;charset=utf-8".parse().unwrap();
//! assert_eq!(plain_text, mime!(Text/Plain; Charset=Utf8));
//! # }
//! ```

#![doc(html_root_url = "https://hyperium.github.io/mime.rs")]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(all(feature = "nightly", test), feature(test))]

#[macro_use]
extern crate log;

#[cfg(feature = "nightly")]
#[cfg(test)]
extern crate test;

use std::ascii::AsciiExt;
use std::fmt;
use std::iter::Enumerate;
use std::str::{FromStr, Chars};

macro_rules! inspect(
    ($s:expr, $t:expr) => ({
        let t = $t;
        trace!("inspect {}: {:?}", $s, t);
        t
    })
);

/// Mime, or Media Type. Encapsulates common registers types.
///
/// Consider that a traditional mime type contains a "top level type",
/// a "sub level type", and 0-N "parameters". And they're all strings.
/// Strings everywhere. Strings mean typos. Rust has type safety. We should
/// use types!
///
/// So, Mime bundles together this data into types so the compiler can catch
/// your typos.
///
/// This improves things so you use match without Strings:
///
/// ```rust
/// use mime::{Mime, TopLevel, SubLevel};
///
/// let mime: Mime = "application/json".parse().unwrap();
///
/// match mime {
///     Mime(TopLevel::Application, SubLevel::Json, _) => println!("matched json!"),
///     _ => ()
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Mime<T: AsRef<[Param]> = Vec<Param>>(pub TopLevel, pub SubLevel, pub T);

impl<LHS: AsRef<[Param]>, RHS: AsRef<[Param]>> PartialEq<Mime<RHS>> for Mime<LHS> {
    fn eq(&self, other: &Mime<RHS>) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2.as_ref() == other.2.as_ref()
    }
}

/// Easily create a Mime without having to import so many enums.
///
/// # Example
///
/// ```
/// # #[macro_use] extern crate mime;
///
/// # fn main() {
/// let json = mime!(Application/Json);
/// let plain = mime!(Text/Plain; Charset=Utf8);
/// let text = mime!(Text/Html; Charset=("bar"), ("baz")=("quux"));
/// let img = mime!(Image/_);
/// # }
/// ```
#[macro_export]
macro_rules! mime {
    ($top:tt / $sub:tt) => (
        mime!($top / $sub;)
    );

    ($top:tt / $sub:tt ; $($attr:tt = $val:tt),*) => (
        $crate::Mime(
            __mime__ident_or_ext!(TopLevel::$top),
            __mime__ident_or_ext!(SubLevel::$sub),
            vec![ $((__mime__ident_or_ext!(Attr::$attr), __mime__ident_or_ext!(Value::$val))),* ]
        )
    );
}

#[doc(hidden)]
#[macro_export]
macro_rules! __mime__ident_or_ext {
    ($enoom:ident::_) => (
        $crate::$enoom::Star
    );
    ($enoom:ident::($inner:expr)) => (
        $crate::$enoom::Ext($inner.to_string())
    );
    ($enoom:ident::$var:ident) => (
        $crate::$enoom::$var
    )
}

macro_rules! enoom {
    (pub enum $en:ident; $ext:ident; $($ty:ident, $text:expr;)*) => (

        #[derive(Clone, Debug)]
        pub enum $en {
            $($ty),*,
            $ext(String)
        }

        impl $en {
            pub fn as_str(&self) -> &str {
                match *self {
                    $($en::$ty => $text),*,
                    $en::$ext(ref s) => &s
                }
            }
        }

        impl ::std::ops::Deref for $en {
            type Target = str;
            fn deref(&self) -> &str {
                self.as_str()
            }
        }

        impl PartialEq for $en {
            fn eq(&self, other: &$en) -> bool {
                match (self, other) {
                    $( (&$en::$ty, &$en::$ty) => true ),*,
                    (&$en::$ext(ref a), &$en::$ext(ref b)) => a == b,
                    _ => self.as_str() == other.as_str()
                }
            }
        }

        impl PartialEq<String> for $en {
            fn eq(&self, other: &String) -> bool {
                self.as_str() == other
            }
        }

        impl PartialEq<str> for $en {
            fn eq(&self, other: &str) -> bool {
                self.as_str() == other
            }
        }

        impl<'a> PartialEq<&'a str> for $en {
            fn eq(&self, other: &&'a str) -> bool {
                self.as_str() == *other
            }
        }

        impl PartialEq<$en> for String {
            fn eq(&self, other: &$en) -> bool {
                self == other.as_str()
            }
        }

        impl PartialEq<$en> for str {
            fn eq(&self, other: &$en) -> bool {
                self == other.as_str()
            }
        }

        impl<'a> PartialEq<$en> for &'a str {
            fn eq(&self, other: &$en) -> bool {
                *self == other.as_str()
            }
        }

        impl fmt::Display for $en {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str(match *self {
                    $($en::$ty => $text),*,
                    $en::$ext(ref s) => s
                })
            }
        }

        impl FromStr for $en {
            type Err = ();
            fn from_str(s: &str) -> Result<$en, ()> {
                Ok(match s {
                    $(_s if _s == $text => $en::$ty),*,
                    s => $en::$ext(inspect!(stringify!($ext), s).to_string())
                })
            }
        }
    )
}

enoom! {
    pub enum TopLevel;
    Ext;
    Star, "*";
    Text, "text";
    Image, "image";
    Audio, "audio";
    Video, "video";
    Application, "application";
    Multipart, "multipart";
    Message, "message";
    Model, "model";
}

enoom! {
    pub enum SubLevel;
    Ext;
    Star, "*";

    // common text/*
    Plain, "plain";
    Html, "html";
    Xml, "xml";
    Javascript, "javascript";
    Css, "css";

    // common application/*
    Json, "json";
    WwwFormUrlEncoded, "x-www-form-urlencoded";

    // multipart/*
    FormData, "form-data";

    // common image/*
    Png, "png";
    Gif, "gif";
    Bmp, "bmp";
    Jpeg, "jpeg";
}

enoom! {
    pub enum Attr;
    Ext;
    Charset, "charset";
    Boundary, "boundary";
    Q, "q";
}

enoom! {
    pub enum Value;
    Ext;
    Utf8, "utf-8";
}

pub type Param = (Attr, Value);

impl<T: AsRef<[Param]>> fmt::Display for Mime<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Mime(ref top, ref sub, ref params) = *self;
        try!(write!(fmt, "{}/{}", top, sub));
        fmt_params(params.as_ref(), fmt)
    }
}

impl<P: AsRef<[Param]>> Mime<P> {
    pub fn get_param<A: PartialEq<Attr>>(&self, attr: A) -> Option<&Value> {
        self.2.as_ref().iter().find(|&&(ref name, _)| attr == *name).map(|&(_, ref value)| value)
    }
}

impl FromStr for Mime {
    type Err = ();
    fn from_str(raw: &str) -> Result<Mime, ()> {
        let ascii = raw.to_ascii_lowercase(); // lifetimes :(
        let len = ascii.len();
        let mut iter = ascii.chars().enumerate();
        let mut params = vec![];
        // toplevel
        let mut start;
        let top;
        loop {
            match inspect!("top iter", iter.next()) {
                Some((0, c)) if is_restricted_name_first_char(c) => (),
                Some((i, c)) if i > 0 && is_restricted_name_char(c) => (),
                Some((i, '/')) if i > 0 => match FromStr::from_str(&ascii[..i]) {
                    Ok(t) => {
                        top = t;
                        start = i + 1;
                        break;
                    }
                    Err(_) => return Err(())
                },
                _ => return Err(()) // EOF and no toplevel is no Mime
            };

        }

        // sublevel
        let sub;
        loop {
            match inspect!("sub iter", iter.next()) {
                Some((i, c)) if i == start && is_restricted_name_first_char(c) => (),
                Some((i, c)) if i > start && is_restricted_name_char(c) => (),
                Some((i, ';')) if i > start => match FromStr::from_str(&ascii[start..i]) {
                    Ok(s) => {
                        sub = s;
                        start = i + 1;
                        break;
                    }
                    Err(_) => return Err(())
                },
                None => match FromStr::from_str(&ascii[start..]) {
                    Ok(s) => return Ok(Mime(top, s, params)),
                    Err(_) => return Err(())
                },
                _ => return Err(())
            };
        }

        // params
        debug!("starting params, len={}", len);
        loop {
            match inspect!("param", param_from_str(raw, &ascii, &mut iter, start)) {
                Some((p, end)) => {
                    params.push(p);
                    start = end;
                    if start >= len {
                        break;
                    }
                }
                None => break
            }
        }

        Ok(Mime(top, sub, params))
    }
}

fn param_from_str(raw: &str, ascii: &str, iter: &mut Enumerate<Chars>, mut start: usize) -> Option<(Param, usize)> {
    let attr;
    debug!("param_from_str, start={}", start);
    loop {
        match inspect!("attr iter", iter.next()) {
            Some((i, ' ')) if i == start => start = i + 1,
            Some((i, c)) if i == start && is_restricted_name_first_char(c) => (),
            Some((i, c)) if i > start && is_restricted_name_char(c) => (),
            Some((i, '=')) if i > start => match FromStr::from_str(&ascii[start..i]) {
                Ok(a) => {
                    attr = inspect!("attr", a);
                    start = i + 1;
                    break;
                },
                Err(_) => return None
            },
            _ => return None
        }
    }

    let value;
    // values must be restrict-name-char or "anything goes"
    let mut is_quoted = false;

    {
        let substr = |a,b| { if attr==Attr::Charset { &ascii[a..b] } else { &raw[a..b] } };
        let endstr = |a| { if attr==Attr::Charset { &ascii[a..] } else { &raw[a..] } };
        loop {
            match inspect!("value iter", iter.next()) {
                Some((i, '"')) if i == start => {
                    debug!("quoted");
                    is_quoted = true;
                    start = i + 1;
                },
                Some((i, c)) if i == start && is_restricted_name_first_char(c) => (),
                Some((i, '"')) if i > start && is_quoted => match FromStr::from_str(substr(start,i)) {
                    Ok(v) => {
                        value = v;
                        start = i + 1;
                        break;
                    },
                    Err(_) => return None
                },
                Some((i, c)) if i > start && is_quoted || is_restricted_name_char(c) => (),
                Some((i, ';')) if i > start => match FromStr::from_str(substr(start,i)) {
                    Ok(v) => {
                        value = v;
                        start = i + 1;
                        break;
                    },
                    Err(_) => return None
                },
                None => match FromStr::from_str(endstr(start)) {
                    Ok(v) => {
                        value = v;
                        start = raw.len();
                        break;
                    },
                    Err(_) => return None
                },

                _ => return None
            }
        }
    }

    Some(((attr, value), start))
}

// From [RFC6838](http://tools.ietf.org/html/rfc6838#section-4.2):
//
// > All registered media types MUST be assigned top-level type and
// > subtype names.  The combination of these names serves to uniquely
// > identify the media type, and the subtype name facet (or the absence
// > of one) identifies the registration tree.  Both top-level type and
// > subtype names are case-insensitive.
// >
// > Type and subtype names MUST conform to the following ABNF:
// >
// >     type-name = restricted-name
// >     subtype-name = restricted-name
// >
// >     restricted-name = restricted-name-first *126restricted-name-chars
// >     restricted-name-first  = ALPHA / DIGIT
// >     restricted-name-chars  = ALPHA / DIGIT / "!" / "#" /
// >                              "$" / "&" / "-" / "^" / "_"
// >     restricted-name-chars =/ "." ; Characters before first dot always
// >                                  ; specify a facet name
// >     restricted-name-chars =/ "+" ; Characters after last plus always
// >                                  ; specify a structured syntax suffix
//
fn is_restricted_name_first_char(c: char) -> bool {
    match c {
        'a'...'z' |
        '0'...'9' => true,
        _ => false
    }
}

fn is_restricted_name_char(c: char) -> bool {
    if is_restricted_name_first_char(c) {
        true
    } else {
        match c {
            '!' |
            '#' |
            '$' |
            '&' |
            '-' |
            '^' |
            '.' |
            '+' |
            '_' => true,
            _ => false
        }
    }
}


#[inline]
fn fmt_params(params: &[Param], fmt: &mut fmt::Formatter) -> fmt::Result {
    for param in params.iter() {
        try!(fmt_param(param, fmt));
    }
    Ok(())
}

#[inline]
fn fmt_param(param: &Param, fmt: &mut fmt::Formatter) -> fmt::Result {
    let (ref attr, ref value) = *param;
    write!(fmt, "; {}={}", attr, value)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    #[cfg(feature = "nightly")]
    use test::Bencher;
    use super::{Mime, Value, Attr};

    #[test]
    fn test_mime_show() {
        let mime = mime!(Text/Plain);
        assert_eq!(mime.to_string(), "text/plain".to_string());
        let mime = mime!(Text/Plain; Charset=Utf8);
        assert_eq!(mime.to_string(), "text/plain; charset=utf-8".to_string());
    }

    #[test]
    fn test_mime_from_str() {
        assert_eq!(Mime::from_str("text/plain").unwrap(), mime!(Text/Plain));
        assert_eq!(Mime::from_str("TEXT/PLAIN").unwrap(), mime!(Text/Plain));
        assert_eq!(Mime::from_str("text/plain; charset=utf-8").unwrap(), mime!(Text/Plain; Charset=Utf8));
        assert_eq!(Mime::from_str("text/plain;charset=\"utf-8\"").unwrap(), mime!(Text/Plain; Charset=Utf8));
        assert_eq!(Mime::from_str("text/plain; charset=utf-8; foo=bar").unwrap(),
            mime!(Text/Plain; Charset=Utf8, ("foo")=("bar")));
    }

    #[test]
    fn test_case_sensitive_values() {
        assert_eq!(Mime::from_str("multipart/form-data; boundary=ABCDEFG").unwrap(),
                   mime!(Multipart/FormData; Boundary=("ABCDEFG")));
        assert_eq!(Mime::from_str("multipart/form-data; charset=BASE64; boundary=ABCDEFG").unwrap(),
                   mime!(Multipart/FormData; Charset=("base64"), Boundary=("ABCDEFG")));
    }

    #[test]
    fn test_get_param() {
        let mime = Mime::from_str("text/plain; charset=utf-8; foo=bar").unwrap();
        assert_eq!(mime.get_param(Attr::Charset), Some(&Value::Utf8));
        assert_eq!(mime.get_param("charset"), Some(&Value::Utf8));
        assert_eq!(mime.get_param("foo").unwrap(), "bar");
        assert_eq!(mime.get_param("baz"), None);
    }

    #[test]
    fn test_value_as_str() {
        assert_eq!(Value::Utf8.as_str(), "utf-8");
    }

    #[test]
    fn test_value_eq_str() {
        assert_eq!(Value::Utf8, "utf-8");
        assert_eq!("utf-8", Value::Utf8);
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_show(b: &mut Bencher) {
        let mime = mime!(Text/Plain; Charset=Utf8, ("foo")=("bar"));
        b.bytes = mime.to_string().as_bytes().len() as u64;
        b.iter(|| mime.to_string())
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_from_str(b: &mut Bencher) {
        let s = "text/plain; charset=utf-8; foo=bar";
        b.bytes = s.as_bytes().len() as u64;
        b.iter(|| s.parse::<Mime>())
    }
}
