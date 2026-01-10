#[cfg(test)]
mod tests {
    use crate::{
        are_phone_numbers_equal, detect_phone_number_type, extract_country, format_phone_number,
        generate_random_phone_number, is_landline_number, is_mobile_number,
        is_potentially_valid_phone_number, is_toll_free_number, is_valid_phone_number,
        normalize_phone_number, normalize_phone_number_in_place, suggest_phone_number_corrections,
        validate_phone_numbers_batch, PhoneFormat, PhoneNumberType,
    };

    struct PhoneNumber {
        country_code: &'static str,
        phone_number: &'static str,
    }

    const PHONE_NUMBERS: [PhoneNumber; 212] = [
        PhoneNumber {
            country_code: "US",
            phone_number: "+12025550173",
        },
        PhoneNumber {
            country_code: "GB",
            phone_number: "+442079460958",
        },
        PhoneNumber {
            country_code: "IN",
            phone_number: "+919876543210",
        },
        PhoneNumber {
            country_code: "DE",
            phone_number: "+493012345678",
        },
        PhoneNumber {
            country_code: "AU",
            phone_number: "+61412345678",
        },
        PhoneNumber {
            country_code: "CN",
            phone_number: "+8613800138000",
        },
        PhoneNumber {
            country_code: "FR",
            phone_number: "+33123456789",
        },
        PhoneNumber {
            country_code: "BR",
            phone_number: "+5511912345678",
        },
        PhoneNumber {
            country_code: "JP",
            phone_number: "+819012345678",
        },
        PhoneNumber {
            country_code: "ZA",
            phone_number: "+27821234567",
        },
        PhoneNumber {
            country_code: "LB",
            phone_number: "+96179123123",
        },
        PhoneNumber {
            country_code: "SY",
            phone_number: "+963944567890",
        },
        PhoneNumber {
            country_code: "IQ",
            phone_number: "+9647901234567",
        },
        PhoneNumber {
            country_code: "KW",
            phone_number: "+96550012345",
        },
        PhoneNumber {
            country_code: "SA",
            phone_number: "+966512345678",
        },
        PhoneNumber {
            country_code: "YE",
            phone_number: "+967711234567",
        },
        PhoneNumber {
            country_code: "OM",
            phone_number: "+96892123456",
        },
        PhoneNumber {
            country_code: "PS",
            phone_number: "+970599123456",
        },
        PhoneNumber {
            country_code: "AE",
            phone_number: "+971501234567",
        },
        PhoneNumber {
            country_code: "IL",
            phone_number: "+972501234567",
        },
        PhoneNumber {
            country_code: "BH",
            phone_number: "+97336012345",
        },
        PhoneNumber {
            country_code: "QA",
            phone_number: "+97433123456",
        },
        PhoneNumber {
            country_code: "BT",
            phone_number: "+97517123456",
        },
        PhoneNumber {
            country_code: "MN",
            phone_number: "+97699123456",
        },
        PhoneNumber {
            country_code: "NP",
            phone_number: "+9779841234567",
        },
        PhoneNumber {
            country_code: "IR",
            phone_number: "+989123456789",
        },
        PhoneNumber {
            country_code: "TJ",
            phone_number: "+992931234567",
        },
        PhoneNumber {
            country_code: "TM",
            phone_number: "+99365123456",
        },
        PhoneNumber {
            country_code: "AZ",
            phone_number: "+994401234567",
        },
        PhoneNumber {
            country_code: "GE",
            phone_number: "+995591234567",
        },
        PhoneNumber {
            country_code: "KG",
            phone_number: "+996551234567",
        },
        PhoneNumber {
            country_code: "UZ",
            phone_number: "+998971234567",
        },
        PhoneNumber {
            country_code: "BS",
            phone_number: "+12425571234",
        },
        PhoneNumber {
            country_code: "BB",
            phone_number: "+12462311234",
        },
        PhoneNumber {
            country_code: "AI",
            phone_number: "+12642351234",
        },
        PhoneNumber {
            country_code: "AG",
            phone_number: "+12684641234",
        },
        PhoneNumber {
            country_code: "VG",
            phone_number: "+12844681234",
        },
        PhoneNumber {
            country_code: "VI",
            phone_number: "+13406901234",
        },
        PhoneNumber {
            country_code: "KY",
            phone_number: "+13453211234",
        },
        PhoneNumber {
            country_code: "BM",
            phone_number: "+14412341234",
        },
        PhoneNumber {
            country_code: "GD",
            phone_number: "+14732341234",
        },
        PhoneNumber {
            country_code: "TC",
            phone_number: "+16492311234",
        },
        PhoneNumber {
            country_code: "MS",
            phone_number: "+16642351234",
        },
        PhoneNumber {
            country_code: "MP",
            phone_number: "+16702351234",
        },
        PhoneNumber {
            country_code: "GU",
            phone_number: "+16712351234",
        },
        PhoneNumber {
            country_code: "AS",
            phone_number: "+16842351234",
        },
        PhoneNumber {
            country_code: "SX",
            phone_number: "+17215431234",
        },
        PhoneNumber {
            country_code: "LC",
            phone_number: "+17582841234",
        },
        PhoneNumber {
            country_code: "DM",
            phone_number: "+17672351234",
        },
        PhoneNumber {
            country_code: "VC",
            phone_number: "+17842351234",
        },
        PhoneNumber {
            country_code: "PR",
            phone_number: "+17872351234",
        },
        PhoneNumber {
            country_code: "DO",
            phone_number: "+18092351234",
        },
        PhoneNumber {
            country_code: "DO",
            phone_number: "+18292351234",
        },
        PhoneNumber {
            country_code: "DO",
            phone_number: "+18492351234",
        },
        PhoneNumber {
            country_code: "TT",
            phone_number: "+18682351234",
        },
        PhoneNumber {
            country_code: "KN",
            phone_number: "+18692351234",
        },
        PhoneNumber {
            country_code: "JM",
            phone_number: "+18762351234",
        },
        PhoneNumber {
            country_code: "RE",
            phone_number: "+262692691234",
        },
        PhoneNumber {
            country_code: "ZW",
            phone_number: "+263772112345",
        },
        PhoneNumber {
            country_code: "NA",
            phone_number: "+264601234567",
        },
        PhoneNumber {
            country_code: "MW",
            phone_number: "+265991234567",
        },
        PhoneNumber {
            country_code: "LS",
            phone_number: "+26662012345",
        },
        PhoneNumber {
            country_code: "BW",
            phone_number: "+26771123456",
        },
        PhoneNumber {
            country_code: "SZ",
            phone_number: "+26876123456",
        },
        PhoneNumber {
            country_code: "KM",
            phone_number: "+2693112345",
        },
        PhoneNumber {
            country_code: "SH",
            phone_number: "+29022123",
        },
        PhoneNumber {
            country_code: "ER",
            phone_number: "+2917111234",
        },
        PhoneNumber {
            country_code: "AW",
            phone_number: "+2975601234",
        },
        PhoneNumber {
            country_code: "FO",
            phone_number: "+298201234",
        },
        PhoneNumber {
            country_code: "GL",
            phone_number: "+299201234",
        },
        PhoneNumber {
            country_code: "GI",
            phone_number: "+35056012345",
        },
        PhoneNumber {
            country_code: "PT",
            phone_number: "+351201234567",
        },
        PhoneNumber {
            country_code: "LU",
            phone_number: "+35220123456",
        },
        PhoneNumber {
            country_code: "IE",
            phone_number: "+353201234567",
        },
        PhoneNumber {
            country_code: "IS",
            phone_number: "+3544101234",
        },
        PhoneNumber {
            country_code: "AL",
            phone_number: "+355691234567",
        },
        PhoneNumber {
            country_code: "MT",
            phone_number: "+35679012345",
        },
        PhoneNumber {
            country_code: "CY",
            phone_number: "+35796123456",
        },
        PhoneNumber {
            country_code: "FI",
            phone_number: "+358201234567",
        },
        PhoneNumber {
            country_code: "BG",
            phone_number: "+35920123456",
        },
        PhoneNumber {
            country_code: "LT",
            phone_number: "+37061234567",
        },
        PhoneNumber {
            country_code: "LV",
            phone_number: "+37120123456",
        },
        PhoneNumber {
            country_code: "EE",
            phone_number: "+37251234567",
        },
        PhoneNumber {
            country_code: "MD",
            phone_number: "+37368123456",
        },
        PhoneNumber {
            country_code: "AM",
            phone_number: "+37491234567",
        },
        PhoneNumber {
            country_code: "BY",
            phone_number: "+375291234567",
        },
        PhoneNumber {
            country_code: "AD",
            phone_number: "+376312345",
        },
        PhoneNumber {
            country_code: "MC",
            phone_number: "+37761234567",
        },
        PhoneNumber {
            country_code: "SM",
            phone_number: "+378661234567",
        },
        PhoneNumber {
            country_code: "VA",
            phone_number: "+379612345678",
        },
        PhoneNumber {
            country_code: "UA",
            phone_number: "+380501234567",
        },
        PhoneNumber {
            country_code: "RS",
            phone_number: "+381601234567",
        },
        PhoneNumber {
            country_code: "ME",
            phone_number: "+38267123456",
        },
        PhoneNumber {
            country_code: "HR",
            phone_number: "+385911234567",
        },
        PhoneNumber {
            country_code: "SI",
            phone_number: "+38631234567",
        },
        PhoneNumber {
            country_code: "BA",
            phone_number: "+38761123456",
        },
        PhoneNumber {
            country_code: "MK",
            phone_number: "+38970123456",
        },
        PhoneNumber {
            country_code: "CZ",
            phone_number: "+420601123456",
        },
        PhoneNumber {
            country_code: "SK",
            phone_number: "+421912345678",
        },
        PhoneNumber {
            country_code: "LI",
            phone_number: "+4236608811",
        },
        PhoneNumber {
            country_code: "FK",
            phone_number: "+50051234",
        },
        PhoneNumber {
            country_code: "BZ",
            phone_number: "+5018221234",
        },
        PhoneNumber {
            country_code: "GT",
            phone_number: "+50251234567",
        },
        PhoneNumber {
            country_code: "SV",
            phone_number: "+50370123456",
        },
        PhoneNumber {
            country_code: "HN",
            phone_number: "+50491234567",
        },
        PhoneNumber {
            country_code: "NI",
            phone_number: "+50581234567",
        },
        PhoneNumber {
            country_code: "CR",
            phone_number: "+50670123456",
        },
        PhoneNumber {
            country_code: "PA",
            phone_number: "+50761234567",
        },
        PhoneNumber {
            country_code: "PM",
            phone_number: "+50850123456",
        },
        PhoneNumber {
            country_code: "HT",
            phone_number: "+50928123456",
        },
        PhoneNumber {
            country_code: "GP",
            phone_number: "+590590123456",
        },
        PhoneNumber {
            country_code: "BO",
            phone_number: "+59171234567",
        },
        PhoneNumber {
            country_code: "GY",
            phone_number: "+5926091234",
        },
        PhoneNumber {
            country_code: "EC",
            phone_number: "+593991234567",
        },
        PhoneNumber {
            country_code: "GF",
            phone_number: "+594694201234",
        },
        PhoneNumber {
            country_code: "PY",
            phone_number: "+595961456789",
        },
        PhoneNumber {
            country_code: "MQ",
            phone_number: "+596696201234",
        },
        PhoneNumber {
            country_code: "SR",
            phone_number: "+5977412345",
        },
        PhoneNumber {
            country_code: "UY",
            phone_number: "+59894231234",
        },
        PhoneNumber {
            country_code: "CW",
            phone_number: "+59995181234",
        },
        PhoneNumber {
            country_code: "TL",
            phone_number: "+67077231234",
        },
        PhoneNumber {
            country_code: "NF",
            phone_number: "+672312345",
        },
        PhoneNumber {
            country_code: "BN",
            phone_number: "+6737123456",
        },
        PhoneNumber {
            country_code: "NR",
            phone_number: "+6745571234",
        },
        PhoneNumber {
            country_code: "PG",
            phone_number: "+67570123456",
        },
        PhoneNumber {
            country_code: "TO",
            phone_number: "+67677151234",
        },
        PhoneNumber {
            country_code: "SB",
            phone_number: "+67762123",
        },
        PhoneNumber {
            country_code: "VU",
            phone_number: "+67824612",
        },
        PhoneNumber {
            country_code: "FJ",
            phone_number: "+6797012345",
        },
        PhoneNumber {
            country_code: "PW",
            phone_number: "+6806201234",
        },
        PhoneNumber {
            country_code: "WF",
            phone_number: "+681501234",
        },
        PhoneNumber {
            country_code: "CK",
            phone_number: "+68222123",
        },
        PhoneNumber {
            country_code: "KI",
            phone_number: "+68660123",
        },
        PhoneNumber {
            country_code: "NC",
            phone_number: "+687501234",
        },
        PhoneNumber {
            country_code: "TV",
            phone_number: "+688901234",
        },
        PhoneNumber {
            country_code: "PF",
            phone_number: "+68987123456",
        },
        PhoneNumber {
            country_code: "TK",
            phone_number: "+6903012",
        },
        PhoneNumber {
            country_code: "FM",
            phone_number: "+6913501234",
        },
        PhoneNumber {
            country_code: "MH",
            phone_number: "+6922471234",
        },
        PhoneNumber {
            country_code: "KP",
            phone_number: "+8501912345678",
        },
        PhoneNumber {
            country_code: "HK",
            phone_number: "+85251234567",
        },
        PhoneNumber {
            country_code: "MO",
            phone_number: "+85366123456",
        },
        PhoneNumber {
            country_code: "KH",
            phone_number: "+85512345678",
        },
        PhoneNumber {
            country_code: "LA",
            phone_number: "+85620911234",
        },
        PhoneNumber {
            country_code: "BD",
            phone_number: "+8801812345678",
        },
        PhoneNumber {
            country_code: "TW",
            phone_number: "+886912345678",
        },
        PhoneNumber {
            country_code: "MV",
            phone_number: "+9607712345",
        },
        PhoneNumber {
            country_code: "JO",
            phone_number: "+962791234567",
        },
        PhoneNumber {
            country_code: "SG",
            phone_number: "+6581234567",
        }, // Singapore
        PhoneNumber {
            country_code: "MY",
            phone_number: "+60121234567",
        }, // Malaysia
        PhoneNumber {
            country_code: "PH",
            phone_number: "+639171234567",
        }, // Philippines
        PhoneNumber {
            country_code: "TH",
            phone_number: "+66812345678",
        }, // Thailand
        PhoneNumber {
            country_code: "VN",
            phone_number: "+84912345678",
        }, // Vietnam
        PhoneNumber {
            country_code: "PK",
            phone_number: "+923001234567",
        }, // Pakistan
        PhoneNumber {
            country_code: "LK",
            phone_number: "+94771234567",
        }, // Sri Lanka
        PhoneNumber {
            country_code: "AF",
            phone_number: "+93700123456",
        }, // Afghanistan
        PhoneNumber {
            country_code: "MM",
            phone_number: "+959123456789",
        }, // Myanmar
        PhoneNumber {
            country_code: "KZ",
            phone_number: "+77012345678",
        }, // Kazakhstan
        PhoneNumber {
            country_code: "UZ",
            phone_number: "+998901234567",
        }, // Uzbekistan
        PhoneNumber {
            country_code: "TJ",
            phone_number: "+992551234567",
        }, // Tajikistan
        PhoneNumber {
            country_code: "KG",
            phone_number: "+996701234567",
        }, // Kyrgyzstan
        PhoneNumber {
            country_code: "TM",
            phone_number: "+993651234567",
        }, // Turkmenistan
        PhoneNumber {
            country_code: "MN",
            phone_number: "+97688123456",
        }, // Mongolia
        PhoneNumber {
            country_code: "NP",
            phone_number: "+9779812345678",
        }, // Nepal
        PhoneNumber {
            country_code: "BT",
            phone_number: "+97517123456",
        }, // Bhutan
        PhoneNumber {
            country_code: "AM",
            phone_number: "+37494123456",
        }, // Armenia
        PhoneNumber {
            country_code: "AZ",
            phone_number: "+994501234567",
        }, // Azerbaijan
        PhoneNumber {
            country_code: "GE",
            phone_number: "+995555123456",
        }, // Georgia
        PhoneNumber {
            country_code: "TM",
            phone_number: "+99312123456",
        }, // Turkmenistan
        PhoneNumber {
            country_code: "AE",
            phone_number: "+971501234567",
        }, // UAE
        PhoneNumber {
            country_code: "BH",
            phone_number: "+97333123456",
        }, // Bahrain
        PhoneNumber {
            country_code: "QA",
            phone_number: "+97450123456",
        }, // Qatar
        PhoneNumber {
            country_code: "KW",
            phone_number: "+96550123456",
        }, // Kuwait
        PhoneNumber {
            country_code: "OM",
            phone_number: "+96892123456",
        }, // Oman
        PhoneNumber {
            country_code: "YE",
            phone_number: "+967711234567",
        }, // Yemen
        PhoneNumber {
            country_code: "SO",
            phone_number: "+252615123456",
        }, // Somalia
        PhoneNumber {
            country_code: "KE",
            phone_number: "+254701234567",
        }, // Kenya
        PhoneNumber {
            country_code: "TZ",
            phone_number: "+255621234567",
        }, // Tanzania
        PhoneNumber {
            country_code: "UG",
            phone_number: "+256701234567",
        }, // Uganda
        PhoneNumber {
            country_code: "ET",
            phone_number: "+251911234567",
        }, // Ethiopia
        PhoneNumber {
            country_code: "NG",
            phone_number: "+234701234567",
        }, // Nigeria
        PhoneNumber {
            country_code: "GH",
            phone_number: "+233501234567",
        }, // Ghana
        PhoneNumber {
            country_code: "SN",
            phone_number: "+221771234567",
        }, // Senegal
        PhoneNumber {
            country_code: "CI",
            phone_number: "+22551234567",
        }, // Ivory Coast
        PhoneNumber {
            country_code: "ML",
            phone_number: "+22365123456",
        }, // Mali
        PhoneNumber {
            country_code: "ZM",
            phone_number: "+260961234567",
        }, // Zambia
        PhoneNumber {
            country_code: "ZW",
            phone_number: "+263771234567",
        }, // Zimbabwe
        PhoneNumber {
            country_code: "BW",
            phone_number: "+26772123456",
        }, // Botswana
        PhoneNumber {
            country_code: "NA",
            phone_number: "+264811234567",
        }, // Namibia
        PhoneNumber {
            country_code: "MG",
            phone_number: "+261341234567",
        }, // Madagascar
        PhoneNumber {
            country_code: "RE",
            phone_number: "+262692123456",
        }, // Reunion
        PhoneNumber {
            country_code: "MU",
            phone_number: "+23057123456",
        }, // Mauritius
        PhoneNumber {
            country_code: "SC",
            phone_number: "+2482512345",
        }, // Seychelles
        PhoneNumber {
            country_code: "MW",
            phone_number: "+265991234567",
        }, // Malawi
        PhoneNumber {
            country_code: "LS",
            phone_number: "+26650123456",
        }, // Lesotho
        PhoneNumber {
            country_code: "SZ",
            phone_number: "+26876123456",
        }, // Eswatini
        PhoneNumber {
            country_code: "CV",
            phone_number: "+2389912345",
        }, // Cape Verde
        PhoneNumber {
            country_code: "ST",
            phone_number: "+2399912345",
        }, // Sao Tome and Principe
        PhoneNumber {
            country_code: "GQ",
            phone_number: "+240222123456",
        }, // Equatorial Guinea
        PhoneNumber {
            country_code: "CD",
            phone_number: "+243991234567",
        }, // Democratic Republic of the Congo
        PhoneNumber {
            country_code: "AO",
            phone_number: "+244921234567",
        }, // Angola
        PhoneNumber {
            country_code: "GW",
            phone_number: "+245501234511",
        }, // Guinea-Bissau
        PhoneNumber {
            country_code: "IO",
            phone_number: "+2463801234",
        }, // British Indian Ocean Territory
        PhoneNumber {
            country_code: "AC",
            phone_number: "+2473612",
        }, // Ascension Island
        PhoneNumber {
            country_code: "SC",
            phone_number: "+2482512345",
        }, // Seychelles
        PhoneNumber {
            country_code: "SD",
            phone_number: "+249911231234",
        }, // Sudan
        PhoneNumber {
            country_code: "RW",
            phone_number: "+250720123456",
        }, // Rwanda
        PhoneNumber {
            country_code: "SO",
            phone_number: "+252615123456",
        }, // Somalia
        PhoneNumber {
            country_code: "DJ",
            phone_number: "+25377123123",
        }, // Djibouti
        PhoneNumber {
            country_code: "BI",
            phone_number: "+25779123456",
        }, // Burundi
        PhoneNumber {
            country_code: "MZ",
            phone_number: "+258821234567",
        }, // Mozambiquez
        PhoneNumber {
            country_code: "GB-CYM",
            phone_number: "+442079460958",
        }, // Great Britain - Cymru
    ];

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
    fn test_normalize_phone_number_in_place() {
        // Valid phone number
        assert_eq!(
            normalize_phone_number_in_place(&mut "+12345678912".to_string()),
            Some("+12345678912".to_string())
        );
        // Invalid characters
        assert_eq!(
            normalize_phone_number_in_place(&mut "invalid_phone_number".to_string()),
            None
        );
        // // Valid phone number with leading zeros
        assert_eq!(
            normalize_phone_number_in_place(&mut "+0012345678912".to_string()),
            Some("+12345678912".to_string())
        );
        assert_eq!(
            normalize_phone_number_in_place(&mut "+96109123123".to_string()),
            Some("+9619123123".to_string())
        );
        // // Valid phone number with country code and leading zeros
        assert_eq!(
            normalize_phone_number_in_place(&mut "+0012345678901".to_string()),
            Some("+12345678901".to_string())
        );
        // // Valid phone number with parentheses and spaces
        assert_eq!(
            normalize_phone_number_in_place(&mut "+1 (234) 567-8910".to_string()),
            Some("+12345678910".to_string())
        );
    }

    #[test]
    fn test_extract_country() {
        // Valid country code
        assert_eq!(
            extract_country("+11231231232".to_string())
                .unwrap()
                .code
                .to_string(),
            "US".to_string()
        );
        // Invalid country code
        assert_eq!(extract_country("+987654321".to_string()), None);
    }

    #[test]
    fn test_normalize_phone_number() {
        for phone_number in PHONE_NUMBERS.iter() {
            let normalized_phone_number =
                normalize_phone_number(phone_number.phone_number.to_string());
            assert_eq!(
                normalized_phone_number,
                Some(phone_number.phone_number.to_string())
            );
            // Use country_code to avoid dead_code warning
            assert!(!phone_number.country_code.is_empty());
        }

        assert_eq!(
            normalize_phone_number("invalid_phone_number".to_string()),
            None
        );
    }

    #[test]
    fn test_cases() {
        let test_cases = vec![
            (String::from("+61485906541"), true),
            (String::from("+4306935893571"), true),
            (String::from("+32468799972"), true),
            (String::from("+5561981737725"), true),
            (String::from("+44 7406514755"), true),
            (String::from("+54 9119298464"), true),
            (String::from("+61 4129228042"), true),
            (String::from("+43 6642428349"), true),
            (String::from("+32 4706460538"), true),
            (String::from("+420 601139706"), true),
        ];

        for (phone, valid) in test_cases {
            let is_valid = is_valid_phone_number(phone.clone());
            assert_eq!(is_valid, valid);
        }
    }

    #[test]
    fn test_phone_number_formatting() {
        let number = "+12345678901".to_string();

        // Test E.164 format
        let e164 = format_phone_number(number.clone(), PhoneFormat::E164);
        assert_eq!(e164, Some("+12345678901".to_string()));

        // Test International format
        let intl = format_phone_number(number.clone(), PhoneFormat::International);
        assert!(intl.is_some());

        // Test National format
        let national = format_phone_number(number.clone(), PhoneFormat::National);
        assert!(national.is_some());

        // Test RFC3966 format
        let rfc = format_phone_number(number.clone(), PhoneFormat::RFC3966);
        assert!(rfc.is_some());
    }

    #[test]
    fn test_phone_number_type_detection() {
        // Test US toll-free number
        let toll_free = is_toll_free_number("18001234567".to_string());
        assert!(toll_free || !is_valid_phone_number("18001234567".to_string()));

        // Test mobile detection function
        let mobile_result = is_mobile_number("447123456789".to_string());
        assert!(mobile_result || !is_valid_phone_number("447123456789".to_string()));

        // Test landline detection function
        let landline_result = is_landline_number("12025551234".to_string());
        assert!(landline_result || !is_valid_phone_number("12025551234".to_string()));
    }

    #[test]
    fn test_random_phone_number_generation() {
        let random_us = generate_random_phone_number("US");
        if let Some(number) = random_us {
            assert!(is_valid_phone_number(number));
        }

        let random_gb = generate_random_phone_number("GB");
        if let Some(number) = random_gb {
            assert!(is_valid_phone_number(number));
        }

        // Test invalid country code
        let invalid = generate_random_phone_number("XX");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_phone_number_equality() {
        let num1 = "+12345678901".to_string();
        let num2 = "12345678901".to_string();
        let num3 = "+12345678902".to_string();

        assert!(are_phone_numbers_equal(num1.clone(), num2));
        assert!(!are_phone_numbers_equal(num1, num3));
    }

    #[test]
    fn test_batch_processing() {
        let numbers = vec![
            "+12345678901".to_string(),
            "invalid".to_string(),
            "+442079460958".to_string(),
        ];

        let results = validate_phone_numbers_batch(numbers.clone());
        assert_eq!(results.len(), 3);

        // First and third should be valid, second should be invalid
        assert!(results[0]);
        assert!(!results[1]);
        assert!(results[2]);
    }

    #[test]
    fn test_phone_number_suggestions() {
        let suggestions = suggest_phone_number_corrections("123456789".to_string(), Some("US"));
        assert!(!suggestions.is_empty());

        // Test potentially valid check
        let potentially_valid = is_potentially_valid_phone_number("123-456-7890".to_string());
        assert!(potentially_valid);

        let not_valid = is_potentially_valid_phone_number("123".to_string());
        assert!(!not_valid);
    }

    #[test]
    fn test_type_detection_specific_cases() {
        // Test with known patterns
        let phone_type = detect_phone_number_type("447123456789".to_string());
        // Should be Some(Mobile) or None if invalid
        assert!(phone_type.is_some() || !is_valid_phone_number("447123456789".to_string()));

        if let Some(ptype) = phone_type {
            assert!(ptype == PhoneNumberType::Mobile || ptype == PhoneNumberType::Unknown);
        }
    }

    // ========================================================================
    // Text Parsing Tests
    // ========================================================================

    #[test]
    fn test_extract_phone_numbers_from_text() {
        use crate::extract_phone_numbers_from_text;

        let text = "Call me at +12025550173 or +442079460958 for support.";
        let numbers = extract_phone_numbers_from_text(text);

        assert_eq!(numbers.len(), 2);
        assert!(numbers[0].raw.contains("12025550173"));
        assert!(numbers[1].raw.contains("442079460958"));
    }

    #[test]
    fn test_extract_phone_with_various_formats() {
        use crate::extract_phone_numbers_from_text;

        let text = "Numbers: (202) 555-0173, 202.555.0174, 202-555-0175";
        let numbers = extract_phone_numbers_from_text(text);

        // Should find multiple number patterns
        assert!(!numbers.is_empty());
    }

    #[test]
    fn test_extract_valid_phone_numbers_only() {
        use crate::extract_valid_phone_numbers_from_text;

        let text = "Valid: +12025550173, Invalid: 123";
        let numbers = extract_valid_phone_numbers_from_text(text);

        // Should only return valid numbers
        for num in &numbers {
            assert!(num.is_valid);
        }
    }

    #[test]
    fn test_extract_with_country_hint() {
        use crate::extract_phone_numbers_with_country_hint;

        let text = "Call (202) 555-0173";
        let numbers = extract_phone_numbers_with_country_hint(text, "US");

        assert!(!numbers.is_empty());
        // With US hint, national number should be recognized
    }

    #[test]
    fn test_count_phone_numbers() {
        use crate::count_phone_numbers_in_text;

        let text = "Contact: +12025550173, +442079460958";
        let count = count_phone_numbers_in_text(text);

        assert_eq!(count, 2);
    }

    #[test]
    fn test_replace_phone_numbers() {
        use crate::replace_phone_numbers_in_text;

        let text = "Call +12025550173 today!";
        let replaced = replace_phone_numbers_in_text(text, |_| "[PHONE]".to_string());

        assert!(replaced.contains("[PHONE]"));
        assert!(!replaced.contains("12025550173"));
    }

    #[test]
    fn test_redact_phone_numbers() {
        use crate::redact_phone_numbers;

        let text = "Call +12025550173";
        let redacted = redact_phone_numbers(text, 4);

        // Should have stars and visible digits
        assert!(redacted.contains("*") || redacted.contains("[PHONE]"));
    }

    #[test]
    fn test_extract_positions() {
        use crate::extract_phone_numbers_from_text;

        let text = "Phone: +12025550173";
        let numbers = extract_phone_numbers_from_text(text);

        if !numbers.is_empty() {
            let num = &numbers[0];
            // Verify positions make sense
            assert!(num.start < num.end);
            assert!(num.end <= text.len());
        }
    }

    // ========================================================================
    // PhoneNumber Struct and Equality Tests
    // ========================================================================

    #[test]
    fn test_phone_number_struct_parse() {
        use crate::PhoneNumber as PhoneNum;

        let phone = PhoneNum::parse("+12025550173");
        assert!(phone.is_some());

        let phone = phone.unwrap();
        assert_eq!(phone.e164(), "+12025550173");
    }

    #[test]
    fn test_phone_number_equality_trait() {
        use crate::PhoneNumber as PhoneNum;

        let num1 = PhoneNum::parse("+12025550173").unwrap();
        let num2 = PhoneNum::parse("12025550173").unwrap();
        let num3 = PhoneNum::parse("+442079460958").unwrap();

        // Same number, different formats should be equal
        assert_eq!(num1, num2);
        // Different numbers should not be equal
        assert_ne!(num1, num3);
    }

    #[test]
    fn test_phone_number_hash() {
        use crate::PhoneNumber as PhoneNum;
        use std::collections::HashSet;

        let mut set = HashSet::new();

        let num1 = PhoneNum::parse("+12025550173").unwrap();
        let num2 = PhoneNum::parse("12025550173").unwrap();

        set.insert(num1);
        set.insert(num2); // Should not add duplicate

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_phone_number_with_country_hint() {
        use crate::PhoneNumber as PhoneNum;

        // National number with country hint
        let phone = PhoneNum::parse_with_country("2025550173", "US");

        if let Some(p) = phone {
            assert!(p.e164().starts_with("+1"));
        }
    }

    #[test]
    fn test_phone_number_national_number() {
        use crate::PhoneNumber as PhoneNum;

        let phone = PhoneNum::parse("+12025550173").unwrap();
        let national = phone.national_number();

        // National number should not include country code
        assert!(!national.starts_with("+"));
        assert!(!national.starts_with("1") || national.len() < 11);
    }

    #[test]
    fn test_phone_number_format_method() {
        use crate::PhoneFormat;
        use crate::PhoneNumber as PhoneNum;

        let phone = PhoneNum::parse("+12025550173").unwrap();

        let e164 = phone.format(PhoneFormat::E164);
        assert!(e164.starts_with("+"));
    }

    #[test]
    fn test_phone_number_display() {
        use crate::PhoneNumber as PhoneNum;

        let phone = PhoneNum::parse("+12025550173").unwrap();
        let display = format!("{}", phone);

        assert_eq!(display, "+12025550173");
    }

    #[test]
    fn test_phone_number_from_str() {
        use crate::PhoneNumber as PhoneNum;

        let phone: Result<PhoneNum, _> = "+12025550173".parse();
        assert!(phone.is_ok());

        let invalid: Result<PhoneNum, _> = "invalid".parse();
        assert!(invalid.is_err());
    }

    // ========================================================================
    // PhoneNumberSet Tests
    // ========================================================================

    #[test]
    fn test_phone_number_set_basic() {
        use crate::PhoneNumberSet;

        let mut set = PhoneNumberSet::new();

        assert!(set.add("+12025550173"));
        assert!(!set.add("12025550173")); // Duplicate

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_phone_number_set_contains() {
        use crate::PhoneNumberSet;

        let mut set = PhoneNumberSet::new();
        set.add("+12025550173");

        assert!(set.contains("+12025550173"));
        assert!(set.contains("12025550173")); // Same number, different format
        assert!(!set.contains("+442079460958"));
    }

    #[test]
    fn test_phone_number_set_remove() {
        use crate::PhoneNumberSet;

        let mut set = PhoneNumberSet::new();
        set.add("+12025550173");

        assert!(set.remove("12025550173")); // Remove using different format
        assert!(set.is_empty());
    }

    #[test]
    fn test_phone_number_set_from_iterator() {
        use crate::PhoneNumberSet;

        let numbers = vec!["+12025550173", "12025550173", "+442079460958"];
        let set: PhoneNumberSet = numbers.into_iter().collect();

        assert_eq!(set.len(), 2); // First two are duplicates
    }

    #[test]
    fn test_phone_number_set_iter() {
        use crate::PhoneNumberSet;

        let mut set = PhoneNumberSet::new();
        set.add("+12025550173");
        set.add("+442079460958");

        let count = set.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_phone_number_type_checks() {
        use crate::PhoneNumber as PhoneNum;

        // These tests depend on the type detection logic
        let phone = PhoneNum::parse("+18005551234"); // US toll-free pattern
        if let Some(p) = phone {
            // Just verify the methods work
            let _ = p.is_mobile();
            let _ = p.is_landline();
            let _ = p.is_toll_free();
        }
    }
}
