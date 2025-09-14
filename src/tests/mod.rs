#[cfg(test)]
mod tests {
    use crate::{
        extract_country, is_valid_phone_number, normalize_phone_number,
        normalize_phone_number_in_place,
    };

    struct PhoneNumber {
        country_code: &'static str,
        phone_number: &'static str,
    }

    const PHONE_NUMBERS: [PhoneNumber; 211] = [
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
        }, // Mozambique
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
}
