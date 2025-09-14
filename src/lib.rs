use constants::COUNTRIES;
use definitions::Country;

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
