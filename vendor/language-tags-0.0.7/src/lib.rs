#![deny(missing_docs)]

//! Language tags can be used identify human languages, scripts e.g. Latin script, countries and
//! other regions.
//!
//! Language tags are defined in [BCP47](http://tools.ietf.org/html/bcp47), an introduction is
//! ["Language tags in HTML and XML"](http://www.w3.org/International/articles/language-tags/) by
//! the W3C. They are commonly used in HTML and HTTP `Content-Language` and `Accept-Language`
//! header fields.
//!
//! This package currently supports parsing (fully conformant parser), formatting and comparing
//! language tags.
//!
//! # Examples
//! Create a simple language tag representing the French language as spoken
//! in Belgium and print it:
//!
//! ```rust
//! use language_tags::LanguageTag;
//! let mut langtag: LanguageTag = Default::default();
//! langtag.language = Some("fr".to_owned());
//! langtag.region = Some("BE".to_owned());
//! assert_eq!(format!("{}", langtag), "fr-BE");
//! ```
//!
//! Parse a tag representing a special type of English specified by private agreement:
//!
//! ```rust
//! use language_tags::LanguageTag;
//! let langtag: LanguageTag = "en-x-twain".parse().unwrap();
//! assert_eq!(format!("{}", langtag.language.unwrap()), "en");
//! assert_eq!(format!("{:?}", langtag.privateuse), "[\"twain\"]");
//! ```
//!
//! You can check for equality, but more often you should test if two tags match.
//! In this example we check if the resource in German language is suitable for
//! a user from Austria. While people speaking Austrian German normally understand
//! standard German the opposite is not always true. So the resource can be presented
//! to the user but if the resource was in `de-AT` and a user asked for a representation
//! in `de` the request should be rejected.
//!
//! ```rust
//! use language_tags::LanguageTag;
//! let mut langtag1: LanguageTag = Default::default();
//! langtag1.language = Some("de".to_owned());
//! langtag1.region = Some("AT".to_owned());
//! let mut langtag2: LanguageTag = Default::default();
//! langtag2.language = Some("de".to_owned());
//! assert!(langtag2.matches(&langtag1));
//! ```
//!
//! There is also the `langtag!` macro for creating language tags.

use std::ascii::AsciiExt;
use std::collections::BTreeMap;
use std::error::Error as ErrorTrait;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

fn is_alphabetic(s: &str) -> bool {
    s.chars().all(|x| x >= 'A' && x <= 'Z' || x >= 'a' && x <= 'z')
}

fn is_numeric(s: &str) -> bool {
    s.chars().all(|x| x >= '0' && x <= '9')
}

fn is_alphanumeric_or_dash(s: &str) -> bool {
    s.chars().all(|x| x >= 'A' && x <= 'Z' || x >= 'a' && x <= 'z' ||
                      x >= '0' && x <= '9' || x == '-')
}

/// Defines an Error type for langtags.
///
/// Errors occur mainly during parsing of language tags.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// The same extension subtag is only allowed once in a tag before the private use part.
    DuplicateExtension,
    /// If an extension subtag is present, it must not be empty.
    EmptyExtension,
    /// If the `x` subtag is present, it must not be empty.
    EmptyPrivateUse,
    /// The langtag contains a char that is not A-Z, a-z, 0-9 or the dash.
    ForbiddenChar,
    /// A subtag fails to parse, it does not match any other subtags.
    InvalidSubtag,
    /// The given language subtag is invalid.
    InvalidLanguage,
    /// A subtag may be eight characters in length at maximum.
    SubtagTooLong,
    /// At maximum three extlangs are allowed, but zero to one extlangs are preferred.
    TooManyExtlangs,
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::DuplicateExtension => "The same extension subtag is only allowed once in a tag",
            Error::EmptyExtension => "If an extension subtag is present, it must not be empty",
            Error::EmptyPrivateUse => "If the `x` subtag is present, it must not be empty",
            Error::ForbiddenChar => "The langtag contains a char not allowed",
            Error::InvalidSubtag => "A subtag fails to parse, it does not match any other subtags",
            Error::InvalidLanguage => "The given language subtag is invalid",
            Error::SubtagTooLong => "A subtag may be eight characters in length at maximum",
            Error::TooManyExtlangs => "At maximum three extlangs are allowed",
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

/// Result type used for this library.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Contains the 17 irregular old language tags not matching the standard grammer of tags.
pub const GRANDFATHERED_IRREGULAR: [&'static str; 17] = [
    "en-GB-oed",
    "i-ami",
    "i-bnn",
    "i-default",
    "i-enochian",
    "i-hak",
    "i-klingon",
    "i-lux",
    "i-mingo",
    "i-navajo",
    "i-pwn",
    "i-tao",
    "i-tay",
    "i-tsu",
    "sgn-BE-FR",
    "sgn-BE-NL",
    "sgn-CH-DE"];

/// Contains the 9 regular grandfathered tags having special semantics.
pub const GRANDFATHERED_REGULAR: [&'static str; 9] = [
    "art-lojban",
    "cel-gaulish",
    "no-bok",
    "no-nyn",
    "zh-guoyu",
    "zh-hakka",
    "zh-min",
    "zh-min-nan",
    "zh-xiang"];

/// A language tag as described in [BCP47](http://tools.ietf.org/html/bcp47).
///
/// Language tags are used to help identify languages, whether spoken,
/// written, signed, or otherwise signaled, for the purpose of
/// communication.  This includes constructed and artificial languages
/// but excludes languages not intended primarily for human
/// communication, such as programming languages.
#[derive(Debug, Eq, Clone)]
pub struct LanguageTag {
    /// Language subtags are used to indicate the language, ignoring all
    /// other aspects such as script, region or spefic invariants.
    pub language: Option<String>,
    /// Extended language subtags are used to identify certain specially
    /// selected languages that, for various historical and compatibility
    /// reasons, are closely identified with or tagged using an existing
    /// primary language subtag.
    pub extlang: Option<String>,
    /// Script subtags are used to indicate the script or writing system
    /// variations that distinguish the written forms of a language or its
    /// dialects.
    pub script: Option<String>,
    /// Region subtags are used to indicate linguistic variations associated
    /// with or appropriate to a specific country, territory, or region.
    /// Typically, a region subtag is used to indicate variations such as
    /// regional dialects or usage, or region-specific spelling conventions.
    /// It can also be used to indicate that content is expressed in a way
    /// that is appropriate for use throughout a region, for instance,
    /// Spanish content tailored to be useful throughout Latin America.
    pub region: Option<String>,
    /// Variant subtags are used to indicate additional, well-recognized
    /// variations that define a language or its dialects that are not
    /// covered by other available subtags.
    pub variants: Vec<String>,
    /// Extensions provide a mechanism for extending language tags for use in
    /// various applications.  They are intended to identify information that
    /// is commonly used in association with languages or language tags but
    /// that is not part of language identification.
    pub extensions: BTreeMap<u8, Vec<String>>,
    /// Private use subtags are used to indicate distinctions in language
    /// that are important in a given context by private agreement.
    pub privateuse: Vec<String>
}

impl LanguageTag {
    /// Matches language tags like described in
    /// [RFC4647#Extended filtering](https://tools.ietf.org/html/rfc4647#section-3.3.2)
    ///
    /// For example `en-GB` matches only `en-GB` and `en-Arab-GB` but not `en`. While `en` matches
    /// all of `en`, `en-GB` ,`en-Arab` and `en-Arab-GB`.
    pub fn matches(&self, other: &LanguageTag) -> bool {
        return matches_option_ignore_ascii_case(&self.language, &other.language) &&
        matches_option_ignore_ascii_case(&self.extlang, &other.extlang) &&
        matches_option_ignore_ascii_case(&self.script, &other.script) &&
        matches_option_ignore_ascii_case(&self.region, &other.region) &&
        self.variants.iter().all(|x| other.variants.iter().all(|y| x.eq_ignore_ascii_case(y))) &&
        self.privateuse.len() == other.privateuse.len() &&
        self.privateuse.iter().zip(other.privateuse.iter()).all(|(x, y)| x.eq_ignore_ascii_case(y));

        fn matches_option_ignore_ascii_case(a: &Option<String>, b: &Option<String>) -> bool {
            match (a.is_some(), b.is_some()) {
                (true, true) => a.as_ref().unwrap().eq_ignore_ascii_case(b.as_ref().unwrap()),
                (false, false) => true,
                (true, false) => false,
                (false, true) => true,
            }

        }
    }
}

impl PartialEq for LanguageTag {
    fn eq(&self, other: &LanguageTag) -> bool {
        return eq_option_ignore_ascii_case(&self.language, &other.language) &&
        eq_option_ignore_ascii_case(&self.extlang, &other.extlang) &&
        eq_option_ignore_ascii_case(&self.script, &other.script) &&
        eq_option_ignore_ascii_case(&self.region, &other.region) &&
        self.variants.iter().all(|x| other.variants.iter().all(|y| x.eq_ignore_ascii_case(y))) &&
        self.privateuse.len() == other.privateuse.len() &&
        self.privateuse.iter().zip(other.privateuse.iter()).all(|(x, y)| x.eq_ignore_ascii_case(y));

        fn eq_option_ignore_ascii_case(a: &Option<String>, b: &Option<String>) -> bool {
            match (a.is_some(), b.is_some()) {
                (true, true) => a.as_ref().unwrap().eq_ignore_ascii_case(b.as_ref().unwrap()),
                (false, false) => true,
                _ => false,
            }

        }
    }
}

impl Default for LanguageTag {
    fn default() -> LanguageTag {
        LanguageTag {
            language: None,
            extlang: None,
            script: None,
            region: None,
            variants: Vec::new(),
            extensions: BTreeMap::new(),
            privateuse: Vec::new(),
        }
    }
}

impl std::str::FromStr for LanguageTag {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let t = s.trim();
        if !is_alphanumeric_or_dash(t)  {
            return Err(Error::ForbiddenChar);
        }
        // Handle grandfathered tags
        if let Some(tag) = GRANDFATHERED_IRREGULAR.iter().find(|x| x.eq_ignore_ascii_case(t)) {
            return Ok(simple_langtag(tag))
        }
        if let Some(tag) = GRANDFATHERED_REGULAR.iter().find(|x| x.eq_ignore_ascii_case(t)) {
            return Ok(simple_langtag(tag))
        }
        // Handle normal tags
        // The parser has a position from 0 to 6. Bigger positions reepresent the ASCII codes of
        // single character extensions
        // language-extlang-script-region-variant-extension-privateuse
        // --- 0 -- -- 1 -- -- 2 - -- 3 - -- 4 -- --- x --- ---- 6 ---
        let mut langtag: LanguageTag = Default::default();
        let mut position: u8 = 0;
        for subtag in t.split('-') {
            if subtag.len() > 8 {
                // > All subtags have a maximum length of eight characters.
                return Err(Error::SubtagTooLong);
            }
            if position == 6 {
                langtag.privateuse.push(subtag.to_owned());
            } else if subtag.eq_ignore_ascii_case("x") {
                position = 6;
            } else if position == 0 {
                // Primary language
                if subtag.len() < 2 || !is_alphabetic(subtag) {
                    return Err(Error::InvalidLanguage)
                }
                langtag.language = Some(subtag.to_owned());
                if subtag.len() < 4 {
                    // Extlangs are only allowed for short language tags
                    position = 1;
                } else {
                    position = 2;
                }
            } else if position == 1 && subtag.len() == 3 && is_alphabetic(subtag) {
                // Extlang
                langtag.extlang = Some(subtag.to_owned());
                position = 2;
            } else if position == 2 && subtag.len() == 3 && is_alphabetic(subtag)
                    && langtag.extlang.is_some() {
                // Multiple extlangs
                let x = [langtag.extlang.unwrap(), subtag.to_owned()].connect("-");
                if x.len() > 11 {
                    // maximum 3 extlangs
                    return Err(Error::TooManyExtlangs);
                }
                langtag.extlang = Some(x);
            } else if position <= 2 && subtag.len() == 4 && is_alphabetic(subtag) {
                // Script
                langtag.script = Some(subtag.to_owned());
                position = 3;
            } else if position <= 3 && (subtag.len() == 2 && is_alphabetic(subtag) ||
                    subtag.len() == 3 && is_numeric(subtag)) {
                langtag.region = Some(subtag.to_owned());
                position = 4;
            } else if position <= 4 && (subtag.len() >= 5 && is_alphabetic(&subtag[0..1]) ||
                    subtag.len() >= 4 && is_numeric(&subtag[0..1])) {
                // Variant
                langtag.variants.push(subtag.to_owned());
                position = 4;
            } else if subtag.len() == 1 {
                position = subtag.chars().next().unwrap() as u8;
                if langtag.extensions.contains_key(&position) {
                    return Err(Error::DuplicateExtension);
                }
                langtag.extensions.insert(position, Vec::new());
            } else if position > 6 {
                langtag.extensions.get_mut(&position).unwrap().push(subtag.to_owned());
            } else {
                return Err(Error::InvalidSubtag);
            }
        }
        if langtag.extensions.values().any(|x| x.is_empty()) {
            // Extensions and privateuse must not be empty if present
            return Err(Error::EmptyExtension);
        }
        if position == 6 && langtag.privateuse.is_empty() {
            return Err(Error::EmptyPrivateUse);
        }
        return Ok(langtag);

        fn simple_langtag(s: &str) -> LanguageTag {
            let mut x: LanguageTag = Default::default();
            x.language = Some(s.to_owned());
            x
        }
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref x) = self.language {
            try!(Display::fmt(x, f))
        }
        if let Some(ref x) = self.extlang {
            try!(write!(f, "-{}", x));
        }
        if let Some(ref x) = self.script {
            try!(write!(f, "-{}", x));
        }
        if let Some(ref x) = self.region {
            try!(write!(f, "-{}", x));
        }
        for x in self.variants.iter() {
            try!(write!(f, "-{}", x));
        }
        for (raw_key, values) in self.extensions.iter() {
            let mut key = String::new();
            key.push(*raw_key as char);
            try!(write!(f, "-{}", key));
            for value in values {
                try!(write!(f, "-{}", value));
            }
        }
        if !self.privateuse.is_empty() {
            if self.language.is_none() {
                try!(f.write_str("x"));
            } else {
                try!(f.write_str("-x"));
            }
            for value in self.privateuse.iter() {
                try!(write!(f, "-{}", value));
            }
        }
        Ok(())
    }
}

#[macro_export]
/// Utility for creating simple language tags.
///
/// The macro supports the language, exlang, script and region parts of language tags,
/// they are separated by semicolons, omitted parts are denoted with mulitple semicolons.
///
/// # Examples
/// * `it`: `langtag!(it)`
/// * `it-LY`: `langtag!(it;;;LY)`
/// * `it-Arab-LY`: `langtag!(it;;Arab;LY)`
/// * `ar-afb`: `langtag!(ar;afb)`
/// * `i-enochian`: `langtag!(i-enochian)`
macro_rules! langtag {
    ( $language:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlang: None,
            script: None,
            region: None,
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;;;$region:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlang: None,
            script: None,
            region: Some(stringify!($region).to_owned()),
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;;$script:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlang: None,
            script: Some(stringify!($script).to_owned()),
            region: None,
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;;$script:expr;$region:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlang: None,
            script: Some(stringify!($script).to_owned()),
            region: Some(stringify!($region).to_owned()),
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;$extlang:expr) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlang: Some(stringify!($extlang).to_owned()),
            script: None,
            region: None,
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;$extlang:expr;$script:expr) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlang: Some(stringify!($extlang).to_owned()),
            script: Some(stringify!($script).to_owned()),
            region: None,
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;$extlang:expr;;$region:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlang: Some(stringify!($extlang).to_owned()),
            script: None,
            region: Some(stringify!($region).to_owned()),
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;$extlang:expr;$script:expr;$region:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlang: Some(stringify!($extlang).to_owned()),
            script: Some(stringify!($script).to_owned()),
            region: Some(stringify!($region).to_owned()),
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
}
