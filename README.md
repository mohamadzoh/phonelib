# Phonelib

[![Crates.io](https://img.shields.io/crates/v/phonelib.svg)](https://crates.io/crates/phonelib)
[![Documentation](https://docs.rs/phonelib/badge.svg)](https://docs.rs/phonelib)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

Phonelib is a comprehensive Rust library for handling phone numbers. It provides functions for validation, formatting, type detection, text extraction, batch processing, and much more.

## Features

- ✅ **Phone Number Validation** - Check if phone numbers are valid
- 🌍 **Country Detection** - Extract country information from phone numbers  
- 🔧 **Normalization** - Clean and standardize phone number formats
- 🎨 **Multiple Format Support** - E.164, International, National, RFC3966
- 📱 **Type Detection** - Identify mobile, landline, toll-free, premium numbers
- 📝 **Text Extraction** - Parse phone numbers from free-form text
- ⚖️ **Comparison & Equality** - Compare numbers regardless of format with `PhoneNumber` struct
- 🎲 **Random Number Generation** - Generate valid random phone numbers
- 🚀 **Batch Processing** - Process multiple numbers efficiently
- 🔍 **Smart Suggestions** - Get correction suggestions for invalid numbers
- 🔒 **Privacy Tools** - Redact phone numbers in text

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
phonelib = "1.0.1"
```

## Quick Start

```rust
use phonelib::*;

// Basic validation
let is_valid = is_valid_phone_number("+12025550173");

// Format a number
let formatted = format_phone_number(
    "12025550173", 
    PhoneFormat::International
);

// Detect number type
let number_type = detect_phone_number_type("+442079460958");

// Extract phone numbers from text
let text = "Call me at +12025550173 or +442079460958";
let numbers = extract_phone_numbers_from_text(text);

// Compare phone numbers (PhoneNumber struct with Eq trait)
let num1 = PhoneNumber::parse("+12025550173").unwrap();
let num2 = PhoneNumber::parse("12025550173").unwrap();
assert_eq!(num1, num2); // Same number, different formats
```

## API Reference

### Core Types

```rust
pub struct Country {
    pub name: &'static str,
    pub code: &'static str,
    pub phone_lengths: &'static [u8],
    pub prefix: u32,
}

pub enum PhoneFormat {
    E164,          // +1234567890
    International, // +1 234 567-890
    National,      // (234) 567-890
    RFC3966,       // tel:+1-234-567-890
}

pub enum PhoneNumberType {
    Mobile, FixedLine, TollFree, PremiumRate, 
    SharedCost, Voip, PersonalNumber, Pager, 
    Uan, Emergency, Voicemail, Unknown,
}
```

### Basic Functions

#### Phone Number Validation

```rust
use phonelib::is_valid_phone_number;

let phone_number = "+1234567890";
if is_valid_phone_number(phone_number) {
    println!("Valid phone number!");
}
```

#### Country Extraction

```rust
use phonelib::extract_country;

let phone_number = "+1234567890";
match extract_country(phone_number) {
    Some(country) => println!("Country: {} ({})", country.name, country.code),
    None => println!("Country not found"),
}
```

#### Phone Number Normalization

```rust
use phonelib::{normalize_phone_number, normalize_phone_number_in_place};

// Returns normalized number without modifying input
let normalized = normalize_phone_number("+1 (234) 567-890");
println!("Normalized: {:?}", normalized); // Some("+1234567890")

// Modifies the input string in place
let mut phone = "+1 (234) 567-890".to_string();
normalize_phone_number_in_place(&mut phone);
println!("In-place normalized: {}", phone);
```

### Phone Number Formatting

```rust
use phonelib::{format_phone_number, PhoneFormat};

let number = "1234567890";

// E.164 format
let e164 = format_phone_number(number, PhoneFormat::E164);
// Result: Some("+1234567890")

// International format
let intl = format_phone_number(number, PhoneFormat::International);
// Result: Some("+1 234 567-890")

// National format
let national = format_phone_number(number, PhoneFormat::National);
// Result: Some("(234) 567-890")

// RFC3966 format
let rfc = format_phone_number(number, PhoneFormat::RFC3966);
// Result: Some("tel:+1-234-567-890")
```

### Phone Number Type Detection

```rust
use phonelib::{
    detect_phone_number_type, is_mobile_number, 
    is_landline_number, is_toll_free_number, PhoneNumberType
};

let mobile = "447123456789";
let landline = "442079460958";
let toll_free = "18001234567";

// Detect specific type
match detect_phone_number_type(mobile) {
    Some(PhoneNumberType::Mobile) => println!("It's a mobile number!"),
    Some(other_type) => println!("It's a {:?}", other_type),
    None => println!("Invalid or unknown type"),
}

// Quick type checks
if is_mobile_number(mobile) {
    println!("Mobile number detected");
}

if is_landline_number(landline) {
    println!("Landline number detected");
}

if is_toll_free_number(toll_free) {
    println!("Toll-free number detected");
}
```

### Random Phone Number Generation

```rust
use phonelib::{generate_random_phone_number, generate_random_phone_numbers};

// Generate a single random number
let random_us = generate_random_phone_number("US");
println!("Random US number: {:?}", random_us);

// Generate multiple random numbers
let random_numbers = generate_random_phone_numbers("GB", 5);
println!("5 random UK numbers: {:?}", random_numbers);
```

### Phone Number Comparison

```rust
use phonelib::{are_phone_numbers_equal, group_equivalent_phone_numbers};

// Compare two numbers
let num1 = "+1234567890";
let num2 = "(234) 567-890";

if are_phone_numbers_equal(num1, num2) {
    println!("Numbers are equivalent!");
}

// Group equivalent numbers
let numbers = [
    "+1234567890",
    "(234) 567-890",
    "+9876543210",
    "987-654-3210",
];

let groups = group_equivalent_phone_numbers(&numbers);
for (i, group) in groups.iter().enumerate() {
    println!("Group {}: {:?}", i + 1, group);
}
```

### Batch Processing

```rust
use phonelib::{
    validate_phone_numbers_batch, 
    normalize_phone_numbers_batch,
    detect_phone_number_types_batch,
    analyze_phone_numbers_batch
};

let numbers = [
    "1234567890",
    "invalid",
    "447123456789",
];

// Batch validation
let valid_results = validate_phone_numbers_batch(&numbers);
println!("Validation results: {:?}", valid_results);

// Batch normalization
let normalized_results = normalize_phone_numbers_batch(&numbers);
println!("Normalized results: {:?}", normalized_results);

// Batch type detection
let type_results = detect_phone_number_types_batch(&numbers);
println!("Type results: {:?}", type_results);

// Comprehensive batch analysis
let analyses = analyze_phone_numbers_batch(&numbers);
for analysis in analyses {
    println!("Original: {}", analysis.original);
    println!("Valid: {}", analysis.is_valid);
    println!("Normalized: {:?}", analysis.normalized);
    println!("Country: {:?}", analysis.country.map(|c| c.name));
    println!("Type: {:?}", analysis.phone_type);
    println!("---");
}
```

### Smart Suggestions & Intelligence

```rust
use phonelib::{
    suggest_phone_number_corrections, 
    is_potentially_valid_phone_number,
    guess_country_from_number
};

// Get suggestions for invalid numbers
let invalid_number = "123456789";
let suggestions = suggest_phone_number_corrections(invalid_number, Some("US"));
println!("Suggestions: {:?}", suggestions);

// Check if a number might be valid with different formatting
let maybe_valid = "123-456-7890";
if is_potentially_valid_phone_number(maybe_valid) {
    println!("This number might be valid with proper formatting");
}

// Guess country from number patterns
let mystery_number = "442079460958";
match guess_country_from_number(mystery_number) {
    Some(country) => println!("Likely from: {}", country.name),
    None => println!("Cannot determine country"),
}
```

### Text Extraction

Extract phone numbers from free-form text:

```rust
use phonelib::{
    extract_phone_numbers_from_text,
    extract_valid_phone_numbers_from_text,
    extract_phone_numbers_with_country_hint,
    replace_phone_numbers_in_text,
    redact_phone_numbers,
    count_phone_numbers_in_text,
};

let text = "Contact us at +12025550173 or call our UK office at +442079460958";

// Extract all phone numbers
let numbers = extract_phone_numbers_from_text(text);
for num in &numbers {
    println!("Found: {} at position {}-{}", num.raw, num.start, num.end);
    println!("  Normalized: {:?}", num.normalized);
    println!("  Valid: {}", num.is_valid);
}

// Extract only valid numbers
let valid_numbers = extract_valid_phone_numbers_from_text(text);

// Extract with country hint for national numbers
let us_text = "Call (202) 555-0173 for assistance";
let numbers = extract_phone_numbers_with_country_hint(us_text, "US");

// Count phone numbers
let count = count_phone_numbers_in_text(text);
println!("Found {} phone numbers", count);

// Replace phone numbers
let replaced = replace_phone_numbers_in_text(text, |num| {
    format!("[PHONE: {}]", num.normalized.as_deref().unwrap_or(&num.raw))
});

// Redact for privacy (show last 4 digits)
let redacted = redact_phone_numbers(text, 4);
println!("{}", redacted); // "Contact us at ********0173 or..."
```

### PhoneNumber Struct with Equality

The `PhoneNumber` struct provides type-safe phone number handling with proper equality comparison:

```rust
use phonelib::{PhoneNumber, PhoneNumberSet, PhoneFormat};
use std::collections::HashSet;

// Parse phone numbers
let num1 = PhoneNumber::parse("+12025550173").unwrap();
let num2 = PhoneNumber::parse("12025550173").unwrap();
let num3 = PhoneNumber::parse("+442079460958").unwrap();

// Equality comparison (same number, different formats)
assert_eq!(num1, num2);
assert_ne!(num1, num3);

// Use in HashSet for deduplication
let mut set = HashSet::new();
set.insert(num1.clone());
set.insert(num2.clone()); // Won't be added - duplicate
assert_eq!(set.len(), 1);

// PhoneNumber methods
println!("E.164: {}", num1.e164());
println!("National: {}", num1.national_number());
println!("Country code: {:?}", num1.country_code());
println!("Is mobile: {}", num1.is_mobile());
println!("Formatted: {}", num1.format(PhoneFormat::International));

// Parse with country hint for national numbers
let national = PhoneNumber::parse_with_country("2025550173", "US");

// PhoneNumberSet for efficient deduplication
let mut phone_set = PhoneNumberSet::new();
phone_set.add("+12025550173");
phone_set.add("12025550173");     // Duplicate - not added
phone_set.add("+442079460958");
assert_eq!(phone_set.len(), 2);

// Check membership
assert!(phone_set.contains("12025550173"));

// Create from iterator
let numbers = vec!["+12025550173", "12025550173", "+442079460958"];
let set: PhoneNumberSet = numbers.into_iter().collect();
assert_eq!(set.len(), 2);
```

## Country Support

The library supports **246 countries** with accurate:
- Country codes and prefixes
- Valid phone number lengths
- Mobile vs. landline detection patterns
- Toll-free and premium number identification

### Supported Countries Include:
- 🇺🇸 United States & Canada (NANP)
- 🇬🇧 United Kingdom
- 🇩🇪 Germany
- 🇫🇷 France
- 🇮🇳 India
- 🇦🇺 Australia
- And 240+ more countries worldwide


## Examples

### Complete Example

```rust
use phonelib::*;

fn main() {
    let numbers = [
        "1234567890",
        "+44 7123 456789",
        "(555) 123-4567",
        "invalid-number",
    ];
    
    for number in numbers {
        println!("\n--- Analyzing: {} ---", number);
        
        // Basic validation
        let is_valid = is_valid_phone_number(number);
        println!("Valid: {}", is_valid);
        
        if is_valid {
            // Format in different styles
            if let Some(e164) = format_phone_number(number, PhoneFormat::E164) {
                println!("E.164: {}", e164);
            }
            
            if let Some(intl) = format_phone_number(number, PhoneFormat::International) {
                println!("International: {}", intl);
            }
            
            // Detect country
            if let Some(country) = extract_country(number) {
                println!("Country: {} ({})", country.name, country.code);
            }
            
            // Detect type
            if let Some(phone_type) = detect_phone_number_type(number) {
                println!("Type: {:?}", phone_type);
            }
        } else {
            // Suggest corrections
            let suggestions = suggest_phone_number_corrections(number, Some("US"));
            if !suggestions.is_empty() {
                println!("Suggestions: {:?}", suggestions);
            }
        }
    }
}
```

## Contributing

Contributions to the Phonelib library are welcome! Here's how you can help:

- 🐛 **Report bugs** - Open an issue if you find any problems
- 💡 **Suggest features** - Share ideas for new functionality  
- 🔧 **Submit pull requests** - Help improve the code
- 📖 **Improve documentation** - Help make the docs better
- 🧪 **Add tests** - Increase test coverage

### Development Setup

```bash
# Clone the repository
git clone https://github.com/mohamadzoh/phonelib.git
cd phonelib

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check code formatting
cargo fmt

# Run clippy for linting
cargo clippy
```

## Changelog

### v1.0.1
**Minor Bug Fixes**
- Fixed edge case in phone number normalization for certain country codes

### v1.0.0 (Latest)

**First Stable Release**

This release marks the first stable version of phonelib, with a production-ready API.

- **Breaking Change**: All functions now accept `&str` instead of `String` for better ergonomics
  - No more `.to_string()` calls needed!
  - `is_valid_phone_number("+12025550173")` just works
- **Breaking Change**: Batch functions now accept `&[T]` where `T: AsRef<str>` instead of `Vec<String>`
  - Works with arrays, slices, or Vecs of `&str` or `String`
  - `validate_phone_numbers_batch(&["123", "456"])` just works
- Zero allocations for read-only operations
- Added support for Cymru (Wales) with country code `GB-CYM`

### v0.3.0

🏴󠁧󠁢󠁷󠁬󠁳󠁿 **Cymru Support**

- Added support for Cymru (Wales) with country code `GB-CYM`

### v0.2.1

**Text Extraction & Equality Release**

- 📝 **Text Extraction** - Extract phone numbers from free-form text
  - `extract_phone_numbers_from_text` - Find all phone numbers in text
  - `extract_valid_phone_numbers_from_text` - Find only valid numbers
  - `extract_phone_numbers_with_country_hint` - Parse with default country
  - `replace_phone_numbers_in_text` - Custom replacement function
  - `redact_phone_numbers` - Privacy-focused masking
  - `count_phone_numbers_in_text` - Quick count
- ⚖️ **PhoneNumber Struct** - Type-safe phone numbers with equality
  - Implements `Eq`, `PartialEq`, `Hash` for use in collections
  - Implements `Display`, `FromStr` for easy conversion
  - Methods: `e164()`, `national_number()`, `format()`, `is_mobile()`, etc.
- 🗂️ **PhoneNumberSet** - Efficient deduplication collection
- 📚 **Improved Documentation** - Complete rustdoc coverage
- 🔧 **Code Quality** - All clippy warnings resolved

### v0.2.0

**Major Feature Release**

- ✨ **Phone Number Formatting** - Multiple format support (E.164, International, National, RFC3966)
- 📱 **Type Detection** - Identify mobile, landline, toll-free, premium numbers
- 🎲 **Random Generation** - Generate valid random phone numbers by country
- ⚖️ **Number Comparison** - Compare numbers regardless of format
- 🚀 **Batch Processing** - Process multiple numbers efficiently
- 🔍 **Smart Suggestions** - Get correction suggestions for invalid numbers
- 📊 **Comprehensive Analysis** - Detailed phone number analysis
- 🌍 **Enhanced Country Support** - Better patterns for major countries

### v0.1.6

- 🐛 Bug fixes and performance improvements
- 📚 Documentation updates

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Rusty Rails Project

Phonelib is part of the larger **Rusty Rails** project, which aims to bridge the gap between Rust and Ruby/Ruby on Rails ecosystems. We're actively working on recreating Ruby libraries in Rust to make working with Rust more easy and fun for new developers.

### Related Projects

- 🔗 More Rust libraries coming soon!
- 🚀 Performance-focused Ruby alternatives
- 📦 Easy-to-use APIs familiar to Ruby developers

---

**Made with ❤️ by the Rusty Rails team**