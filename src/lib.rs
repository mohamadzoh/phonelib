//! # Phonelib
//!
//! A dependency-free Rust library for validating, parsing, formatting and
//! manipulating international phone numbers.
//!
//! ## Features
//!
//! - **Validation** - structural (the plan's lengths), with NANP area codes
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
//! Validation checks the *structure* of a numbering plan rather than which of
//! its ranges are allocated today: the country calling code must be known and
//! the national significant number must have a length the plan uses for some
//! kind of number — fixed line, mobile, freephone, premium rate and so on. The
//! lengths are those of Google's libphonenumber metadata, so no number it
//! considers valid is rejected here; and because allocations change monthly
//! while lengths change once in a decade, the table does not need chasing to
//! stay right. A Lebanese number is accepted in any block, but never with 6 or
//! 10 digits:
//!
//! ```rust
//! use phonelib::{is_valid_phone_number, try_normalize_phone_number, ParseError};
//!
//! assert!(is_valid_phone_number("+961 81 123 456"));
//! assert_eq!(try_normalize_phone_number("+961 123 456"), Err(ParseError::InvalidLength));
//! ```
//!
//! For numbers in the North American Numbering Plan the area code is also
//! checked against the assigned NANP ranges, which is what makes `US` / `CA` /
//! Caribbean territory attribution exact. Elsewhere a calling code shared by
//! several territories is split by each territory's own blocks — `+7 7…` is
//! Kazakhstan, `+44 1481` Guernsey — and those prefixes only ever choose which
//! territory is reported, never whether a number is valid.
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
//! The flip side is that a number of a valid length in a block nobody has
//! opened yet is accepted. If you need to know a number is in a range that is
//! *allocated* today, use a library backed by the complete libphonenumber
//! metadata and keep that metadata current.
//!
//! [`detect_phone_number_type`] classifies every region from range tables
//! derived from the same metadata. A number in a block no table claims is
//! [`PhoneNumberType::Unknown`] rather than a guess, and a number's type never
//! affects whether it is valid.
//!
//! The ITU's non-geographic calling codes — `+800` freephone, `+870` Inmarsat,
//! `+881`–`+883` networks and the rest — resolve to entries whose `code` is
//! `"001"`, as in libphonenumber.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Compiles and runs every Rust example in the README as a doctest, so the
// README cannot drift from the crate's actual behaviour.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

use constants::COUNTRIES;

pub use definitions::{Country, ParseError, PhoneNumberType};
pub use text::ExtractedPhoneNumber;

mod constants;
mod definitions;
mod nanp;
mod ranges;
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
/// range by default and is therefore returned by [`extract_country`] for every
/// number that no [`constants::TERRITORY_PREFIXES`] rule gives to a secondary
/// territory. Without it the winner would be an accident of declaration order.
///
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
    (290, "SH"),
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

/// The `code` of the entries for the ITU's non-geographic calling codes: the
/// UN M.49 code for the world, as libphonenumber uses.
const NON_GEOGRAPHIC_CODE: &str = "001";

/// The fewest digits read as a phone number when nothing marks them as one —
/// no `+`, and no country hint.
///
/// Seven is the length of a local subscriber number in most plans. The table
/// has shorter international numbers — a four-digit Austrian or Iranian
/// number behind its two-digit calling code is six digits in all — but a bare
/// run of six digits is an order number, a date or a price far more often
/// than it is one of those, which therefore have to be written with their `+`.
pub(crate) const MIN_UNMARKED_DIGITS: usize = 7;

/// `(first position in PREFIX_INDEX, entry count)` for every calling code
/// below 1000, so finding a code's entries is an array index rather than a
/// search. A count of 0 means the code is not in the table. The four-digit
/// NANP territory codes are looked up through [`NANP_AREA_OWNER`] instead.
static CALLING_CODES: [(u16, u16); 1000] = build_calling_codes();

const fn build_calling_codes() -> [(u16, u16); 1000] {
    let mut codes = [(0u16, 0u16); 1000];
    let mut i = 0;
    while i < COUNTRY_COUNT {
        let prefix = PREFIX_INDEX[i].0 as usize;
        if prefix < 1000 {
            // PREFIX_INDEX is sorted, so a code's entries are contiguous.
            let (start, count) = codes[prefix];
            codes[prefix] = if count == 0 {
                (i as u16, 1)
            } else {
                (start, count + 1)
            };
        }
        i += 1;
    }
    codes
}

// Resolution reads a calling code off the front of the digits and stops at the
// first code it knows. That is only sound because no code is a prefix of
// another — `7` and `77`, say — which the ITU guarantees and this checks.
const _: () = {
    let codes = build_calling_codes();
    let mut code = 1;
    while code < 1000 {
        if codes[code].1 > 0 {
            let mut shorter = code / 10;
            while shorter > 0 {
                assert!(
                    codes[shorter].1 == 0,
                    "a calling code is a prefix of another calling code"
                );
                shorter /= 10;
            }
        }
        code += 1;
    }
};

/// Every table entry that uses `prefix`, as `(prefix, index)`, in resolution
/// order.
fn prefix_entries(prefix: u32) -> &'static [(u32, u16)] {
    if prefix < 1000 {
        let (start, count) = CALLING_CODES[prefix as usize];
        return &PREFIX_INDEX[start as usize..(start + count) as usize];
    }
    let lo = PREFIX_INDEX.partition_point(|&(p, _)| p < prefix);
    let hi = PREFIX_INDEX.partition_point(|&(p, _)| p <= prefix);
    &PREFIX_INDEX[lo..hi]
}

/// Every table entry that uses `prefix`, in resolution order.
fn entries_for_prefix(prefix: u32) -> impl Iterator<Item = &'static Country> {
    prefix_entries(prefix)
        .iter()
        .map(|&(_, idx)| &COUNTRIES[idx as usize])
}

// ============================================================================
// Numbering-plan structure
//
// What the resolver needs to know about an entry, folded at compile time out
// of the tables in `constants` into one record per entry, so that a lookup is
// an array index and a length check is a bit test.
// ============================================================================

/// The structure of one entry's numbering plan.
#[derive(Clone, Copy)]
struct Plan {
    /// Bit `n` is set when an NSN of `n` digits that does not begin with `0`
    /// is valid.
    lengths: u32,
    /// Bit `n` is set when an NSN of `n` digits that begins with `0` is valid;
    /// see [`constants::ZERO_LED`].
    zero_led: u32,
    /// Whether some NSN begins with `00`, so that `00` cannot be taken for an
    /// international access code when reading one of the plan's own numbers.
    double_zero_led: bool,
    /// Trunk prefix digits, `""` when the plan has none.
    trunk: &'static str,
    /// International access codes beyond `00`, `0011` and `011` that no
    /// national number is dialled like, and those some might be; see
    /// [`constants::INTERNATIONAL_PREFIXES`].
    early_access: &'static [&'static str],
    late_access: &'static [&'static str],
    /// NSN prefixes that give a number to this entry rather than to the main
    /// region of its calling code; see [`constants::TERRITORY_PREFIXES`].
    territory: &'static [&'static str],
    /// Bit `d` is set when some `territory` prefix starts with the digit `d`,
    /// so most numbers skip the prefix comparisons altogether.
    territory_digits: u16,
    /// Number-type rules; see [`ranges::TYPE_RANGES`].
    types: &'static [u32],
    /// For each first digit, the index into [`ranges::TYPES`] of a one-digit
    /// rule that covers every length, or [`NO_TYPE`]: the common case,
    /// answered without a search.
    first_digit_type: [u8; 10],
}

/// No one-digit rule decides the type; see [`Plan::first_digit_type`].
const NO_TYPE: u8 = u8::MAX;

impl Plan {
    /// Whether `nsn` — which must not be empty — has a length this plan uses.
    ///
    /// Only a plan with `00…` numbers of its own accepts one: everywhere else
    /// `00` is the international access code, never the start of a number.
    #[inline]
    fn accepts(&self, nsn: &[u8]) -> bool {
        let mask = if nsn[0] != b'0' {
            self.lengths
        } else if nsn.get(1) == Some(&b'0') && !self.double_zero_led {
            return false;
        } else {
            self.zero_led
        };
        nsn.len() < 32 && mask & (1 << nsn.len()) != 0
    }

    #[inline]
    fn claims(&self, nsn: &[u8]) -> bool {
        self.territory_digits & (1 << (nsn[0] - b'0')) != 0
            && self
                .territory
                .iter()
                .any(|prefix| nsn.starts_with(prefix.as_bytes()))
    }
}

static PLANS: [Plan; COUNTRY_COUNT] = build_plans();

const fn length_mask(lengths: &[u8]) -> u32 {
    let mut mask = 0u32;
    let mut i = 0;
    while i < lengths.len() {
        mask |= 1 << lengths[i];
        i += 1;
    }
    mask
}

const fn build_plans() -> [Plan; COUNTRY_COUNT] {
    use constants::{INTERNATIONAL_PREFIXES, NATIONAL_PREFIX, TERRITORY_PREFIXES, ZERO_LED};

    let empty = Plan {
        lengths: 0,
        zero_led: 0,
        double_zero_led: false,
        trunk: "",
        early_access: &[],
        late_access: &[],
        territory: &[],
        territory_digits: 0,
        types: &[],
        first_digit_type: [NO_TYPE; 10],
    };
    let mut plans = [empty; COUNTRY_COUNT];
    let mut i = 0;
    while i < COUNTRY_COUNT {
        let country = &COUNTRIES[i];
        let mut plan = Plan {
            lengths: length_mask(country.phone_lengths),
            ..empty
        };

        let mut j = 0;
        while j < ZERO_LED.len() {
            let (code, prefix, zero, other, double_zero) = ZERO_LED[j];
            if prefix == country.prefix && const_str_eq(code, country.code) {
                plan.zero_led = length_mask(zero);
                plan.lengths = length_mask(other);
                plan.double_zero_led = double_zero;
            }
            j += 1;
        }

        // Calling code 1's trunk `1` is handled by the NANP code.
        if !is_nanp(country.prefix) {
            let mut j = 0;
            while j < NATIONAL_PREFIX.len() {
                if const_str_eq(NATIONAL_PREFIX[j].0, country.code) {
                    plan.trunk = NATIONAL_PREFIX[j].1;
                }
                j += 1;
            }
        }

        let mut j = 0;
        while j < INTERNATIONAL_PREFIXES.len() {
            let (code, early, late) = INTERNATIONAL_PREFIXES[j];
            if const_str_eq(code, country.code) {
                plan.early_access = early;
                plan.late_access = late;
            }
            j += 1;
        }

        let mut j = 0;
        while j < TERRITORY_PREFIXES.len() {
            if const_str_eq(TERRITORY_PREFIXES[j].0, country.code) {
                let prefixes = TERRITORY_PREFIXES[j].1;
                plan.territory = prefixes;
                let mut k = 0;
                while k < prefixes.len() {
                    plan.territory_digits |= 1 << (prefixes[k].as_bytes()[0] - b'0');
                    k += 1;
                }
            }
            j += 1;
        }

        let mut j = 0;
        while j < ranges::TYPE_RANGES.len() {
            let (code, prefix, rules) = ranges::TYPE_RANGES[j];
            if prefix == country.prefix && const_str_eq(code, country.code) {
                plan.types = rules;
                let mut r = 0;
                while r < rules.len() {
                    let (key, lengths) = (rules[r] >> 16, rules[r] & 0xFFF);
                    if key < 10 && lengths == 0 {
                        plan.first_digit_type[key as usize] = (rules[r] >> 12 & 0xF) as u8;
                    }
                    r += 1;
                }
            }
            j += 1;
        }

        plans[i] = plan;
        i += 1;
    }
    plans
}

// The side tables are keyed by ISO code, so a typo would silently do nothing.
// Check every key, that each zero-led split adds up to the entry's lengths,
// and that each range table is sorted, since classification binary-searches
// it.
const _: () = {
    use constants::{
        EXTRA_NATIONAL_PREFIXES, INTERNATIONAL_PREFIXES, NATIONAL_FORMAT_PREFIX, NATIONAL_PREFIX,
        TERRITORY_PREFIXES, ZERO_LED,
    };

    /// `true` when some entry has `code`, and — if `prefix` is not 0 — that
    /// calling code.
    const fn has_entry(code: &str, prefix: u32) -> bool {
        let mut i = 0;
        while i < COUNTRY_COUNT {
            if const_str_eq(COUNTRIES[i].code, code)
                && (prefix == 0 || COUNTRIES[i].prefix == prefix)
            {
                return true;
            }
            i += 1;
        }
        false
    }

    let mut j = 0;
    while j < ranges::TYPE_RANGES.len() {
        let (code, prefix, rules) = ranges::TYPE_RANGES[j];
        assert!(has_entry(code, prefix), "a TYPE_RANGES row names no entry");
        let mut r = 1;
        while r < rules.len() {
            assert!(rules[r - 1] < rules[r], "TYPE_RANGES rules must be sorted");
            r += 1;
        }
        j += 1;
    }

    let mut j = 0;
    while j < ZERO_LED.len() {
        let (code, prefix, zero, other, _) = ZERO_LED[j];
        let mut found = false;
        let mut i = 0;
        while i < COUNTRY_COUNT {
            if COUNTRIES[i].prefix == prefix && const_str_eq(COUNTRIES[i].code, code) {
                found = true;
                assert!(
                    length_mask(zero) | length_mask(other)
                        == length_mask(COUNTRIES[i].phone_lengths),
                    "a ZERO_LED row does not add up to its entry's phone_lengths"
                );
            }
            i += 1;
        }
        assert!(found, "a ZERO_LED row names no entry");
        j += 1;
    }

    let mut j = 0;
    while j < NATIONAL_PREFIX.len() {
        assert!(
            has_entry(NATIONAL_PREFIX[j].0, 0),
            "unknown code in NATIONAL_PREFIX"
        );
        j += 1;
    }
    let mut j = 0;
    while j < EXTRA_NATIONAL_PREFIXES.len() {
        assert!(
            has_entry(EXTRA_NATIONAL_PREFIXES[j].0, 0),
            "unknown code in EXTRA_NATIONAL_PREFIXES"
        );
        j += 1;
    }
    // Looked up by binary search on the calling code.
    let mut j = 1;
    while j < NATIONAL_FORMAT_PREFIX.len() {
        assert!(
            NATIONAL_FORMAT_PREFIX[j - 1].0 < NATIONAL_FORMAT_PREFIX[j].0,
            "NATIONAL_FORMAT_PREFIX must be sorted by calling code"
        );
        j += 1;
    }
    let mut j = 0;
    while j < INTERNATIONAL_PREFIXES.len() {
        assert!(
            has_entry(INTERNATIONAL_PREFIXES[j].0, 0),
            "unknown code in INTERNATIONAL_PREFIXES"
        );
        j += 1;
    }

    // A territory rule only means something for a secondary entry: the main
    // region already takes every number no rule claims.
    let mut j = 0;
    while j < TERRITORY_PREFIXES.len() {
        let code = TERRITORY_PREFIXES[j].0;
        let mut i = 0;
        let mut found = false;
        while i < COUNTRY_COUNT {
            if const_str_eq(COUNTRIES[i].code, code) {
                found = true;
                assert!(
                    primary_rank(i) == 1 && build_calling_codes()[COUNTRIES[i].prefix as usize].1 > 1,
                    "TERRITORY_PREFIXES names an entry that does not share its calling code as a secondary"
                );
            }
            i += 1;
        }
        assert!(found, "unknown code in TERRITORY_PREFIXES");
        j += 1;
    }
};

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

        // A marker only counts once some digits have been seen, and every
        // marker starts with a letter or one of `#`, `,` and `;`.
        if !at_word_start
            || !seen_digit
            || !(bytes[i].is_ascii_alphabetic() || matches!(bytes[i], b'#' | b',' | b';'))
        {
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
    /// `+` or by the `00` access code. See [`resolve_input`] for why it
    /// matters.
    explicit_international: bool,
}

impl Digits {
    fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// `calling_code` followed by `national`: a number read in a country's
    /// national format, rewritten as the global number it stands for.
    fn global(calling_code: u32, national: &[u8]) -> Result<Self, ParseError> {
        let mut out = Digits {
            buf: [0u8; 32],
            len: prefix_digit_count(calling_code),
            explicit_international: true,
        };
        let mut code = calling_code;
        for slot in out.buf[..out.len].iter_mut().rev() {
            *slot = b'0' + (code % 10) as u8;
            code /= 10;
        }
        if out.len + national.len() > E164_MAX_DIGITS {
            return Err(ParseError::TooLong);
        }
        out.buf[out.len..out.len + national.len()].copy_from_slice(national);
        out.len += national.len();
        Ok(out)
    }
}

/// Validate the characters of `input` and extract its significant digits, with
/// any international access code stripped; see [`scan_digits`].
fn extract_digits(input: &str) -> Result<Digits, ParseError> {
    let mut out = scan_digits(input)?;

    // Strip the zeros an access code leaves before the calling code ("0044…").
    // The "011…" form needs country context and is left to
    // `PhoneNumber::try_parse_with_country`.
    let start = leading_zeros(out.as_slice());
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

/// Validate the characters of `input` and collect its digits exactly as
/// written.
///
/// Vanity letters are only converted when the input already looks like a phone
/// number — at least three digits must precede the first letter. That is what
/// stops arbitrary prose such as `"HELLO WORLD"` from being read as a number.
/// Any other character is rejected, so validation and normalization can never
/// disagree about what is acceptable.
fn scan_digits(input: &str) -> Result<Digits, ParseError> {
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

    Ok(out)
}

// ============================================================================
// Country resolution
// ============================================================================

/// A resolved number: which country owns it and where its national significant
/// number starts within the digit string.
struct Resolved {
    /// Index of the owning entry in `COUNTRIES` (and `PLANS`).
    index: usize,
    country: &'static Country,
    /// Offset of the national significant number: just past the calling code,
    /// or further when a national trunk prefix was stripped.
    national_start: usize,
}

/// Count of leading `'0'` bytes.
fn leading_zeros(digits: &[u8]) -> usize {
    digits.iter().take_while(|&&b| b == b'0').count()
}

/// Resolve the country for a digit string.
///
/// Country calling codes are prefix-free outside the NANP, so the calling code
/// is determined by the digits alone; [`resolve_national`] does the rest.
fn resolve_country(digits: &[u8]) -> Result<Resolved, ParseError> {
    match digits.first() {
        None => return Err(ParseError::Empty),
        // No calling code starts with 0.
        Some(b'0') => return Err(ParseError::UnknownCountryCode),
        Some(b'1') => return resolve_nanp(digits),
        Some(_) => {}
    }

    for prefix_len in 1..=3usize {
        if digits.len() <= prefix_len {
            break;
        }
        let entries = prefix_entries(prefix_value(&digits[..prefix_len]));
        if !entries.is_empty() {
            // No other calling code can match: none is a prefix of another.
            return resolve_national(entries, digits, prefix_len);
        }
    }

    Err(ParseError::UnknownCountryCode)
}

/// Whether some entry among `entries` â€” the entries sharing one calling code â€”
/// accepts `nsn` as a national significant number.
#[inline]
fn any_accepts(entries: &[(u32, u16)], nsn: &[u8]) -> bool {
    !nsn.is_empty() && entries.iter().any(|&(_, i)| PLANS[i as usize].accepts(nsn))
}

/// Find the national significant number behind a calling code, and the entry
/// it belongs to.
///
/// The digits after the calling code are the NSN as written, unless they start
/// with a trunk prefix copied from the national format (`+44 (0)20 â€¦`,
/// `+7 8 916 â€¦`, `+36 06 1 â€¦`). A leading `0` is the delicate case, because in
/// a few plans the NSN itself begins with one ([`constants::ZERO_LED`]). Where
/// the plan's trunk prefix is `0`, a single zero is read as that trunk first
/// and `00` as part of the number; where it has none, the zero is kept first.
/// Either way the other reading is the fallback, and a reading only counts if
/// the plan has numbers of its length.
fn resolve_national(
    entries: &'static [(u32, u16)],
    digits: &[u8],
    prefix_len: usize,
) -> Result<Resolved, ParseError> {
    let national = &digits[prefix_len..];
    let trunk = PLANS[entries[0].1 as usize].trunk.as_bytes();
    let zeros = leading_zeros(national);

    let offset = if zeros == 0 {
        if any_accepts(entries, national) {
            Some(0)
        } else if !trunk.is_empty()
            && national.starts_with(trunk)
            && any_accepts(entries, &national[trunk.len()..])
        {
            Some(trunk.len())
        } else {
            None
        }
    } else {
        // A trunk prefix is a single `0` — or Hungary's `06` — never two
        // zeros: what follows `00` is not a national number with its trunk.
        let strip = if trunk.len() > 1 && national.starts_with(trunk) {
            trunk.len()
        } else {
            1
        };
        let keep_first = trunk.first() != Some(&b'0') || zeros >= 2;
        let order = if keep_first { [0, strip] } else { [strip, 0] };
        order
            .into_iter()
            .find(|&start| any_accepts(entries, &national[start..]))
    };

    let offset = offset.ok_or(ParseError::InvalidLength)?;
    let index = attribute(entries, &national[offset..]);
    Ok(Resolved {
        index,
        country: &COUNTRIES[index],
        national_start: prefix_len + offset,
    })
}

/// The entry `nsn` belongs to among the `entries` sharing its calling code: a
/// secondary territory whose prefix it starts with, otherwise the first entry â€”
/// the main region â€” whose plan has numbers of its length.
fn attribute(entries: &[(u32, u16)], nsn: &[u8]) -> usize {
    if let [(_, only)] = entries {
        return *only as usize;
    }
    let mut fallback = None;
    for &(_, i) in entries {
        let plan = &PLANS[i as usize];
        if !plan.accepts(nsn) {
            continue;
        }
        if plan.claims(nsn) {
            return i as usize;
        }
        fallback.get_or_insert(i as usize);
    }
    fallback.unwrap_or(entries[0].1 as usize)
}

/// For every NANP area code, the entry that owns it, or [`NO_OWNER`] when the
/// code is unassigned. A territory owns its four-digit `1XXX` entry and the US
/// and Canada their shared `1` ones. NANP-wide service codes â€” toll-free,
/// premium rate, personal numbering â€” belong to no country and fall to the
/// primary `+1` entry, leaving classification to report the service.
static NANP_AREA_OWNER: [u16; 1000] = build_nanp_area_owner();

const NO_OWNER: u16 = u16::MAX;

const fn build_nanp_area_owner() -> [u16; 1000] {
    let mut us = NO_OWNER;
    let mut ca = NO_OWNER;
    let mut i = 0;
    while i < COUNTRY_COUNT {
        if COUNTRIES[i].prefix == 1 {
            if const_str_eq(COUNTRIES[i].code, "US") {
                us = i as u16;
            } else if const_str_eq(COUNTRIES[i].code, "CA") {
                ca = i as u16;
            }
        }
        i += 1;
    }
    assert!(
        us != NO_OWNER && ca != NO_OWNER,
        "the +1 entries are missing"
    );

    let mut owner = [NO_OWNER; 1000];
    let mut j = 0;
    while j < nanp::NANP_AREA_CODES.len() {
        let (area, iso) = nanp::NANP_AREA_CODES[j];
        owner[area as usize] = if const_str_eq(iso, "CA") { ca } else { us };
        j += 1;
    }

    let mut i = 0;
    while i < COUNTRY_COUNT {
        let prefix = COUNTRIES[i].prefix;
        if is_nanp_territory(prefix) {
            owner[(prefix - 1000) as usize] = i as u16;
        }
        i += 1;
    }

    // Every territory tag in the area-code table must have reached its entry;
    // otherwise its numbers would silently be attributed to the US.
    let mut j = 0;
    while j < nanp::NANP_AREA_CODES.len() {
        let (area, iso) = nanp::NANP_AREA_CODES[j];
        let is_territory =
            !const_str_eq(iso, "US") && !const_str_eq(iso, "CA") && !nanp::is_non_geographic(iso);
        if is_territory {
            assert!(
                COUNTRIES[owner[area as usize] as usize].prefix == 1000 + area as u32,
                "a NANP territory area code has no four-digit entry in COUNTRIES"
            );
        }
        j += 1;
    }
    owner
}

/// Resolve a `+1` number using the assigned NANP area codes.
///
/// Length alone cannot tell the United States, Canada and the Caribbean
/// territories apart â€” they all use `1` + 10 digits. Checking the area code
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

    let owner = NANP_AREA_OWNER[area as usize];
    if owner == NO_OWNER {
        return Err(ParseError::InvalidAreaCode);
    }
    let country = &COUNTRIES[owner as usize];

    // Territories have their own four-digit calling code entry (1268 = AG);
    // the US and Canada share the bare `1`.
    let prefix_len = if is_nanp_territory(country.prefix) {
        4
    } else {
        1
    };
    Ok(Resolved {
        index: owner as usize,
        country,
        national_start: prefix_len,
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

impl Parsed {
    /// `spare` is extra capacity to reserve in `normalized`, for a caller that
    /// will append to it.
    fn new(digits: &[u8], resolved: &Resolved, extension: Option<&str>, spare: usize) -> Self {
        let normalized = normalized(digits, resolved, spare);
        let national = &normalized[1 + prefix_digit_count(resolved.country.prefix)..];
        let phone_type = classify_phone_number_type(national, resolved.index);
        Parsed {
            normalized,
            country: resolved.country,
            phone_type,
            extension: extension.map(str::to_string),
        }
    }
}

/// Validate and resolve `input` without building anything, which is all that
/// validation and country lookup need.
///
/// Input that does not mark itself as international â€” no `+`, no `00` access
/// code â€” is only accepted when the resolution is *checked* rather than guessed,
/// which today means the North American Numbering Plan, where the area code is
/// verified against the assigned ranges. Everywhere else, reading the leading
/// digits as a calling code is indistinguishable from a national number that
/// happens to start with those digits, and answering anyway turns a US number
/// into an Egyptian one. See [`ParseError::MissingCallingCode`].
fn resolve_input(input: &str) -> Result<(Digits, Resolved, Option<&str>), ParseError> {
    let input = strip_tel_uri(input.trim());
    let (base, extension) = split_extension(input);
    let digits = extract_digits(base)?;
    let resolved = resolve_digits(digits.as_slice(), digits.explicit_international)?;
    Ok((digits, resolved, extension))
}

/// Resolve a digit string that starts with its calling code.
fn resolve_digits(digits: &[u8], explicit_international: bool) -> Result<Resolved, ParseError> {
    // Early-out with a better error than the resolver would give; the floor is
    // the shortest national number plus at least one calling-code digit.
    if digits.len() < MIN_NATIONAL_LEN + 1 {
        return Err(ParseError::TooShort);
    }

    let resolved = resolve_country(digits)?;

    if !explicit_international && !is_nanp(resolved.country.prefix) {
        return Err(ParseError::MissingCallingCode);
    }
    Ok(resolved)
}

/// The national significant number of a resolved digit string.
fn national_digits<'a>(digits: &'a [u8], resolved: &Resolved) -> &'a str {
    // Infallible: digit buffers only ever hold ASCII digits.
    std::str::from_utf8(&digits[resolved.national_start..]).expect("digits are ASCII")
}

/// The E.164 form of a resolved digit string, with `spare` bytes of capacity
/// left over.
fn normalized(digits: &[u8], resolved: &Resolved, spare: usize) -> String {
    let national = national_digits(digits, resolved);
    let mut out = String::with_capacity(national.len() + 5 + spare);
    out.push('+');
    push_prefix_digits(&mut out, resolved.country.prefix);
    out.push_str(national);
    out
}

/// Parse, validate, normalize and classify in a single pass.
fn parse_internal(input: &str) -> Result<Parsed, ParseError> {
    let (digits, resolved, extension) = resolve_input(input)?;
    Ok(Parsed::new(digits.as_slice(), &resolved, extension, 0))
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
    resolve_input(phone_number).is_ok()
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
    let (digits, resolved, _) = resolve_input(phone_number)?;
    Ok(normalized(digits.as_slice(), &resolved, 0))
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
    resolve_input(phone_number)
        .ok()
        .map(|(_, resolved, _)| resolved.country)
}

/// Every country whose numbering plan could account for this number.
///
/// Some calling codes are shared by countries that a number's length cannot
/// distinguish (`+7` is Russia and Kazakhstan, `+44` is the UK, Guernsey, the
/// Isle of Man and Jersey). [`extract_country`] returns the one the number's
/// prefix points to — the main region unless a secondary territory's own
/// block claims it; this returns every territory the length fits, that one
/// first.
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
/// // Kazakh numbers start with 7, so the prefix points to Kazakhstan first.
/// let codes: Vec<_> = extract_countries("+77012345678")
///     .iter()
///     .map(|c| c.code)
///     .collect();
/// assert_eq!(codes, vec!["KZ", "RU"]);
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
    let Ok((digits, resolved, _)) = resolve_input(phone_number) else {
        return Vec::new();
    };

    // Resolved by area code, not by length: nothing to disambiguate.
    if is_nanp(resolved.country.prefix) {
        return vec![resolved.country];
    }

    let nsn = &digits.as_slice()[resolved.national_start..];
    let mut candidates = vec![resolved.country];
    candidates.extend(
        prefix_entries(resolved.country.prefix)
            .iter()
            .filter(|&&(_, i)| PLANS[i as usize].accepts(nsn))
            .map(|&(_, i)| &COUNTRIES[i as usize])
            .filter(|&country| country != resolved.country),
    );
    candidates
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
    country_index(code).map(|i| &COUNTRIES[i])
}

fn country_index(code: &str) -> Option<usize> {
    COUNTRIES
        .iter()
        .position(|c| c.code.eq_ignore_ascii_case(code))
}

/// The first reading that parses, or the error of the first one.
fn first_ok<T>(readings: Vec<Result<T, ParseError>>) -> Result<T, ParseError> {
    let mut first_error = None;
    for reading in readings {
        match reading {
            Ok(value) => return Ok(value),
            Err(error) => {
                first_error.get_or_insert(error);
            }
        }
    }
    Err(first_error.unwrap_or(ParseError::Empty))
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
    /// `(202) 555-0173`, `020 7946 0958`. A number under one of the ITU's
    /// non-geographic calling codes (`+800` freephone, `+870` Inmarsat, …) has
    /// no national form and is written as in `International`.
    National,
    /// RFC 3966 `tel:` URI: `tel:+12025550173`, with `;ext=` when the number
    /// carries an extension.
    RFC3966,
}

/// The trunk prefix digits of `code`'s plan, `""` when it has none.
fn national_prefix_for(code: &str) -> &'static str {
    constants::NATIONAL_PREFIX
        .iter()
        .find(|(c, _)| *c == code)
        .map_or("", |(_, p)| *p)
}

/// What is written in front of the first group of `national`, a national
/// significant number under `calling_code`; see
/// [`constants::NATIONAL_FORMAT_PREFIX`].
fn national_format_prefix(calling_code: u32, national: &str) -> &'static str {
    let table = constants::NATIONAL_FORMAT_PREFIX;
    let Ok(row) = table.binary_search_by_key(&calling_code, |&(code, _, _)| code) else {
        return "";
    };
    let (_, usual, exceptions) = table[row];
    exceptions
        .iter()
        .filter(|(prefix, _)| national.starts_with(prefix))
        .max_by_key(|(prefix, _)| prefix.len())
        .map_or(usual, |&(_, written)| written)
}

/// Whether `national` written behind `prefix` still reads back as itself.
///
/// It does not where the plan has no trunk prefix of its own and the written
/// `0` followed by the number is itself one of the plan's `0…` numbers:
/// Gabon's seven-digit `790 3896` against its old-plan `0790 3896`. Such a
/// number is written without the prefix, so its national form stays
/// unambiguous.
fn reads_back(country: &Country, prefix: &str, national: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    let written = || {
        prefix
            .bytes()
            .filter(u8::is_ascii_digit)
            .chain(national.bytes())
    };
    // Nor as a number dialled out through one of the plan's access codes:
    // Russia's `8` in front of a number starting `10` would read as `8 10`.
    let entries = prefix_entries(country.prefix);
    let Some(&(_, own)) = entries
        .iter()
        .find(|&&(_, i)| COUNTRIES[i as usize].code == country.code)
    else {
        return true;
    };
    let dialled_out = PLANS[own as usize].early_access.iter().any(|code| {
        let mut digits = written();
        code.bytes().all(|digit| digits.next() == Some(digit))
    });
    if dialled_out {
        return false;
    }

    let zeros = prefix.bytes().filter(|&b| b == b'0').count();
    if zeros == 0 || !PLANS[entries[0].1 as usize].trunk.is_empty() {
        return true;
    }
    let written_len = written().count();
    !entries
        .iter()
        .any(|&(_, i)| written_len < 32 && PLANS[i as usize].zero_led & (1 << written_len) != 0)
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
    let (digits, resolved, extension) = resolve_input(phone_number).ok()?;
    Some(render(
        &normalized(digits.as_slice(), &resolved, 0),
        resolved.country,
        extension,
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
            // A non-geographic number is dialled the same way from everywhere,
            // so the international form is the only one it has.
            if country.code == NON_GEOGRAPHIC_CODE {
                return render(normalized, country, extension, PhoneFormat::International);
            }
            // The NANP convention omits the trunk `1` in written form.
            if let Some(nanp) = nanp_national(country, national) {
                return format_nanp_national(&nanp);
            }
            // A number that itself begins with `0` is dialled as written,
            // never behind a trunk prefix.
            let trunk = if national.starts_with('0') {
                ""
            } else {
                national_format_prefix(country.prefix, national)
            };
            let trunk = if reads_back(country, trunk, national) {
                trunk
            } else {
                ""
            };
            let grouped = group_national(national, country, ' ');
            let mut out = String::with_capacity(trunk.len() + grouped.len());
            out.push_str(trunk);
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
    national_group_sizes(country.prefix, number)
        .or_else(|| default_group_sizes(number.len()))
        .and_then(|groups| format_grouped(number, groups, separator))
        .unwrap_or_else(|| number.to_string())
}

/// Group sizes for a national number of the plan under `calling_code`, where
/// the plan has a convention the length-based default does not match. Keyed by
/// calling code, so a territory inside another country's plan — Guernsey in
/// the UK's, the Vatican in Italy's — is written like it.
fn national_group_sizes(calling_code: u32, number: &str) -> Option<&'static [usize]> {
    // Italy keeps its trunk zero; major-city area codes are two digits, while
    // mobile numbers have no zero and group in threes.
    if calling_code == 39 && number.len() == 10 {
        return Some(if number.starts_with('0') {
            &[2, 4, 4]
        } else {
            &[3, 3, 4]
        });
    }

    match (calling_code, number.len()) {
        // London, Cardiff and the other two-digit UK area codes.
        (44, 10) => Some(&[2, 4, 4]),
        (44, 9) => Some(&[3, 3, 3]),
        (49, 10) => Some(&[2, 4, 4]),
        (49, 11) => Some(&[3, 4, 4]),
        (33, 9) => Some(&[1, 2, 2, 2, 2]),
        // "01 234 5678" / "085 123 4567" once the trunk zero is prepended.
        (353, 8) => Some(&[1, 3, 4]),
        (353, 9) => Some(&[2, 3, 4]),
        // "082 123 4567" once the trunk zero is prepended.
        (27, 9) => Some(&[2, 3, 4]),
        // Poland has no trunk prefix: "12 345 67 89".
        (48, 9) => Some(&[2, 3, 2, 2]),
        (91, 10) => Some(&[5, 5]),
        (61, 9) => Some(&[1, 4, 4]),
        (31, 9) => Some(&[1, 4, 4]),
        (81, 10) => Some(&[2, 4, 4]),
        (86, 11) => Some(&[3, 4, 4]),
        (55, 11) => Some(&[2, 5, 4]),
        (55, 10) => Some(&[2, 4, 4]),
        (7, 10) => Some(&[3, 3, 2, 2]),
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
    let (digits, resolved, _) = resolve_input(phone_number).ok()?;
    Some(classify_phone_number_type(
        national_digits(digits.as_slice(), &resolved),
        resolved.index,
    ))
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
/// Calling code 1 is classified from the NANP area code. Everywhere else the
/// number's leading digits are looked up in the entry's range table
/// ([`ranges::TYPE_RANGES`]); a number no range claims is
/// [`PhoneNumberType::Unknown`] rather than a guess.
fn classify_phone_number_type(national: &str, index: usize) -> PhoneNumberType {
    use PhoneNumberType::*;

    let country = &COUNTRIES[index];
    let b = national.as_bytes();

    // Calling code 1 is classified from the area code, which territories carry
    // in their prefix and the US and Canada in the national number.
    if is_nanp(country.prefix) {
        let area = if is_nanp_territory(country.prefix) {
            (country.prefix - 1000) as u16
        } else if b.len() >= 3 {
            prefix_value(&b[..3]) as u16
        } else {
            return Unknown;
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

    let Some(&first) = b.first() else {
        return Unknown;
    };
    let plan = &PLANS[index];
    let fast = plan.first_digit_type[(first - b'0') as usize];
    if fast != NO_TYPE {
        return ranges::TYPES[fast as usize];
    }

    // For any one length the rules are prefix-free, so the first rule that
    // covers the number's length is the only one that can.
    const KEY_OFFSET: [u32; ranges::MAX_PREFIX_DIGITS] = [0, 10, 110];
    let rules = plan.types;
    let length_bit = b
        .len()
        .checked_sub(ranges::MIN_RULE_LENGTH)
        .map_or(0, |shift| 1u32 << shift);
    let mut value = 0u32;
    for (len, &digit) in b.iter().take(ranges::MAX_PREFIX_DIGITS).enumerate() {
        value = value * 10 + (digit - b'0') as u32;
        let key = KEY_OFFSET[len] + value;
        let start = rules.partition_point(|&rule| rule >> 16 < key);
        for &rule in rules[start..].iter().take_while(|&&rule| rule >> 16 == key) {
            let lengths = rule & 0xFFF;
            if lengths == 0 || lengths & length_bit != 0 {
                return ranges::TYPES[(rule >> 12 & 0xF) as usize];
            }
        }
    }
    Unknown
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
/// Territories that share a calling code — Jersey inside `+44`, Kazakhstan
/// inside `+7`, the Vatican inside `+39` — get a number from one of their own
/// blocks.
///
/// # Returns `None` when
/// - the code is not in the country table, or
/// - the territory has no numbers of its own to draw from: Western Sahara is
///   served entirely from Morocco's `+212` ranges, and South Georgia from the
///   Falklands' `+500`, so every number resolves to the main region.
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
    let index = country_index(country_code)?;
    let mut rng = Rng::from_entropy();
    generate_for(index, &mut rng)
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
    let Some(index) = country_index(country_code) else {
        return Vec::new();
    };

    // Do not trust `count` as an allocation hint; a caller-controlled value
    // reaching `Vec::with_capacity` can abort the process.
    const PREALLOC_CAP: usize = 4096;
    let mut numbers = Vec::with_capacity(count.min(PREALLOC_CAP));
    let mut rng = Rng::from_entropy();

    for _ in 0..count {
        match generate_for(index, &mut rng) {
            Some(number) => numbers.push(number),
            None => break,
        }
    }

    numbers
}

fn generate_for(index: usize, rng: &mut Rng) -> Option<String> {
    let country = &COUNTRIES[index];
    if country.phone_lengths.is_empty() {
        return None;
    }
    // A territory sharing a calling code is only reported for numbers in its
    // own blocks, so draw from those.
    let territory = PLANS[index].territory;

    // Retry rather than emit an invalid number: a random national can land on
    // a length the validator rejects for its leading digit.
    for _ in 0..64 {
        let pick = rng.below(country.phone_lengths.len() as u64) as usize;
        let length = country.phone_lengths[pick] as usize;

        let national = if country.prefix == 1 {
            generate_nanp_national(country.code, rng)?
        } else if territory.is_empty() {
            generate_national(length, rng)
        } else {
            let block = territory[rng.below(territory.len() as u64) as usize];
            let mut national = String::with_capacity(length);
            national.push_str(block);
            while national.len() < length {
                national.push(char::from(rng.digit()));
            }
            national
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
    // Central office code: first digit 2-9. N11 exchanges exist, but only in
    // a few places, so a generated number avoids them.
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
    // Without a hint, digits too few to read as a number even after the
    // one-digit insertion below are not a phone number with a typo in it.
    if hinted.is_none() && digits.len() + 1 < MIN_UNMARKED_DIGITS {
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
#[derive(Clone)]
pub struct PhoneNumber {
    /// The E.164 form immediately followed by the input as supplied: one
    /// allocation for both.
    text: String,
    /// Length of the E.164 form at the front of `text`.
    e164_len: usize,
    country: &'static Country,
    phone_type: PhoneNumberType,
    extension: Option<String>,
}

impl std::fmt::Debug for PhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhoneNumber")
            .field("original", &self.original())
            .field("normalized", &self.e164())
            .field("country", &self.country)
            .field("phone_type", &self.phone_type)
            .field("extension", &self.extension)
            .finish()
    }
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
        Self::parse_as(input, input)
    }

    /// Parse `input`, recording `original` as what the caller supplied.
    fn parse_as(input: &str, original: &str) -> Result<Self, ParseError> {
        let (digits, resolved, extension) = resolve_input(input)?;
        let parsed = Parsed::new(digits.as_slice(), &resolved, extension, original.len());
        Ok(Self::from_parsed(parsed, original))
    }

    /// `parsed` must have been built with room for `original` to spare.
    fn from_parsed(parsed: Parsed, original: &str) -> Self {
        let mut text = parsed.normalized;
        let e164_len = text.len();
        text.push_str(original);
        PhoneNumber {
            text,
            e164_len,
            country: parsed.country,
            phone_type: parsed.phone_type,
            extension: parsed.extension,
        }
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
    /// The hint country's trunk prefix is understood (`0` almost everywhere,
    /// `8` in Russia, `06` in Hungary), and so are the international access
    /// codes dialled from it (`00`, `011` from North America, `010` from Japan,
    /// `810` from Russia).
    ///
    /// # Examples
    /// ```
    /// use phonelib::PhoneNumber;
    ///
    /// // National format resolved through the hint.
    /// let n = PhoneNumber::parse_with_country("020 7946 0958", "GB").unwrap();
    /// assert_eq!(n.e164(), "+442079460958");
    ///
    /// // Hungary's trunk prefix is `06`, not `0`.
    /// let n = PhoneNumber::parse_with_country("06 1 234 5678", "HU").unwrap();
    /// assert_eq!(n.e164(), "+3612345678");
    ///
    /// // A number dialled from Japan, through its `010` access code.
    /// let n = PhoneNumber::parse_with_country("010 44 20 7946 0958", "JP").unwrap();
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
        let index = country_index(country_code).ok_or(ParseError::UnknownCountry)?;
        let hint = &COUNTRIES[index];

        // A `tel:` URI and an explicit '+' both carry their own country code,
        // so the hint must not be applied on top.
        let trimmed = strip_tel_uri(input.trim());
        let (base, extension) = split_extension(trimmed);
        let scanned = scan_digits(base)?;
        if trimmed.len() != input.trim().len() || scanned.explicit_international {
            return Self::parse_as(trimmed, input);
        }

        let digits = scanned.as_slice();
        let finish = |global: &[u8], resolved: Resolved| {
            Self::from_parsed(
                Parsed::new(global, &resolved, extension, input.len()),
                input,
            )
        };
        // A reading of the digits that spells out the calling code itself.
        let as_global =
            |global: &[u8]| resolve_digits(global, true).map(|resolved| finish(global, resolved));
        // A reading of the digits as a national number of `calling_code`.
        let as_national = |calling_code: u32, national: &[u8]| {
            let global = Digits::global(calling_code, national)?;
            resolve_digits(global.as_slice(), true)
                .map(|resolved| finish(global.as_slice(), resolved))
        };
        let (early, late) = (PLANS[index].early_access, PLANS[index].late_access);
        let after_access_code = |codes: &mut dyn Iterator<Item = &str>| {
            codes
                .filter_map(|code| digits.strip_prefix(code.as_bytes()))
                .find_map(|rest| as_global(rest).ok())
        };

        // A national number never begins with two zeros, so a `00…` access
        // code cannot be mistaken for one — except in the few plans that do
        // have `00…` numbers of their own (Japanese and Korean freephone, say),
        // which get their national reading first.
        let double_zero = digits.starts_with(b"00");
        let zero_led_plan = PLANS[index].double_zero_led;
        let (early_now, early_later): (Vec<&str>, Vec<&str>) = early
            .iter()
            .partition(|code| !(zero_led_plan && code.starts_with("00")));

        // The plan's own access codes that no national number is dialled like
        // come first: `8 10 44 …` from Russia is never a Russian number.
        if let Some(parsed) = after_access_code(&mut early_now.into_iter()) {
            return Ok(parsed);
        }

        if double_zero && !zero_led_plan {
            if let Some(parsed) = after_access_code(&mut ["0011", "00"].into_iter()) {
                return Ok(parsed);
            }
        }

        let national = if is_nanp(hint.prefix) {
            // Ten flat digits, written bare, with the long-distance trunk
            // `1`, or — for a territory hint — as a local number needing its
            // area code back. The area code, not the hint, picks the country.
            let mut readings = vec![as_national(1, digits)];
            if let Some(rest) = digits.strip_prefix(b"1") {
                readings.push(as_national(1, rest));
            }
            if is_nanp_territory(hint.prefix) {
                readings.push(as_national(hint.prefix, digits));
            }
            first_ok(readings)
        } else {
            // With the trunk prefix removed, or as written — the resolver
            // itself reads a leading `0` as a trunk where the plan has one.
            // What is left after a trunk never starts with `0`: a number that
            // does is one of the plan's own `0…` numbers, dialled as written.
            let trunks = std::iter::once(national_prefix_for(hint.code)).chain(
                constants::EXTRA_NATIONAL_PREFIXES
                    .iter()
                    .filter(|(code, _)| *code == hint.code)
                    .map(|(_, trunk)| *trunk),
            );
            let mut readings: Vec<_> = trunks
                .filter(|trunk| !trunk.is_empty())
                .filter_map(|trunk| digits.strip_prefix(trunk.as_bytes()))
                .filter(|rest| rest.first().is_some_and(|&d| d != b'0'))
                .map(|rest| as_national(hint.prefix, rest))
                .collect();
            readings.push(as_national(hint.prefix, digits));
            first_ok(readings)
        };
        let national_error = match national {
            Ok(parsed) => return Ok(parsed),
            Err(error) => error,
        };

        if double_zero && zero_led_plan {
            if let Some(parsed) =
                after_access_code(&mut early_later.into_iter().chain(["0011", "00"]))
            {
                return Ok(parsed);
            }
        }

        // The plan's other access codes, then `011`, after the national
        // readings: they can begin a national number, as Turin's `011` does.
        if let Some(parsed) =
            after_access_code(&mut late.iter().copied().chain(std::iter::once("011")))
        {
            return Ok(parsed);
        }

        // Last resort: the hint's own calling code written without its '+',
        // perhaps behind a stray trunk prefix (`0 44 20 7946 0958`). Only the
        // hint's — reading a national number as whatever country its digits
        // happen to spell would contradict the hint.
        let mut code_digits = String::with_capacity(4);
        push_prefix_digits(&mut code_digits, hint.prefix);
        let trunk = national_prefix_for(hint.code).as_bytes();
        let behind_trunk = digits.strip_prefix(trunk).filter(|_| !trunk.is_empty());
        for global in [Some(digits), behind_trunk].into_iter().flatten() {
            if global.starts_with(code_digits.as_bytes()) {
                if let Ok(parsed) = as_global(global) {
                    return Ok(parsed);
                }
            }
        }

        Err(national_error)
    }

    /// The input exactly as it was supplied.
    #[must_use]
    pub fn original(&self) -> &str {
        &self.text[self.e164_len..]
    }

    /// The E.164 form, without any extension.
    #[must_use]
    pub fn e164(&self) -> &str {
        &self.text[..self.e164_len]
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
        &self.e164()[1 + prefix_digit_count(self.country.prefix)..]
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
        render(self.e164(), self.country, self.extension.as_deref(), fmt)
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
        self.e164() == other.e164()
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
        self.e164().cmp(other.e164())
    }
}

impl std::hash::Hash for PhoneNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.e164().hash(state);
    }
}

impl std::fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.e164())
    }
}

impl AsRef<str> for PhoneNumber {
    fn as_ref(&self) -> &str {
        self.e164()
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
        if self.numbers.contains_key(phone.e164()) {
            return Ok(false);
        }
        self.numbers.insert(phone.e164().to_string(), phone);
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
