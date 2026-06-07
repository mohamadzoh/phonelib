# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2026-06-07

### Added

- Added the missing NANP overlay area codes so multi-code regions resolve from any of their area codes: Dominican Republic `829` and `849` (in addition to `809`), and Puerto Rico (`787` and `939`), which was previously absent from the dataset

### Fixed

- Fixed `normalize_phone_number()` (and `normalize_phone_number_in_place()`) emitting strings the library itself rejected: stripping a national trunk-prefix zero could shorten a number below a valid length. Normalization now re-validates the national length after stripping and returns `None` instead of a non-idempotent/invalid result
- Fixed `generate_random_phone_number()`/`generate_random_phone_numbers()` producing invalid numbers: removed the bogus `GB` leading-zero special case (which made ~10% of `GB` numbers invalid) so national numbers always start with a non-zero digit, and the generator now re-validates each candidate and retries, guarding against rare collisions (e.g. a `US` national beginning with a Caribbean area code plus a trunk zero)
- Fixed `RFC3966` formatting leaving an orphaned trailing digit (e.g. `tel:+1-202-555-017-3`); it now groups the national number using the same group-size tables as the national/international formats, joined with `-` (e.g. `tel:+1-202-555-0173`)
- Removed dead code in `guess_country_from_number()` (a no-op `continue` branch)
- Fixed country detection for the 22 NANP territories that use a 4-digit dialing prefix (country code 1 plus a 3-digit area code, e.g. `1268` Antigua, `1876` Jamaica). `extract_country()` now resolves these to their own country instead of collapsing them into `US`, and `prefix_digit_count()`/`push_prefix_digits()` handle 4-digit prefixes. Corrected the affected `phone_lengths` entries to the 7 significant digits that follow the prefix
- Fixed `strip_extension()` to only treat `ext`/`ext.` as an extension marker at a word boundary, so words such as "next" or "text" are no longer mistaken for an extension
- Fixed a potential panic in random seed generation when the system clock is set before the Unix epoch

### Performance

- Kept country lookup on the original fast path: codes outside the NANP are prefix-free and resolve shortest-prefix-first in one or two probes, so they match 1.1.3 speed. `+1` numbers add only a few-nanosecond compile-time bitmap test to rule out the common US/Canada case before considering the new 4-digit territory prefixes
- `generate_random_phone_number()` only validates/retries for the bare NANP code (US/CA); every other country is built directly

### Changed

- Documented that the phone-number generator uses a non-cryptographic PRNG suitable only for sample/test data
- Resolved all Clippy lints across the crate, tests, and benches; `cargo clippy --all-targets -- -D warnings` now passes clean

## [1.1.3] - 2026-03-15

### Fixed

- Fixed `generate_random_phone_numbers()` reseeding logic so batch generation no longer produces repeated values when numbers are created in rapid succession
- Fixed `contains_invalid_character()` so top-level validation accepts leading `+`, reports invalid characters correctly, and stays consistent with normalization
- Fixed `suggest_phone_number_corrections()` repeated country hint lookups inside inner loops
- Fixed `classify_phone_number_type()` for `GB` numbers by matching single-digit prefixes with `d0` and service ranges with `n2`
- Fixed `analyze_phone_numbers_batch()` to derive validation, normalization, country, and type from a single parse pass per input

### Changed

- Extended national formatting with table-driven grouping for `GB`, `DE`, `FR`, `IN`, and `AU`, and replaced the generic split-in-half fallback with length-based grouping
- Wired extension stripping and vanity-letter conversion into the shared normalization path and aligned `extract_country()` with that preprocessing
- Aligned `normalize_phone_number_in_place()` with shared preprocessing for vanity numbers and extension suffixes

## [1.1.0] - 2026-03-09

### Performance

Major performance optimizations making phonelib the fastest phone number formatting library in the Rust ecosystem.

- Eliminated redundant re-normalization in `PhoneNumber::format()` by using cached normalized data
- Replaced all `format!()` macro usage in hot paths with `String::with_capacity()` + `push_str()`
- Optimized country code prefix handling with `#[inline(always)] const fn` digit counting
- Replaced `u32::to_string()` allocations with zero-alloc `push_prefix_digits()` helper
- Optimized `extract_country_data()` with byte-level prefix comparison instead of `str::parse::<u32>()`

### Added

- Added `strip_extension()` for removing extension markers (e.g., "ext. 1234", "ext 987")
- Added `convert_vanity_letters()` for phone keypad letter-to-digit conversion (e.g., "1-800-FLOWERS")
- Added IDD prefix stripping in `parse_with_country()` (handles "0011", "011", "00" prefixes)
- Added trunk prefix handling in `parse_with_country()` (strips leading 0 for national format numbers)
- Added country hint validation - `parse_with_country()` now verifies parsed country matches the hint
- Added Canada (CA) to the country database (prefix 1, 10-digit national numbers)
- Added comparative benchmarks against `rlibphonenumber` and `rust-phonenumber` for parsing and formatting

### Fixed

- Fixed `parse_with_country()` incorrectly matching "(650) 253-0000" as Singapore instead of US
- Fixed Argentina phone_lengths to include 11-digit mobile numbers

### Changed

- **Country data corrections (18 countries fixed):**
  - Brazil (BR): Added 10-digit landline support (was mobile-only 11 digits)
  - Italy (IT): Expanded phone lengths to [6-11] (was only [10])
  - New Zealand (NZ): Expanded phone lengths to [8, 9, 10] (was only [8])
  - Belgium (BE): Added 8-digit geographic number support
  - China (CN): Expanded phone lengths to [7, 8, 10, 11] (was only [11])
  - Vietnam (VN): Added 10-digit numbers (2018 number expansion)
  - Iran (IR): Corrected phone lengths to [10] (removed incorrect 11)
  - Turkey (TR): Corrected phone lengths to [10] (removed incorrect 11)
  - Reunion (RE): Corrected phone lengths to [9] (was incorrect [10])
  - Libya (LY): Expanded phone lengths to [8, 9, 10] (was only [10])
  - Finland (FI): Expanded phone lengths to [5-12] (was only [9, 11])
  - Taiwan (TW): Added 8-digit landline support (was only 9)
  - Ireland (IE): Expanded phone lengths to [7, 8, 9] (was only [9])
- **Country name updates:**
  - "Swaziland" renamed to "Eswatini"
  - "Macedonia, the Former Yugoslav Republic of" renamed to "North Macedonia"

## [1.0.0] - 2026-01-10

### First Stable Release

This release marks the first stable version of phonelib, with a production-ready API.

### Changed (Breaking from 0.3.x)

- **API Improvement**: All functions now accept `&str` instead of `String` for better ergonomics and performance:
  - `is_valid_phone_number(&str)` - no longer requires `.to_string()`
  - `extract_country(&str)` - works directly with string slices
  - `normalize_phone_number(&str)` - accepts borrowed data
  - `format_phone_number(&str, PhoneFormat)` - more flexible input
  - `detect_phone_number_type(&str)` - no allocation needed
  - `is_mobile_number(&str)` - accepts references
  - `is_landline_number(&str)` - accepts references
  - `is_toll_free_number(&str)` - accepts references
  - `are_phone_numbers_equal(&str, &str)` - compare without cloning
  - `suggest_phone_number_corrections(&str, Option<&str>)` - accepts references
  - `is_potentially_valid_phone_number(&str)` - accepts references
  - `guess_country_from_number(&str)` - accepts references

- **API Improvement**: Batch functions now accept `&[T]` where `T: AsRef<str>` instead of `Vec<String>`:
  - `validate_phone_numbers_batch(&[T])` - works with slices of any string-like type
  - `normalize_phone_numbers_batch(&[T])` - no need to create Vec<String>
  - `extract_countries_batch(&[T])` - accepts arrays, slices, or Vecs
  - `detect_phone_number_types_batch(&[T])` - flexible input types
  - `analyze_phone_numbers_batch(&[T])` - works with borrowed data
  - `group_equivalent_phone_numbers(&[T])` - accepts slices

### Added

- Added support for Cymru (Wales) with country code `GB-CYM`, prefix `44`, and phone length of 10 digits
- **Comprehensive Benchmark Suite**: Added Criterion-based benchmarks covering:
  - Single and batch validation performance
  - Normalization (clean and dirty numbers)
  - Country extraction
  - Phone formatting (E.164, International, National)
  - Type detection (mobile, landline, toll-free)
  - Text extraction from paragraphs
  - Phone number comparison
  - Scaling tests (10, 100, 1000 numbers)

### Migration Guide from 0.3.x

Before (0.3.x):
```rust
is_valid_phone_number("+12025550173".to_string());
validate_phone_numbers_batch(vec!["+12025550173".to_string()]);
```

After (1.0.0):
```rust
is_valid_phone_number("+12025550173");
validate_phone_numbers_batch(&["+12025550173"]);
```

## [0.3.0] - 2026-01-10

### Added

- Added support for Cymru (Wales) with country code `GB-CYM`, prefix `44`, and phone length of 10 digits

## [0.2.1] - Initial tracked release

- Initial release with phone number validation, formatting, parsing, and manipulation
- Support for international phone numbers with country-specific validation
