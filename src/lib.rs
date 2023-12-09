use constants::COUNTRIES;
use definitions::Country;

mod constants;
mod definitions;
mod tests;
pub fn is_valid_phone_number(phone_number: String) -> bool {
    if contains_invalid_character(&phone_number) {
        return false;
    }
    cleaned_phone_number(phone_number).is_some()
}

pub fn extract_country(phone_number: String) -> Option<&'static Country> {
    let mut phone_number = phone_number;
    remove_unwanted_character(&mut phone_number);
    extract_country_data(&phone_number)
}
pub fn cleaned_phone_number(phone_number: String) -> Option<String> {
    phone_number_cleaner(&mut phone_number.clone())
}

pub fn phone_number_cleaner(phone_number: &mut String) -> Option<String> {
    remove_unwanted_character(phone_number);
    println!("xxxxxx{}xxxxxxxx", phone_number);
    // Extrpub fn extract_country(phone_number: &mut String) -> Option<>act country data
    let country = extract_country_data(&phone_number)?;

    // Remove country code from phone number
    phone_number.replace_range(0..country.prefix.to_string().len(), "");

    // Remove all leading zeros if present
    leading_zero_remover(phone_number);

    // Add country code again
    let cleaned_phone_number = format!("+{}{}", country.prefix, phone_number);

    Some(cleaned_phone_number)
}

fn remove_unwanted_character(phone_number: &mut String) {
    remove_non_digit_character(phone_number);
    // Remove leading zero before country code
    leading_zero_remover(phone_number);
}


fn contains_invalid_character(phone_number: &String) -> bool {
    let mut parentheses_count = 0;

    for c in phone_number.chars() {
        match c {
            '0'..='9' | '-' | ' ' => {}
            '(' => parentheses_count += 1,
            ')' if parentheses_count == 0 => return false,
            ')' => parentheses_count -= 1,
            _ => return false,
        }
    }

    parentheses_count == 0
}


fn remove_non_digit_character(phone_number: &mut String) {
    phone_number.retain(|c| c.is_numeric());
}

fn leading_zero_remover(phone_number: &mut String) {
    while phone_number.starts_with('0') {
        phone_number.remove(0);
    }
}

fn extract_country_data(phone_number: &str) -> Option<&'static Country> {
    for country in COUNTRIES.iter() {
        if phone_number.starts_with(&country.prefix.to_string()) {
            if country
                .phone_lengths
                .contains(&(phone_number.len() as u8 - country.prefix.to_string().len() as u8))
            {
                return Some(country);
            }
        }
    }
    None
}
