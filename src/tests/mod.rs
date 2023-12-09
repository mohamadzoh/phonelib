#[cfg(test)]
mod tests {
    use crate::{constants::COUNTRIES, {is_valid_phone_number, extract_country, phone_number_cleaner, cleaned_phone_number}};

    #[test]
    fn test_is_valid_phone_number() {
        // Valid phone number
        assert!(is_valid_phone_number("+96179123123".to_string()));
        assert!(!is_valid_phone_number("invalid_phone_number".to_string()));
        // Valid phone number with parentheses
        assert!(is_valid_phone_number("+1 (234) 567-8990".to_string()));
        assert!(!is_valid_phone_number("+1 (234) 567-890".to_string()));
    }

    #[test]
    fn test_cleaned_phone_number() {
        // Valid phone number
        assert_eq!(cleaned_phone_number("+96179123123".to_string()), Some("+96179123123".to_string()));
        // // Invalid characters
        assert_eq!(cleaned_phone_number("invalid_phone_number".to_string()), None);
        // // Valid phone number with leading zeros
        assert_eq!(cleaned_phone_number("+096179123123".to_string()), Some("+96179123123".to_string()));
    }

    #[test]
    fn test_phone_number_cleaner() {
        // Valid phone number
        assert_eq!(phone_number_cleaner(&mut "+12345678912".to_string()), Some("+12345678912".to_string()));
        // Invalid characters
        assert_eq!(phone_number_cleaner(&mut "invalid_phone_number".to_string()), None);
        // // Valid phone number with leading zeros
        assert_eq!(phone_number_cleaner(&mut "+0012345678912".to_string()), Some("+12345678912".to_string()));
        assert_eq!(phone_number_cleaner(&mut "+96109123123".to_string()), Some("+9619123123".to_string()));
        // // Valid phone number with country code and leading zeros
        assert_eq!(phone_number_cleaner(&mut "+0012345678901".to_string()), Some("+12345678901".to_string()));
        // // Valid phone number with parentheses and spaces
        assert_eq!(phone_number_cleaner(&mut "+1 (234) 567-8910".to_string()), Some("+12345678910".to_string()));
    }

    #[test]
    fn test_extract_country() {
        // Valid country code
        assert_eq!(extract_country("+11231231232".to_string()).unwrap().code.to_string(), "US".to_string());
        // Invalid country code
        assert_eq!(extract_country("+987654321".to_string()), None);
    }
}
