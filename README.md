# Phonelib

Phonelib is a comprehensive Rust library for handling phone numbers. It provides functions for validation, formatting, type detection, batch processing, and much more.

## Features

- ✅ **Phone Number Validation** - Check if phone numbers are valid
- 🌍 **Country Detection** - Extract country information from phone numbers  
- 🔧 **Normalization** - Clean and standardize phone number formats
- 🎨 **Multiple Format Support** - E.164, International, National, RFC3966
- 📱 **Type Detection** - Identify mobile, landline, toll-free, premium numbers
- 🎲 **Random Number Generation** - Generate valid random phone numbers
- ⚖️ **Number Comparison** - Compare numbers regardless of format
- 🚀 **Batch Processing** - Process multiple numbers efficiently
- 🔍 **Smart Suggestions** - Get correction suggestions for invalid numbers

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
phonelib = "0.2.0"
```

## Quick Start

```rust
use phonelib::*;

// Basic validation
let is_valid = is_valid_phone_number("+1234567890".to_string());

// Format a number
let formatted = format_phone_number(
    "1234567890".to_string(), 
    PhoneFormat::International
);

// Detect number type
let number_type = detect_phone_number_type("+447123456789".to_string());
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

let phone_number = "+1234567890".to_string();
if is_valid_phone_number(phone_number) {
    println!("Valid phone number!");
}
```

#### Country Extraction

```rust
use phonelib::extract_country;

let phone_number = "+1234567890".to_string();
match extract_country(phone_number) {
    Some(country) => println!("Country: {} ({})", country.name, country.code),
    None => println!("Country not found"),
}
```

#### Phone Number Normalization

```rust
use phonelib::{normalize_phone_number, normalize_phone_number_in_place};

// Returns normalized number without modifying input
let normalized = normalize_phone_number("+1 (234) 567-890".to_string());
println!("Normalized: {:?}", normalized); // Some("+1234567890")

// Modifies the input string in place
let mut phone = "+1 (234) 567-890".to_string();
normalize_phone_number_in_place(&mut phone);
println!("In-place normalized: {}", phone);
```

### Phone Number Formatting

```rust
use phonelib::{format_phone_number, PhoneFormat};

let number = "1234567890".to_string();

// E.164 format
let e164 = format_phone_number(number.clone(), PhoneFormat::E164);
// Result: Some("+1234567890")

// International format
let intl = format_phone_number(number.clone(), PhoneFormat::International);
// Result: Some("+1 234 567-890")

// National format
let national = format_phone_number(number.clone(), PhoneFormat::National);
// Result: Some("(234) 567-890")

// RFC3966 format
let rfc = format_phone_number(number.clone(), PhoneFormat::RFC3966);
// Result: Some("tel:+1-234-567-890")
```

### Phone Number Type Detection

```rust
use phonelib::{
    detect_phone_number_type, is_mobile_number, 
    is_landline_number, is_toll_free_number, PhoneNumberType
};

let mobile = "447123456789".to_string();
let landline = "442079460958".to_string();
let toll_free = "18001234567".to_string();

// Detect specific type
match detect_phone_number_type(mobile.clone()) {
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
let num1 = "+1234567890".to_string();
let num2 = "(234) 567-890".to_string();

if are_phone_numbers_equal(num1, num2) {
    println!("Numbers are equivalent!");
}

// Group equivalent numbers
let numbers = vec![
    "+1234567890".to_string(),
    "(234) 567-890".to_string(),
    "+9876543210".to_string(),
    "987-654-3210".to_string(),
];

let groups = group_equivalent_phone_numbers(numbers);
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

let numbers = vec![
    "1234567890".to_string(),
    "invalid".to_string(),
    "447123456789".to_string(),
];

// Batch validation
let valid_results = validate_phone_numbers_batch(numbers.clone());
println!("Validation results: {:?}", valid_results);

// Batch normalization
let normalized_results = normalize_phone_numbers_batch(numbers.clone());
println!("Normalized results: {:?}", normalized_results);

// Batch type detection
let type_results = detect_phone_number_types_batch(numbers.clone());
println!("Type results: {:?}", type_results);

// Comprehensive batch analysis
let analyses = analyze_phone_numbers_batch(numbers);
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
let invalid_number = "123456789".to_string();
let suggestions = suggest_phone_number_corrections(invalid_number, Some("US"));
println!("Suggestions: {:?}", suggestions);

// Check if a number might be valid with different formatting
let maybe_valid = "123-456-7890".to_string();
if is_potentially_valid_phone_number(maybe_valid) {
    println!("This number might be valid with proper formatting");
}

// Guess country from number patterns
let mystery_number = "447123456789".to_string();
match guess_country_from_number(mystery_number) {
    Some(country) => println!("Likely from: {}", country.name),
    None => println!("Cannot determine country"),
}
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

## Performance

The library is optimized for performance:
- ✅ Zero external dependencies
- ✅ Efficient string processing
- ✅ Batch processing capabilities
- ✅ In-place operations available

Benchmark results show excellent performance for validation and normalization operations.

## Examples

### Complete Example

```rust
use phonelib::*;

fn main() {
    let numbers = vec![
        "1234567890",
        "+44 7123 456789",
        "(555) 123-4567",
        "invalid-number",
    ];
    
    for number in numbers {
        let number = number.to_string();
        println!("\n--- Analyzing: {} ---", number);
        
        // Basic validation
        let is_valid = is_valid_phone_number(number.clone());
        println!("Valid: {}", is_valid);
        
        if is_valid {
            // Format in different styles
            if let Some(e164) = format_phone_number(number.clone(), PhoneFormat::E164) {
                println!("E.164: {}", e164);
            }
            
            if let Some(intl) = format_phone_number(number.clone(), PhoneFormat::International) {
                println!("International: {}", intl);
            }
            
            // Detect country
            if let Some(country) = extract_country(number.clone()) {
                println!("Country: {} ({})", country.name, country.code);
            }
            
            // Detect type
            if let Some(phone_type) = detect_phone_number_type(number.clone()) {
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

### v0.2.0 (Latest)

🎉 **Major Feature Release**

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
