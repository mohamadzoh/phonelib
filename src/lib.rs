//! # Phonelib
//!
//! A comprehensive Rust library for handling phone numbers.
//!
//! ## Features
//!
//! - **Validation** - Check if phone numbers are valid
//! - **Country Detection** - Extract country information from phone numbers
//! - **Normalization** - Clean and standardize phone number formats
//! - **Multiple Format Support** - E.164, International, National, RFC3966
//! - **Type Detection** - Identify mobile, landline, toll-free, premium numbers
//! - **Text Extraction** - Parse phone numbers from free-form text
//! - **Comparison/Equality** - Compare numbers regardless of format
//! - **Batch Processing** - Process multiple numbers efficiently
//!
//! ## Quick Start
//!
//! ```rust
//! use phonelib::*;
//!
//! // Basic validation
//! let is_valid = is_valid_phone_number("+12025550173");
//! assert!(is_valid);
//!
//! // Normalize a number
//! let normalized = normalize_phone_number("12025550173");
//! assert_eq!(normalized, Some("+12025550173".to_string()));
//!
//! // Extract phone numbers from text
//! let text = "Call me at +12025550173 or +442079460958";
//! let numbers = extract_phone_numbers_from_text(text);
//! assert_eq!(numbers.len(), 2);
//!
//! // Compare phone numbers (different formats, same number)
//! let num1 = PhoneNumber::parse("+12025550173").unwrap();
//! let num2 = PhoneNumber::parse("12025550173").unwrap();
//! assert_eq!(num1, num2);
//! ```

use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};

use constants::COUNTRIES;
use definitions::Country;

pub use definitions::PhoneNumberType;

mod constants;
mod definitions;
mod tests;

struct CountryEntry {
    country: &'static Country,
    length_mask: u32,
}

fn prefix_table() -> &'static [Vec<CountryEntry>] {
    static TABLE: OnceLock<Vec<Vec<CountryEntry>>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table: Vec<Vec<CountryEntry>> = (0..1000).map(|_| Vec::new()).collect();
        for country in COUNTRIES.iter() {
            // Only index prefixes up to 3 digits; extract_country_data
            // only tries 1/2/3-digit prefix lookups.
            if country.prefix >= 1000 { continue; }
            let mut mask = 0u32;
            for &len in country.phone_lengths {
                mask |= 1u32 << (len as u32);
            }
            table[country.prefix as usize].push(CountryEntry {
                country,
                length_mask: mask,
            });
        }
        table
    })
}

fn random_seed() -> u64 {
    static SEED_COUNTER: AtomicU64 = AtomicU64::new(0);

    let time_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    let counter = SEED_COUNTER.fetch_add(1, Ordering::Relaxed);

    time_seed ^ counter.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn next_random_digit(seed: &mut u64) -> u8 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    ((*seed / 65536) % 10) as u8
}

fn generate_random_national_number(length: usize, allow_leading_zero: bool, seed: &mut u64) -> String {
    let mut national_number = String::with_capacity(length);

    for index in 0..length {
        let mut digit = next_random_digit(seed);
        if index == 0 && digit == 0 && !allow_leading_zero {
            digit = 1;
        }

        national_number.push(char::from(b'0' + digit));
    }

    national_number
}

/// Validates whether a phone number is valid.
///
/// This function checks if the provided phone number:
/// - Contains only valid characters (digits, spaces, dashes, parentheses)
/// - Can be normalized to a valid E.164 format
/// - Matches a known country's phone number pattern
///
/// # Arguments
/// * `phone_number` - The phone number string to validate
///
/// # Returns
/// * `true` if the phone number is valid
/// * `false` if the phone number is invalid
///
/// # Examples
/// ```
/// use phonelib::is_valid_phone_number;
///
/// assert!(is_valid_phone_number("+12025550173"));
/// assert!(!is_valid_phone_number("invalid"));
/// ```
pub fn is_valid_phone_number(phone_number: &str) -> bool {
    // check if the phone number contains invalid character
    if contains_invalid_character(phone_number) {
        return false;
    }

    // normalize the phone number and check if it is valid or not
    normalize_phone_number(phone_number).is_some()
}

/// Extracts country information from a phone number.
///
/// # Arguments
/// * `phone_number` - The phone number to analyze
///
/// # Returns
/// * `Some(Country)` - The country data if the phone number matches a known country
/// * `None` - If no country could be determined
///
/// # Examples
/// ```
/// use phonelib::extract_country;
///
/// let country = extract_country("+12025550173");
/// assert!(country.is_some());
/// assert_eq!(country.unwrap().code, "US");
/// ```
pub fn extract_country(phone_number: &str) -> Option<&'static Country> {
    normalize_and_extract(phone_number).map(|(_, country, _)| country)
}

/// Normalizes a phone number to E.164 format.
///
/// Takes a phone number in various formats and returns it in the
/// standard E.164 format (+[country code][national number]).
///
/// # Arguments
/// * `phone_number` - The phone number to normalize
///
/// # Returns
/// * `Some(String)` - The normalized phone number in E.164 format
/// * `None` - If the phone number is invalid
///
/// # Examples
/// ```
/// use phonelib::normalize_phone_number;
///
/// let normalized = normalize_phone_number("12025550173");
/// assert_eq!(normalized, Some("+12025550173".to_string()));
/// ```
pub fn normalize_phone_number(phone_number: &str) -> Option<String> {
    normalize_and_extract(phone_number).map(|(normalized, _, _)| normalized)
}

/// Normalizes a phone number in place to E.164 format.
///
/// Similar to [`normalize_phone_number`] but modifies the input string
/// in place for better performance when you already own the string.
///
/// # Arguments
/// * `phone_number` - Mutable reference to the phone number to normalize
///
/// # Returns
/// * `Some(String)` - The normalized phone number in E.164 format
/// * `None` - If the phone number is invalid
///
/// # Examples
/// ```
/// use phonelib::normalize_phone_number_in_place;
///
/// let mut phone = "12025550173".to_string();
/// let normalized = normalize_phone_number_in_place(&mut phone);
/// assert_eq!(normalized, Some("+12025550173".to_string()));
/// ```
pub fn normalize_phone_number_in_place(phone_number: &mut String) -> Option<String> {
    let stripped_len = strip_extension(phone_number).len();
    phone_number.truncate(stripped_len);

    if phone_number.bytes().any(|byte| byte.is_ascii_alphabetic()) {
        let converted = convert_vanity_letters(phone_number);
        phone_number.clear();
        phone_number.push_str(&converted);
    }

    remove_unwanted_character(phone_number);

    // extract country data
    let country = extract_country_data(phone_number)?;

    // Remove country code from phone number
    let plen = prefix_digit_count(country.prefix);
    phone_number.drain(0..plen);

    // Remove all leading zeros if present
    leading_zero_remover(phone_number);

    // Build E.164 string without intermediate allocations
    let mut normalized = String::with_capacity(phone_number.len() + plen + 1);
    normalized.push('+');
    push_prefix_digits(&mut normalized, country.prefix);
    normalized.push_str(phone_number);

    Some(normalized)
}

/// Internal: normalize input and extract all metadata in a single pass.
/// Avoids redundant normalization and country lookups.
fn normalize_and_extract(input: &str) -> Option<(String, &'static Country, PhoneNumberType)> {
    let stripped = strip_extension(input);
    let processed_storage;
    let processed_input = if stripped.bytes().any(|byte| byte.is_ascii_alphabetic()) {
        processed_storage = convert_vanity_letters(stripped);
        processed_storage.as_str()
    } else {
        stripped
    };

    // Extract digits into stack buffer (avoids heap allocation for cleaning)
    let mut digits = [0u8; 20];
    let mut digit_count = 0;
    let input_bytes = processed_input.as_bytes();
    // Skip leading '+' if present
    let scan_start = if input_bytes.first() == Some(&b'+') { 1 } else { 0 };
    for &b in &input_bytes[scan_start..] {
        if b.is_ascii_digit() {
            if digit_count >= 20 { return None; }
            digits[digit_count] = b;
            digit_count += 1;
        }
    }

    // Skip leading zeros
    let mut start = 0;
    while start < digit_count && digits[start] == b'0' {
        start += 1;
    }
    if start >= digit_count { return None; }

    // SAFETY: buffer contains only ASCII digit bytes (0x30..=0x39), which are valid UTF-8
    let cleaned = unsafe { std::str::from_utf8_unchecked(&digits[start..digit_count]) };

    let country = extract_country_data(cleaned)?;

    let plen = prefix_digit_count(country.prefix);
    let national_bytes = &digits[start + plen..digit_count];

    // Skip leading zeros in national part (trunk prefix)
    let mut nat_start = 0;
    while nat_start < national_bytes.len() && national_bytes[nat_start] == b'0' {
        nat_start += 1;
    }

    // SAFETY: sub-slice of ASCII digit buffer
    let national = unsafe { std::str::from_utf8_unchecked(&national_bytes[nat_start..]) };

    // Classify type from national number
    let phone_type = classify_phone_number_type(national, country);

    // Build E.164 string
    let mut normalized = String::with_capacity(national.len() + plen + 1);
    normalized.push('+');
    push_prefix_digits(&mut normalized, country.prefix);
    normalized.push_str(national);

    Some((normalized, country, phone_type))
}

/// Phone number format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhoneFormat {
    /// E.164 format: +1234567890
    E164,
    /// International format: +1 234 567-890
    International,
    /// National format: (234) 567-890
    National,
    /// RFC3966 format: tel:+1-234-567-890
    RFC3966,
}

/// Format a phone number according to the specified format
/// 
/// # Arguments
/// * `phone_number` - The phone number to format
/// * `format` - The desired format
/// 
/// # Returns
/// * `Some(String)` - The formatted phone number if valid
/// * `None` - If the phone number is invalid
/// 
/// # Examples
/// ```
/// use phonelib::{format_phone_number, PhoneFormat};
/// 
/// let formatted = format_phone_number("12345678901", PhoneFormat::E164);
/// // Returns Some("+12345678901") if valid
/// ```
pub fn format_phone_number(phone_number: &str, format: PhoneFormat) -> Option<String> {
    let (normalized, country, _) = normalize_and_extract(phone_number)?;

    match format {
        PhoneFormat::E164 => Some(normalized),
        _ => {
            let plen = prefix_digit_count(country.prefix);
            let national_number = &normalized[1 + plen..];

            match format {
                PhoneFormat::International => {
                    let formatted = format_national_number(national_number, country);
                    let mut result = String::with_capacity(2 + plen + formatted.len());
                    result.push('+');
                    push_prefix_digits(&mut result, country.prefix);
                    result.push(' ');
                    result.push_str(&formatted);
                    Some(result)
                },
                PhoneFormat::National => {
                    Some(format_national_number(national_number, country))
                },
                PhoneFormat::RFC3966 => {
                    let mut result = String::with_capacity(5 + normalized.len() + 4);
                    result.push_str("tel:+");
                    push_prefix_digits(&mut result, country.prefix);
                    result.push('-');
                    for (i, b) in national_number.bytes().enumerate() {
                        if i > 0 && i % 3 == 0 {
                            result.push('-');
                        }
                        result.push(b as char);
                    }
                    Some(result)
                },
                _ => unreachable!(),
            }
        }
    }
}

fn format_national_number(number: &str, country: &Country) -> String {
    match country.code {
        "US" | "CA" if number.len() == 10 => {
            // (XXX) XXX-XXXX = 14 chars
            let mut s = String::with_capacity(14);
            s.push('(');
            s.push_str(&number[0..3]);
            s.push_str(") ");
            s.push_str(&number[3..6]);
            s.push('-');
            s.push_str(&number[6..]);
            s
        }
        _ => {
            let group_sizes = national_format_group_sizes(country.code, number.len())
                .or_else(|| default_national_group_sizes(number.len()));

            group_sizes
                .and_then(|groups| format_grouped_number(number, groups))
                .unwrap_or_else(|| number.to_string())
        }
    }
}

fn national_format_group_sizes(country_code: &str, number_len: usize) -> Option<&'static [usize]> {
    match (country_code, number_len) {
        ("GB", 10) => Some(&[4, 3, 3]),
        ("DE", 10) => Some(&[2, 4, 4]),
        ("DE", 11) => Some(&[3, 4, 4]),
        ("FR", 9) => Some(&[1, 2, 2, 2, 2]),
        ("IN", 10) => Some(&[5, 5]),
        ("AU", 9) => Some(&[1, 4, 4]),
        _ => None,
    }
}

fn default_national_group_sizes(number_len: usize) -> Option<&'static [usize]> {
    match number_len {
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

fn format_grouped_number(number: &str, groups: &[usize]) -> Option<String> {
    if groups.iter().sum::<usize>() != number.len() {
        return None;
    }

    let mut formatted = String::with_capacity(number.len() + groups.len().saturating_sub(1));
    let mut start = 0;

    for (index, &group_len) in groups.iter().enumerate() {
        if index > 0 {
            formatted.push(' ');
        }

        let end = start + group_len;
        formatted.push_str(&number[start..end]);
        start = end;
    }

    Some(formatted)
}

/// Detect the type of a phone number (mobile, landline, toll-free, etc.)
/// 
/// # Arguments
/// * `phone_number` - The phone number to analyze
/// 
/// # Returns
/// * `Some(PhoneNumberType)` - The detected phone number type if valid
/// * `None` - If the phone number is invalid
/// 
/// # Examples
/// ```
/// use phonelib::{detect_phone_number_type, PhoneNumberType};
/// 
/// let number_type = detect_phone_number_type("12345678901");
/// // Returns Some(PhoneNumberType) if valid
/// ```
pub fn detect_phone_number_type(phone_number: &str) -> Option<PhoneNumberType> {
    normalize_and_extract(phone_number).map(|(_, _, phone_type)| phone_type)
}

/// Check if a phone number is a mobile number
/// 
/// # Arguments
/// * `phone_number` - The phone number to check
/// 
/// # Returns
/// * `true` - If the number is a mobile number
/// * `false` - If the number is not mobile or invalid
pub fn is_mobile_number(phone_number: &str) -> bool {
    detect_phone_number_type(phone_number) == Some(PhoneNumberType::Mobile)
}

/// Check if a phone number is a landline number
/// 
/// # Arguments
/// * `phone_number` - The phone number to check
/// 
/// # Returns
/// * `true` - If the number is a landline number
/// * `false` - If the number is not landline or invalid
pub fn is_landline_number(phone_number: &str) -> bool {
    detect_phone_number_type(phone_number) == Some(PhoneNumberType::FixedLine)
}

/// Check if a phone number is a toll-free number
/// 
/// # Arguments
/// * `phone_number` - The phone number to check
/// 
/// # Returns
/// * `true` - If the number is toll-free
/// * `false` - If the number is not toll-free or invalid
pub fn is_toll_free_number(phone_number: &str) -> bool {
    detect_phone_number_type(phone_number) == Some(PhoneNumberType::TollFree)
}

fn classify_phone_number_type(national_number: &str, country: &Country) -> PhoneNumberType {
    let bytes = national_number.as_bytes();
    if bytes.is_empty() {
        return PhoneNumberType::Unknown;
    }

    let d0 = bytes[0];
    // Compute numeric prefix values for fast matching
    let n2: u16 = if bytes.len() >= 2 {
        (bytes[0] - b'0') as u16 * 10 + (bytes[1] - b'0') as u16
    } else {
        u16::MAX
    };
    let n3: u16 = if bytes.len() >= 3 {
        (bytes[0] - b'0') as u16 * 100 + (bytes[1] - b'0') as u16 * 10 + (bytes[2] - b'0') as u16
    } else {
        u16::MAX
    };

    match country.code {
        "US" | "CA" => {
            match n3 {
                800 | 833 | 844 | 855 | 866 | 877 | 888 => PhoneNumberType::TollFree,
                900 | 976 => PhoneNumberType::PremiumRate,
                _ => {
                    if bytes.len() == 10 {
                        PhoneNumberType::FixedLine
                    } else {
                        PhoneNumberType::Unknown
                    }
                }
            }
        },
        "GB" => {
            match d0 {
                b'7' => PhoneNumberType::Mobile,
                b'8' => match n2 {
                    80 | 84 | 87 => PhoneNumberType::TollFree,
                    81 | 82 | 89 => PhoneNumberType::PremiumRate,
                    _ => PhoneNumberType::SharedCost,
                },
                b'1' | b'2' => PhoneNumberType::FixedLine,
                b'3' => PhoneNumberType::Uan,
                b'5' => PhoneNumberType::Voip,
                _ => PhoneNumberType::Unknown,
            }
        },
        "DE" => {
            match d0 {
                b'1' => match n2 {
                    15 | 16 | 17 => PhoneNumberType::Mobile,
                    18 => PhoneNumberType::SharedCost,
                    19 => PhoneNumberType::PremiumRate,
                    _ => PhoneNumberType::Unknown,
                },
                b'0' => PhoneNumberType::TollFree,
                _ => PhoneNumberType::FixedLine,
            }
        },
        "FR" => {
            match d0 {
                b'6' | b'7' => PhoneNumberType::Mobile,
                b'8' => PhoneNumberType::TollFree,
                b'1' | b'2' | b'3' | b'4' | b'5' | b'9' => PhoneNumberType::FixedLine,
                _ => PhoneNumberType::Unknown,
            }
        },
        "AU" => {
            match d0 {
                b'4' => PhoneNumberType::Mobile,
                b'1' => match n3 {
                    180 | 188 => PhoneNumberType::TollFree,
                    190 => PhoneNumberType::PremiumRate,
                    _ => PhoneNumberType::Unknown,
                },
                b'2' | b'3' | b'7' | b'8' => PhoneNumberType::FixedLine,
                _ => PhoneNumberType::Unknown,
            }
        },
        "IN" => {
            match d0 {
                b'9' | b'8' | b'7' | b'6' => PhoneNumberType::Mobile,
                b'1' | b'2' | b'3' | b'4' | b'5' => PhoneNumberType::FixedLine,
                _ => PhoneNumberType::Unknown,
            }
        },
        _ => {
            match d0 {
                b'9' | b'8' | b'7' | b'6' => PhoneNumberType::Mobile,
                b'1' | b'2' | b'3' | b'4' | b'5' => PhoneNumberType::FixedLine,
                b'0' => PhoneNumberType::TollFree,
                _ => PhoneNumberType::Unknown,
            }
        }
    }
}

/// Generate a random valid phone number for a specific country
/// 
/// # Arguments
/// * `country_code` - The ISO 3166-1 alpha-2 country code (e.g., "US", "GB", "DE")
/// 
/// # Returns
/// * `Some(String)` - A random valid phone number for the country
/// * `None` - If the country code is not found
/// 
/// # Examples
/// ```
/// use phonelib::generate_random_phone_number;
/// 
/// let random_us_number = generate_random_phone_number("US");
/// // Returns Some("+1234567890") or similar
/// ```
pub fn generate_random_phone_number(country_code: &str) -> Option<String> {
    let country = COUNTRIES.iter().find(|c| c.code == country_code)?;

    // Use the first valid length for simplicity
    let length = *country.phone_lengths.first()? as usize;

    let mut seed = random_seed();
    let national_number = generate_random_national_number(length, country_code == "GB", &mut seed);

    // Format as E.164
    Some(format!("+{}{}", country.prefix, national_number))
}

/// Generate multiple random valid phone numbers for a specific country
/// 
/// # Arguments
/// * `country_code` - The ISO 3166-1 alpha-2 country code
/// * `count` - Number of phone numbers to generate
/// 
/// # Returns
/// * `Vec<String>` - A vector of random valid phone numbers
/// 
/// # Examples
/// ```
/// use phonelib::generate_random_phone_numbers;
/// 
/// let numbers = generate_random_phone_numbers("US", 5);
/// // Returns a vector with 5 US phone numbers
/// ```
pub fn generate_random_phone_numbers(country_code: &str, count: usize) -> Vec<String> {
    let country = match COUNTRIES.iter().find(|c| c.code == country_code) {
        Some(country) => country,
        None => return Vec::new(),
    };
    let length = match country.phone_lengths.first() {
        Some(length) => *length as usize,
        None => return Vec::new(),
    };

    let mut numbers = Vec::with_capacity(count);
    let mut seed = random_seed();

    for _ in 0..count {
        let national_number = generate_random_national_number(length, country_code == "GB", &mut seed);
        numbers.push(format!("+{}{}", country.prefix, national_number));
    }

    numbers
}

/// Check if two phone numbers are equivalent (same number, different formats)
/// 
/// # Arguments
/// * `number1` - First phone number
/// * `number2` - Second phone number
/// 
/// # Returns
/// * `true` - If the numbers represent the same phone number
/// * `false` - If the numbers are different or invalid
/// 
/// # Examples
/// ```
/// use phonelib::are_phone_numbers_equal;
/// 
/// let equal = are_phone_numbers_equal("+1234567890", "(234) 567-890");
/// // Returns true if both represent the same number
/// ```
pub fn are_phone_numbers_equal(number1: &str, number2: &str) -> bool {
    match (normalize_phone_number(number1), normalize_phone_number(number2)) {
        (Some(norm1), Some(norm2)) => norm1 == norm2,
        _ => false,
    }
}

/// Compare multiple phone numbers and group them by equivalence
/// 
/// # Arguments
/// * `phone_numbers` - Slice of phone numbers to compare
/// 
/// # Returns
/// * `Vec<Vec<String>>` - Groups of equivalent phone numbers
/// 
/// # Examples
/// ```
/// use phonelib::group_equivalent_phone_numbers;
/// 
/// let numbers = ["+1234567890", "(234) 567-890", "+9876543210"];
/// let groups = group_equivalent_phone_numbers(&numbers);
/// // Returns groups of equivalent numbers
/// ```
pub fn group_equivalent_phone_numbers<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    
    for number in phone_numbers {
        let number_str = number.as_ref();
        let mut found_group = false;
        
        // Try to find an existing group for this number
        for group in &mut groups {
            if let Some(representative) = group.first() {
                if are_phone_numbers_equal(number_str, representative) {
                    group.push(number_str.to_string());
                    found_group = true;
                    break;
                }
            }
        }
        
        // If no group found, create a new one
        if !found_group {
            groups.push(vec![number_str.to_string()]);
        }
    }
    
    groups
}

/// Validate multiple phone numbers at once
/// 
/// # Arguments
/// * `phone_numbers` - Slice of phone numbers to validate
/// 
/// # Returns
/// * `Vec<bool>` - Vector of validation results in the same order
/// 
/// # Examples
/// ```
/// use phonelib::validate_phone_numbers_batch;
/// 
/// let numbers = ["1234567890", "invalid"];
/// let results = validate_phone_numbers_batch(&numbers);
/// // Returns [true, false]
/// ```
pub fn validate_phone_numbers_batch<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<bool> {
    phone_numbers
        .iter()
        .map(|n| is_valid_phone_number(n.as_ref()))
        .collect()
}

/// Normalize multiple phone numbers at once
/// 
/// # Arguments
/// * `phone_numbers` - Slice of phone numbers to normalize
/// 
/// # Returns
/// * `Vec<Option<String>>` - Vector of normalized numbers (None for invalid ones)
/// 
/// # Examples
/// ```
/// use phonelib::normalize_phone_numbers_batch;
/// 
/// let numbers = ["1234567890", "(234) 567-890"];
/// let normalized = normalize_phone_numbers_batch(&numbers);
/// // Returns [Some("+1234567890"), Some("+1234567890")]
/// ```
pub fn normalize_phone_numbers_batch<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<Option<String>> {
    phone_numbers
        .iter()
        .map(|n| normalize_phone_number(n.as_ref()))
        .collect()
}

/// Extract countries for multiple phone numbers at once
/// 
/// # Arguments
/// * `phone_numbers` - Slice of phone numbers to analyze
/// 
/// # Returns
/// * `Vec<Option<&'static Country>>` - Vector of country data (None for invalid ones)
/// 
/// # Examples
/// ```
/// use phonelib::extract_countries_batch;
/// 
/// let numbers = ["1234567890", "44123456789"];
/// let countries = extract_countries_batch(&numbers);
/// // Returns country data for each number
/// ```
pub fn extract_countries_batch<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<Option<&'static Country>> {
    phone_numbers
        .iter()
        .map(|n| extract_country(n.as_ref()))
        .collect()
}

/// Detect phone number types for multiple numbers at once
/// 
/// # Arguments
/// * `phone_numbers` - Slice of phone numbers to analyze
/// 
/// # Returns
/// * `Vec<Option<PhoneNumberType>>` - Vector of phone number types
/// 
/// # Examples
/// ```
/// use phonelib::detect_phone_number_types_batch;
/// 
/// let numbers = ["1234567890", "447123456789"];
/// let types = detect_phone_number_types_batch(&numbers);
/// // Returns phone number types for each number
/// ```
pub fn detect_phone_number_types_batch<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<Option<PhoneNumberType>> {
    phone_numbers
        .iter()
        .map(|n| detect_phone_number_type(n.as_ref()))
        .collect()
}

/// Comprehensive batch analysis of phone numbers
/// 
/// # Arguments
/// * `phone_numbers` - Slice of phone numbers to analyze
/// 
/// # Returns
/// * `Vec<PhoneNumberAnalysis>` - Detailed analysis for each number
/// 
/// # Examples
/// ```
/// use phonelib::analyze_phone_numbers_batch;
/// 
/// let numbers = ["1234567890"];
/// let analyses = analyze_phone_numbers_batch(&numbers);
/// ```
pub fn analyze_phone_numbers_batch<T: AsRef<str>>(phone_numbers: &[T]) -> Vec<PhoneNumberAnalysis> {
    phone_numbers
        .iter()
        .map(|number| {
            let number_str = number.as_ref();
            match normalize_and_extract(number_str) {
                Some((normalized, country, phone_type)) => PhoneNumberAnalysis {
                    original: number_str.to_string(),
                    is_valid: true,
                    normalized: Some(normalized),
                    country: Some(country),
                    phone_type: Some(phone_type),
                },
                None => PhoneNumberAnalysis {
                    original: number_str.to_string(),
                    is_valid: false,
                    normalized: None,
                    country: None,
                    phone_type: None,
                },
            }
        })
        .collect()
}

/// Detailed analysis result for a phone number
#[derive(Debug, Clone)]
pub struct PhoneNumberAnalysis {
    pub original: String,
    pub is_valid: bool,
    pub normalized: Option<String>,
    pub country: Option<&'static Country>,
    pub phone_type: Option<PhoneNumberType>,
}

/// Suggest corrections for an invalid phone number
/// 
/// # Arguments
/// * `phone_number` - The invalid phone number
/// * `country_hint` - Optional country code hint for better suggestions
/// 
/// # Returns
/// * `Vec<String>` - Vector of suggested corrections
/// 
/// # Examples
/// ```
/// use phonelib::suggest_phone_number_corrections;
/// 
/// let suggestions = suggest_phone_number_corrections("123456789", Some("US"));
/// // Returns possible corrections like "+1123456789"
/// ```
pub fn suggest_phone_number_corrections(phone_number: &str, country_hint: Option<&str>) -> Vec<String> {
    if is_valid_phone_number(phone_number) {
        return vec![phone_number.to_string()]; // Already valid
    }

    let mut suggestions = Vec::new();
    let mut cleaned = phone_number.to_string();
    remove_non_digit_character(&mut cleaned);
    let hinted_country = country_hint.and_then(|hint| COUNTRIES.iter().find(|c| c.code == hint));

    // Try adding country codes
    if let Some(country) = hinted_country {
        let suggestion = format!("+{}{}", country.prefix, cleaned);
        if is_valid_phone_number(&suggestion) {
            suggestions.push(suggestion);
        }
    } else {
        // Try common country codes
        let common_countries = ["US", "GB", "DE", "FR", "IN", "AU", "CA"];
        for &country_code in &common_countries {
            if let Some(country) = COUNTRIES.iter().find(|c| c.code == country_code) {
                let suggestion = format!("+{}{}", country.prefix, cleaned);
                if is_valid_phone_number(&suggestion) {
                    suggestions.push(suggestion);
                }
            }
        }
    }

    // Try removing leading digits if number is too long
    if cleaned.len() > 15 {
        for i in 1..=(cleaned.len() - 7) {
            let shortened = cleaned[i..].to_string();
            if let Some(country) = hinted_country {
                let suggestion = format!("+{}{}", country.prefix, shortened);
                if is_valid_phone_number(&suggestion) {
                    suggestions.push(suggestion);
                    break;
                }
            }
        }
    }

    // Try adding leading digits if number is too short
    if cleaned.len() < 10 {
        for prefix in &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"] {
            let extended = format!("{}{}", prefix, cleaned);
            if let Some(country) = hinted_country {
                let suggestion = format!("+{}{}", country.prefix, extended);
                if is_valid_phone_number(&suggestion) {
                    suggestions.push(suggestion);
                }
            }
        }
    }

    // Remove duplicates and limit suggestions
    suggestions.sort();
    suggestions.dedup();
    suggestions.truncate(5); // Limit to 5 suggestions
    
    suggestions
}

/// Check if a phone number is potentially valid but incorrectly formatted
/// 
/// # Arguments
/// * `phone_number` - The phone number to check
/// 
/// # Returns
/// * `bool` - True if the number might be valid with different formatting
/// 
/// # Examples
/// ```
/// use phonelib::is_potentially_valid_phone_number;
/// 
/// let might_be_valid = is_potentially_valid_phone_number("123-456-7890");
/// ```
pub fn is_potentially_valid_phone_number(phone_number: &str) -> bool {
    let mut cleaned = phone_number.to_string();
    remove_non_digit_character(&mut cleaned);
    
    // Check if length is reasonable for a phone number
    cleaned.len() >= 7 && cleaned.len() <= 15 && !cleaned.chars().all(|c| c == '0')
}

/// Get the most likely country for a phone number based on patterns
/// 
/// # Arguments
/// * `phone_number` - The phone number to analyze
/// 
/// # Returns
/// * `Option<&'static Country>` - The most likely country, if any
/// 
/// # Examples
/// ```
/// use phonelib::guess_country_from_number;
/// 
/// let country = guess_country_from_number("1234567890");
/// ```
pub fn guess_country_from_number(phone_number: &str) -> Option<&'static Country> {
    let mut cleaned = phone_number.to_string();
    remove_non_digit_character(&mut cleaned);
    
    if cleaned.is_empty() {
        return None;
    }
    
    // Try to match based on length and common patterns
    for country in COUNTRIES.iter() {
        let prefix_len = count_digits(country.prefix);
        
        // Check if number starts with country code
        if cleaned.len() >= prefix_len {
            if let Ok(parsed_prefix) = cleaned[0..prefix_len].parse::<u32>() {
                if parsed_prefix == country.prefix {
                    let remaining_len = cleaned.len() - prefix_len;
                    if country.phone_lengths.contains(&(remaining_len as u8)) {
                        return Some(country);
                    }
                }
            }
        }
        
        // Check if number length matches country patterns (without country code)
        if country.phone_lengths.contains(&(cleaned.len() as u8)) {
            // This is a weak match, prefer exact country code matches
            continue;
        }
    }
    
    // Fallback: guess based on common patterns
    match cleaned.len() {
        10 => COUNTRIES.iter().find(|c| c.code == "US"), // Common US format
        11 if cleaned.starts_with('1') => COUNTRIES.iter().find(|c| c.code == "US"),
        11 if cleaned.starts_with("44") => COUNTRIES.iter().find(|c| c.code == "GB"),
        12 if cleaned.starts_with("49") => COUNTRIES.iter().find(|c| c.code == "DE"),
        _ => None,
    }
}

fn remove_unwanted_character(phone_number: &mut String) {
    remove_non_digit_character(phone_number);
    // Remove leading zero before country code
    leading_zero_remover(phone_number);
}


fn contains_invalid_character(phone_number: &str) -> bool {
    let mut parentheses_count = 0;
    // check if the phone number contains invalid character
    // use as_bytes() for better performance when checking ASCII characters

    for (index, &byte) in phone_number.as_bytes().iter().enumerate() {
        match byte {
            b'0'..=b'9' | b'-' | b' ' | b'.' => {}
            b'+' if index == 0 => {}
            b'A'..=b'Z' | b'a'..=b'z' => {}
            b'(' => parentheses_count += 1,
            b')' if parentheses_count == 0 => return true,
            b')' => parentheses_count -= 1,
            _ => return true,
        }
    }

    parentheses_count != 0
}


fn remove_non_digit_character(phone_number: &mut String) {
    // remove all non digit character - use is_ascii_digit for better performance
    phone_number.retain(|c| c.is_ascii_digit());
}

/// Strip extension markers (e.g., "ext. 1234", "ext 987") and everything after them.
fn strip_extension(input: &str) -> &str {
    let bytes = input.as_bytes();
    let len = bytes.len();
    for i in 0..len {
        if i + 4 <= len
            && (bytes[i] == b'e' || bytes[i] == b'E')
            && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X')
            && (bytes[i + 2] == b't' || bytes[i + 2] == b'T')
            && (bytes[i + 3] == b'.' || bytes[i + 3] == b' ')
        {
            return input[..i].trim_end();
        }
    }
    input
}

/// Convert phone keypad vanity letters to digits (e.g., "FLOWERS" → "3569377").
fn convert_vanity_letters(input: &str) -> String {
    input.chars().map(|c| {
        match c.to_ascii_uppercase() {
            'A' | 'B' | 'C' => '2',
            'D' | 'E' | 'F' => '3',
            'G' | 'H' | 'I' => '4',
            'J' | 'K' | 'L' => '5',
            'M' | 'N' | 'O' => '6',
            'P' | 'Q' | 'R' | 'S' => '7',
            'T' | 'U' | 'V' => '8',
            'W' | 'X' | 'Y' | 'Z' => '9',
            _ => c,
        }
    }).collect()
}

fn leading_zero_remover(phone_number: &mut String) {
    // remove all leading zeros - more efficient approach
    let first_non_zero = phone_number.find(|c: char| c != '0').unwrap_or(phone_number.len());
    if first_non_zero > 0 {
        phone_number.drain(0..first_non_zero);
    }
}

fn extract_country_data(phone_number: &str) -> Option<&'static Country> {
    let bytes = phone_number.as_bytes();
    let len = bytes.len();
    if len == 0 { return None; }

    let table = prefix_table();

    let d0 = (bytes[0] - b'0') as u32;
    if d0 > 9 { return None; }

    // Try 1-digit prefix
    let remaining = len - 1;
    if remaining < 32 {
        let bit = 1u32 << remaining;
        for entry in &table[d0 as usize] {
            if entry.length_mask & bit != 0 {
                return Some(entry.country);
            }
        }
    }

    if len >= 2 {
        let d1 = (bytes[1] - b'0') as u32;
        if d1 <= 9 {
            let prefix2 = d0 * 10 + d1;

            // Try 2-digit prefix
            let remaining = len - 2;
            if remaining < 32 {
                let bit = 1u32 << remaining;
                for entry in &table[prefix2 as usize] {
                    if entry.length_mask & bit != 0 {
                        return Some(entry.country);
                    }
                }
            }

            // Try 3-digit prefix
            if len >= 3 {
                let d2 = (bytes[2] - b'0') as u32;
                if d2 <= 9 {
                    let prefix3 = prefix2 * 10 + d2;
                    let remaining = len - 3;
                    if remaining < 32 {
                        let bit = 1u32 << remaining;
                        for entry in &table[prefix3 as usize] {
                            if entry.length_mask & bit != 0 {
                                return Some(entry.country);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

#[inline(always)]
const fn prefix_digit_count(prefix: u32) -> usize {
    if prefix >= 100 { 3 }
    else if prefix >= 10 { 2 }
    else { 1 }
}

/// Push a u32 country prefix as ASCII digits without allocating a String.
#[inline(always)]
fn push_prefix_digits(s: &mut String, prefix: u32) {
    if prefix >= 100 {
        s.push((b'0' + (prefix / 100) as u8) as char);
    }
    if prefix >= 10 {
        s.push((b'0' + ((prefix / 10) % 10) as u8) as char);
    }
    s.push((b'0' + (prefix % 10) as u8) as char);
}

fn count_digits(n: u32) -> usize {
    prefix_digit_count(n)
}

// ============================================================================
// Text Parsing - Extract phone numbers from free-form text
// ============================================================================

/// Result of extracting a phone number from text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedPhoneNumber {
    /// The phone number as it appeared in the text
    pub raw: String,
    /// The normalized E.164 format if valid
    pub normalized: Option<String>,
    /// Start position in the original text (byte index)
    pub start: usize,
    /// End position in the original text (byte index)
    pub end: usize,
    /// Whether the extracted number is valid
    pub is_valid: bool,
}

/// Extract all phone numbers from free-form text
/// 
/// This function scans text and extracts potential phone numbers in various formats:
/// - International format: +1 234 567 8901
/// - With parentheses: (234) 567-8901
/// - With dashes: 234-567-8901
/// - With dots: 234.567.8901
/// - Plain digits: 2345678901
/// 
/// # Arguments
/// * `text` - The text to search for phone numbers
/// 
/// # Returns
/// * `Vec<ExtractedPhoneNumber>` - All phone numbers found in the text
/// 
/// # Examples
/// ```
/// use phonelib::extract_phone_numbers_from_text;
/// 
/// let text = "Call me at +1-202-555-0173 or (415) 555-2671";
/// let numbers = extract_phone_numbers_from_text(text);
/// assert_eq!(numbers.len(), 2);
/// ```
pub fn extract_phone_numbers_from_text(text: &str) -> Vec<ExtractedPhoneNumber> {
    let mut results = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        // Look for potential phone number starts
        if is_phone_number_start(&chars, i) {
            if let Some((phone_str, start_byte, end_byte)) = extract_phone_candidate(text, &chars, i) {
                let normalized = normalize_phone_number(&phone_str);
                let is_valid = normalized.is_some();
                
                // Only include if it looks like a real phone number (7+ digits)
                let digit_count = phone_str.chars().filter(|c| c.is_ascii_digit()).count();
                if digit_count >= 7 {
                    results.push(ExtractedPhoneNumber {
                        raw: phone_str,
                        normalized,
                        start: start_byte,
                        end: end_byte,
                        is_valid,
                    });
                    
                    // Skip past this phone number
                    i = char_index_from_byte(text, end_byte);
                    continue;
                }
            }
        }
        i += 1;
    }
    
    results
}

/// Extract only valid phone numbers from text
/// 
/// Similar to `extract_phone_numbers_from_text` but only returns numbers
/// that pass validation.
/// 
/// # Arguments
/// * `text` - The text to search for phone numbers
/// 
/// # Returns
/// * `Vec<ExtractedPhoneNumber>` - Only valid phone numbers found in the text
/// 
/// # Examples
/// ```
/// use phonelib::extract_valid_phone_numbers_from_text;
/// 
/// let text = "Call +12025550173 or 123 (invalid)";
/// let numbers = extract_valid_phone_numbers_from_text(text);
/// // Returns only the valid +12025550173
/// ```
pub fn extract_valid_phone_numbers_from_text(text: &str) -> Vec<ExtractedPhoneNumber> {
    extract_phone_numbers_from_text(text)
        .into_iter()
        .filter(|n| n.is_valid)
        .collect()
}

/// Extract phone numbers from text with a country hint
/// 
/// This function attempts to parse national numbers by assuming
/// a default country when no country code is present.
/// 
/// # Arguments
/// * `text` - The text to search for phone numbers
/// * `default_country` - ISO 3166-1 alpha-2 country code to use as default
/// 
/// # Returns
/// * `Vec<ExtractedPhoneNumber>` - Phone numbers found in the text
/// 
/// # Examples
/// ```
/// use phonelib::extract_phone_numbers_with_country_hint;
/// 
/// let text = "Call (202) 555-0173";
/// let numbers = extract_phone_numbers_with_country_hint(text, "US");
/// // The number will be normalized as +12025550173
/// ```
pub fn extract_phone_numbers_with_country_hint(text: &str, default_country: &str) -> Vec<ExtractedPhoneNumber> {
    let country = COUNTRIES.iter().find(|c| c.code == default_country);
    
    let mut results = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if is_phone_number_start(&chars, i) {
            if let Some((phone_str, start_byte, end_byte)) = extract_phone_candidate(text, &chars, i) {
                let digit_count = phone_str.chars().filter(|c| c.is_ascii_digit()).count();
                
                if digit_count >= 7 {
                    let mut normalized = None;
                    
                    // If we have a country hint, try it first
                    if let Some(c) = country {
                        let mut cleaned = phone_str.clone();
                        remove_non_digit_character(&mut cleaned);
                        // Remove leading zeros (trunk prefix) from national format numbers
                        leading_zero_remover(&mut cleaned);
                        let with_country = format!("+{}{}", c.prefix, cleaned);
                        normalized = normalize_phone_number(&with_country);
                    }
                    
                    // Fallback: try to normalize as-is if country hint failed
                    if normalized.is_none() {
                        normalized = normalize_phone_number(&phone_str);
                    }
                    
                    let is_valid = normalized.is_some();
                    
                    results.push(ExtractedPhoneNumber {
                        raw: phone_str,
                        normalized,
                        start: start_byte,
                        end: end_byte,
                        is_valid,
                    });
                    
                    i = char_index_from_byte(text, end_byte);
                    continue;
                }
            }
        }
        i += 1;
    }
    
    results
}

/// Count how many phone numbers are in the text
/// 
/// # Arguments
/// * `text` - The text to search
/// 
/// # Returns
/// * `usize` - Number of phone numbers found
pub fn count_phone_numbers_in_text(text: &str) -> usize {
    extract_phone_numbers_from_text(text).len()
}

/// Replace phone numbers in text with a placeholder or transformed version
/// 
/// # Arguments
/// * `text` - The text containing phone numbers
/// * `replacement` - Function that takes an ExtractedPhoneNumber and returns the replacement string
/// 
/// # Returns
/// * `String` - The text with phone numbers replaced
/// 
/// # Examples
/// ```
/// use phonelib::replace_phone_numbers_in_text;
/// 
/// let text = "Call me at +12025550173";
/// let redacted = replace_phone_numbers_in_text(text, |_| "[REDACTED]".to_string());
/// assert_eq!(redacted, "Call me at [REDACTED]");
/// ```
pub fn replace_phone_numbers_in_text<F>(text: &str, replacement: F) -> String
where
    F: Fn(&ExtractedPhoneNumber) -> String,
{
    let numbers = extract_phone_numbers_from_text(text);
    
    if numbers.is_empty() {
        return text.to_string();
    }
    
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;
    
    for number in &numbers {
        // Add text before this phone number
        result.push_str(&text[last_end..number.start]);
        // Add the replacement
        result.push_str(&replacement(number));
        last_end = number.end;
    }
    
    // Add remaining text after last phone number
    result.push_str(&text[last_end..]);
    
    result
}

/// Redact (mask) phone numbers in text for privacy
/// 
/// # Arguments
/// * `text` - The text containing phone numbers
/// * `visible_digits` - Number of digits to keep visible at the end (0 to hide all)
/// 
/// # Returns
/// * `String` - The text with phone numbers redacted
/// 
/// # Examples
/// ```
/// use phonelib::redact_phone_numbers;
/// 
/// let text = "Call +12025550173";
/// let redacted = redact_phone_numbers(text, 4);
/// // Returns "Call ***-***-0173" or similar
/// ```
pub fn redact_phone_numbers(text: &str, visible_digits: usize) -> String {
    replace_phone_numbers_in_text(text, |number| {
        let digits: Vec<char> = number.raw.chars().filter(|c| c.is_ascii_digit()).collect();
        let total = digits.len();
        
        if visible_digits == 0 || visible_digits >= total {
            return "[PHONE]".to_string();
        }
        
        let hidden_count = total - visible_digits;
        let mut result = String::new();
        
        for _ in 0..hidden_count {
            result.push('*');
        }
        
        for &d in &digits[hidden_count..] {
            result.push(d);
        }
        
        result
    })
}

// Helper function to check if position might be start of phone number
fn is_phone_number_start(chars: &[char], pos: usize) -> bool {
    if pos >= chars.len() {
        return false;
    }
    
    let c = chars[pos];
    
    // Check for + prefix
    if c == '+' {
        return pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit();
    }
    
    // Check for opening parenthesis (area code)
    if c == '(' {
        return pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit();
    }
    
    // Check for digit that's not part of a longer number/word
    if c.is_ascii_digit() {
        // Make sure it's not in the middle of a word/number
        if pos > 0 {
            let prev = chars[pos - 1];
            if prev.is_alphanumeric() && prev != ' ' && prev != '\n' && prev != '\t' {
                return false;
            }
        }
        return true;
    }
    
    false
}

// Helper function to extract a phone number candidate starting at position
fn extract_phone_candidate(text: &str, chars: &[char], start_pos: usize) -> Option<(String, usize, usize)> {
    let mut end_pos = start_pos;
    let mut digit_count = 0;
    let mut last_digit_pos = start_pos;
    let mut paren_depth = 0;
    
    // Valid phone number characters
    while end_pos < chars.len() {
        let c = chars[end_pos];
        
        match c {
            '+' if end_pos == start_pos => {
                end_pos += 1;
            }
            '0'..='9' => {
                digit_count += 1;
                last_digit_pos = end_pos;
                end_pos += 1;
            }
            '(' => {
                paren_depth += 1;
                end_pos += 1;
            }
            ')' if paren_depth > 0 => {
                paren_depth -= 1;
                end_pos += 1;
            }
            '-' | '.' | ' ' => {
                // Only allow these if we've seen digits and more might follow
                if digit_count > 0 && end_pos + 1 < chars.len() && 
                   (chars[end_pos + 1].is_ascii_digit() || chars[end_pos + 1] == '(') {
                    end_pos += 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
        
        // Stop if we have too many digits
        if digit_count > 15 {
            break;
        }
    }
    
    if digit_count < 7 {
        return None;
    }
    
    // Convert char positions to byte positions
    let start_byte = byte_index_from_char(text, start_pos);
    let end_byte = byte_index_from_char(text, last_digit_pos + 1);
    
    let phone_str = text[start_byte..end_byte].to_string();
    
    Some((phone_str, start_byte, end_byte))
}

// Helper to convert char index to byte index
fn byte_index_from_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

// Helper to convert byte index to char index
fn char_index_from_byte(text: &str, byte_index: usize) -> usize {
    text[..byte_index].chars().count()
}

// ============================================================================
// PhoneNumber struct with equality comparison
// ============================================================================

/// A parsed and validated phone number with equality comparison
/// 
/// Two `PhoneNumber` instances are considered equal if they represent
/// the same phone number, regardless of their original formatting.
/// 
/// # Examples
/// ```
/// use phonelib::PhoneNumber;
/// 
/// let num1 = PhoneNumber::parse("+12025550173").unwrap();
/// let num2 = PhoneNumber::parse("12025550173").unwrap();
/// assert_eq!(num1, num2); // Same number, different formats
/// ```
#[derive(Debug, Clone)]
pub struct PhoneNumber {
    /// The original input string
    pub original: String,
    /// The normalized E.164 format
    pub normalized: String,
    /// The country information
    pub country: Option<&'static Country>,
    /// The phone number type
    pub phone_type: Option<PhoneNumberType>,
}

impl PhoneNumber {
    /// Parse a string into a PhoneNumber
    /// 
    /// # Arguments
    /// * `input` - The phone number string to parse
    /// 
    /// # Returns
    /// * `Some(PhoneNumber)` - If the input is a valid phone number
    /// * `None` - If the input is invalid
    pub fn parse(input: &str) -> Option<Self> {
        let (normalized, country, phone_type) = normalize_and_extract(input)?;
        Some(PhoneNumber {
            original: input.to_string(),
            normalized,
            country: Some(country),
            phone_type: Some(phone_type),
        })
    }
    
    /// Parse a phone number with a country hint for national numbers
    /// 
    /// # Arguments
    /// * `input` - The phone number string to parse
    /// * `country_code` - ISO 3166-1 alpha-2 country code
    /// 
    /// # Returns
    /// * `Some(PhoneNumber)` - If the input is a valid phone number
    /// * `None` - If the input is invalid
    pub fn parse_with_country(input: &str, country_code: &str) -> Option<Self> {
        let country = COUNTRIES.iter().find(|c| c.code == country_code);

        // Strip extension markers (e.g., "ext. 1234")
        let without_ext = strip_extension(input);

        // Convert vanity letters to digits (e.g., 1-800-FLOWERS)
        let processed = convert_vanity_letters(without_ext);

        // Try parsing as-is
        if let Some(phone) = Self::parse(&processed) {
            // If the hint country shares the same prefix as the parsed country
            // (e.g., US and CA both use prefix 1), prefer the hint
            let resolved_country = if let (Some(hint), Some(parsed)) = (country, phone.country) {
                if hint.prefix == parsed.prefix && hint.code != parsed.code {
                    Some(hint)
                } else {
                    phone.country
                }
            } else {
                phone.country
            };

            if processed.trim_start().starts_with('+') {
                return Some(PhoneNumber {
                    original: input.to_string(),
                    country: resolved_country,
                    ..phone
                });
            }
            // For national format, only accept if country matches hint
            if country.map_or(true, |hint| resolved_country.map(|c| c.code) == Some(hint.code)) {
                return Some(PhoneNumber {
                    original: input.to_string(),
                    country: resolved_country,
                    ..phone
                });
            }
        }

        let country = country?;
        let mut cleaned = processed.to_string();
        remove_non_digit_character(&mut cleaned);

        // Try stripping common IDD prefixes and parsing as international
        for idd in &["0011", "011", "00"] {
            if cleaned.starts_with(idd) && cleaned.len() > idd.len() + 5 {
                let remaining = &cleaned[idd.len()..];
                let with_plus = format!("+{}", remaining);
                if let Some(mut phone) = Self::parse(&with_plus) {
                    phone.original = input.to_string();
                    return Some(phone);
                }
            }
        }

        // Try with country code, stripping trunk prefix (leading 0)
        if cleaned.starts_with('0') {
            let without_trunk = cleaned.trim_start_matches('0');
            let with_country = format!("+{}{}", country.prefix, without_trunk);
            if let Some(mut phone) = Self::parse(&with_country) {
                phone.original = input.to_string();
                phone.country = Some(country);
                return Some(phone);
            }
        }

        // Try with country code prepended as-is
        let with_country = format!("+{}{}", country.prefix, cleaned);
        Self::parse(&with_country).map(|mut phone| {
            phone.original = input.to_string();
            phone.country = Some(country);
            phone
        })
    }
    
    /// Get the E.164 formatted number
    pub fn e164(&self) -> &str {
        &self.normalized
    }
    
    /// Get the national number (without country code)
    pub fn national_number(&self) -> String {
        if let Some(country) = self.country {
            let prefix_len = count_digits(country.prefix) + 1; // +1 for '+'
            self.normalized[prefix_len..].to_string()
        } else {
            self.normalized.clone()
        }
    }
    
    /// Get the country code digits
    pub fn country_code(&self) -> Option<u32> {
        self.country.map(|c| c.prefix)
    }
    
    /// Format the phone number
    pub fn format(&self, fmt: PhoneFormat) -> String {
        match fmt {
            PhoneFormat::E164 => self.normalized.clone(),
            _ => {
                if let Some(country) = self.country {
                    let plen = prefix_digit_count(country.prefix);
                    let national = &self.normalized[1 + plen..];
                    match fmt {
                        PhoneFormat::International => {
                            let formatted = format_national_number(national, country);
                            let mut result = String::with_capacity(2 + plen + formatted.len());
                            result.push('+');
                            push_prefix_digits(&mut result, country.prefix);
                            result.push(' ');
                            result.push_str(&formatted);
                            result
                        }
                        PhoneFormat::National => {
                            format_national_number(national, country)
                        }
                        PhoneFormat::RFC3966 => {
                            let mut result = String::with_capacity(5 + self.normalized.len() + 4);
                            result.push_str("tel:+");
                            push_prefix_digits(&mut result, country.prefix);
                            result.push('-');
                            for (i, b) in national.bytes().enumerate() {
                                if i > 0 && i % 3 == 0 {
                                    result.push('-');
                                }
                                result.push(b as char);
                            }
                            result
                        }
                        _ => unreachable!(),
                    }
                } else {
                    self.normalized.clone()
                }
            }
        }
    }
    
    /// Check if this number is mobile
    pub fn is_mobile(&self) -> bool {
        self.phone_type == Some(PhoneNumberType::Mobile)
    }
    
    /// Check if this number is a landline
    pub fn is_landline(&self) -> bool {
        self.phone_type == Some(PhoneNumberType::FixedLine)
    }
    
    /// Check if this number is toll-free
    pub fn is_toll_free(&self) -> bool {
        self.phone_type == Some(PhoneNumberType::TollFree)
    }
}

impl PartialEq for PhoneNumber {
    fn eq(&self, other: &Self) -> bool {
        self.normalized == other.normalized
    }
}

impl Eq for PhoneNumber {}

impl std::hash::Hash for PhoneNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.normalized.hash(state);
    }
}

impl std::fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.normalized)
    }
}

impl std::str::FromStr for PhoneNumber {
    type Err = &'static str;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PhoneNumber::parse(s).ok_or("Invalid phone number")
    }
}

/// A collection of phone numbers that can be compared and deduplicated
/// 
/// This struct provides efficient deduplication and comparison of phone numbers.
/// 
/// # Examples
/// ```
/// use phonelib::PhoneNumberSet;
/// 
/// let mut set = PhoneNumberSet::new();
/// set.add("+1-202-555-0173");
/// set.add("(202) 555-0173");
/// assert_eq!(set.len(), 1); // Same number, different formats
/// ```
#[derive(Debug, Clone, Default)]
pub struct PhoneNumberSet {
    numbers: std::collections::HashMap<String, PhoneNumber>,
}

impl PhoneNumberSet {
    /// Create a new empty PhoneNumberSet
    pub fn new() -> Self {
        PhoneNumberSet {
            numbers: std::collections::HashMap::new(),
        }
    }
    
    /// Add a phone number to the set
    /// 
    /// # Returns
    /// * `true` - If the number was added (not a duplicate)
    /// * `false` - If the number was already in the set
    pub fn add(&mut self, phone_number: &str) -> bool {
        if let Some(phone) = PhoneNumber::parse(phone_number) {
            if !self.numbers.contains_key(&phone.normalized) {
                self.numbers.insert(phone.normalized.clone(), phone);
                return true;
            }
        }
        false
    }
    
    /// Check if a phone number is in the set
    pub fn contains(&self, phone_number: &str) -> bool {
        if let Some(normalized) = normalize_phone_number(phone_number) {
            self.numbers.contains_key(&normalized)
        } else {
            false
        }
    }
    
    /// Get the number of unique phone numbers
    pub fn len(&self) -> usize {
        self.numbers.len()
    }
    
    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.numbers.is_empty()
    }
    
    /// Get all unique phone numbers
    pub fn iter(&self) -> impl Iterator<Item = &PhoneNumber> {
        self.numbers.values()
    }
    
    /// Get all normalized phone numbers
    pub fn normalized_numbers(&self) -> Vec<&str> {
        self.numbers.keys().map(|s| s.as_str()).collect()
    }
    
    /// Remove a phone number from the set
    pub fn remove(&mut self, phone_number: &str) -> bool {
        if let Some(normalized) = normalize_phone_number(phone_number) {
            self.numbers.remove(&normalized).is_some()
        } else {
            false
        }
    }
    
    /// Find all duplicates of a phone number (different formats)
    pub fn find_duplicates(&self, phone_number: &str) -> Option<&PhoneNumber> {
        let normalized = normalize_phone_number(phone_number)?;
        self.numbers.get(&normalized)
    }
}

impl FromIterator<String> for PhoneNumberSet {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut set = PhoneNumberSet::new();
        for number in iter {
            set.add(&number);
        }
        set
    }
}

impl<'a> FromIterator<&'a str> for PhoneNumberSet {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut set = PhoneNumberSet::new();
        for number in iter {
            set.add(number);
        }
        set
    }
}
