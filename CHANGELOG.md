# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
