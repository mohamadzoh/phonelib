use constants::COUNTRIES;
use definitions::Country;

pub use definitions::PhoneNumberType;

mod constants;
mod definitions;
mod tests;

pub fn is_valid_phone_number(phone_number: String) -> bool {
    // check if the phone number contains invalid character
    if contains_invalid_character(&phone_number) {
        return false;
    }

    // normalize the phone number and check if it is valid or not
    normalize_phone_number(phone_number).is_some()
}

pub fn extract_country(phone_number: String) -> Option<&'static Country> {
    let mut phone_number = phone_number;
    remove_unwanted_character(&mut phone_number);
    extract_country_data(&phone_number)
}

pub fn normalize_phone_number(mut phone_number: String) -> Option<String> {
    // normalize the phone number in place to avoid cloning
    normalize_phone_number_in_place(&mut phone_number)
}

pub fn normalize_phone_number_in_place(phone_number: &mut String) -> Option<String> {
    remove_unwanted_character(phone_number);

    // extract country data
    let country = extract_country_data(&phone_number)?;

    // Remove country code from phone number
    let prefix_digits = count_digits(country.prefix);
    phone_number.drain(0..prefix_digits);

    // Remove all leading zeros if present
    leading_zero_remover(phone_number);

    // Add country code again to the phone number and return it
    // Use capacity hint and push_str for better performance
    let mut normalized = String::with_capacity(phone_number.len() + prefix_digits + 1);
    normalized.push('+');
    normalized.push_str(&country.prefix.to_string());
    normalized.push_str(phone_number);

    Some(normalized)
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
/// let formatted = format_phone_number("12345678901".to_string(), PhoneFormat::E164);
/// // Returns Some("+12345678901") if valid
/// ```
pub fn format_phone_number(phone_number: String, format: PhoneFormat) -> Option<String> {
    let normalized = normalize_phone_number(phone_number)?;
    let country = extract_country(normalized.clone())?;
    
    // Remove the '+' from normalized number for processing
    let digits = &normalized[1..];
    let country_code = &normalized[1..1 + count_digits(country.prefix)];
    let national_number = &digits[count_digits(country.prefix)..];
    
    match format {
        PhoneFormat::E164 => Some(normalized),
        PhoneFormat::International => {
            Some(format!("+{} {}", country_code, format_national_number(national_number, country)))
        },
        PhoneFormat::National => {
            Some(format_national_number(national_number, country))
        },
        PhoneFormat::RFC3966 => {
            Some(format!("tel:+{}-{}", country_code, national_number.chars().collect::<Vec<_>>().chunks(3).map(|chunk| chunk.iter().collect::<String>()).collect::<Vec<_>>().join("-")))
        }
    }
}

fn format_national_number(number: &str, country: &Country) -> String {
    // Simple formatting based on common patterns
    match country.code {
        "US" | "CA" => {
            if number.len() == 10 {
                format!("({}) {}-{}", &number[0..3], &number[3..6], &number[6..])
            } else {
                number.to_string()
            }
        },
        "GB" => {
            if number.len() >= 10 {
                format!("{} {} {}", &number[0..4], &number[4..7], &number[7..])
            } else {
                number.to_string()
            }
        },
        "DE" => {
            if number.len() >= 10 {
                format!("{} {}", &number[0..3], &number[3..])
            } else {
                number.to_string()
            }
        },
        _ => {
            // Generic formatting for other countries
            if number.len() >= 7 {
                let mid = number.len() / 2;
                format!("{} {}", &number[0..mid], &number[mid..])
            } else {
                number.to_string()
            }
        }
    }
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
/// let number_type = detect_phone_number_type("12345678901".to_string());
/// // Returns Some(PhoneNumberType) if valid
/// ```
pub fn detect_phone_number_type(phone_number: String) -> Option<PhoneNumberType> {
    let normalized = normalize_phone_number(phone_number)?;
    let country = extract_country(normalized.clone())?;
    
    // Remove the '+' and country code to get national number
    let digits = &normalized[1..];
    let national_number = &digits[count_digits(country.prefix)..];
    
    Some(classify_phone_number_type(national_number, country))
}

/// Check if a phone number is a mobile number
/// 
/// # Arguments
/// * `phone_number` - The phone number to check
/// 
/// # Returns
/// * `true` - If the number is a mobile number
/// * `false` - If the number is not mobile or invalid
pub fn is_mobile_number(phone_number: String) -> bool {
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
pub fn is_landline_number(phone_number: String) -> bool {
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
pub fn is_toll_free_number(phone_number: String) -> bool {
    detect_phone_number_type(phone_number) == Some(PhoneNumberType::TollFree)
}

fn classify_phone_number_type(national_number: &str, country: &Country) -> PhoneNumberType {
    if national_number.is_empty() {
        return PhoneNumberType::Unknown;
    }
    
    let first_digit = national_number.chars().next().unwrap();
    let first_two = if national_number.len() >= 2 {
        &national_number[0..2]
    } else {
        national_number
    };
    let first_three = if national_number.len() >= 3 {
        &national_number[0..3]
    } else {
        national_number
    };
    
    match country.code {
        "US" | "CA" => {
            // North American Numbering Plan
            match first_three {
                "800" | "833" | "844" | "855" | "866" | "877" | "888" => PhoneNumberType::TollFree,
                "900" | "976" => PhoneNumberType::PremiumRate,
                _ => {
                    // In NANP, mobile and landline numbers use the same format
                    // This is a simplified classification
                    if national_number.len() == 10 {
                        PhoneNumberType::FixedLine // Default to fixed line for NANP
                    } else {
                        PhoneNumberType::Unknown
                    }
                }
            }
        },
        "GB" => {
            match first_two {
                "07" => PhoneNumberType::Mobile,
                "08" => match first_three {
                    "080" | "084" | "087" => PhoneNumberType::TollFree,
                    "081" | "082" | "089" => PhoneNumberType::PremiumRate,
                    _ => PhoneNumberType::SharedCost,
                },
                "01" | "02" => PhoneNumberType::FixedLine,
                "03" => PhoneNumberType::Uan,
                "05" => PhoneNumberType::Voip,
                _ => PhoneNumberType::Unknown,
            }
        },
        "DE" => {
            match first_digit {
                '1' => match first_two {
                    "15" | "16" | "17" => PhoneNumberType::Mobile,
                    "18" => PhoneNumberType::SharedCost,
                    "19" => PhoneNumberType::PremiumRate,
                    _ => PhoneNumberType::Unknown,
                },
                '0' => PhoneNumberType::TollFree,
                _ => PhoneNumberType::FixedLine,
            }
        },
        "FR" => {
            match first_digit {
                '6' | '7' => PhoneNumberType::Mobile,
                '8' => PhoneNumberType::TollFree,
                '1' | '2' | '3' | '4' | '5' | '9' => PhoneNumberType::FixedLine,
                _ => PhoneNumberType::Unknown,
            }
        },
        "AU" => {
            match first_digit {
                '4' => PhoneNumberType::Mobile,
                '1' => match first_three {
                    "180" | "188" => PhoneNumberType::TollFree,
                    "190" => PhoneNumberType::PremiumRate,
                    _ => PhoneNumberType::Unknown,
                },
                '2' | '3' | '7' | '8' => PhoneNumberType::FixedLine,
                _ => PhoneNumberType::Unknown,
            }
        },
        "IN" => {
            match first_digit {
                '9' | '8' | '7' | '6' => PhoneNumberType::Mobile,
                '1' | '2' | '3' | '4' | '5' => PhoneNumberType::FixedLine,
                _ => PhoneNumberType::Unknown,
            }
        },
        _ => {
            // Generic classification for other countries
            // This is a very basic heuristic
            match first_digit {
                '9' | '8' | '7' | '6' => PhoneNumberType::Mobile,
                '1' | '2' | '3' | '4' | '5' => PhoneNumberType::FixedLine,
                '0' => PhoneNumberType::TollFree,
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
    
    // Generate random digits
    let mut national_number = String::with_capacity(length);
    
    // Use a simple pseudo-random generator based on system time
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    
    // Generate digits for the national number
    for _ in 0..length {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let digit = (seed / 65536) % 10;
        national_number.push_str(&digit.to_string());
    }
    
    // Ensure first digit is not 0 for most countries
    if national_number.starts_with('0') && country_code != "GB" {
        national_number = format!("1{}", &national_number[1..]);
    }
    
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
    let mut numbers = Vec::with_capacity(count);
    
    for i in 0..count {
        // Add some variation to the seed for each number
        if let Some(number) = generate_random_phone_number(country_code) {
            // Add slight variation based on index to ensure different numbers
            let mut seed = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64)
                .wrapping_add(i as u64 * 1000);
            
            // Modify last few digits to ensure uniqueness
            let number_chars: Vec<char> = number.chars().collect();
            let mut modified = String::with_capacity(number.len());
            
            for (idx, &ch) in number_chars.iter().enumerate() {
                if idx >= number_chars.len() - 3 && ch.is_ascii_digit() {
                    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    let new_digit = (seed / 65536) % 10;
                    modified.push_str(&new_digit.to_string());
                } else {
                    modified.push(ch);
                }
            }
            
            numbers.push(modified);
        }
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
/// let equal = are_phone_numbers_equal("+1234567890".to_string(), "(234) 567-890".to_string());
/// // Returns true if both represent the same number
/// ```
pub fn are_phone_numbers_equal(number1: String, number2: String) -> bool {
    match (normalize_phone_number(number1), normalize_phone_number(number2)) {
        (Some(norm1), Some(norm2)) => norm1 == norm2,
        _ => false,
    }
}

/// Compare multiple phone numbers and group them by equivalence
/// 
/// # Arguments
/// * `phone_numbers` - Vector of phone numbers to compare
/// 
/// # Returns
/// * `Vec<Vec<String>>` - Groups of equivalent phone numbers
/// 
/// # Examples
/// ```
/// use phonelib::group_equivalent_phone_numbers;
/// 
/// let numbers = vec!["+1234567890".to_string(), "(234) 567-890".to_string(), "+9876543210".to_string()];
/// let groups = group_equivalent_phone_numbers(numbers);
/// // Returns groups of equivalent numbers
/// ```
pub fn group_equivalent_phone_numbers(phone_numbers: Vec<String>) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    
    for number in phone_numbers {
        let mut found_group = false;
        
        // Try to find an existing group for this number
        for group in &mut groups {
            if let Some(representative) = group.first() {
                if are_phone_numbers_equal(number.clone(), representative.clone()) {
                    group.push(number.clone());
                    found_group = true;
                    break;
                }
            }
        }
        
        // If no group found, create a new one
        if !found_group {
            groups.push(vec![number]);
        }
    }
    
    groups
}

/// Validate multiple phone numbers at once
/// 
/// # Arguments
/// * `phone_numbers` - Vector of phone numbers to validate
/// 
/// # Returns
/// * `Vec<bool>` - Vector of validation results in the same order
/// 
/// # Examples
/// ```
/// use phonelib::validate_phone_numbers_batch;
/// 
/// let numbers = vec!["1234567890".to_string(), "invalid".to_string()];
/// let results = validate_phone_numbers_batch(numbers);
/// // Returns [true, false]
/// ```
pub fn validate_phone_numbers_batch(phone_numbers: Vec<String>) -> Vec<bool> {
    phone_numbers
        .into_iter()
        .map(is_valid_phone_number)
        .collect()
}

/// Normalize multiple phone numbers at once
/// 
/// # Arguments
/// * `phone_numbers` - Vector of phone numbers to normalize
/// 
/// # Returns
/// * `Vec<Option<String>>` - Vector of normalized numbers (None for invalid ones)
/// 
/// # Examples
/// ```
/// use phonelib::normalize_phone_numbers_batch;
/// 
/// let numbers = vec!["1234567890".to_string(), "(234) 567-890".to_string()];
/// let normalized = normalize_phone_numbers_batch(numbers);
/// // Returns [Some("+1234567890"), Some("+1234567890")]
/// ```
pub fn normalize_phone_numbers_batch(phone_numbers: Vec<String>) -> Vec<Option<String>> {
    phone_numbers
        .into_iter()
        .map(normalize_phone_number)
        .collect()
}

/// Extract countries for multiple phone numbers at once
/// 
/// # Arguments
/// * `phone_numbers` - Vector of phone numbers to analyze
/// 
/// # Returns
/// * `Vec<Option<&'static Country>>` - Vector of country data (None for invalid ones)
/// 
/// # Examples
/// ```
/// use phonelib::extract_countries_batch;
/// 
/// let numbers = vec!["1234567890".to_string(), "44123456789".to_string()];
/// let countries = extract_countries_batch(numbers);
/// // Returns country data for each number
/// ```
pub fn extract_countries_batch(phone_numbers: Vec<String>) -> Vec<Option<&'static Country>> {
    phone_numbers
        .into_iter()
        .map(extract_country)
        .collect()
}

/// Detect phone number types for multiple numbers at once
/// 
/// # Arguments
/// * `phone_numbers` - Vector of phone numbers to analyze
/// 
/// # Returns
/// * `Vec<Option<PhoneNumberType>>` - Vector of phone number types
/// 
/// # Examples
/// ```
/// use phonelib::detect_phone_number_types_batch;
/// 
/// let numbers = vec!["1234567890".to_string(), "447123456789".to_string()];
/// let types = detect_phone_number_types_batch(numbers);
/// // Returns phone number types for each number
/// ```
pub fn detect_phone_number_types_batch(phone_numbers: Vec<String>) -> Vec<Option<PhoneNumberType>> {
    phone_numbers
        .into_iter()
        .map(detect_phone_number_type)
        .collect()
}

/// Comprehensive batch analysis of phone numbers
/// 
/// # Arguments
/// * `phone_numbers` - Vector of phone numbers to analyze
/// 
/// # Returns
/// * `Vec<PhoneNumberAnalysis>` - Detailed analysis for each number
/// 
/// # Examples
/// ```
/// use phonelib::analyze_phone_numbers_batch;
/// 
/// let numbers = vec!["1234567890".to_string()];
/// let analyses = analyze_phone_numbers_batch(numbers);
/// ```
pub fn analyze_phone_numbers_batch(phone_numbers: Vec<String>) -> Vec<PhoneNumberAnalysis> {
    phone_numbers
        .into_iter()
        .map(|number| {
            let is_valid = is_valid_phone_number(number.clone());
            let normalized = normalize_phone_number(number.clone());
            let country = extract_country(number.clone());
            let phone_type = detect_phone_number_type(number.clone());
            
            PhoneNumberAnalysis {
                original: number,
                is_valid,
                normalized,
                country,
                phone_type,
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
/// let suggestions = suggest_phone_number_corrections("123456789".to_string(), Some("US"));
/// // Returns possible corrections like "+1123456789"
/// ```
pub fn suggest_phone_number_corrections(phone_number: String, country_hint: Option<&str>) -> Vec<String> {
    if is_valid_phone_number(phone_number.clone()) {
        return vec![phone_number]; // Already valid
    }
    
    let mut suggestions = Vec::new();
    let mut cleaned = phone_number.clone();
    remove_non_digit_character(&mut cleaned);
    
    // Try adding country codes
    if let Some(hint) = country_hint {
        if let Some(country) = COUNTRIES.iter().find(|c| c.code == hint) {
            let suggestion = format!("+{}{}", country.prefix, cleaned);
            if is_valid_phone_number(suggestion.clone()) {
                suggestions.push(suggestion);
            }
        }
    } else {
        // Try common country codes
        let common_countries = ["US", "GB", "DE", "FR", "IN", "AU", "CA"];
        for &country_code in &common_countries {
            if let Some(country) = COUNTRIES.iter().find(|c| c.code == country_code) {
                let suggestion = format!("+{}{}", country.prefix, cleaned);
                if is_valid_phone_number(suggestion.clone()) {
                    suggestions.push(suggestion);
                }
            }
        }
    }
    
    // Try removing leading digits if number is too long
    if cleaned.len() > 15 {
        for i in 1..=(cleaned.len() - 7) {
            let shortened = cleaned[i..].to_string();
            if let Some(hint) = country_hint {
                if let Some(country) = COUNTRIES.iter().find(|c| c.code == hint) {
                    let suggestion = format!("+{}{}", country.prefix, shortened);
                    if is_valid_phone_number(suggestion.clone()) {
                        suggestions.push(suggestion);
                        break;
                    }
                }
            }
        }
    }
    
    // Try adding leading digits if number is too short
    if cleaned.len() < 10 {
        for prefix in &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"] {
            let extended = format!("{}{}", prefix, cleaned);
            if let Some(hint) = country_hint {
                if let Some(country) = COUNTRIES.iter().find(|c| c.code == hint) {
                    let suggestion = format!("+{}{}", country.prefix, extended);
                    if is_valid_phone_number(suggestion.clone()) {
                        suggestions.push(suggestion);
                    }
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
/// let might_be_valid = is_potentially_valid_phone_number("123-456-7890".to_string());
/// ```
pub fn is_potentially_valid_phone_number(phone_number: String) -> bool {
    let mut cleaned = phone_number;
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
/// let country = guess_country_from_number("1234567890".to_string());
/// ```
pub fn guess_country_from_number(phone_number: String) -> Option<&'static Country> {
    let mut cleaned = phone_number;
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


fn contains_invalid_character(phone_number: &String) -> bool {
    let mut parentheses_count = 0;
    // check if the phone number contains invalid character
    // use as_bytes() for better performance when checking ASCII characters

    for &byte in phone_number.as_bytes() {
        match byte {
            b'0'..=b'9' | b'-' | b' ' => {}
            b'(' => parentheses_count += 1,
            b')' if parentheses_count == 0 => return false,
            b')' => parentheses_count -= 1,
            _ => return false,
        }
    }

    parentheses_count == 0
}


fn remove_non_digit_character(phone_number: &mut String) {
    // remove all non digit character - use is_ascii_digit for better performance
    phone_number.retain(|c| c.is_ascii_digit());
}

fn leading_zero_remover(phone_number: &mut String) {
    // remove all leading zeros - more efficient approach
    let first_non_zero = phone_number.find(|c: char| c != '0').unwrap_or(phone_number.len());
    if first_non_zero > 0 {
        phone_number.drain(0..first_non_zero);
    }
}

fn extract_country_data(phone_number: &str) -> Option<&'static Country> {
    // check if the phone number starts with country code or not and return country data if found
    // Avoid string allocation by comparing digits directly
    for country in COUNTRIES.iter() {
        let prefix_digits = count_digits(country.prefix);
        if phone_number.len() >= prefix_digits {
            // Parse the beginning digits of phone_number and compare with prefix
            if let Ok(parsed_prefix) = phone_number[0..prefix_digits].parse::<u32>() {
                if parsed_prefix == country.prefix {
                    let remaining_len = phone_number.len() - prefix_digits;
                    if country.phone_lengths.contains(&(remaining_len as u8)) {
                        return Some(country);
                    }
                }
            }
        }
    }
    None
}

fn count_digits(mut n: u32) -> usize {
    if n == 0 { return 1; }
    let mut count = 0;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}
