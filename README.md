# Phonelib

[![Crates.io](https://img.shields.io/crates/v/phonelib.svg)](https://crates.io/crates/phonelib)
[![Documentation](https://docs.rs/phonelib/badge.svg)](https://docs.rs/phonelib)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.74%2B-blue.svg)](https://www.rust-lang.org)

Phonelib is a dependency-free Rust library for validating, parsing, formatting and
manipulating international phone numbers.

- No dependencies, no runtime initialization, no global state
- `#![forbid(unsafe_code)]`
- Country tables built entirely at compile time

## Features

- **Validation** - structural: every length a plan uses, for every kind of number,
  so a number is never rejected just because its range opened recently
- **Deterministic country resolution** - a shared calling code is split by each
  territory's own blocks, exactly as libphonenumber attributes them
- **Real NANP validation** - a 490-entry area-code table sourced from NANPA
- **Normalization** - canonical, idempotent E.164 output
- **Formatting** - E.164, international, national, and a conformant RFC 3966 `tel:` URI
- **Type detection** - mobile, fixed line, toll-free, premium rate and more, for
  every region
- **Extensions** - parsed, preserved, and emitted in RFC 3966 output
- **Text extraction** - find phone numbers in free-form text, in linear time
- **Error reporting** - `ParseError` and a `try_*` API tell you *why* a number was rejected
- **Country table access** - enumerate, or look up by ISO code or calling code
- **Batch processing** - validate, normalize, classify and analyze whole slices

## Accuracy and scope

Read this before choosing the crate.

**Validation checks the structure of a numbering plan, not the allocation of
its ranges.** A number is accepted when its country calling code is known and
its national significant number has a length the plan uses for *some* kind of
number — fixed line, mobile, freephone, premium rate, shared cost, UAN, VoIP or
pager. The lengths are the dialable-from-abroad lengths in Google's
libphonenumber metadata (last reconciled against release v9.0.40), taken as the
union over every number type.

The crate deliberately does not track which prefixes are in service. Operators
open new ranges every month; a plan's lengths change once in a decade. So a
Lebanese number in the `81` block is accepted whether or not that block has been
allocated yet — but a Lebanese number of 6 or 10 digits never is:

```rust
use phonelib::{is_valid_phone_number, try_normalize_phone_number, ParseError};

assert!(is_valid_phone_number("+961 81 123 456")); // 8 digits: a Lebanese length
assert!(is_valid_phone_number("+961 1 234 567")); // 7 digits: a Beirut landline
assert_eq!(
    try_normalize_phone_number("+961 123 456"),
    Err(ParseError::InvalidLength)
);
assert_eq!(
    try_normalize_phone_number("+961 12 3456 7890"),
    Err(ParseError::InvalidLength)
);
```

That trade is measurable, against libphonenumber's own data:

- **Nothing real is rejected.** Of 35,306 valid numbers generated from
  libphonenumber's per-type patterns across all 254 of its regions, every one
  is accepted and normalized to the same E.164 string, except 15 Dominican
  toll-free numbers whose exchange starts with `1` — which the NANP's structure
  forbids, and which phonelib rejects on purpose. Every one of them is also
  attributed to the same country libphonenumber picks.
- **Wrong lengths are.** All 51,550 numbers of a length their plan does not use
  are rejected.
- **It ages well.** Length rules taken from libphonenumber releases of November
  2018, March 2020 and November 2022 still accept 97.3%, 98.0% and 99.5% of the
  numbers valid today. The full range metadata of those same releases accepts
  only 79.9%, 83.8% and 91.9% of them. A range-based validator that is not kept
  up to date starts rejecting real numbers; a structural one mostly does not.

The flip side is that a number of the right length in a block nobody has opened
yet is accepted. No offline metadata can tell you a number is *in service*; if
you need to know it is in a range that is *allocated* today, use a library
backed by the complete
[libphonenumber](https://github.com/google/libphonenumber) metadata, and keep
that metadata current.

**A number has to say which country it belongs to.** Give it a leading `+`, an
`00` international access code, or a country via `parse_with_country`. A bare
national number is rejected with `ParseError::MissingCallingCode` instead of
being guessed at:

```rust
use phonelib::{normalize_phone_number, try_normalize_phone_number, ParseError, PhoneNumber};

// Ambiguous: this is a Washington DC number, and also a well-formed
// Egyptian +20 2555 0173. Neither reading is offered.
assert_eq!(normalize_phone_number("(202) 555-0173"), None);
assert_eq!(
    try_normalize_phone_number("(202) 555-0173").unwrap_err(),
    ParseError::MissingCallingCode
);

// Say which country you mean, and it resolves.
assert_eq!(
    normalize_phone_number("+1 (202) 555-0173").as_deref(),
    Some("+12025550173")
);
assert_eq!(
    PhoneNumber::parse_with_country("(202) 555-0173", "US").unwrap().e164(),
    "+12025550173"
);
```

The exception is calling code 1, where the area code is checked against the
numbering plan rather than merely fitting a length — so `12025550173` resolves
without a `+`.

**Inside the North American Numbering Plan (`+1`) the area code is checked
too.** Every NANP number has the same length, so length says nothing about
*which* of its 25 countries a number belongs to. The area code is checked
against a 490-entry table of assigned NPAs derived from the NANP Administrator's
public NPA report, and the central office code must start with 2-9. That is what
makes `US` / `CA` / Caribbean-territory attribution exact and what rejects
numbers such as `+1 111 111 1111`.

**Shared calling codes are split by block.** `+7` is Russia and Kazakhstan,
`+44` the UK and the Crown Dependencies, `+39` Italy and the Vatican. A number
in a territory's own block — Kazakhstan's leading `7`, Guernsey's `1481`, the
Vatican's `06 698` — is reported as that territory, and every other number as
the main region, exactly as libphonenumber attributes them. These prefixes only
choose *which* country is reported; they never make a number invalid, so a
block opened later is still accepted, as the main region. `extract_countries`
returns every candidate.

**Type detection** uses range tables derived from libphonenumber for every
region. A number in a block no table claims is `PhoneNumberType::Unknown`
rather than a guess, and its type never affects whether it is valid.

The ITU's non-geographic calling codes — international freephone `+800`,
Inmarsat `+870`, the satellite and international networks `+881` to `+883` and
the rest — are supported too, under the code `"001"` libphonenumber also uses.

`generate_random_phone_number` returns `None` only for Western Sahara (`EH`)
and South Georgia (`GS`), which have no numbers of their own: every `+212` and
`+500` number belongs to Morocco and the Falklands.

## Installation

```toml
[dependencies]
phonelib = "2.1.0"
```

### Dependencies

**None**, and no feature flags — that is a deliberate property of the crate, not
an accident of its current size. Adding a dependency, even an optional one, puts
it into every consumer's resolution graph, so it is treated as a design decision
rather than a convenience.

If you need to serialize a number, it is one line either way: a `PhoneNumber` is
fully described by its RFC 3966 `tel:` URI, which carries the E.164 number and
any extension together and parses straight back.

```rust
use phonelib::{PhoneNumber, PhoneFormat};

let number = PhoneNumber::parse("+1 202-555-0173 ext. 42").unwrap();

// Store this.
let stored = number.format(PhoneFormat::RFC3966);
assert_eq!(stored, "tel:+12025550173;ext=42");

// Read it back. Parsing is the validation, so a corrupted value fails loudly
// instead of becoming a number the parser would never have produced.
let restored = PhoneNumber::parse(&stored).unwrap();
assert_eq!(restored, number);
assert_eq!(restored.extension(), Some("42"));
```

`criterion` appears under `[dev-dependencies]` for the benchmarks and never
reaches a consumer's build.

Minimum supported Rust version: **1.74**. The MSRV applies to the library only.
Building the benchmarks additionally requires the `criterion` dev-dependency,
which needs a much newer rustc (1.86 at the time of writing) — which is why the
MSRV of 1.70 declared before 2.0.0 was never testable through `cargo test`.

## Quick start

```rust
use phonelib::*;

// Validation
assert!(is_valid_phone_number("+12025550173"));
assert!(!is_valid_phone_number("+11111111111")); // unassigned NANP area code

// Normalization to E.164
assert_eq!(
    normalize_phone_number("+1 (202) 555-0173"),
    Some("+12025550173".to_string())
);

// Country resolution
assert_eq!(extract_country("+12025550173").unwrap().code, "US");
assert_eq!(extract_country("+14165550198").unwrap().code, "CA");
assert_eq!(extract_country("+442079460958").unwrap().code, "GB");

// Formatting
assert_eq!(
    format_phone_number("+12025550173", PhoneFormat::National),
    Some("(202) 555-0173".to_string())
);

// Text extraction
let text = "Call me at +12025550173 or +442079460958";
assert_eq!(extract_phone_numbers_from_text(text).len(), 2);

// Comparison, regardless of formatting
let a = PhoneNumber::parse("+1 202-555-0173").unwrap();
let b = PhoneNumber::parse("12025550173").unwrap();
assert_eq!(a, b);
```

## API Reference

### Core types

```rust
use phonelib::{Country, ParseError, PhoneFormat, PhoneNumberType};

// Country: a static entry in the crate's table. `#[non_exhaustive]`.
let c: &'static Country = phonelib::country_by_code("JP").unwrap();
assert_eq!(c.name, "Japan");
assert_eq!(c.code, "JP");          // ISO 3166-1 alpha-2
assert_eq!(c.prefix, 81);          // E.164 calling code
assert_eq!(c.phone_lengths, &[8, 9, 10, 11, 12, 13][..]); // national significant lengths

// PhoneFormat and PhoneNumberType are `#[non_exhaustive]` enums, so a `match`
// over them needs a wildcard arm.
let f = PhoneFormat::International;
let label = match f {
    PhoneFormat::E164 => "e164",
    PhoneFormat::International => "international",
    PhoneFormat::National => "national",
    PhoneFormat::RFC3966 => "rfc3966",
    _ => "other",
};
assert_eq!(label, "international");

// Both types, and ParseError, implement Display.
assert_eq!(PhoneNumberType::FixedLineOrMobile.to_string(), "fixed line or mobile");
assert_eq!(ParseError::TooShort.to_string(), "fewer digits than any numbering plan accepts");
assert_eq!(c.to_string(), "Japan (+81)");
```

### Validation

```rust
use phonelib::is_valid_phone_number;

assert!(is_valid_phone_number("+12025550173"));
assert!(is_valid_phone_number("+1 (202) 555-0173"));
assert!(is_valid_phone_number("+442079460958"));
assert!(is_valid_phone_number("+390669821234"));

assert!(!is_valid_phone_number("invalid"));
assert!(!is_valid_phone_number("+11111111111")); // area code 111 is not assigned
assert!(!is_valid_phone_number("+12021111111")); // exchange 111 is structurally invalid
```

`is_valid_phone_number(n)` is exactly `normalize_phone_number(n).is_some()`, so
the two can never disagree.

### Knowing *why* a number was rejected

Every fallible entry point has a `try_*` variant returning
`Result<_, ParseError>`.

```rust
use phonelib::{try_normalize_phone_number, ParseError, PhoneNumber};

assert_eq!(try_normalize_phone_number(""), Err(ParseError::Empty));
assert_eq!(try_normalize_phone_number("abc"), Err(ParseError::InvalidCharacter));
assert_eq!(try_normalize_phone_number("+1234567890123456"), Err(ParseError::TooLong));
assert_eq!(try_normalize_phone_number("+1 202 555"), Err(ParseError::InvalidLength));
assert_eq!(try_normalize_phone_number("+9999999999999"), Err(ParseError::UnknownCountryCode));
assert_eq!(try_normalize_phone_number("+11111111111"), Err(ParseError::InvalidAreaCode));
assert_eq!(try_normalize_phone_number("+12021111111"), Err(ParseError::InvalidExchange));

// The hint itself can be wrong.
assert_eq!(
    PhoneNumber::try_parse_with_country("020 7946 0958", "ZZ"),
    Err(ParseError::UnknownCountry)
);

// ParseError implements std::error::Error.
fn takes_error(_: &dyn std::error::Error) {}
takes_error(&ParseError::TooShort);
```

### Normalization

```rust
use phonelib::{normalize_phone_number, normalize_phone_number_in_place};

assert_eq!(
    normalize_phone_number("+1 (202) 555-0173"),
    Some("+12025550173".to_string())
);

// A national trunk zero is removed...
assert_eq!(
    normalize_phone_number("+44 (0)20 7946 0958"),
    Some("+442079460958".to_string())
);

// ...but Italy's leading zero is significant and is preserved.
assert_eq!(
    normalize_phone_number("+390669821234"),
    Some("+390669821234".to_string())
);

// Normalization is idempotent.
let once = normalize_phone_number("+1 202-555-0173").unwrap();
assert_eq!(normalize_phone_number(&once), Some(once.clone()));

// In place: the buffer ends up holding the normalized value, and the return
// value always agrees with it.
let mut phone = "+1 (202) 555-0173".to_string();
assert_eq!(
    normalize_phone_number_in_place(&mut phone),
    Some("+12025550173".to_string())
);
assert_eq!(phone, "+12025550173");

// On failure the buffer is left untouched.
let mut invalid = "nonsense".to_string();
assert_eq!(normalize_phone_number_in_place(&mut invalid), None);
assert_eq!(invalid, "nonsense");
```

### Country resolution

Resolution is deterministic. A calling code shared by several countries has one
main region; a number in another territory's own block is reported as that
territory, and every other number as the main region. `extract_countries`
returns every candidate, the resolved one first.

```rust
use phonelib::{extract_countries, extract_country};

assert_eq!(extract_country("+442079460958").unwrap().code, "GB");
assert_eq!(extract_country("+441481256789").unwrap().code, "GG"); // Guernsey
assert_eq!(extract_country("+79161234567").unwrap().code, "RU");
assert_eq!(extract_country("+77012345678").unwrap().code, "KZ"); // Kazakhstan
assert_eq!(extract_country("+390649911234").unwrap().code, "IT"); // Rome
assert_eq!(extract_country("+390669812345").unwrap().code, "VA"); // the Vatican
assert_eq!(extract_country("+870773111632").unwrap().code, "001"); // Inmarsat

// Every country whose plan could account for the number.
let codes: Vec<_> = extract_countries("+442079460958")
    .iter()
    .map(|c| c.code)
    .collect();
assert_eq!(codes, ["GB", "GG", "IM", "JE"]);

let codes: Vec<_> = extract_countries("+77012345678")
    .iter()
    .map(|c| c.code)
    .collect();
assert_eq!(codes, ["KZ", "RU"]);
```

### NANP resolution

`+1` numbers are resolved through the assigned area codes rather than guessed,
so the United States, Canada and each Caribbean/Pacific territory come back
correctly.

```rust
use phonelib::{detect_phone_number_type, extract_country, PhoneNumberType};

assert_eq!(extract_country("+12025550173").unwrap().code, "US");
assert_eq!(extract_country("+16045550198").unwrap().code, "CA");
assert_eq!(extract_country("+18764561234").unwrap().code, "JM");
assert_eq!(extract_country("+12684641234").unwrap().code, "AG");
assert_eq!(extract_country("+17875551234").unwrap().code, "PR");
assert_eq!(extract_country("+18095551234").unwrap().code, "DO");

// Territories carry the area code in their four-digit calling code.
assert_eq!(extract_country("+18764561234").unwrap().prefix, 1876);

// NANP-wide service ranges (800/900/500/700/710) are non-geographic, so they
// are resolved by *type*. They are still attributed to a country: the NANP has
// no non-geographic entry of its own, so `extract_country` reports the United
// States.
assert_eq!(
    detect_phone_number_type("+18005550199"),
    Some(PhoneNumberType::TollFree)
);
assert_eq!(
    detect_phone_number_type("+19005550199"),
    Some(PhoneNumberType::PremiumRate)
);
assert_eq!(extract_country("+18005550199").unwrap().code, "US");
assert_eq!(extract_country("+19005550199").unwrap().code, "US");
```

Do not read `US` on an 8XX/900/5XX/70X number as a geographic fact: those
ranges are reachable from anywhere in the NANP and say nothing about where the
subscriber is.

### Country table access

```rust
use phonelib::{countries, countries_by_calling_code, countries_by_code, country_by_code};

assert_eq!(countries().len(), 259);
assert!(countries().iter().any(|c| c.code == "JP"));

assert_eq!(country_by_code("JP").unwrap().prefix, 81);
assert!(country_by_code("ZZ").is_none());

// The Dominican Republic operates three overlay calling codes.
let prefixes: Vec<_> = countries_by_code("DO").iter().map(|c| c.prefix).collect();
assert_eq!(prefixes, [1809, 1829, 1849]);

// Everyone sharing a calling code, in resolution order.
let codes: Vec<_> = countries_by_calling_code(44).iter().map(|c| c.code).collect();
assert_eq!(codes, ["GB", "GG", "IM", "JE"]);
```

### Formatting

```rust
use phonelib::{format_phone_number, PhoneFormat};

assert_eq!(
    format_phone_number("+12025550173", PhoneFormat::E164),
    Some("+12025550173".to_string())
);
assert_eq!(
    format_phone_number("+12025550173", PhoneFormat::International),
    Some("+1 202 555 0173".to_string())
);
assert_eq!(
    format_phone_number("+12025550173", PhoneFormat::National),
    Some("(202) 555-0173".to_string())
);
assert_eq!(
    format_phone_number("+12025550173", PhoneFormat::RFC3966),
    Some("tel:+12025550173".to_string())
);

// National format includes the country's trunk prefix where it has one.
assert_eq!(
    format_phone_number("+442079460958", PhoneFormat::National),
    Some("020 7946 0958".to_string())
);
assert_eq!(
    format_phone_number("+493012345678", PhoneFormat::National),
    Some("030 1234 5678".to_string())
);

// ...but only where the country dials it: Chinese mobiles take no 0, and
// Russia writes its trunk 8 as a token of its own.
assert_eq!(
    format_phone_number("+8613112345678", PhoneFormat::National),
    Some("131 1234 5678".to_string())
);
assert_eq!(
    format_phone_number("+79161234567", PhoneFormat::National),
    Some("8 916 123 45 67".to_string())
);

// Italy has no trunk prefix; the leading zero is part of the number.
assert_eq!(
    format_phone_number("+390649911234", PhoneFormat::National),
    Some("06 4991 1234".to_string())
);
assert_eq!(
    format_phone_number("+390649911234", PhoneFormat::International),
    Some("+39 06 4991 1234".to_string())
);

// An invalid number formats to None rather than to nonsense.
assert_eq!(format_phone_number("nonsense", PhoneFormat::E164), None);
```

`E164` and `RFC3966` output re-parses; `National` output deliberately does not,
because it carries no country calling code. Round-trip it with
`PhoneNumber::parse_with_country`.

```rust
use phonelib::{PhoneFormat, PhoneNumber};

let n = PhoneNumber::parse("+442079460958").unwrap();
let national = n.format(PhoneFormat::National);
assert_eq!(national, "020 7946 0958");
assert_eq!(
    PhoneNumber::parse_with_country(&national, "GB").unwrap().e164(),
    "+442079460958"
);
```

### Extensions

Extensions are parsed, kept on the `PhoneNumber`, and emitted in RFC 3966
output. E.164 has no representation for an extension, so `e164()` never carries
one.

```rust
use phonelib::{PhoneFormat, PhoneNumber};

let n = PhoneNumber::parse("+1 202-555-0173 ext. 42").unwrap();
assert_eq!(n.extension(), Some("42"));
assert_eq!(n.e164(), "+12025550173");
assert_eq!(n.format(PhoneFormat::RFC3966), "tel:+12025550173;ext=42");

// The RFC 3966 output is conformant and re-parses, extension included.
let round_tripped = PhoneNumber::parse(&n.format(PhoneFormat::RFC3966)).unwrap();
assert_eq!(round_tripped.e164(), "+12025550173");
assert_eq!(round_tripped.extension(), Some("42"));

// Markers are only recognised at a word boundary, so a word that merely starts
// with one is not an extension — it stays a stray word, and the input is
// rejected rather than silently acquiring an extension.
assert!(PhoneNumber::parse("+12025550173 next 42").is_none());
assert!(PhoneNumber::parse("+12025550173 text 42").is_none());
assert_eq!(
    PhoneNumber::parse("+12025550173 extension 42").unwrap().extension(),
    Some("42")
);

// A number with no marker at all simply has no extension.
assert_eq!(PhoneNumber::parse("+12025550173").unwrap().extension(), None);
```

### Type detection

```rust
use phonelib::{
    detect_phone_number_type, is_landline_number, is_mobile_number, is_toll_free_number,
    PhoneNumberType,
};

assert_eq!(
    detect_phone_number_type("+447911123456"),
    Some(PhoneNumberType::Mobile)
);
assert_eq!(
    detect_phone_number_type("+442079460958"),
    Some(PhoneNumberType::FixedLine)
);
assert_eq!(
    detect_phone_number_type("+18005550199"),
    Some(PhoneNumberType::TollFree)
);

// North American geographic ranges carry both fixed lines and mobiles, and
// number portability has decoupled the prefix from the technology, so the
// honest answer is FixedLineOrMobile rather than a coin flip.
assert_eq!(
    detect_phone_number_type("+12025550173"),
    Some(PhoneNumberType::FixedLineOrMobile)
);

// Every country has a range table, derived from libphonenumber's metadata.
assert_eq!(
    detect_phone_number_type("+96171123456"),
    Some(PhoneNumberType::Mobile)
);
assert_eq!(
    detect_phone_number_type("+9611234567"),
    Some(PhoneNumberType::FixedLine)
);

// A valid number in a block no range claims — the UK has never opened `4` —
// is Unknown, not a guess.
assert_eq!(
    detect_phone_number_type("+444123456789"),
    Some(PhoneNumberType::Unknown)
);

// An invalid number has no type at all.
assert_eq!(detect_phone_number_type("nonsense"), None);

assert!(is_mobile_number("+447911123456"));
assert!(is_landline_number("+442079460958"));
assert!(is_toll_free_number("+18005550199"));

// `false` here means "not known to be mobile", not "known not to be mobile".
assert!(!is_mobile_number("+12025550173"));
```

### Text extraction

```rust
use phonelib::{
    count_phone_numbers_in_text, extract_phone_numbers_from_text,
    extract_phone_numbers_with_country_hint, extract_valid_phone_numbers_from_text,
    redact_phone_numbers, replace_phone_numbers_in_text,
};

let text = "Contact +1 202-555-0173 or our London office on +44 20 7946 0958.";

let numbers = extract_phone_numbers_from_text(text);
assert_eq!(numbers.len(), 2);
assert_eq!(numbers[0].raw, "+1 202-555-0173");
assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
assert!(numbers[0].is_valid);

// Spans are exact and never overlap.
for n in &numbers {
    assert_eq!(&text[n.start..n.end], n.raw);
}

assert_eq!(count_phone_numbers_in_text(text), 2);
assert_eq!(extract_valid_phone_numbers_from_text("Call +12025550173 or 123").len(), 1);

// National-format numbers need a country hint.
let hinted = extract_phone_numbers_with_country_hint("Call (202) 555-0173 for help", "US");
assert_eq!(hinted[0].normalized.as_deref(), Some("+12025550173"));

// A number that carries its own country code is never given a second one.
let hinted = extract_phone_numbers_with_country_hint("Call +12025550173", "DE");
assert_eq!(hinted[0].normalized.as_deref(), Some("+12025550173"));

// Rewrite
assert_eq!(
    replace_phone_numbers_in_text(text, |n| {
        format!("[{}]", n.normalized.as_deref().unwrap_or(&n.raw))
    }),
    "Contact [+12025550173] or our London office on [+442079460958]."
);

// Redact, keeping the last four digits. Separators and the leading '+' are not
// preserved: the output is '*' characters followed by the visible digits.
assert_eq!(
    redact_phone_numbers(text, 4),
    "Contact *******0173 or our London office on ********0958."
);

// visible_digits == 0 replaces the whole number.
assert_eq!(
    redact_phone_numbers("Call +12025550173 now", 0),
    "Call [PHONE] now"
);
```

### `PhoneNumber`

`PhoneNumber` has private fields and accessor methods, so a value can never be
constructed in an inconsistent state. `country()` and `phone_type()` are not
`Option` — a parsed number always has both.

```rust
use phonelib::{PhoneFormat, PhoneNumber, PhoneNumberType};
use std::collections::HashSet;

let n = PhoneNumber::parse("+1 (202) 555-0173").unwrap();

assert_eq!(n.original(), "+1 (202) 555-0173");
assert_eq!(n.e164(), "+12025550173");
assert_eq!(n.national_number(), "2025550173");
assert_eq!(n.country().code, "US");
assert_eq!(n.country_code(), 1);
assert_eq!(n.phone_type(), PhoneNumberType::FixedLineOrMobile);
assert_eq!(n.extension(), None);
assert_eq!(n.format(PhoneFormat::International), "+1 202 555 0173");

// Equality and Hash both follow the E.164 form, so the Eq/Hash contract holds.
let a = PhoneNumber::parse("+1 202-555-0173").unwrap();
let b = PhoneNumber::parse("12025550173").unwrap();
assert_eq!(a, b);

let mut set = HashSet::new();
set.insert(a.clone());
set.insert(b.clone());
assert_eq!(set.len(), 1);

// FromStr / TryFrom, with a real error type.
let parsed: PhoneNumber = "+12025550173".parse().unwrap();
assert_eq!(parsed.e164(), "+12025550173");
assert!("nonsense".parse::<PhoneNumber>().is_err());
assert!(PhoneNumber::try_from("+12025550173").is_ok());

// A country hint resolves national-format input.
let uk = PhoneNumber::parse_with_country("020 7946 0958", "GB").unwrap();
assert_eq!(uk.e164(), "+442079460958");

// The number's own country code always wins over a contradicting hint.
let us = PhoneNumber::parse_with_country("+12025550173", "DE").unwrap();
assert_eq!(us.country().code, "US");
```

### `PhoneNumberSet`

```rust
use phonelib::PhoneNumberSet;

let mut set = PhoneNumberSet::new();
assert!(set.add("+1 202-555-0173"));
assert!(!set.add("+1 (202) 555-0173")); // same number in another formatting
assert!(set.add("+442079460958"));
assert_eq!(set.len(), 2);

// Membership and lookup work from any formatting.
assert!(set.contains("+1-202-555-0173"));
assert_eq!(set.get("+1 202 555 0173").unwrap().e164(), "+12025550173");

// `try_add` tells an already-present number apart from an unparseable one.
assert_eq!(set.try_add("+12025550173"), Ok(false));
assert!(set.try_add("nonsense").is_err());

assert!(set.remove("+442079460958"));
assert_eq!(set.len(), 1);

// Build one from an iterator.
let set: PhoneNumberSet = vec!["+12025550173", "+1 202 555 0173", "+442079460958"]
    .into_iter()
    .collect();
assert_eq!(set.len(), 2);
```

### Batch processing

```rust
use phonelib::{
    analyze_phone_numbers_batch, detect_phone_number_types_batch, extract_countries_batch,
    normalize_phone_numbers_batch, validate_phone_numbers_batch, ParseError, PhoneNumberType,
};

let numbers = ["+1 202 555 0173", "nonsense"];

assert_eq!(validate_phone_numbers_batch(&numbers), vec![true, false]);
assert_eq!(
    normalize_phone_numbers_batch(&numbers),
    vec![Some("+12025550173".to_string()), None]
);

let countries: Vec<_> = extract_countries_batch(&numbers)
    .iter()
    .map(|c| c.map(|c| c.code))
    .collect();
assert_eq!(countries, vec![Some("US"), None]);

assert_eq!(
    detect_phone_number_types_batch(&["+18005550199"]),
    vec![Some(PhoneNumberType::TollFree)]
);

// Full analysis, including the rejection reason.
let analyses = analyze_phone_numbers_batch(&numbers);
assert!(analyses[0].is_valid);
assert_eq!(analyses[0].country.unwrap().code, "US");
assert_eq!(analyses[0].normalized.as_deref(), Some("+12025550173"));
assert!(!analyses[1].is_valid);
assert_eq!(analyses[1].error, Some(ParseError::InvalidCharacter));
```

### Comparison and grouping

```rust
use phonelib::{are_phone_numbers_equal, group_equivalent_phone_numbers};

assert!(are_phone_numbers_equal("+1 202-555-0173", "+1 (202) 555-0173"));
assert!(!are_phone_numbers_equal("+12025550173", "+12025550174"));

// A significant leading zero is not the same number without it.
assert!(!are_phone_numbers_equal("+390669821234", "+39669821234"));

// Grouping is hash-based and linear; unparseable inputs each form their own
// group, so every input appears exactly once in the output.
let groups = group_equivalent_phone_numbers(&[
    "+1 202-555-0173",
    "+1 (202) 555-0173",
    "+442079460958",
    "nonsense",
]);
assert_eq!(groups.len(), 3);
assert_eq!(groups[0].len(), 2);
```

### Suggestions

```rust
use phonelib::{
    guess_country_from_number, is_potentially_valid_phone_number,
    is_valid_phone_number, suggest_phone_number_corrections,
};

// An already valid number comes back unchanged.
assert_eq!(
    suggest_phone_number_corrections("+12025550173", None),
    vec!["+12025550173".to_string()]
);

// Repairs are ranked by edit cost and every suggestion is itself valid.
let suggestions = suggest_phone_number_corrections("+1 202 555 017", Some("US"));
assert!(!suggestions.is_empty());
assert!(suggestions.iter().all(|s| is_valid_phone_number(s)));

assert!(is_potentially_valid_phone_number("202-555-0173"));
assert!(!is_potentially_valid_phone_number("12"));
// Anything valid is also potentially valid.
assert!(is_potentially_valid_phone_number("+12025550173"));

assert_eq!(guess_country_from_number("442079460958").unwrap().code, "GB");
```

### Random generation

The generator uses a non-cryptographic PRNG and is intended for sample and test
data only. Every number it returns is valid and resolves back to the country
that was asked for.

```rust
use phonelib::{
    extract_country, generate_random_phone_number, generate_random_phone_numbers,
    is_valid_phone_number,
};

let number = generate_random_phone_number("CA").unwrap();
assert!(is_valid_phone_number(&number));
assert_eq!(extract_country(&number).unwrap().code, "CA");

let numbers = generate_random_phone_numbers("GB", 5);
assert_eq!(numbers.len(), 5);
assert!(numbers.iter().all(|n| is_valid_phone_number(n)));

// A territory sharing a calling code gets a number from its own block.
let jersey = generate_random_phone_number("JE").unwrap();
assert_eq!(extract_country(&jersey).unwrap().code, "JE");

// `None` for Western Sahara and South Georgia, which have no numbers of their
// own...
assert!(generate_random_phone_number("EH").is_none());
// ...and for an unknown ISO code.
assert!(generate_random_phone_number("ZZ").is_none());
```

## Country support

The table holds **259 entries**: 250 countries and territories, including the
NANP territories that operate their own four-digit `1` + area code calling
code, and the nine calling codes the ITU assigns to global services rather than
to a country (`+800`, `+808`, `+870`, `+878`, `+881`, `+882`, `+883`, `+888`,
`+979`), which carry the code `"001"`.

```rust
use phonelib::countries;

assert_eq!(countries().len(), 259);

// Every entry has an ISO 3166-1 alpha-2 code — or "001" for a global
// service — and at least one valid length.
assert!(countries()
    .iter()
    .all(|c| (c.code.len() == 2 || c.code == "001") && !c.phone_lengths.is_empty()));
```

Three ISO codes appear on more than one entry, because the country operates
several calling codes: the Dominican Republic's `1809`/`1829`/`1849` overlays,
Jamaica's `1876`/`1658`, and Puerto Rico's `1787`/`1939` — as does `"001"`, once
per global service. Use `countries_by_code` to see all of them.

```rust
use phonelib::{countries, countries_by_code};

let multi: Vec<_> = countries()
    .iter()
    .map(|c| c.code)
    .filter(|code| countries_by_code(code).len() > 1)
    .collect::<std::collections::BTreeSet<_>>()
    .into_iter()
    .collect();
assert_eq!(multi, ["001", "DO", "JM", "PR"]);
```

## Performance

Everything the parser consults is folded into flat tables at compile time, so a
lookup is an array index and a length check is a bit test: a calling code is
found by direct index, a NANP area code likewise, and a number's lengths are a
bitmask per entry. There is no runtime initialization and no first-call cliff,
and numbers are extracted from text in a single linear pass.

`is_valid_phone_number`, `extract_country` and `detect_phone_number_type` do
not allocate at all. `normalize_phone_number` and the formatters allocate the
string they return, and a `PhoneNumber` holds its E.164 form and the original
input in a single allocation.

Against 2.0.0 on the same machine, validating a number takes 48-60% less
time, country extraction 48-56% less, type detection about 32% less,
`parse_with_country` about 50% less and a rejected parse about 30% less;
`PhoneNumber::parse` is 7-16% faster, normalization, E.164 and international
formatting and text extraction 4-15% faster, and national formatting — which
now writes the prefix each country actually dials — takes the same time.

Run the suite yourself with `cargo bench`; the numbers depend heavily on the
machine, so none are quoted here. The benchmarks cover validation, normalization,
country extraction, all four formats, type detection, batch processing, parsing,
comparison and grouping, redaction, and text extraction at sizes from a single
sentence up to 256 KB.

## Migration from 1.x

2.0.0 is a breaking release.

**`PhoneNumber` fields are private.** Use the accessors.

```rust
use phonelib::PhoneNumber;

let n = PhoneNumber::parse("+12025550173").unwrap();

// 1.x: n.original / n.normalized / n.country / n.phone_type / n.extension
assert_eq!(n.original(), "+12025550173");
assert_eq!(n.e164(), "+12025550173");
assert_eq!(n.national_number(), "2025550173");
assert_eq!(n.country().code, "US");
assert_eq!(n.country_code(), 1);
assert_eq!(n.extension(), None);
```

**Two accessors that already existed changed signature.**
`national_number()` returns `&str` rather than `String`, and `country_code()`
returns `u32` rather than `Option<u32>`. Code that called either compiles
differently even though it never touched the fields.

```rust
use phonelib::PhoneNumber;

let n = PhoneNumber::parse("+12025550173").unwrap();

// 1.x: let nsn: String = n.national_number();
let nsn: &str = n.national_number();
assert_eq!(nsn, "2025550173");

// 1.x: let cc: Option<u32> = n.country_code();
let cc: u32 = n.country_code();
assert_eq!(cc, 1);
```

**`country()` and `phone_type()` are no longer `Option`.**

```rust
use phonelib::{PhoneNumber, PhoneNumberType};

let n = PhoneNumber::parse("+447400123456").unwrap();

// 1.x: n.country.map(|c| c.code) == Some("GB")
assert_eq!(n.country().code, "GB");
// 1.x: n.phone_type == Some(PhoneNumberType::Mobile)
assert_eq!(n.phone_type(), PhoneNumberType::Mobile);
```

**`FromStr::Err` is now `ParseError` instead of `&'static str`.**

```rust
use phonelib::{ParseError, PhoneNumber};

let err: ParseError = "nonsense".parse::<PhoneNumber>().unwrap_err();
assert_eq!(err, ParseError::InvalidCharacter);
```

**`PhoneNumberSet::find_duplicates` is renamed to `get`.**

```rust
use phonelib::PhoneNumberSet;

let mut set = PhoneNumberSet::new();
set.add("+12025550173");
// 1.x: set.find_duplicates("+1 202 555 0173")
assert_eq!(set.get("+1 202 555 0173").unwrap().e164(), "+12025550173");
```

**`Country`, `PhoneFormat`, `PhoneNumberType`, `ParseError` and
`PhoneNumberAnalysis` are `#[non_exhaustive]`.** A `match` over any of these
enums now needs a wildcard arm, and `Country` and `PhoneNumberAnalysis` can no
longer be constructed or exhaustively destructured outside the crate —
`PhoneNumberAnalysis` had all-public fields in 1.x, so hand-built test fixtures
and mocks need to come from `analyze_phone_numbers_batch` instead.

```rust
use phonelib::PhoneNumberType;

let t = PhoneNumberType::Mobile;
let s = match t {
    PhoneNumberType::Mobile => "mobile",
    PhoneNumberType::FixedLine => "fixed",
    _ => "other", // required
};
assert_eq!(s, "mobile");
```

**`PhoneNumberType` gained `FixedLineOrMobile` and lost `Emergency` and
`Voicemail`.** North American geographic numbers now report `FixedLineOrMobile`
instead of being guessed as `FixedLine` or `Mobile`. Neither removed variant was
ever produced by the classifier, but they were public, so any code that named
them — in a `match` arm or a constructor — no longer compiles.

**Country resolution results changed.** If you persisted resolved countries,
re-resolve them:

| Calling code | 1.2.0 result | 2.0.0 result |
|---|---|---|
| `+44` | `GB-CYM` ("Cymru") | `GB` |
| `+1` | `US` for everything except the 26 four-digit territory prefixes — Canada included | `US` / `CA` / the correct territory, from the area code |
| `+7` | `KZ` | `RU` |
| `+212` | `EH` | `MA` |
| `+358` | `AX` | `FI` |
| `+379` | `VA` | no longer a calling code — see below |

```rust
use phonelib::extract_country;

assert_eq!(extract_country("+442079460958").unwrap().code, "GB");
assert_eq!(extract_country("+16045550198").unwrap().code, "CA");
assert_eq!(extract_country("+79161234567").unwrap().code, "RU");
assert_eq!(extract_country("+212612345678").unwrap().code, "MA");
assert_eq!(extract_country("+358401234567").unwrap().code, "FI");

// Canada is the one to re-check: 1.2.0 already resolved the four-digit
// Caribbean/Pacific territory prefixes, but collapsed every Canadian number
// into `US`.
assert_eq!(extract_country("+14165550198").unwrap().code, "CA");
assert_eq!(extract_country("+18764561234").unwrap().code, "JM"); // also correct in 1.2.0
```

The `GB-CYM` ("Cymru") entry has been **removed**. It was not an ISO 3166-1
code, and because it shared calling code `44` and a 10-digit length with the
real `GB` entry it shadowed it, so every UK number resolved to "Cymru" and the
`GB` type-detection and formatting tables were unreachable. Previous releases
advertised it as a feature; it was a bug.

**Vatican City moved off `+379`.** `+379` is assigned to the Holy See by the
ITU but has never been brought into service; Vatican numbers are dialled through
Italy. The `VA` entry now sits on `+39`, so `+379` numbers that validated in
1.2.0 are rejected with `ParseError::UnknownCountryCode`.

```rust
use phonelib::{extract_country, try_normalize_phone_number, ParseError};

// 1.2.0: Some("VA")
assert_eq!(try_normalize_phone_number("+3791234567"), Err(ParseError::UnknownCountryCode));
assert_eq!(extract_country("+390649911234").unwrap().code, "IT");
assert_eq!(phonelib::country_by_code("VA").unwrap().prefix, 39);
```

## Contributing

Contributions are welcome.

- **Report bugs** - open an issue if you find a problem
- **Suggest features** - share ideas for new functionality
- **Submit pull requests** - help improve the code
- **Improve documentation** - help make the docs better
- **Add tests** - increase coverage

### Development setup

```bash
git clone https://github.com/mohamadzoh/phonelib.git
cd phonelib

# Tests, including all doctests
cargo test
cargo test --release

# Lints and formatting
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Documentation
cargo doc --no-deps

# Benchmarks (needs a newer rustc than the library MSRV)
cargo bench

# MSRV check (rustup needs the full version)
cargo +1.74.0 check --lib

# The README's Rust examples are real doctests - run them against the crate
cargo build --lib
rustdoc --test README.md --edition 2021   --extern phonelib=target/debug/libphonelib.rlib   -L dependency=target/debug/deps
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## License

MIT — see [LICENSE](LICENSE).

## Rusty Rails Project

Phonelib is part of the larger **Rusty Rails** project, which aims to bridge the
gap between the Rust and Ruby/Ruby on Rails ecosystems.

---

**Made with love by the Rusty Rails team**
