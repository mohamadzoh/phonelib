//! # Phonelib
//!
//! A dependency-free Rust library for validating, parsing, formatting and
//! manipulating international phone numbers.
//!
//! ## Features
//!
//! - **Validation** - length- and (for the NANP) range-based validation
//! - **Country detection** - deterministic resolution of the calling code
//! - **Normalization** - canonical E.164 output
//! - **Multiple formats** - E.164, international, national, RFC 3966
//! - **Type detection** - mobile, landline, toll-free, premium, and more
//! - **Text extraction** - find phone numbers in free-form text
//! - **Comparison** - compare numbers regardless of formatting
//! - **Batch processing** - process many numbers in one call
//!
//! ## Quick start
//!
//! ```rust
//! use phonelib::*;
//!
//! assert!(is_valid_phone_number("+12025550173"));
//!
//! assert_eq!(
//!     normalize_phone_number("+1 (202) 555-0173"),
//!     Some("+12025550173".to_string())
//! );
//!
//! let text = "Call me at +12025550173 or +442079460958";
//! assert_eq!(extract_phone_numbers_from_text(text).len(), 2);
//!
//! let a = PhoneNumber::parse("+1 202-555-0173").unwrap();
//! let b = PhoneNumber::parse("12025550173").unwrap();
//! assert_eq!(a, b);
//! ```
//!
//! ## Dependencies
//!
//! The crate has none, and no feature flags. Everything it needs is `core` and
//! `alloc` by way of `std`: the country and area-code tables are `const` data
//! built at compile time, so there is no runtime initialization and no global
//! state either.
//!
//! ## Accuracy and scope
//!
//! Validation is driven by the country table in this crate: the country
//! calling code must be known and the national significant number must have a
//! length that country accepts. For numbers in the North American Numbering
//! Plan the area code and central office code are additionally checked against
//! the assigned NANP ranges, which is what makes `US` / `CA` / Caribbean
//! territory attribution exact.
//!
//! **A number must say which country it belongs to.** Give it a leading `+`,
//! an `00` international access code, or a country via
//! [`PhoneNumber::parse_with_country`]. A bare national number is rejected with
//! [`ParseError::MissingCallingCode`] rather than guessed at, because its
//! leading digits usually match *some* country's calling code —
//! `(202) 555-0173` is a Washington DC number, and also a well-formed Egyptian
//! `+20 2555 0173`. The exception is calling code 1, where the area code is
//! checked against the numbering plan, so `12025550173` resolves without a `+`.
//!
//! Outside the NANP the crate does **not** carry per-operator prefix ranges, so
//! a number with a plausible length is accepted even if its prefix is not in
//! service. If you need full carrier-grade validation for every country, use a
//! library backed by the complete libphonenumber metadata.
//!
//! [`detect_phone_number_type`] is likewise metadata-driven. Countries without
//! a published range table in this crate return [`PhoneNumberType::Unknown`]
//! rather than a guess. Range tables are carried for the North American
//! Numbering Plan and for GB, IE, DE, FR, IT, ES, NL, BE, CH, AT, SE, NO, PL,
//! PT, RU, KZ, TR, IN, CN, JP, KR, AU, NZ, BR, ZA and NG.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use constants::COUNTRIES;

pub use definitions::{Country, ParseError, PhoneNumberType};
pub use text::ExtractedPhoneNumber;

mod constants;
mod definitions;
mod nanp;
mod text;

#[cfg(test)]
mod tests;

// ============================================================================
// Country calling code index
//
// Built entirely at compile time: a sorted `(prefix, country index)` array that
// is binary-searched at lookup time. There is no runtime initialization, no
// allocation, and no leaked table.
// ============================================================================

/// For calling codes shared by more than one entry, the ISO code that owns the
/// range by default and is therefore returned by [`extract_country`]. Without
/// it the winner would be an accident of declaration order.
///
/// Ambiguity length alone cannot resolve is inherent to these ranges;
/// [`extract_countries`] returns every candidate.
const PRIMARY_FOR_PREFIX: &[(u32, &str)] = &[
    (1, "US"),
    (7, "RU"),
    (39, "IT"),
    (44, "GB"),
    (47, "NO"),
    (61, "AU"),
    (212, "MA"),
    (262, "RE"),
    (358, "FI"),
    (500, "FK"),
    (590, "GP"),
    (599, "CW"),
];

const COUNTRY_COUNT: usize = COUNTRIES.len();

// A stale entry here would silently do nothing, so check it at compile time.
const _: () = {
    let mut i = 0;
    while i < PRIMARY_FOR_PREFIX.len() {
        let (prefix, code) = PRIMARY_FOR_PREFIX[i];
        let mut found = false;
        let mut j = 0;
        while j < COUNTRY_COUNT {
            if COUNTRIES[j].prefix == prefix && const_str_eq(COUNTRIES[j].code, code) {
                found = true;
            }
            j += 1;
        }
        assert!(
            found,
            "PRIMARY_FOR_PREFIX names a country that is not in COUNTRIES under that calling code"
        );
        i += 1;
    }
};

// `resolve_nanp` looks a country up by area code in one table and indexes the
// other, so the two must agree on every `1XXX` territory.
const _: () = {
    let mut i = 0;
    while i < COUNTRY_COUNT {
        let prefix = COUNTRIES[i].prefix;
        if prefix >= 1000 && prefix < 2000 {
            let area = (prefix - 1000) as u16;
            let mut agrees = false;
            let mut j = 0;
            while j < nanp::NANP_AREA_CODES.len() {
                let (code, iso) = nanp::NANP_AREA_CODES[j];
                if code == area && const_str_eq(iso, COUNTRIES[i].code) {
                    agrees = true;
                }
                j += 1;
            }
            assert!(
                agrees,
                "a NANP territory in COUNTRIES has no matching entry in nanp::NANP_AREA_CODES"
            );
        }
        i += 1;
    }
};

/// `(calling code, index into COUNTRIES)`, sorted ascending by calling code and,
/// within a calling code, by resolution priority then declaration order.
const PREFIX_INDEX: [(u32, u16); COUNTRY_COUNT] = build_prefix_index();

const fn const_str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// `0` if this entry is the designated primary for its calling code, else `1`.
const fn primary_rank(index: usize) -> u8 {
    let prefix = COUNTRIES[index].prefix;
    let mut i = 0;
    while i < PRIMARY_FOR_PREFIX.len() {
        if PRIMARY_FOR_PREFIX[i].0 == prefix {
            if const_str_eq(COUNTRIES[index].code, PRIMARY_FOR_PREFIX[i].1) {
                return 0;
            }
            return 1;
        }
        i += 1;
    }
    1
}

const fn build_prefix_index() -> [(u32, u16); COUNTRY_COUNT] {
    // (calling code, priority rank, declaration index)
    let mut items = [(0u32, 0u8, 0u16); COUNTRY_COUNT];
    let mut i = 0;
    while i < COUNTRY_COUNT {
        items[i] = (COUNTRIES[i].prefix, primary_rank(i), i as u16);
        i += 1;
    }

    // Insertion sort: stable, and const-compatible.
    let mut i = 1;
    while i < COUNTRY_COUNT {
        let mut j = i;
        while j > 0 {
            let (pp, pr, pi) = items[j - 1];
            let (cp, cr, ci) = items[j];
            let swap = pp > cp || (pp == cp && (pr > cr || (pr == cr && pi > ci)));
            if !swap {
                break;
            }
            items[j - 1] = (cp, cr, ci);
            items[j] = (pp, pr, pi);
            j -= 1;
        }
        i += 1;
    }

    let mut out = [(0u32, 0u16); COUNTRY_COUNT];
    let mut i = 0;
    while i < COUNTRY_COUNT {
        out[i] = (items[i].0, items[i].2);
        i += 1;
    }
    out
}

/// Shortest national significant number any country in the table accepts.
pub(crate) const MIN_NATIONAL_LEN: usize = min_national_len();
/// Longest national significant number any country in the table accepts.
pub(crate) const MAX_NATIONAL_LEN: usize = max_national_len();

const fn min_national_len() -> usize {
    let mut best = u8::MAX;
    let mut i = 0;
    while i < COUNTRY_COUNT {
        let lens = COUNTRIES[i].phone_lengths;
        let mut j = 0;
        while j < lens.len() {
            if lens[j] < best {
                best = lens[j];
            }
            j += 1;
        }
        i += 1;
    }
    best as usize
}

const fn max_national_len() -> usize {
    let mut best = 0u8;
    let mut i = 0;
    while i < COUNTRY_COUNT {
        let lens = COUNTRIES[i].phone_lengths;
        let mut j = 0;
        while j < lens.len() {
            if lens[j] > best {
                best = lens[j];
            }
            j += 1;
        }
        i += 1;
    }
    best as usize
}

/// Maximum number of significant digits in an E.164 number.
const E164_MAX_DIGITS: usize = 15;

/// Every table entry that uses `prefix`, in resolution order.
fn entries_for_prefix(prefix: u32) -> impl Iterator<Item = &'static Country> {
    let lo = PREFIX_INDEX.partition_point(|&(p, _)| p < prefix);
    let hi = PREFIX_INDEX.partition_point(|&(p, _)| p <= prefix);
    PREFIX_INDEX[lo..hi]
        .iter()
        .map(|&(_, idx)| &COUNTRIES[idx as usize])
}

// ============================================================================
// Input preparation: extensions, vanity letters, digit extraction
// ============================================================================

/// Split a trailing extension from the number, returning both parts.
///
/// Recognises `ext`, `ext.`, `ext:`, `ext-`, `extn`, `extension`, a bare `x`,
/// `#`, `,`, `;` and the DTMF pause characters `p` and `w`. A marker only
/// counts at a word boundary and only when digits (or nothing) follow, so
/// `next`, `text` and `1-800-FLOWERS` are never mistaken for one.
fn split_extension(input: &str) -> (&str, Option<&str>) {
    let bytes = input.as_bytes();
    // Tracked incrementally; rescanning at each marker would be quadratic.
    let mut seen_digit = false;

    for i in 0..bytes.len() {
        let at_word_start = i == 0 || !bytes[i - 1].is_ascii_alphabetic();

        // A marker only counts once some digits have been seen.
        if !at_word_start || !seen_digit {
            seen_digit |= bytes[i].is_ascii_digit();
            continue;
        }

        let Some(marker_len) = extension_marker_len(&bytes[i..]) else {
            seen_digit |= bytes[i].is_ascii_digit();
            continue;
        };

        let rest = input[i + marker_len..].trim();
        let rest = rest.trim_start_matches([':', '-', '.', '=', ' ']).trim();
        if rest.is_empty() {
            return (input[..i].trim_end(), None);
        }
        if rest.bytes().all(|b| b.is_ascii_digit()) {
            return (input[..i].trim_end(), Some(rest));
        }
        // Not an extension after all (more letters in a vanity string, say).
    }

    (input, None)
}

/// Length of the extension marker at the start of `bytes`, if there is one.
fn extension_marker_len(bytes: &[u8]) -> Option<usize> {
    const WORD_MARKERS: &[&[u8]] = &[b"extension", b"extn", b"ext", b"x"];

    // RFC 3966 spells the extension as a URI parameter.
    if bytes.len() >= 5 && bytes[..5].eq_ignore_ascii_case(b";ext=") {
        return Some(5);
    }

    for marker in WORD_MARKERS {
        if bytes.len() >= marker.len()
            && bytes[..marker.len()].eq_ignore_ascii_case(marker)
            // A longer word starting with the marker is not a marker.
            && !bytes
                .get(marker.len())
                .is_some_and(|b| b.is_ascii_alphabetic())
        {
            return Some(marker.len());
        }
    }

    match bytes.first() {
        Some(b'#') | Some(b',') | Some(b';') => Some(1),
        Some(b'p') | Some(b'P') | Some(b'w') | Some(b'W')
            if !bytes.get(1).is_some_and(|b| b.is_ascii_alphabetic()) =>
        {
            Some(1)
        }
        _ => None,
    }
}

/// Strip the `tel:` scheme from an RFC 3966 URI so the crate can re-parse its
/// own [`PhoneFormat::RFC3966`] output.
fn strip_tel_uri(input: &str) -> &str {
    if input.len() >= 4 && input.as_bytes()[..4].eq_ignore_ascii_case(b"tel:") {
        return input[4..].trim_start();
    }
    input
}

/// Map a phone-keypad vanity letter to its digit.
fn vanity_digit(c: char) -> Option<u8> {
    let d = match c.to_ascii_uppercase() {
        'A' | 'B' | 'C' => b'2',
        'D' | 'E' | 'F' => b'3',
        'G' | 'H' | 'I' => b'4',
        'J' | 'K' | 'L' => b'5',
        'M' | 'N' | 'O' => b'6',
        'P' | 'Q' | 'R' | 'S' => b'7',
        'T' | 'U' | 'V' => b'8',
        'W' | 'X' | 'Y' | 'Z' => b'9',
        _ => return None,
    };
    Some(d)
}

/// Characters accepted between digits.
fn is_separator(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t'
            | '-'
            | '.'
            | '/'
            | '\u{00A0}' // no-break space
            | '\u{202F}' // narrow no-break space
            | '\u{2007}' // figure space
            | '\u{2013}' // en dash
            | '\u{2014}' // em dash
            | '\u{2012}' // figure dash
            | '\u{2212}' // minus sign
            | '\u{FF0D}' // fullwidth hyphen-minus
    )
}

/// ASCII digit value of `c`, accepting fullwidth forms.
fn digit_value(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8),
        '\u{FF10}'..='\u{FF19}' => Some(b'0' + (c as u32 - 0xFF10) as u8),
        _ => None,
    }
}

/// Digits of a phone number, extracted from arbitrary formatting.
struct Digits {
    buf: [u8; 32],
    len: usize,
    /// Whether the input marked itself as an international number, by a leading
    /// `+` or by the `00` access code. See [`parse_internal`] for why it
    /// matters.
    explicit_international: bool,
}

impl Digits {
    fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    fn as_str(&self) -> &str {
        // Infallible: the buffer only ever receives ASCII digit bytes.
        std::str::from_utf8(self.as_slice()).expect("digit buffer is ASCII")
    }
}

/// Validate the characters of `input` and extract its significant digits.
///
/// Vanity letters are only converted when the input already looks like a phone
/// number — at least three digits must precede the first letter. That is what
/// stops arbitrary prose such as `"HELLO WORLD"` from being read as a number.
/// Any other character is rejected, so validation and normalization can never
/// disagree about what is acceptable.
fn extract_digits(input: &str) -> Result<Digits, ParseError> {
    const VANITY_MIN_LEADING_DIGITS: usize = 3;

    let mut out = Digits {
        buf: [0u8; 32],
        len: 0,
        explicit_international: false,
    };
    let mut paren_depth: i32 = 0;
    let mut seen_significant = false;
    let mut seen_plus = false;

    for c in input.trim().chars() {
        if let Some(d) = digit_value(c) {
            if out.len == out.buf.len() {
                return Err(ParseError::TooLong);
            }
            out.buf[out.len] = d;
            out.len += 1;
            seen_significant = true;
            continue;
        }

        match c {
            // Once only, and before any digit, so `(+44) 20 …` works.
            '+' | '\u{FF0B}' if out.len == 0 && !seen_plus => {
                seen_plus = true;
                out.explicit_international = true;
            }
            '(' | '\u{FF08}' => paren_depth += 1,
            ')' | '\u{FF09}' => {
                paren_depth -= 1;
                if paren_depth < 0 {
                    return Err(ParseError::InvalidCharacter);
                }
            }
            c if is_separator(c) => {}
            c if c.is_ascii_alphabetic() => {
                if out.len < VANITY_MIN_LEADING_DIGITS {
                    return Err(ParseError::InvalidCharacter);
                }
                let Some(d) = vanity_digit(c) else {
                    return Err(ParseError::InvalidCharacter);
                };
                if out.len == out.buf.len() {
                    return Err(ParseError::TooLong);
                }
                out.buf[out.len] = d;
                out.len += 1;
            }
            _ => return Err(ParseError::InvalidCharacter),
        }
    }

    if paren_depth != 0 {
        return Err(ParseError::InvalidCharacter);
    }
    if !seen_significant {
        return Err(ParseError::Empty);
    }

    // Strip the zeros an access code leaves before the calling code ("0044…").
    // The "011…" form needs country context and is left to
    // `PhoneNumber::try_parse_with_country`.
    let mut start = 0;
    while start < out.len && out.buf[start] == b'0' {
        start += 1;
    }
    if start >= out.len {
        return Err(ParseError::Empty);
    }
    if start > 0 {
        // One zero is a national trunk prefix; two or more can only be the
        // international access code, since no national number starts with `00`.
        out.explicit_international |= start >= 2;
        out.buf.copy_within(start..out.len, 0);
        out.len -= start;
    }

    if out.len > E164_MAX_DIGITS {
        return Err(ParseError::TooLong);
    }

    Ok(out)
}

// ============================================================================
// Country resolution
// ============================================================================

/// A resolved number: which country owns it and where its national significant
/// number starts within the digit string.
struct Resolved {
    country: &'static Country,
    prefix_len: usize,
    /// Offset of the national significant number, at or after `prefix_len`.
    /// Greater than `prefix_len` when a national trunk zero was stripped.
    national_start: usize,
}

/// Count of leading `'0'` bytes.
fn leading_zeros(digits: &[u8]) -> usize {
    digits.iter().take_while(|&&b| b == b'0').count()
}

/// Countries whose national significant number legitimately starts with `0`.
///
/// Everywhere else a leading zero is a trunk prefix and is stripped. Here it is
/// part of the number: `IT` (`+39 06 …`, and `VA` shares the plan), `SM`
/// (`0549 xxxxxx`), `CI` and `BJ` (post-renumbering, zero mandatory from
/// abroad). Writing the zero nationally is not enough to qualify — Gabon does,
/// but the ITU has international callers dial the digits after it.
const LEADING_ZERO_SIGNIFICANT: &[&str] = &["BJ", "CI", "IT", "SM", "VA"];

fn leading_zero_significant(code: &str) -> bool {
    LEADING_ZERO_SIGNIFICANT.contains(&code)
}

/// Resolve the country for a digit string.
///
/// Country calling codes are prefix-free outside the NANP, so the calling code
/// is determined by the digits alone; only the reading of a leading zero on the
/// national part varies, and that is decided per country by
/// [`LEADING_ZERO_SIGNIFICANT`].
fn resolve_country(digits: &[u8]) -> Result<Resolved, ParseError> {
    if digits.is_empty() {
        return Err(ParseError::Empty);
    }

    if digits[0] == b'1' {
        return resolve_nanp(digits);
    }

    let mut saw_prefix = false;

    for prefix_len in 1..=3usize {
        if digits.len() <= prefix_len {
            break;
        }
        let prefix = prefix_value(&digits[..prefix_len]);
        let zeros = leading_zeros(&digits[prefix_len..]);

        for country in entries_for_prefix(prefix) {
            saw_prefix = true;

            // Where the zero is not significant it is a trunk prefix, so
            // keeping it is not an alternative reading at all.
            let (starts, count) = if zeros == 0 {
                ([prefix_len, 0], 1)
            } else if leading_zero_significant(country.code) {
                ([prefix_len, prefix_len + zeros], 2)
            } else {
                ([prefix_len + zeros, 0], 1)
            };

            for &national_start in &starts[..count] {
                let national_len = digits.len() - national_start;
                if country.phone_lengths.contains(&(national_len as u8)) {
                    return Ok(Resolved {
                        country,
                        prefix_len,
                        national_start,
                    });
                }
            }
        }
    }

    if saw_prefix {
        Err(ParseError::InvalidLength)
    } else {
        Err(ParseError::UnknownCountryCode)
    }
}

/// Resolve a `+1` number using the assigned NANP area codes.
///
/// Length alone cannot tell the United States, Canada and the Caribbean
/// territories apart — they all use `1` + 10 digits. Checking the area code
/// against the numbering plan makes the attribution exact and rejects
/// structurally impossible numbers such as `+1 111 111 1111`.
fn resolve_nanp(digits: &[u8]) -> Result<Resolved, ParseError> {
    const NANP_NSN_LEN: usize = 10;

    let national = &digits[1..];
    if national.len() != NANP_NSN_LEN {
        return Err(ParseError::InvalidLength);
    }

    let area = prefix_value(&national[..3]) as u16;
    let exchange = prefix_value(&national[3..6]) as u16;

    if !nanp::is_valid_area_code_format(area) {
        return Err(ParseError::InvalidAreaCode);
    }
    if !nanp::is_valid_exchange_format(exchange) {
        return Err(ParseError::InvalidExchange);
    }

    let iso = nanp::country_for_area_code(area).ok_or(ParseError::InvalidAreaCode)?;

    // Territories have their own four-digit calling code entry (1268 = AG);
    // the US and Canada share the bare `1`.
    let territory_prefix = 1000 + area as u32;
    if let Some(country) = entries_for_prefix(territory_prefix)
        .find(|c| c.phone_lengths.contains(&((NANP_NSN_LEN - 3) as u8)))
    {
        return Ok(Resolved {
            country,
            prefix_len: 4,
            national_start: 4,
        });
    }

    let country = if nanp::is_non_geographic(iso) {
        // NANP-wide service ranges belong to no country; fall back to the
        // primary `+1` entry and let classification report the service class.
        entries_for_prefix(1).next()
    } else {
        entries_for_prefix(1).find(|c| c.code == iso)
    }
    .ok_or(ParseError::UnknownCountryCode)?;

    if !country.phone_lengths.contains(&(NANP_NSN_LEN as u8)) {
        return Err(ParseError::InvalidLength);
    }

    Ok(Resolved {
        country,
        prefix_len: 1,
        national_start: 1,
    })
}

/// Whether `prefix` is under calling code 1 — either the bare `1` shared by the
/// US and Canada, or a four-digit `1` + area code territory entry.
#[inline]
const fn is_nanp(prefix: u32) -> bool {
    prefix == 1 || is_nanp_territory(prefix)
}

/// Whether `prefix` is a four-digit NANP territory entry such as `1268`.
#[inline]
const fn is_nanp_territory(prefix: u32) -> bool {
    prefix >= 1000 && prefix < 2000
}

/// Interpret up to four ASCII digit bytes as a number.
fn prefix_value(digits: &[u8]) -> u32 {
    digits
        .iter()
        .fold(0u32, |acc, &b| acc * 10 + (b - b'0') as u32)
}

// ============================================================================
// Core parsing
// ============================================================================

/// Everything the crate knows about one parsed number.
struct Parsed {
    normalized: String,
    country: &'static Country,
    phone_type: PhoneNumberType,
    extension: Option<String>,
}

/// Parse, validate and normalize in a single pass.
///
/// Input that does not mark itself as international — no `+`, no `00` access
/// code — is only accepted when the resolution is *checked* rather than guessed,
/// which today means the North American Numbering Plan, where the area code is
/// verified against the assigned ranges. Everywhere else, reading the leading
/// digits as a calling code is indistinguishable from a national number that
/// happens to start with those digits, and answering anyway turns a US number
/// into an Egyptian one. See [`ParseError::MissingCallingCode`].
fn parse_internal(input: &str) -> Result<Parsed, ParseError> {
    let input = strip_tel_uri(input.trim());
    let (base, extension) = split_extension(input);
    let digits = extract_digits(base)?;
    let slice = digits.as_slice();

    // Early-out with a better error than the resolver would give; the floor is
    // the shortest national number plus at least one calling-code digit.
    if slice.len() < MIN_NATIONAL_LEN + 1 {
        return Err(ParseError::TooShort);
    }

    let resolved = resolve_country(slice)?;

    if !digits.explicit_international && !is_nanp(resolved.country.prefix) {
        return Err(ParseError::MissingCallingCode);
    }

    let national = &digits.as_str()[resolved.national_start..];

    let mut normalized = String::with_capacity(national.len() + 5);
    normalized.push('+');
    push_prefix_digits(&mut normalized, resolved.country.prefix);
    normalized.push_str(national);

    let phone_type = classify_phone_number_type(national, resolved.country);

    Ok(Parsed {
        normalized,
        country: resolved.country,
        phone_type,
        extension: extension.map(str::to_string),
    })
}

/// Validates whether a phone number is valid.
///
/// A number is valid when its characters are acceptable, its country calling
/// code is known, and its national significant number has a length that
/// country's numbering plan accepts. Numbers in the North American Numbering
/// Plan are additionally checked against the assigned area codes.
///
/// This is exactly `normalize_phone_number(..).is_some()`, so the two can never
/// disagree.
///
/// # Examples
/// ```
/// use phonelib::is_valid_phone_number;
///
/// assert!(is_valid_phone_number("+12025550173"));
/// assert!(is_valid_phone_number("+1 (202) 555-0173"));
/// assert!(!is_valid_phone_number("invalid"));
/// // Unassigned NANP area code.
/// assert!(!is_valid_phone_number("+11111111111"));
/// ```
#[must_use]
pub fn is_valid_phone_number(phone_number: &str) -> bool {
    parse_internal(phone_number).is_ok()
}

/// Normalizes a phone number to E.164 format.
///
/// Returns `None` if the number is not valid. Any extension is removed; E.164
/// has no representation for one. Use [`PhoneNumber::parse`] and
/// [`PhoneNumber::extension`] to keep it.
///
/// The result is idempotent: normalizing an already-normalized number returns
/// it unchanged.
///
/// # Examples
/// ```
/// use phonelib::normalize_phone_number;
///
/// assert_eq!(
///     normalize_phone_number("12025550173"),
///     Some("+12025550173".to_string())
/// );
/// // A leading zero that is part of the number is preserved.
/// assert_eq!(
///     normalize_phone_number("+390669821234"),
///     Some("+390669821234".to_string())
/// );
/// // A national trunk zero is removed.
/// assert_eq!(
///     normalize_phone_number("+44 (0)20 7946 0958"),
///     Some("+442079460958".to_string())
/// );
/// ```
#[must_use]
pub fn normalize_phone_number(phone_number: &str) -> Option<String> {
    try_normalize_phone_number(phone_number).ok()
}

/// Like [`normalize_phone_number`] but reports why the number was rejected.
///
/// # Examples
/// ```
/// use phonelib::{try_normalize_phone_number, ParseError};
///
/// assert_eq!(
///     try_normalize_phone_number("+12025550173"),
///     Ok("+12025550173".to_string())
/// );
/// assert_eq!(
///     try_normalize_phone_number("+9999999999999"),
///     Err(ParseError::UnknownCountryCode)
/// );
/// ```
pub fn try_normalize_phone_number(phone_number: &str) -> Result<String, ParseError> {
    parse_internal(phone_number).map(|p| p.normalized)
}

/// Normalizes a phone number in place to E.164 format.
///
/// On success the buffer is replaced by the normalized number and the same
/// string is returned, so the buffer and the return value always agree and a
/// second call is a no-op. On failure the buffer is left **unchanged**.
///
/// # Examples
/// ```
/// use phonelib::normalize_phone_number_in_place;
///
/// let mut phone = "+1 (202) 555-0173".to_string();
/// assert_eq!(
///     normalize_phone_number_in_place(&mut phone),
///     Some("+12025550173".to_string())
/// );
/// assert_eq!(phone, "+12025550173");
///
/// let mut invalid = "nonsense".to_string();
/// assert_eq!(normalize_phone_number_in_place(&mut invalid), None);
/// assert_eq!(invalid, "nonsense"); // untouched
/// ```
pub fn normalize_phone_number_in_place(phone_number: &mut String) -> Option<String> {
    let normalized = try_normalize_phone_number(phone_number.as_str()).ok()?;
    phone_number.clear();
    phone_number.push_str(&normalized);
    Some(normalized)
}

/// Extracts country information from a phone number.
///
/// When a calling code is shared by several countries this returns the primary
/// one; use [`extract_countries`] to see every candidate.
///
/// # Examples
/// ```
/// use phonelib::extract_country;
///
/// assert_eq!(extract_country("+12025550173").unwrap().code, "US");
/// assert_eq!(extract_country("+14165550198").unwrap().code, "CA");
/// assert_eq!(extract_country("+442079460958").unwrap().code, "GB");
/// assert_eq!(extract_country("+79161234567").unwrap().code, "RU");
/// ```
#[must_use]
pub fn extract_country(phone_number: &str) -> Option<&'static Country> {
    parse_internal(phone_number).ok().map(|p| p.country)
}

/// Every country whose numbering plan could account for this number.
///
/// Some calling codes are shared by countries that a number's length cannot
/// distinguish (`+7` is Russia and Kazakhstan, `+44` is the UK, Guernsey, the
/// Isle of Man and Jersey). [`extract_country`] returns the primary one; this
/// returns all of them, most likely first.
///
/// Numbers under calling code 1 always yield exactly one country: the NANP area
/// code identifies it unambiguously.
///
/// # Examples
/// ```
/// use phonelib::extract_countries;
///
/// let codes: Vec<_> = extract_countries("+79161234567")
///     .iter()
///     .map(|c| c.code)
///     .collect();
/// assert_eq!(codes, vec!["RU", "KZ"]);
///
/// // The NANP area code leaves no ambiguity.
/// let codes: Vec<_> = extract_countries("+14165550198")
///     .iter()
///     .map(|c| c.code)
///     .collect();
/// assert_eq!(codes, vec!["CA"]);
/// ```
#[must_use]
pub fn extract_countries(phone_number: &str) -> Vec<&'static Country> {
    let Ok((digits, resolved)) = parse_for_candidates(phone_number) else {
        return Vec::new();
    };

    // Resolved by area code, not by length: nothing to disambiguate.
    if is_nanp(resolved.country.prefix) {
        return vec![resolved.country];
    }

    let national_len = digits.len() - resolved.national_start;
    let prefix = prefix_value(&digits[..resolved.prefix_len]);

    entries_for_prefix(prefix)
        .filter(|c| c.phone_lengths.contains(&(national_len as u8)))
        .collect()
}

fn parse_for_candidates(input: &str) -> Result<(Vec<u8>, Resolved), ParseError> {
    let (base, _) = split_extension(strip_tel_uri(input.trim()));
    let digits = extract_digits(base)?;
    let resolved = resolve_country(digits.as_slice())?;
    if !digits.explicit_international && !is_nanp(resolved.country.prefix) {
        return Err(ParseError::MissingCallingCode);
    }
    Ok((digits.as_slice().to_vec(), resolved))
}

// ============================================================================
// Country table access
// ============================================================================

/// Every country and territory the crate knows about.
///
/// # Examples
/// ```
/// use phonelib::countries;
///
/// assert!(countries().iter().any(|c| c.code == "JP"));
/// ```
#[must_use]
pub fn countries() -> &'static [Country] {
    &COUNTRIES
}

/// Look up a country by its ISO 3166-1 alpha-2 code.
///
/// A few countries operate more than one calling code (overlay area codes in
/// the NANP); this returns the first entry for the code. Use
/// [`countries_by_code`] for all of them.
///
/// # Examples
/// ```
/// use phonelib::country_by_code;
///
/// assert_eq!(country_by_code("JP").unwrap().prefix, 81);
/// assert!(country_by_code("ZZ").is_none());
/// ```
#[must_use]
pub fn country_by_code(code: &str) -> Option<&'static Country> {
    COUNTRIES.iter().find(|c| c.code.eq_ignore_ascii_case(code))
}

/// Every table entry for an ISO 3166-1 alpha-2 code.
///
/// # Examples
/// ```
/// use phonelib::countries_by_code;
///
/// // The Dominican Republic has three overlay calling codes.
/// assert!(countries_by_code("DO").len() >= 1);
/// ```
#[must_use]
pub fn countries_by_code(code: &str) -> Vec<&'static Country> {
    COUNTRIES
        .iter()
        .filter(|c| c.code.eq_ignore_ascii_case(code))
        .collect()
}

/// Every country that uses a given E.164 calling code, in resolution order.
///
/// # Examples
/// ```
/// use phonelib::countries_by_calling_code;
///
/// let codes: Vec<_> = countries_by_calling_code(7).iter().map(|c| c.code).collect();
/// assert_eq!(codes.first(), Some(&"RU"));
/// ```
#[must_use]
pub fn countries_by_calling_code(calling_code: u32) -> Vec<&'static Country> {
    entries_for_prefix(calling_code).collect()
}

// ============================================================================
// Formatting
// ============================================================================

/// Phone number format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PhoneFormat {
    /// E.164: `+12025550173`
    E164,
    /// International: `+1 202 555 0173`
    International,
    /// National, including the country's trunk prefix where it has one:
    /// `(202) 555-0173`, `020 7946 0958`
    National,
    /// RFC 3966 `tel:` URI: `tel:+12025550173`, with `;ext=` when the number
    /// carries an extension.
    RFC3966,
}

/// National trunk prefix by ISO code, for countries that use one.
///
/// Countries absent from this table either have no trunk prefix (Italy, Spain,
/// Denmark, Norway, Portugal) or are not yet covered, in which case the
/// national format is the grouped national significant number alone.
const NATIONAL_PREFIX: &[(&str, &str)] = &[
    ("AT", "0"),
    ("AU", "0"),
    ("BE", "0"),
    ("BR", "0"),
    ("CH", "0"),
    ("CN", "0"),
    ("DE", "0"),
    ("FR", "0"),
    ("GB", "0"),
    ("GG", "0"),
    ("IE", "0"),
    ("IM", "0"),
    ("IN", "0"),
    ("JE", "0"),
    ("JP", "0"),
    ("KR", "0"),
    ("NG", "0"),
    ("NL", "0"),
    ("NZ", "0"),
    // Poland abolished its trunk prefix in 2009; numbers are dialled as the
    // bare nine digits, so `PL` deliberately has no entry here.
    ("SE", "0"),
    ("TR", "0"),
    ("ZA", "0"),
    ("RU", "8"),
];

/// What goes between the trunk prefix and the number when writing a national
/// number.
///
/// Nearly every plan joins the trunk `0` straight onto the area code
/// (`020 7946 0958`). Russia's trunk `8` is written as its own token
/// (`8 916 123-45-67`), so gluing it produces `8916 …`, which reads as a
/// four-digit area code.
fn trunk_separator(code: &str) -> &'static str {
    if code == "RU" {
        " "
    } else {
        ""
    }
}

fn national_prefix_for(code: &str) -> &'static str {
    NATIONAL_PREFIX
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, p)| *p)
        .unwrap_or("")
}

/// Format a phone number according to the specified format.
///
/// Returns `None` if the number is not valid.
///
/// Note that [`PhoneFormat::National`] output is, by design, not a global
/// number: it has no country calling code and cannot be re-parsed on its own.
/// Round-trip it with [`PhoneNumber::parse_with_country`]. `E164` and `RFC3966`
/// output re-parse directly.
///
/// # Examples
/// ```
/// use phonelib::{format_phone_number, PhoneFormat};
///
/// assert_eq!(
///     format_phone_number("+12025550173", PhoneFormat::E164),
///     Some("+12025550173".to_string())
/// );
/// assert_eq!(
///     format_phone_number("+12025550173", PhoneFormat::National),
///     Some("(202) 555-0173".to_string())
/// );
/// assert_eq!(
///     format_phone_number("+12025550173", PhoneFormat::RFC3966),
///     Some("tel:+12025550173".to_string())
/// );
/// ```
#[must_use]
pub fn format_phone_number(phone_number: &str, format: PhoneFormat) -> Option<String> {
    let parsed = parse_internal(phone_number).ok()?;
    Some(render(
        &parsed.normalized,
        parsed.country,
        parsed.extension.as_deref(),
        format,
    ))
}

fn render(
    normalized: &str,
    country: &Country,
    extension: Option<&str>,
    format: PhoneFormat,
) -> String {
    let plen = prefix_digit_count(country.prefix);
    let national = &normalized[1 + plen..];

    match format {
        PhoneFormat::E164 => normalized.to_string(),

        PhoneFormat::International => {
            // Under calling code 1 the area code is written as part of the
            // national number, even where the table folds it into the prefix.
            if let Some(nanp) = nanp_national(country, national) {
                return format!("+1 {} {} {}", &nanp[0..3], &nanp[3..6], &nanp[6..]);
            }
            let grouped = group_national(national, country, ' ');
            let mut out = String::with_capacity(2 + plen + grouped.len());
            out.push('+');
            push_prefix_digits(&mut out, country.prefix);
            out.push(' ');
            out.push_str(&grouped);
            out
        }

        PhoneFormat::National => {
            // The NANP convention omits the trunk `1` in written form.
            if let Some(nanp) = nanp_national(country, national) {
                return format_nanp_national(&nanp);
            }
            let trunk = national_prefix_for(country.code);
            let grouped = group_national(national, country, ' ');
            let mut out = String::with_capacity(trunk.len() + grouped.len() + 1);
            out.push_str(trunk);
            out.push_str(trunk_separator(country.code));
            out.push_str(&grouped);
            out
        }

        PhoneFormat::RFC3966 => {
            // RFC 3966 §3: a global number is the `+`-prefixed E.164 digits.
            // Visual separators are optional; omitting them keeps it parseable.
            let mut out = String::with_capacity(4 + normalized.len() + 8);
            out.push_str("tel:");
            out.push_str(normalized);
            if let Some(ext) = extension {
                out.push_str(";ext=");
                out.push_str(ext);
            }
            out
        }
    }
}

/// The ten-digit NANP national number, reuniting the area code with the
/// subscriber number for countries whose area code this crate folds into the
/// calling code. Returns `None` for countries outside calling code 1.
fn nanp_national(country: &Country, national: &str) -> Option<String> {
    if country.prefix == 1 {
        return (national.len() == 10).then(|| national.to_string());
    }
    if is_nanp_territory(country.prefix) {
        let mut full = String::with_capacity(10);
        push_prefix_digits(&mut full, country.prefix - 1000);
        full.push_str(national);
        return (full.len() == 10).then_some(full);
    }
    None
}

fn format_nanp_national(national: &str) -> String {
    if national.len() != 10 {
        return national.to_string();
    }
    let mut s = String::with_capacity(14);
    s.push('(');
    s.push_str(&national[0..3]);
    s.push_str(") ");
    s.push_str(&national[3..6]);
    s.push('-');
    s.push_str(&national[6..]);
    s
}

fn group_national(number: &str, country: &Country, separator: char) -> String {
    if country.code == "US" || country.code == "CA" {
        if let Some(groups) = format_grouped(number, &[3, 3, 4], separator) {
            return groups;
        }
    }

    national_group_sizes(country.code, number)
        .or_else(|| default_group_sizes(number.len()))
        .and_then(|groups| format_grouped(number, groups, separator))
        .unwrap_or_else(|| number.to_string())
}

fn national_group_sizes(country_code: &str, number: &str) -> Option<&'static [usize]> {
    // Italy keeps its trunk zero; major-city area codes are two digits, while
    // mobile numbers have no zero and group in threes.
    if country_code == "IT" && number.len() == 10 {
        return Some(if number.starts_with('0') {
            &[2, 4, 4]
        } else {
            &[3, 3, 4]
        });
    }

    match (country_code, number.len()) {
        // London, Cardiff and the other two-digit UK area codes.
        ("GB", 10) => Some(&[2, 4, 4]),
        ("GB", 9) => Some(&[3, 3, 3]),
        ("DE", 10) => Some(&[2, 4, 4]),
        ("DE", 11) => Some(&[3, 4, 4]),
        ("FR", 9) => Some(&[1, 2, 2, 2, 2]),
        // "01 234 5678" / "085 123 4567" once the trunk zero is prepended.
        ("IE", 8) => Some(&[1, 3, 4]),
        ("IE", 9) => Some(&[2, 3, 4]),
        // "082 123 4567" once the trunk zero is prepended.
        ("ZA", 9) => Some(&[2, 3, 4]),
        // Poland has no trunk prefix: "12 345 67 89".
        ("PL", 9) => Some(&[2, 3, 2, 2]),
        ("IN", 10) => Some(&[5, 5]),
        ("AU", 9) => Some(&[1, 4, 4]),
        ("NL", 9) => Some(&[1, 4, 4]),
        ("JP", 10) => Some(&[2, 4, 4]),
        ("CN", 11) => Some(&[3, 4, 4]),
        ("BR", 11) => Some(&[2, 5, 4]),
        ("BR", 10) => Some(&[2, 4, 4]),
        ("RU", 10) => Some(&[3, 3, 2, 2]),
        _ => None,
    }
}

fn default_group_sizes(number_len: usize) -> Option<&'static [usize]> {
    match number_len {
        6 => Some(&[3, 3]),
        7 => Some(&[3, 4]),
        8 => Some(&[4, 4]),
        9 => Some(&[3, 3, 3]),
        10 => Some(&[3, 3, 4]),
        11 => Some(&[3, 4, 4]),
        12 => Some(&[3, 3, 3, 3]),
        13 => Some(&[3, 3, 3, 4]),
        14 => Some(&[3, 3, 4, 4]),
        15 => Some(&[3, 4, 4, 4]),
        _ => None,
    }
}

fn format_grouped(number: &str, groups: &[usize], separator: char) -> Option<String> {
    if groups.iter().sum::<usize>() != number.len() {
        return None;
    }

    let mut out = String::with_capacity(number.len() + groups.len());
    let mut start = 0;
    for (index, &len) in groups.iter().enumerate() {
        if index > 0 {
            out.push(separator);
        }
        out.push_str(&number[start..start + len]);
        start += len;
    }
    Some(out)
}

// ============================================================================
// Type classification
// ============================================================================

/// Detect the service class of a phone number.
///
/// Returns `None` if the number is not valid, and
/// [`PhoneNumberType::Unknown`] if it is valid but the crate has no range
/// table for its country.
///
/// # Examples
/// ```
/// use phonelib::{detect_phone_number_type, PhoneNumberType};
///
/// assert_eq!(
///     detect_phone_number_type("+18005550199"),
///     Some(PhoneNumberType::TollFree)
/// );
/// assert_eq!(
///     detect_phone_number_type("+447911123456"),
///     Some(PhoneNumberType::Mobile)
/// );
/// assert_eq!(detect_phone_number_type("nonsense"), None);
/// ```
#[must_use]
pub fn detect_phone_number_type(phone_number: &str) -> Option<PhoneNumberType> {
    parse_internal(phone_number).ok().map(|p| p.phone_type)
}

/// Check if a phone number is a mobile number.
///
/// `false` also means "cannot tell": every North American number is
/// [`PhoneNumberType::FixedLineOrMobile`], so this returns `false` for mobiles
/// there. Match on [`detect_phone_number_type`] to tell the two apart.
#[must_use]
pub fn is_mobile_number(phone_number: &str) -> bool {
    detect_phone_number_type(phone_number) == Some(PhoneNumberType::Mobile)
}

/// Check if a phone number is a landline number.
///
/// See [`is_mobile_number`] for the caveat about unknown and
/// fixed-line-or-mobile types.
#[must_use]
pub fn is_landline_number(phone_number: &str) -> bool {
    detect_phone_number_type(phone_number) == Some(PhoneNumberType::FixedLine)
}

/// Check if a phone number is a toll-free number.
///
/// See [`is_mobile_number`] for the caveat about unknown types.
#[must_use]
pub fn is_toll_free_number(phone_number: &str) -> bool {
    detect_phone_number_type(phone_number) == Some(PhoneNumberType::TollFree)
}

/// Classify a national significant number against its country's ranges.
///
/// Range tables are carried for: US, CA and the NANP service codes, GB, IE,
/// DE, FR, IT, ES, NL, BE, CH, AT, SE, NO, PL, PT, RU, TR, IN, CN, JP, KR, AU,
/// NZ, BR, ZA, NG. Every other country returns [`PhoneNumberType::Unknown`] —
/// the crate does not guess from the leading digit, because the guess is wrong
/// more often than it is right.
fn classify_phone_number_type(national: &str, country: &Country) -> PhoneNumberType {
    use PhoneNumberType::*;

    let b = national.as_bytes();
    if b.is_empty() {
        return Unknown;
    }

    let d1 = b[0];
    let n2 = numeric_prefix(b, 2);
    let n3 = numeric_prefix(b, 3);
    let n4 = numeric_prefix(b, 4);

    // Calling code 1 is classified from the area code, which territories carry
    // in their prefix and the US and Canada in the national number.
    if is_nanp(country.prefix) {
        let area = if country.prefix >= 1000 {
            (country.prefix - 1000) as u16
        } else {
            n3
        };
        return match nanp::country_for_area_code(area) {
            Some(nanp::TOLL_FREE) => TollFree,
            Some(nanp::PREMIUM_RATE) => PremiumRate,
            Some(nanp::NON_GEOGRAPHIC) => match area {
                500 | 521..=529 | 533 | 544 | 566 | 577 | 588 => PersonalNumber,
                700 => Uan,
                _ => Unknown,
            },
            // Not split by service, and portability decoupled prefix from
            // technology.
            Some(_) => FixedLineOrMobile,
            None => Unknown,
        };
    }

    match country.code {
        "GB" => match n4 {
            7000..=7099 => PersonalNumber,
            7600..=7699 => Pager,
            _ => match n3 {
                800 | 808 => TollFree,
                843 | 844 | 845 | 870 | 871 | 872 => SharedCost,
                _ => match d1 {
                    b'1' | b'2' => FixedLine,
                    b'3' => Uan,
                    b'5' => Voip,
                    b'7' => Mobile,
                    // All of `09` is premium rate, not just `090`.
                    b'9' => PremiumRate,
                    _ => Unknown,
                },
            },
        },

        "IE" => match n4 {
            1800 => TollFree,
            1850 | 1890 => SharedCost,
            1520 | 1550 | 1560 | 1570 | 1580 => PremiumRate,
            _ => match n3 {
                // `0818` is a shared-cost range inside the `08` block that
                // otherwise belongs to mobiles.
                818 => SharedCost,
                _ => match d1 {
                    b'8' => Mobile,
                    b'1'..=b'7' | b'9' => FixedLine,
                    _ => Unknown,
                },
            },
        },

        "DE" => match n3 {
            800 => TollFree,
            900 => PremiumRate,
            180 => SharedCost,
            700 => PersonalNumber,
            164 | 168 | 169 => Pager,
            _ => match n2 {
                15..=17 => Mobile,
                _ => FixedLine,
            },
        },

        // Three digits once the trunk zero is stripped, so match on `n3`.
        "FR" => match n3 {
            800..=805 => TollFree,
            806..=809 => Uan,
            810..=819 => SharedCost,
            820..=899 => PremiumRate,
            _ => match d1 {
                b'6' | b'7' => Mobile,
                b'9' => Voip,
                b'1'..=b'5' => FixedLine,
                _ => Unknown,
            },
        },

        // Italy keeps its leading zero, so fixed lines start with '0'.
        "IT" => match n3 {
            800 | 803 => TollFree,
            199 | 848 => SharedCost,
            892 | 894 | 895 | 899 => PremiumRate,
            _ => match d1 {
                b'0' => FixedLine,
                b'3' => Mobile,
                _ => Unknown,
            },
        },

        "ES" => match n3 {
            800 | 900 => TollFree,
            803 | 806 | 807 | 905 => PremiumRate,
            901 | 902 => SharedCost,
            _ => match d1 {
                b'6' | b'7' => Mobile,
                b'8' | b'9' => FixedLine,
                _ => Unknown,
            },
        },

        "NL" => match n3 {
            800 => TollFree,
            900 | 906 | 909 => PremiumRate,
            _ => match d1 {
                b'6' => Mobile,
                _ => FixedLine,
            },
        },

        "BE" => match n3 {
            800 => TollFree,
            900 | 901 | 902 | 903 | 905 | 906 | 907 | 909 => PremiumRate,
            _ => match d1 {
                b'4' => Mobile,
                _ => FixedLine,
            },
        },

        "CH" => match n3 {
            800 => TollFree,
            900 | 901 | 906 => PremiumRate,
            842 | 848 => SharedCost,
            _ => match d1 {
                b'7' => Mobile,
                _ => FixedLine,
            },
        },

        "AT" => match n3 {
            800 => TollFree,
            900 | 930 | 931 | 939 => PremiumRate,
            810 | 820 | 821 | 828 => SharedCost,
            _ => match d1 {
                b'6' => Mobile,
                _ => FixedLine,
            },
        },

        // `020` and `077` are two digits once the trunk zero is stripped.
        "SE" => match n2 {
            20 => TollFree,
            77 => SharedCost,
            _ => match n3 {
                900 | 939 => PremiumRate,
                _ => match d1 {
                    b'7' => Mobile,
                    _ => FixedLine,
                },
            },
        },

        "NO" => match n3 {
            800 => TollFree,
            820 | 829 => PremiumRate,
            _ => match d1 {
                b'4' | b'9' => Mobile,
                _ => FixedLine,
            },
        },

        "PL" => match n3 {
            800 => TollFree,
            700 | 704 => PremiumRate,
            801 => SharedCost,
            _ => match n2 {
                45 | 50 | 51 | 53 | 57 | 60 | 66 | 69 | 72 | 73 | 78 | 79 | 88 => Mobile,
                _ => FixedLine,
            },
        },

        "PT" => match n3 {
            800 => TollFree,
            808 | 809 => SharedCost,
            760..=762 => PremiumRate,
            _ => match d1 {
                b'9' => Mobile,
                b'2' => FixedLine,
                _ => Unknown,
            },
        },

        "RU" | "KZ" => match n3 {
            800 => TollFree,
            809 => PremiumRate,
            _ => match d1 {
                b'9' => Mobile,
                _ => FixedLine,
            },
        },

        "TR" => match n3 {
            800 => TollFree,
            900 => PremiumRate,
            444 => Uan,
            _ => match d1 {
                b'5' => Mobile,
                _ => FixedLine,
            },
        },

        // National numbers carry the STD code, so the two-digit metro codes
        // 79 (Ahmedabad) and 80 (Bengaluru) land inside the mobile range. The
        // others (11, 20, 22, 33, 40, 44) are already in the fixed-line range.
        "IN" => match n2 {
            79 | 80 if b.len() == 10 => FixedLine,
            _ => match d1 {
                b'6'..=b'9' => Mobile,
                b'1'..=b'5' => FixedLine,
                _ => Unknown,
            },
        },

        "CN" => match n3 {
            400 | 800 => TollFree,
            _ => match n2 {
                13..=19 => Mobile,
                _ => FixedLine,
            },
        },

        "JP" => match n3 {
            120 | 800 => TollFree,
            570 => SharedCost,
            990 => PremiumRate,
            _ => match n2 {
                70 | 80 | 90 => Mobile,
                50 => Voip,
                _ => FixedLine,
            },
        },

        // `080` / `060` are two digits once the trunk zero is gone.
        "KR" => match n2 {
            80 => TollFree,
            60 => PremiumRate,
            50 => Voip,
            // Representative numbers share their leading pair with the mobile
            // block; only the length tells them apart.
            15 | 16 | 18 if b.len() == 8 => Uan,
            10..=19 => Mobile,
            _ => FixedLine,
        },

        "AU" => match n4 {
            1800 => TollFree,
            1300 | 1345 => SharedCost,
            1900..=1902 => PremiumRate,
            _ => match d1 {
                b'4' => Mobile,
                b'2' | b'3' | b'7' | b'8' => FixedLine,
                _ => Unknown,
            },
        },

        "NZ" => match n3 {
            800 => TollFree,
            900 => PremiumRate,
            _ => match d1 {
                b'2' => Mobile,
                b'3' | b'4' | b'6' | b'7' | b'9' => FixedLine,
                _ => Unknown,
            },
        },

        // Brazil: an 11-digit national number carries the mobile `9` right
        // after the two-digit area code.
        "BR" => match n3 {
            800 => TollFree,
            300 | 400 => SharedCost,
            900 => PremiumRate,
            _ => {
                if b.len() == 11 && b[2] == b'9' {
                    Mobile
                } else if b.len() == 10 {
                    FixedLine
                } else {
                    Unknown
                }
            }
        },

        "ZA" => match n2 {
            80 => TollFree,
            86 => SharedCost,
            87 => Voip,
            _ => match d1 {
                b'6' | b'7' | b'8' => Mobile,
                _ => FixedLine,
            },
        },

        "NG" => match n2 {
            70 | 80 | 81 | 90 | 91 => Mobile,
            _ => FixedLine,
        },

        _ => Unknown,
    }
}

/// First `count` digits of `b` as a number, or `u16::MAX` if it is shorter.
fn numeric_prefix(b: &[u8], count: usize) -> u16 {
    if b.len() < count {
        return u16::MAX;
    }
    b[..count]
        .iter()
        .fold(0u16, |acc, &x| acc * 10 + (x - b'0') as u16)
}

// ============================================================================
// Digit helpers
// ============================================================================

#[inline]
const fn prefix_digit_count(prefix: u32) -> usize {
    if prefix >= 1000 {
        4
    } else if prefix >= 100 {
        3
    } else if prefix >= 10 {
        2
    } else {
        1
    }
}

/// Push a calling code as ASCII digits without allocating.
#[inline]
fn push_prefix_digits(s: &mut String, prefix: u32) {
    if prefix >= 1000 {
        s.push((b'0' + (prefix / 1000) as u8) as char);
    }
    if prefix >= 100 {
        s.push((b'0' + ((prefix / 100) % 10) as u8) as char);
    }
    if prefix >= 10 {
        s.push((b'0' + ((prefix / 10) % 10) as u8) as char);
    }
    s.push((b'0' + (prefix % 10) as u8) as char);
}

// ============================================================================
// Random generation
// ============================================================================

/// SplitMix64 — a small, well-distributed PRNG.
///
/// Suitable for sample and test data only. It is **not** cryptographically
/// secure and must not be used to generate secrets.
struct Rng(u64);

impl Rng {
    fn from_entropy() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        // Mixing the counter through SplitMix64's finalizer keeps two calls in
        // the same nanosecond — including on different threads — from
        // producing the same stream.
        Rng(time ^ Self::mix(counter.wrapping_mul(0x9E37_79B9_7F4A_7C15)))
    }

    fn mix(mut z: u64) -> u64 {
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        Self::mix(self.0)
    }

    /// Uniform in `0..bound`, rejecting the biased tail.
    fn below(&mut self, bound: u64) -> u64 {
        debug_assert!(bound > 0);
        let zone = u64::MAX - (u64::MAX % bound);
        loop {
            let v = self.next_u64();
            if v < zone {
                return v % bound;
            }
        }
    }

    fn digit(&mut self) -> u8 {
        b'0' + self.below(10) as u8
    }
}

/// Generate a random valid phone number for a specific country.
///
/// The length is chosen uniformly from the lengths that country accepts, and
/// for NANP countries the area code is drawn from the codes actually assigned
/// to that country — so a `"CA"` number really does resolve back to `CA`.
///
/// Every returned number is guaranteed to round-trip: it is valid, already in
/// E.164 form, and [`extract_country`] resolves it back to the country asked
/// for.
///
/// # Returns `None` when
/// - the ISO 3166-1 alpha-2 code is not in the country table, or
/// - another country on the same calling code covers its lengths, so no number
///   can be minted that resolves back to it — Jersey and Guernsey inside `+44`,
///   Kazakhstan inside `+7`, the Vatican inside `+39`.
///
/// # Note
/// The digits come from a PRNG intended for sample and test data. It is not
/// cryptographically secure and must not be used to generate secrets.
///
/// # Examples
/// ```
/// use phonelib::{generate_random_phone_number, extract_country, is_valid_phone_number};
///
/// let number = generate_random_phone_number("CA").unwrap();
/// assert!(is_valid_phone_number(&number));
/// assert_eq!(extract_country(&number).unwrap().code, "CA");
/// ```
#[must_use]
pub fn generate_random_phone_number(country_code: &str) -> Option<String> {
    let country = country_by_code(country_code)?;
    let mut rng = Rng::from_entropy();
    generate_for(country, &mut rng)
}

/// Generate several random valid phone numbers for a country.
///
/// Returns an empty vector if the country code is unknown. See
/// [`generate_random_phone_number`] for the randomness caveat.
///
/// # Examples
/// ```
/// use phonelib::{generate_random_phone_numbers, is_valid_phone_number};
///
/// let numbers = generate_random_phone_numbers("US", 5);
/// assert_eq!(numbers.len(), 5);
/// assert!(numbers.iter().all(|n| is_valid_phone_number(n)));
/// ```
#[must_use]
pub fn generate_random_phone_numbers(country_code: &str, count: usize) -> Vec<String> {
    let Some(country) = country_by_code(country_code) else {
        return Vec::new();
    };

    // Do not trust `count` as an allocation hint; a caller-controlled value
    // reaching `Vec::with_capacity` can abort the process.
    const PREALLOC_CAP: usize = 4096;
    let mut numbers = Vec::with_capacity(count.min(PREALLOC_CAP));
    let mut rng = Rng::from_entropy();

    for _ in 0..count {
        match generate_for(country, &mut rng) {
            Some(number) => numbers.push(number),
            None => break,
        }
    }

    numbers
}

fn generate_for(country: &'static Country, rng: &mut Rng) -> Option<String> {
    if country.phone_lengths.is_empty() {
        return None;
    }

    // Retry rather than emit an invalid number: a random national can land on
    // a range the validator rejects (an unassigned NANP exchange, say).
    for _ in 0..64 {
        let index = rng.below(country.phone_lengths.len() as u64) as usize;
        let length = country.phone_lengths[index] as usize;

        let national = if country.prefix == 1 {
            generate_nanp_national(country.code, rng)?
        } else {
            generate_national(length, rng)
        };

        let mut candidate = String::with_capacity(national.len() + 5);
        candidate.push('+');
        push_prefix_digits(&mut candidate, country.prefix);
        candidate.push_str(&national);

        // The generated number must round-trip: same string back, same country.
        if let Ok(parsed) = parse_internal(&candidate) {
            if parsed.normalized == candidate && parsed.country.code == country.code {
                return Some(candidate);
            }
        }
    }

    None
}

fn generate_national(length: usize, rng: &mut Rng) -> String {
    let mut national = String::with_capacity(length);
    for index in 0..length {
        // A leading zero would be read as a trunk prefix, so resample it rather
        // than clamping — clamping doubles the probability of a leading '1'.
        let digit = loop {
            let d = rng.digit();
            if index > 0 || d != b'0' {
                break d;
            }
        };
        national.push(char::from(digit));
    }
    national
}

fn generate_nanp_national(iso: &str, rng: &mut Rng) -> Option<String> {
    let codes: Vec<u16> = nanp::NANP_AREA_CODES
        .iter()
        .filter(|(_, c)| *c == iso)
        .map(|(area, _)| *area)
        .collect();
    if codes.is_empty() {
        return None;
    }

    let area = codes[rng.below(codes.len() as u64) as usize];

    let mut national = String::with_capacity(10);
    national.push_str(&area.to_string());
    // Central office code: first digit 2-9, never the N11 form.
    loop {
        let a = b'2' + rng.below(8) as u8;
        let b = rng.digit();
        let c = rng.digit();
        if !(b == b'1' && c == b'1') {
            national.push(char::from(a));
            national.push(char::from(b));
            national.push(char::from(c));
            break;
        }
    }
    for _ in 0..4 {
        national.push(char::from(rng.digit()));
    }

    Some(national)
}

// ============================================================================
// Comparison
// ============================================================================

/// Check if two phone numbers are equivalent, ignoring formatting.
///
/// Both must be valid; two invalid strings are never equal.
///
/// # Examples
/// ```
/// use phonelib::are_phone_numbers_equal;
///
/// assert!(are_phone_numbers_equal("+1 202-555-0173", "+1 (202) 555-0173"));
/// assert!(!are_phone_numbers_equal("+12025550173", "+12025550174"));
/// // A significant leading zero is not the same number without it.
/// assert!(!are_phone_numbers_equal("+390669821234", "+39669821234"));
/// ```
#[must_use]
pub fn are_phone_numbers_equal(number1: &str, number2: &str) -> bool {
    match (
        normalize_phone_number(number1),
        normalize_phone_number(number2),
    ) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// Group phone numbers by equivalence, preserving input order.
///
/// Numbers that cannot be normalized each form their own group, so the output
/// always contains every input exactly once.
///
/// Runs in linear time via hashing rather than comparing every pair.
///
/// # Examples
/// ```
/// use phonelib::group_equivalent_phone_numbers;
///
/// let numbers = ["+1 202-555-0173", "+1 (202) 555-0173", "+442079460958"];
/// let groups = group_equivalent_phone_numbers(&numbers);
/// assert_eq!(groups.len(), 2);
/// assert_eq!(groups[0].len(), 2);
/// ```
#[must_use]
pub fn group_equivalent_phone_numbers<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<Vec<String>> {
    use std::collections::HashMap;

    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut index_of: HashMap<String, usize> = HashMap::new();

    for number in phone_numbers {
        let raw = number.as_ref();
        match normalize_phone_number(raw) {
            Some(key) => match index_of.get(&key) {
                Some(&i) => groups[i].push(raw.to_string()),
                None => {
                    index_of.insert(key, groups.len());
                    groups.push(vec![raw.to_string()]);
                }
            },
            None => groups.push(vec![raw.to_string()]),
        }
    }

    groups
}

// ============================================================================
// Batch processing
// ============================================================================

/// Validate multiple phone numbers at once.
///
/// # Examples
/// ```
/// use phonelib::validate_phone_numbers_batch;
///
/// let results = validate_phone_numbers_batch(&["+12025550173", "invalid"]);
/// assert_eq!(results, vec![true, false]);
/// ```
#[must_use]
pub fn validate_phone_numbers_batch<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<bool> {
    phone_numbers
        .iter()
        .map(|n| is_valid_phone_number(n.as_ref()))
        .collect()
}

/// Normalize multiple phone numbers at once.
///
/// # Examples
/// ```
/// use phonelib::normalize_phone_numbers_batch;
///
/// let out = normalize_phone_numbers_batch(&["+12025550173", "+1 (202) 555-0173"]);
/// assert_eq!(
///     out,
///     vec![
///         Some("+12025550173".to_string()),
///         Some("+12025550173".to_string())
///     ]
/// );
/// ```
#[must_use]
pub fn normalize_phone_numbers_batch<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<Option<String>> {
    phone_numbers
        .iter()
        .map(|n| normalize_phone_number(n.as_ref()))
        .collect()
}

/// Extract countries for multiple phone numbers at once.
///
/// # Examples
/// ```
/// use phonelib::extract_countries_batch;
///
/// let out = extract_countries_batch(&["+12025550173", "+442079460958"]);
/// assert_eq!(out[0].unwrap().code, "US");
/// assert_eq!(out[1].unwrap().code, "GB");
/// ```
#[must_use]
pub fn extract_countries_batch<T: AsRef<str>>(
    phone_numbers: &[T],
) -> Vec<Option<&'static Country>> {
    phone_numbers
        .iter()
        .map(|n| extract_country(n.as_ref()))
        .collect()
}

/// Detect phone number types for multiple numbers at once.
///
/// # Examples
/// ```
/// use phonelib::{detect_phone_number_types_batch, PhoneNumberType};
///
/// let out = detect_phone_number_types_batch(&["+18005550199"]);
/// assert_eq!(out[0], Some(PhoneNumberType::TollFree));
/// ```
#[must_use]
pub fn detect_phone_number_types_batch<T: AsRef<str>>(
    phone_numbers: &[T],
) -> Vec<Option<PhoneNumberType>> {
    phone_numbers
        .iter()
        .map(|n| detect_phone_number_type(n.as_ref()))
        .collect()
}

/// Detailed analysis result for a phone number.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PhoneNumberAnalysis {
    /// The input exactly as supplied.
    pub original: String,
    /// Whether the number passed validation.
    pub is_valid: bool,
    /// E.164 form, when valid.
    pub normalized: Option<String>,
    /// Resolved country, when valid.
    pub country: Option<&'static Country>,
    /// Service class, when valid.
    pub phone_type: Option<PhoneNumberType>,
    /// Why the number was rejected, when invalid.
    pub error: Option<ParseError>,
}

/// Comprehensive batch analysis of phone numbers.
///
/// `is_valid` here agrees with [`validate_phone_numbers_batch`] for every
/// input, because both run the same parser.
///
/// # Examples
/// ```
/// use phonelib::analyze_phone_numbers_batch;
///
/// let out = analyze_phone_numbers_batch(&["+12025550173"]);
/// assert!(out[0].is_valid);
/// assert_eq!(out[0].country.unwrap().code, "US");
/// ```
#[must_use]
pub fn analyze_phone_numbers_batch<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<PhoneNumberAnalysis> {
    phone_numbers
        .iter()
        .map(|number| {
            let original = number.as_ref().to_string();
            match parse_internal(&original) {
                Ok(parsed) => PhoneNumberAnalysis {
                    original,
                    is_valid: true,
                    normalized: Some(parsed.normalized),
                    country: Some(parsed.country),
                    phone_type: Some(parsed.phone_type),
                    error: None,
                },
                Err(error) => PhoneNumberAnalysis {
                    original,
                    is_valid: false,
                    normalized: None,
                    country: None,
                    phone_type: None,
                    error: Some(error),
                },
            }
        })
        .collect()
}

// ============================================================================
// Correction suggestions
// ============================================================================

/// Suggest corrections for an invalid phone number.
///
/// Candidates are ranked by how little they change the input — a number that is
/// already valid comes back unchanged, then country-code additions, then trunk
/// and IDD prefix repairs, then single-digit insertions and deletions. At most
/// five suggestions are returned.
///
/// With a `country_hint` the input is read as a *national* number of that
/// country first; other interpretations are offered only if no repair is
/// possible under that reading, so a suggestion may still name another country.
/// An unrecognised hint returns nothing rather than falling back to no hint.
///
/// # Examples
/// ```
/// use phonelib::{suggest_phone_number_corrections, is_valid_phone_number};
///
/// // An already valid number is returned as-is.
/// let s = suggest_phone_number_corrections("+12025550173", None);
/// assert_eq!(s, vec!["+12025550173".to_string()]);
///
/// // With a hint, a national number gains that country's calling code.
/// let s = suggest_phone_number_corrections("2025550173", Some("US"));
/// assert_eq!(s, vec!["+12025550173".to_string()]);
/// let s = suggest_phone_number_corrections("020 7946 0958", Some("GB"));
/// assert_eq!(s, vec!["+442079460958".to_string()]);
///
/// // A number one digit short is repaired against the hint country.
/// let s = suggest_phone_number_corrections("+1 202 555 017", Some("US"));
/// assert!(!s.is_empty());
/// assert!(s.iter().all(|c| is_valid_phone_number(c)));
///
/// // An unrecognised hint yields nothing at all.
/// assert!(suggest_phone_number_corrections("2025550173", Some("ZZ")).is_empty());
/// ```
#[must_use]
pub fn suggest_phone_number_corrections(
    phone_number: &str,
    country_hint: Option<&str>,
) -> Vec<String> {
    let hinted = match country_hint {
        // An unrecognised hint fails closed. Treating it as "no hint" would
        // answer with a number from some other country entirely.
        Some(code) => match country_by_code(code) {
            Some(country) => Some(country),
            None => return Vec::new(),
        },
        None => None,
    };

    // Nothing to correct: the input already reads as a valid number. With a
    // hint that reading must go through the hint country, otherwise a national
    // number would come back attributed to whatever country its digits happen
    // to spell.
    match hinted {
        Some(country) => {
            if let Ok(parsed) = PhoneNumber::try_parse_with_country(phone_number, country.code) {
                return vec![parsed.e164().to_string()];
            }
        }
        None => {
            if let Some(normalized) = normalize_phone_number(phone_number) {
                return vec![normalized];
            }
        }
    }

    let digits: String = phone_number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    // The repair tiers below are quadratic in the digit count, so bound the
    // input rather than letting a caller-supplied string of arbitrary length
    // drive the search. Nothing longer than one digit over the E.164 ceiling
    // could be fixed by the single-edit repairs we attempt anyway.
    if digits.is_empty() || digits.len() > E164_MAX_DIGITS + 1 {
        return Vec::new();
    }

    let candidate_countries: Vec<&'static Country> = match hinted {
        Some(country) => vec![country],
        None => ["US", "GB", "DE", "FR", "IN", "AU", "CA", "BR", "JP", "NG"]
            .iter()
            .filter_map(|code| country_by_code(code))
            .collect(),
    };

    // Candidates are generated one edit-cost tier at a time and the search
    // stops at the first tier that produces anything, so the result is always
    // a minimum-edit repair rather than whatever sorted first.
    let mut found: Vec<String> = Vec::new();
    let consider = |candidate: String, found: &mut Vec<String>| {
        if let Some(normalized) = normalize_phone_number(&candidate) {
            if !found.contains(&normalized) {
                found.push(normalized);
            }
        }
    };

    // Read `national` as a national number of each candidate country. The
    // hinted parser already knows every country's trunk digit.
    let add_calling_code = |national: &str, found: &mut Vec<String>| {
        for country in &candidate_countries {
            let Ok(parsed) = PhoneNumber::try_parse_with_country(national, country.code) else {
                continue;
            };
            // That parser falls back to a global reading, which is not a
            // repair in the requested country.
            let resolved = parsed.country().prefix;
            let same_plan =
                resolved == country.prefix || (is_nanp(resolved) && is_nanp(country.prefix));
            if !same_plan {
                continue;
            }
            let e164 = parsed.e164().to_string();
            if !found.contains(&e164) {
                found.push(e164);
            }
        }
    };
    // Read `global` as a number that already spells out its calling code.
    let as_global = |global: &str, found: &mut Vec<String>| consider(format!("+{}", global), found);

    // Within every tier the hinted reading is exhausted before the global one:
    // a foreign reading of the same digits is a worse answer at equal cost.
    let hinted_first = |candidates: &[String], found: &mut Vec<String>| {
        if hinted.is_some() {
            for c in candidates {
                add_calling_code(c, found);
            }
            if found.is_empty() {
                for c in candidates {
                    as_global(c, found);
                }
            }
        } else {
            for c in candidates {
                as_global(c, found);
            }
            if found.is_empty() {
                for c in candidates {
                    add_calling_code(c, found);
                }
            }
        }
    };

    // Tiers 0 and 1: the calling code is missing, or only the '+' is.
    hinted_first(std::slice::from_ref(&digits), &mut found);

    // Tier 2: an IDD prefix was dialled instead of '+'.
    if found.is_empty() {
        for idd in ["0011", "011", "00"] {
            if let Some(rest) = digits.strip_prefix(idd) {
                consider(format!("+{}", rest), &mut found);
            }
        }
    }

    // Tier 3: one digit too many.
    if found.is_empty() {
        // End first: an extra digit is usually picked up at the end.
        let shortened: Vec<String> = (0..digits.len())
            .rev()
            .map(|i| {
                let mut s = digits.clone();
                s.remove(i);
                s
            })
            .collect();
        hinted_first(&shortened, &mut found);
    }

    // Tier 4: one digit missing.
    if found.is_empty() && digits.len() < E164_MAX_DIGITS {
        let mut extended = Vec::with_capacity((digits.len() + 1) * 10);
        // End first: a number that lost a digit most often lost the last one.
        for i in (0..=digits.len()).rev() {
            for d in b'0'..=b'9' {
                let mut s = digits.clone();
                s.insert(i, char::from(d));
                extended.push(s);
            }
        }
        hinted_first(&extended, &mut found);
    }

    found.truncate(5);
    found
}

/// Check if a phone number could plausibly become valid with reformatting.
///
/// The bounds come from the country table rather than a hardcoded window, so
/// this never contradicts [`is_valid_phone_number`]: every valid number is also
/// potentially valid.
///
/// # Examples
/// ```
/// use phonelib::{is_potentially_valid_phone_number, is_valid_phone_number};
///
/// assert!(is_potentially_valid_phone_number("202-555-0173"));
/// assert!(!is_potentially_valid_phone_number("12"));
/// assert!(!is_potentially_valid_phone_number("000000000"));
///
/// // Anything valid is also potentially valid.
/// assert!(is_potentially_valid_phone_number("+12025550173"));
/// assert!(is_valid_phone_number("+12025550173"));
/// ```
#[must_use]
pub fn is_potentially_valid_phone_number(phone_number: &str) -> bool {
    if is_valid_phone_number(phone_number) {
        return true;
    }

    let digits = phone_number.chars().filter(|c| c.is_ascii_digit()).count();

    (MIN_NATIONAL_LEN..=E164_MAX_DIGITS).contains(&digits)
        && !phone_number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .all(|c| c == '0')
}

/// Get the most likely country for a phone number.
///
/// Tries full resolution first, so the answer always agrees with
/// [`extract_country`] when the number is valid. Falls back to matching the
/// leading digits against a known calling code when it is not.
///
/// # Examples
/// ```
/// use phonelib::{guess_country_from_number, extract_country};
///
/// assert_eq!(guess_country_from_number("+442079460958").unwrap().code, "GB");
/// // Agrees with extract_country for valid numbers.
/// let n = "+12025550173";
/// assert_eq!(
///     guess_country_from_number(n).map(|c| c.code),
///     extract_country(n).map(|c| c.code)
/// );
/// ```
#[must_use]
pub fn guess_country_from_number(phone_number: &str) -> Option<&'static Country> {
    if let Some(country) = extract_country(phone_number) {
        return Some(country);
    }

    let digits: Vec<u8> = phone_number
        .bytes()
        .filter(|b| b.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return None;
    }

    // Longest calling code that the digits start with, ignoring length.
    for prefix_len in (1..=4usize).rev() {
        if digits.len() <= prefix_len {
            continue;
        }
        let prefix = prefix_value(&digits[..prefix_len]);
        if let Some(country) = entries_for_prefix(prefix).next() {
            return Some(country);
        }
    }

    None
}

// ============================================================================
// Text parsing
// ============================================================================

/// Extract all phone numbers from free-form text.
///
/// Scans for candidate numbers in the common written formats — international
/// (`+1 202 555 0173`), parenthesised area codes (`(202) 555-0173`), and
/// dash-, dot- or space-separated digits — and reports each with its byte range
/// in the input. Runs in linear time.
///
/// Ranges never overlap and `&text[n.start..n.end] == n.raw` always holds.
///
/// # Examples
/// ```
/// use phonelib::extract_phone_numbers_from_text;
///
/// let text = "Call me at +1-202-555-0173 or (415) 555-2671";
/// let numbers = extract_phone_numbers_from_text(text);
/// assert_eq!(numbers.len(), 2);
/// assert_eq!(&text[numbers[0].start..numbers[0].end], numbers[0].raw);
/// ```
#[must_use]
pub fn extract_phone_numbers_from_text(text: &str) -> Vec<ExtractedPhoneNumber> {
    text::extract(text)
}

/// Extract only the valid phone numbers from text.
///
/// Validity is judged without country context, so a national-format number is
/// read as though its leading digits were a calling code — usually rejecting
/// it, occasionally resolving it to an unrelated country. Use
/// [`extract_phone_numbers_with_country_hint`] when the text has a known origin.
///
/// # Examples
/// ```
/// use phonelib::extract_valid_phone_numbers_from_text;
///
/// let numbers = extract_valid_phone_numbers_from_text("Call +12025550173 or 123");
/// assert_eq!(numbers.len(), 1);
/// assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
/// ```
#[must_use]
pub fn extract_valid_phone_numbers_from_text(text: &str) -> Vec<ExtractedPhoneNumber> {
    text::extract(text)
        .into_iter()
        .filter(|n| n.is_valid)
        .collect()
}

/// Extract phone numbers from text, assuming a default country.
///
/// National-format numbers are resolved against `default_country`. Numbers that
/// already carry their own country code — anything written with a leading `+`,
/// or with an international dialling prefix — are left alone rather than being
/// given a second calling code.
///
/// Returns an empty vector if `default_country` is not in the country table —
/// falling back to no hint would yield a different, valid-looking number rather
/// than an obviously absent one. The code is matched case-insensitively.
///
/// # Examples
/// ```
/// use phonelib::extract_phone_numbers_with_country_hint;
///
/// let numbers = extract_phone_numbers_with_country_hint("Call (202) 555-0173", "US");
/// assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
///
/// // An explicit country code always wins over the hint.
/// let numbers = extract_phone_numbers_with_country_hint("Call +12025550173", "DE");
/// assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
/// ```
#[must_use]
pub fn extract_phone_numbers_with_country_hint(
    text: &str,
    default_country: &str,
) -> Vec<ExtractedPhoneNumber> {
    let Some(country) = country_by_code(default_country) else {
        return Vec::new();
    };
    text::extract_with_hint(text, Some(country))
}

/// Count how many phone numbers are in the text.
///
/// # Examples
/// ```
/// use phonelib::count_phone_numbers_in_text;
///
/// assert_eq!(count_phone_numbers_in_text("+12025550173 and +442079460958"), 2);
/// ```
#[must_use]
pub fn count_phone_numbers_in_text(text: &str) -> usize {
    text::extract(text).len()
}

/// Replace phone numbers in text with a transformed version.
///
/// # Examples
/// ```
/// use phonelib::replace_phone_numbers_in_text;
///
/// let redacted = replace_phone_numbers_in_text("Call me at +12025550173", |_| {
///     "[REDACTED]".to_string()
/// });
/// assert_eq!(redacted, "Call me at [REDACTED]");
/// ```
#[must_use]
pub fn replace_phone_numbers_in_text<F>(text: &str, replacement: F) -> String
where
    F: Fn(&ExtractedPhoneNumber) -> String,
{
    text::replace(text, replacement)
}

/// Redact phone numbers in text for privacy.
///
/// `visible_digits` digits are kept at the end of each number and the rest are
/// replaced by `*`. When `visible_digits` is `0`, or is at least the number's
/// digit count, the whole number is replaced by `[PHONE]`. Separators and the
/// leading `+` are not preserved — the output is `*` characters followed by the
/// visible digits.
///
/// # Examples
/// ```
/// use phonelib::redact_phone_numbers;
///
/// assert_eq!(
///     redact_phone_numbers("Call +12025550173", 4),
///     "Call *******0173"
/// );
/// assert_eq!(redact_phone_numbers("Call +12025550173", 0), "Call [PHONE]");
///
/// // Adjacent numbers are each redacted in full.
/// assert_eq!(
///     redact_phone_numbers("Call 2025550173 4155552671 now", 0),
///     "Call [PHONE] [PHONE] now"
/// );
/// ```
#[must_use]
pub fn redact_phone_numbers(text: &str, visible_digits: usize) -> String {
    text::redact(text, visible_digits)
}

// ============================================================================
// PhoneNumber
// ============================================================================

/// A parsed and validated phone number.
///
/// Two `PhoneNumber` values are equal when they normalize to the same E.164
/// string, regardless of how they were written. `Hash` follows the same rule,
/// so the `Eq`/`Hash` contract holds and equal numbers always hash equally.
///
/// # Examples
/// ```
/// use phonelib::PhoneNumber;
///
/// let a = PhoneNumber::parse("+1 202-555-0173").unwrap();
/// let b = PhoneNumber::parse("+1 (202) 555-0173").unwrap();
/// assert_eq!(a, b);
/// assert_eq!(a.e164(), "+12025550173");
/// assert_eq!(a.country().code, "US");
/// ```
#[derive(Debug, Clone)]
pub struct PhoneNumber {
    original: String,
    normalized: String,
    country: &'static Country,
    phone_type: PhoneNumberType,
    extension: Option<String>,
}

impl PhoneNumber {
    /// Parse a string into a `PhoneNumber`.
    ///
    /// # Examples
    /// ```
    /// use phonelib::PhoneNumber;
    ///
    /// assert!(PhoneNumber::parse("+12025550173").is_some());
    /// assert!(PhoneNumber::parse("nonsense").is_none());
    /// ```
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        Self::try_parse(input).ok()
    }

    /// Parse a string into a `PhoneNumber`, reporting why it failed.
    ///
    /// # Examples
    /// ```
    /// use phonelib::{PhoneNumber, ParseError};
    ///
    /// assert_eq!(PhoneNumber::try_parse("+1202555").unwrap_err(), ParseError::InvalidLength);
    /// ```
    pub fn try_parse(input: &str) -> Result<Self, ParseError> {
        let parsed = parse_internal(input)?;
        Ok(PhoneNumber {
            original: input.to_string(),
            normalized: parsed.normalized,
            country: parsed.country,
            phone_type: parsed.phone_type,
            extension: parsed.extension,
        })
    }

    /// Parse a number that may be in national format, using a country hint.
    ///
    /// The hint applies only when the number does not already carry its own
    /// country code: a leading `+` always wins, so `country()` can never
    /// disagree with the number's own calling code.
    ///
    /// Under calling code 1 the **area code** wins for the same reason —
    /// `parse_with_country("(202) 555-0173", "CA")` is a `US` number. A NANP
    /// hint only says to read the digits as a ten-digit North American number,
    /// and supplies the area code for a local seven-digit one.
    ///
    /// # Examples
    /// ```
    /// use phonelib::PhoneNumber;
    ///
    /// // National format resolved through the hint.
    /// let n = PhoneNumber::parse_with_country("020 7946 0958", "GB").unwrap();
    /// assert_eq!(n.e164(), "+442079460958");
    ///
    /// // The number's own country code wins over a contradicting hint.
    /// let n = PhoneNumber::parse_with_country("+12025550173", "DE").unwrap();
    /// assert_eq!(n.country().code, "US");
    ///
    /// // An unknown hint is an error, not a silently ignored argument.
    /// assert!(PhoneNumber::parse_with_country("020 7946 0958", "ZZ").is_none());
    /// ```
    #[must_use]
    pub fn parse_with_country(input: &str, country_code: &str) -> Option<Self> {
        Self::try_parse_with_country(input, country_code).ok()
    }

    /// Like [`PhoneNumber::parse_with_country`] but reports why it failed.
    pub fn try_parse_with_country(input: &str, country_code: &str) -> Result<Self, ParseError> {
        let hint = country_by_code(country_code).ok_or(ParseError::UnknownCountry)?;

        // Preserved verbatim for `original()`.
        let original = input;

        // A `tel:` URI and an explicit '+' both carry their own country code,
        // so the hint must not be applied on top.
        let trimmed = strip_tel_uri(input.trim());
        if trimmed.len() != input.trim().len() || trimmed.starts_with(['+', '\u{FF0B}']) {
            return Self::try_parse(trimmed).map(|mut parsed| {
                parsed.original = original.to_string();
                parsed
            });
        }
        let input = trimmed;

        let (base, extension) = split_extension(input);
        let digits: String = base
            .chars()
            .filter_map(|c| digit_value(c).map(char::from))
            .collect();
        if digits.is_empty() {
            return Err(ParseError::Empty);
        }

        let suffix = extension
            .map(|e| format!(" ext. {}", e))
            .unwrap_or_default();

        // A national number never begins with two zeros, so `00`/`0011` is
        // unambiguously an access code. `011` is not — a Japanese `0114…` is
        // Sapporo — so it waits until after the national readings below.
        if digits.starts_with("00") {
            for idd in ["0011", "00"] {
                if let Some(rest) = digits.strip_prefix(idd) {
                    if let Ok(mut parsed) = Self::try_parse(&format!("+{}{}", rest, suffix)) {
                        parsed.original = original.to_string();
                        return Ok(parsed);
                    }
                }
            }
        }

        // Candidate global numbers to try, most likely reading first.
        let mut attempts: Vec<String> = Vec::new();

        if is_nanp(hint.prefix) {
            // Ten flat digits, written bare, with the long-distance trunk
            // `1`, or — for a territory hint — as a local number needing its
            // area code back. The area code, not the hint, picks the country.
            attempts.push(format!("+1{}", digits));
            if let Some(rest) = digits.strip_prefix('1') {
                attempts.push(format!("+1{}", rest));
            }
            if is_nanp_territory(hint.prefix) {
                attempts.push(format!("+{}{}", hint.prefix, digits));
            }
        } else {
            // National format. Removing a trunk prefix is the likelier
            // reading where the country has one; otherwise prefer the digits
            // as written, which keeps a significant leading zero intact.
            let trunk = national_prefix_for(hint.code);
            if !trunk.is_empty() {
                if let Some(rest) = digits.strip_prefix(trunk) {
                    attempts.push(format!("+{}{}", hint.prefix, rest));
                }
            }
            attempts.push(format!("+{}{}", hint.prefix, digits));
            if trunk.is_empty() {
                if let Some(rest) = digits.strip_prefix('0') {
                    attempts.push(format!("+{}{}", hint.prefix, rest));
                }
            }
        }

        for candidate in attempts {
            if let Ok(mut parsed) = Self::try_parse(&format!("{}{}", candidate, suffix)) {
                parsed.original = original.to_string();
                return Ok(parsed);
            }
        }

        // `011` as an access code, after the national readings for the
        // reason given above.
        if let Some(rest) = digits.strip_prefix("011") {
            if let Ok(mut parsed) = Self::try_parse(&format!("+{}{}", rest, suffix)) {
                parsed.original = original.to_string();
                return Ok(parsed);
            }
        }

        // Last resort: read the digits as though they spelled out their own
        // calling code. Last, so the hint wins on a coincidental match. The
        // explicit `+` is what lets this past the ambiguity check — the hint
        // is the caller vouching that these digits are a whole number.
        Self::try_parse(&format!("+{}{}", digits, suffix)).map(|mut parsed| {
            parsed.original = original.to_string();
            parsed
        })
    }

    /// The input exactly as it was supplied.
    #[must_use]
    pub fn original(&self) -> &str {
        &self.original
    }

    /// The E.164 form, without any extension.
    #[must_use]
    pub fn e164(&self) -> &str {
        &self.normalized
    }

    /// The national significant number, without the country calling code.
    ///
    /// # Examples
    /// ```
    /// use phonelib::PhoneNumber;
    ///
    /// let n = PhoneNumber::parse("+12025550173").unwrap();
    /// assert_eq!(n.national_number(), "2025550173");
    /// ```
    #[must_use]
    pub fn national_number(&self) -> &str {
        &self.normalized[1 + prefix_digit_count(self.country.prefix)..]
    }

    /// The resolved country.
    #[must_use]
    pub fn country(&self) -> &'static Country {
        self.country
    }

    /// The calling code of the resolved country, as the table records it.
    ///
    /// NANP territories report the four-digit `1` + area code form — Antigua is
    /// `1268`, not `1` — because that is what
    /// [`PhoneNumber::national_number`] strips.
    #[must_use]
    pub fn country_code(&self) -> u32 {
        self.country.prefix
    }

    /// The service class of this number.
    #[must_use]
    pub fn phone_type(&self) -> PhoneNumberType {
        self.phone_type
    }

    /// The extension, if the input carried one.
    ///
    /// # Examples
    /// ```
    /// use phonelib::PhoneNumber;
    ///
    /// let n = PhoneNumber::parse("+1 202-555-0173 ext. 42").unwrap();
    /// assert_eq!(n.extension(), Some("42"));
    /// assert_eq!(n.e164(), "+12025550173");
    /// assert_eq!(
    ///     n.format(phonelib::PhoneFormat::RFC3966),
    ///     "tel:+12025550173;ext=42"
    /// );
    /// ```
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    /// Format this number.
    ///
    /// # Examples
    /// ```
    /// use phonelib::{PhoneNumber, PhoneFormat};
    ///
    /// let n = PhoneNumber::parse("+12025550173").unwrap();
    /// assert_eq!(n.format(PhoneFormat::E164), "+12025550173");
    /// assert_eq!(n.format(PhoneFormat::National), "(202) 555-0173");
    /// assert_eq!(n.format(PhoneFormat::International), "+1 202 555 0173");
    /// ```
    #[must_use]
    pub fn format(&self, fmt: PhoneFormat) -> String {
        render(
            &self.normalized,
            self.country,
            self.extension.as_deref(),
            fmt,
        )
    }

    /// Whether this number is mobile.
    #[must_use]
    pub fn is_mobile(&self) -> bool {
        self.phone_type == PhoneNumberType::Mobile
    }

    /// Whether this number is a landline.
    #[must_use]
    pub fn is_landline(&self) -> bool {
        self.phone_type == PhoneNumberType::FixedLine
    }

    /// Whether this number is toll-free.
    #[must_use]
    pub fn is_toll_free(&self) -> bool {
        self.phone_type == PhoneNumberType::TollFree
    }
}

impl PartialEq for PhoneNumber {
    fn eq(&self, other: &Self) -> bool {
        self.normalized == other.normalized
    }
}

impl Eq for PhoneNumber {}

impl PartialOrd for PhoneNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PhoneNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.normalized.cmp(&other.normalized)
    }
}

impl std::hash::Hash for PhoneNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.normalized.hash(state);
    }
}

impl std::fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.normalized)
    }
}

impl AsRef<str> for PhoneNumber {
    fn as_ref(&self) -> &str {
        &self.normalized
    }
}

impl std::str::FromStr for PhoneNumber {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PhoneNumber::try_parse(s)
    }
}

impl TryFrom<&str> for PhoneNumber {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        PhoneNumber::try_parse(value)
    }
}

impl TryFrom<String> for PhoneNumber {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PhoneNumber::try_parse(&value)
    }
}

// ============================================================================
// PhoneNumberSet
// ============================================================================

/// A set of phone numbers, deduplicated by their E.164 form.
///
/// Backed by a map keyed on the normalized number so that lookups by any
/// formatting of a number can return the stored [`PhoneNumber`], which a plain
/// `HashSet<PhoneNumber>` cannot do.
///
/// # Examples
/// ```
/// use phonelib::PhoneNumberSet;
///
/// let mut set = PhoneNumberSet::new();
/// assert!(set.add("+1-202-555-0173"));
/// assert!(!set.add("+1 (202) 555-0173")); // same number
/// assert_eq!(set.len(), 1);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneNumberSet {
    numbers: std::collections::HashMap<String, PhoneNumber>,
}

impl PhoneNumberSet {
    /// Create an empty set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a phone number.
    ///
    /// Returns `true` if it was inserted, and `false` if the number was already
    /// present **or** could not be parsed. Use [`PhoneNumberSet::try_add`] when
    /// you need to tell those apart.
    pub fn add(&mut self, phone_number: &str) -> bool {
        self.try_add(phone_number).unwrap_or(false)
    }

    /// Add a phone number, reporting a parse failure.
    ///
    /// `Ok(true)` means inserted, `Ok(false)` means already present.
    ///
    /// # Examples
    /// ```
    /// use phonelib::PhoneNumberSet;
    ///
    /// let mut set = PhoneNumberSet::new();
    /// assert_eq!(set.try_add("+12025550173"), Ok(true));
    /// assert_eq!(set.try_add("+1 202-555-0173"), Ok(false));
    /// assert!(set.try_add("nonsense").is_err());
    /// ```
    pub fn try_add(&mut self, phone_number: &str) -> Result<bool, ParseError> {
        let phone = PhoneNumber::try_parse(phone_number)?;
        if self.numbers.contains_key(&phone.normalized) {
            return Ok(false);
        }
        self.numbers.insert(phone.normalized.clone(), phone);
        Ok(true)
    }

    /// Whether a number is in the set, in any formatting.
    #[must_use]
    pub fn contains(&self, phone_number: &str) -> bool {
        self.get(phone_number).is_some()
    }

    /// The stored number matching `phone_number`, in any formatting.
    ///
    /// # Examples
    /// ```
    /// use phonelib::PhoneNumberSet;
    ///
    /// let mut set = PhoneNumberSet::new();
    /// set.add("+1-202-555-0173");
    /// assert_eq!(set.get("+1 (202) 555-0173").unwrap().e164(), "+12025550173");
    /// ```
    #[must_use]
    pub fn get(&self, phone_number: &str) -> Option<&PhoneNumber> {
        let normalized = normalize_phone_number(phone_number)?;
        self.numbers.get(&normalized)
    }

    /// Number of unique phone numbers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.numbers.len()
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.numbers.is_empty()
    }

    /// Iterate over the stored numbers, in unspecified order.
    pub fn iter(&self) -> impl Iterator<Item = &PhoneNumber> {
        self.numbers.values()
    }

    /// The normalized form of every stored number, in unspecified order.
    #[must_use]
    pub fn normalized_numbers(&self) -> Vec<&str> {
        self.numbers.keys().map(String::as_str).collect()
    }

    /// Remove a number, in any formatting. Returns whether it was present.
    pub fn remove(&mut self, phone_number: &str) -> bool {
        match normalize_phone_number(phone_number) {
            Some(normalized) => self.numbers.remove(&normalized).is_some(),
            None => false,
        }
    }
}

impl IntoIterator for PhoneNumberSet {
    type Item = PhoneNumber;
    type IntoIter = std::collections::hash_map::IntoValues<String, PhoneNumber>;

    fn into_iter(self) -> Self::IntoIter {
        self.numbers.into_values()
    }
}

impl<'a> IntoIterator for &'a PhoneNumberSet {
    type Item = &'a PhoneNumber;
    type IntoIter = std::collections::hash_map::Values<'a, String, PhoneNumber>;

    fn into_iter(self) -> Self::IntoIter {
        self.numbers.values()
    }
}

impl<T: AsRef<str>> Extend<T> for PhoneNumberSet {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for number in iter {
            self.add(number.as_ref());
        }
    }
}

impl FromIterator<String> for PhoneNumberSet {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut set = PhoneNumberSet::new();
        set.extend(iter);
        set
    }
}

impl<'a> FromIterator<&'a str> for PhoneNumberSet {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut set = PhoneNumberSet::new();
        set.extend(iter);
        set
    }
}
