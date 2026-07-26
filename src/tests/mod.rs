//! Unit tests for the public API.
//!
//! `src/lib.rs` declares this module as `#[cfg(test)] mod tests;`, so nothing
//! here is compiled into a release build or shipped to crates.io. The module
//! therefore carries no `#[cfg(test)]` wrapper of its own.
//!
//! Conventions used throughout:
//!
//! * Every assertion is unconditional. No test hides an assertion behind
//!   `if let Some(..)` or `if !x.is_empty()`, because such a test passes when
//!   the value it meant to check is absent.
//! * Tables assert a concrete expected value per row, never a constant.
//! * [`regressions`] holds one test per fixed audit finding; each names the
//!   defect it pins.

use crate::*;

// ============================================================================
// Fixtures
// ============================================================================

/// `(expected ISO code, canonical E.164 number)`.
///
/// Every row asserts `extract_country(number).code == expected` **and** that
/// the number is already canonical, so a resolution regression cannot hide
/// behind a weak "the country is non-empty" check.
///
/// Rows whose calling code is shared with another entry that a number's length
/// cannot separate live in [`SHARED_CALLING_CODE`] instead — for those there is
/// no single correct answer from `extract_country` alone.
const COUNTRY_FIXTURE: &[(&str, &str)] = &[
    ("US", "+12025550173"),
    ("GB", "+442079460958"),
    ("IN", "+919876543210"),
    ("DE", "+493012345678"),
    ("AU", "+61412345678"),
    ("CN", "+8613800138000"),
    ("FR", "+33123456789"),
    ("BR", "+5511912345678"),
    ("JP", "+819012345678"),
    ("ZA", "+27821234567"),
    ("LB", "+96179123123"),
    ("SY", "+963944567890"),
    ("IQ", "+9647901234567"),
    ("KW", "+96550012345"),
    ("SA", "+966512345678"),
    ("YE", "+967711234567"),
    ("OM", "+96892123456"),
    ("PS", "+970599123456"),
    ("AE", "+971501234567"),
    ("IL", "+972501234567"),
    ("BH", "+97336012345"),
    ("QA", "+97433123456"),
    ("BT", "+97517123456"),
    ("MN", "+97699123456"),
    ("NP", "+9779841234567"),
    ("IR", "+989123456789"),
    ("TJ", "+992931234567"),
    ("TM", "+99365123456"),
    ("AZ", "+994401234567"),
    ("GE", "+995591234567"),
    ("KG", "+996551234567"),
    ("UZ", "+998971234567"),
    ("BS", "+12425571234"),
    ("BB", "+12462311234"),
    ("AI", "+12642351234"),
    ("AG", "+12684641234"),
    ("VG", "+12844681234"),
    ("VI", "+13406901234"),
    ("KY", "+13453211234"),
    ("BM", "+14412341234"),
    ("GD", "+14732341234"),
    ("TC", "+16492311234"),
    ("MS", "+16642351234"),
    ("MP", "+16702351234"),
    ("GU", "+16712351234"),
    ("AS", "+16842351234"),
    ("SX", "+17215431234"),
    ("LC", "+17582841234"),
    ("DM", "+17672351234"),
    ("VC", "+17842351234"),
    ("PR", "+17872351234"),
    ("DO", "+18092351234"),
    ("DO", "+18292351234"),
    ("DO", "+18492351234"),
    ("TT", "+18682351234"),
    ("KN", "+18692351234"),
    ("JM", "+18762351234"),
    ("RE", "+262692691234"),
    ("ZW", "+263772112345"),
    ("NA", "+264601234567"),
    ("MW", "+265991234567"),
    ("LS", "+26662012345"),
    ("BW", "+26771123456"),
    ("SZ", "+26876123456"),
    ("KM", "+2693112345"),
    ("SH", "+29022123"),
    ("ER", "+2917111234"),
    ("AW", "+2975601234"),
    ("FO", "+298201234"),
    ("GL", "+299201234"),
    ("GI", "+35056012345"),
    ("PT", "+351201234567"),
    ("LU", "+35220123456"),
    ("IE", "+353201234567"),
    ("IS", "+3544101234"),
    ("AL", "+355691234567"),
    ("MT", "+35679012345"),
    ("CY", "+35796123456"),
    ("FI", "+358201234567"),
    ("BG", "+35920123456"),
    ("LT", "+37061234567"),
    ("LV", "+37120123456"),
    ("EE", "+37251234567"),
    ("MD", "+37368123456"),
    ("AM", "+37491234567"),
    ("BY", "+375291234567"),
    ("AD", "+376312345"),
    ("MC", "+37761234567"),
    // San Marino mobiles are 8 digits (6x xxx xxx); the fixed-line plan is
    // 0549 xxxxxx and keeps its leading zero, like Italy's.
    ("SM", "+37866123456"),
    ("UA", "+380501234567"),
    ("RS", "+381601234567"),
    ("ME", "+38267123456"),
    ("HR", "+385911234567"),
    ("SI", "+38631234567"),
    ("BA", "+38761123456"),
    ("MK", "+38970123456"),
    ("CZ", "+420601123456"),
    ("SK", "+421912345678"),
    ("LI", "+4236608811"),
    ("FK", "+50051234"),
    ("BZ", "+5018221234"),
    ("GT", "+50251234567"),
    ("SV", "+50370123456"),
    ("HN", "+50491234567"),
    ("NI", "+50581234567"),
    ("CR", "+50670123456"),
    ("PA", "+50761234567"),
    ("PM", "+508412345"),
    ("HT", "+50928123456"),
    ("GP", "+590590123456"),
    ("BO", "+59171234567"),
    ("GY", "+5926091234"),
    ("EC", "+593991234567"),
    ("GF", "+594694201234"),
    ("PY", "+595961456789"),
    ("MQ", "+596696201234"),
    ("SR", "+5977412345"),
    ("UY", "+59894231234"),
    ("CW", "+59995181234"),
    ("TL", "+67077231234"),
    ("NF", "+672312345"),
    ("BN", "+6737123456"),
    ("NR", "+6745571234"),
    ("PG", "+67570123456"),
    ("TO", "+6767715123"),
    ("SB", "+67762123"),
    ("VU", "+67824612"),
    ("FJ", "+6797012345"),
    ("PW", "+6806201234"),
    ("WF", "+681501234"),
    ("CK", "+68222123"),
    ("KI", "+68660123"),
    ("NC", "+687501234"),
    ("TV", "+688901234"),
    ("PF", "+68987123456"),
    ("TK", "+6903012"),
    ("FM", "+6913501234"),
    ("MH", "+6922471234"),
    ("KP", "+8501912345678"),
    ("HK", "+85251234567"),
    ("MO", "+85366123456"),
    ("KH", "+85512345678"),
    ("LA", "+85620911234"),
    ("BD", "+8801812345678"),
    ("TW", "+886912345678"),
    ("MV", "+9607712345"),
    ("JO", "+962791234567"),
    ("SG", "+6581234567"),    // Singapore
    ("MY", "+60121234567"),   // Malaysia
    ("PH", "+639171234567"),  // Philippines
    ("TH", "+66812345678"),   // Thailand
    ("VN", "+84912345678"),   // Vietnam
    ("PK", "+923001234567"),  // Pakistan
    ("LK", "+94771234567"),   // Sri Lanka
    ("AF", "+93700123456"),   // Afghanistan
    ("MM", "+959123456789"),  // Myanmar
    ("UZ", "+998901234567"),  // Uzbekistan
    ("TJ", "+992551234567"),  // Tajikistan
    ("KG", "+996701234567"),  // Kyrgyzstan
    ("TM", "+99365123456"),   // Turkmenistan
    ("MN", "+97688123456"),   // Mongolia
    ("NP", "+9779812345678"), // Nepal
    ("AM", "+37494123456"),   // Armenia
    ("AZ", "+994501234567"),  // Azerbaijan
    ("GE", "+995555123456"),  // Georgia
    ("TM", "+99312123456"),   // Turkmenistan
    ("BH", "+97333123456"),   // Bahrain
    ("QA", "+97450123456"),   // Qatar
    ("KW", "+96550123456"),   // Kuwait
    ("SO", "+252615123456"),  // Somalia
    ("KE", "+254701234567"),  // Kenya
    ("TZ", "+255621234567"),  // Tanzania
    ("UG", "+256701234567"),  // Uganda
    ("ET", "+251911234567"),  // Ethiopia
    ("NG", "+2347012345678"), // Nigeria
    ("GH", "+233501234567"),  // Ghana
    ("SN", "+221771234567"),  // Senegal
    ("CI", "+2252721234567"), // Ivory Coast
    ("ML", "+22365123456"),   // Mali
    ("ZM", "+260961234567"),  // Zambia
    ("ZW", "+263771234567"),  // Zimbabwe
    ("BW", "+26772123456"),   // Botswana
    ("NA", "+264811234567"),  // Namibia
    ("MG", "+261341234567"),  // Madagascar
    ("RE", "+262692123456"),  // Reunion
    ("MU", "+23057123456"),   // Mauritius
    ("SC", "+2482512345"),    // Seychelles
    ("LS", "+26650123456"),   // Lesotho
    ("CV", "+2389912345"),    // Cape Verde
    ("ST", "+2399912345"),    // Sao Tome and Principe
    ("GQ", "+240222123456"),  // Equatorial Guinea
    ("CD", "+243991234567"),  // Democratic Republic of the Congo
    ("AO", "+244921234567"),  // Angola
    ("GW", "+245501234511"),  // Guinea-Bissau
    ("IO", "+2463801234"),    // British Indian Ocean Territory
    ("AC", "+247612345"),     // Ascension Island
    ("SD", "+249911231234"),  // Sudan
    ("RW", "+250720123456"),  // Rwanda
    ("DJ", "+25377123123"),   // Djibouti
    ("BI", "+25779123456"),   // Burundi
    ("MZ", "+258821234567"),  // Mozambique
];

/// `(number, primary ISO code, an ISO code that also claims the range)`.
///
/// `+7` is Russia and Kazakhstan, `+39` is Italy and the Vatican, `+44` is the
/// UK and the Crown Dependencies. [`extract_country`] must return the
/// documented primary; [`extract_countries`] must list the other candidate.
const SHARED_CALLING_CODE: &[(&str, &str, &str)] = &[
    ("+77012345678", "RU", "KZ"),
    ("+79161234567", "RU", "KZ"),
    ("+390669821234", "IT", "VA"),
    ("+442079460958", "GB", "JE"),
    ("+441481123456", "GB", "GG"),
];

/// Inputs that must be rejected, with the reason each one is rejected for.
///
/// The old suite had four negative cases in total; the parser has nine distinct
/// rejection reasons, and every one of them is exercised here.
const INVALID_CASES: &[(&str, ParseError)] = &[
    // No digits at all.
    ("", ParseError::Empty),
    ("   ", ParseError::Empty),
    ("+", ParseError::Empty),
    ("()", ParseError::Empty),
    ("0", ParseError::Empty),
    ("000000", ParseError::Empty),
    ("---", ParseError::Empty),
    // Characters that cannot appear in a phone number.
    ("abc", ParseError::InvalidCharacter),
    ("HELLO WORLD", ParseError::InvalidCharacter),
    ("CALL ME NOW PLZ", ParseError::InvalidCharacter),
    ("ILOVEYOUXX", ParseError::InvalidCharacter),
    ("invalid_phone_number", ParseError::InvalidCharacter),
    // Vanity letters need three digits ahead of them; one or two is prose.
    ("1CALLNOWXXX", ParseError::InvalidCharacter),
    ("12FLOWERSXX", ParseError::InvalidCharacter),
    ("+1_202_555_0173", ParseError::InvalidCharacter),
    ("+1@202#555$0173", ParseError::InvalidCharacter),
    ("1+2025550173", ParseError::InvalidCharacter),
    ("12025550173+", ParseError::InvalidCharacter),
    ("++12025550173", ParseError::InvalidCharacter),
    ("+1 (234 567-8990", ParseError::InvalidCharacter),
    ("+1 202 555 0173)", ParseError::InvalidCharacter),
    ("(((((", ParseError::InvalidCharacter),
    ("+1\u{0660}2025550173", ParseError::InvalidCharacter),
    ("+1\n202\t555 0173", ParseError::InvalidCharacter),
    // Too many significant digits for E.164.
    ("+1234567890123456", ParseError::TooLong),
    ("+4312345678901234", ParseError::TooLong),
    // Fewer digits than any numbering plan accepts.
    ("12", ParseError::TooShort),
    ("+12", ParseError::TooShort),
    ("999", ParseError::TooShort),
    // The leading digits are not a calling code anyone uses.
    ("+9999999999999", ParseError::UnknownCountryCode),
    ("+8709123456", ParseError::UnknownCountryCode),
    // Known country, wrong national length.
    ("+12025550", ParseError::InvalidLength),
    ("+1202555017", ParseError::InvalidLength),
    ("+120255501730", ParseError::InvalidLength),
    ("+2126123456", ParseError::InvalidLength),
    ("+471234567890", ParseError::InvalidLength),
    ("+6721234567890", ParseError::InvalidLength),
    ("+1202555017x", ParseError::InvalidLength),
    // NANP structural checks.
    ("+11111111111", ParseError::InvalidAreaCode),
    ("+11231231234", ParseError::InvalidAreaCode),
    ("+10005550199", ParseError::InvalidAreaCode),
    ("+19765550199", ParseError::InvalidAreaCode),
    ("+12021110173", ParseError::InvalidExchange),
    ("+12021230173", ParseError::InvalidExchange),
    ("+12020120173", ParseError::InvalidExchange),
];

/// `(input, is_valid)` — a validation table with a genuinely varying expected
/// column. The predecessor of this table asserted the constant `true` on all
/// ten of its rows, which gave it no discriminating power at all.
const VALIDATION_CASES: &[(&str, bool)] = &[
    ("+12025550173", true),
    ("+1 (202) 555-0173", true),
    ("+1.202.555.0173", true),
    ("+1-202-555-0173", true),
    ("+1/202/555/0173", true),
    ("+1 (202) 555-0173 ext. 99", true),
    ("1-800-FLOWERS", true),
    ("+96179123123", true),
    ("+442079460958", true),
    ("+44 (0)20 7946 0958", true),
    ("+390698123456", true),
    ("tel:+12025550173", true),
    ("\u{FF0B}\u{FF18}\u{FF11}\u{FF13}\u{FF11}\u{FF12}\u{FF13}\u{FF14}\u{FF15}\u{FF16}\u{FF17}\u{FF18}", true),
    ("000000000012025550173", true),
    ("+12025550173\n", true),
    (" +12025550173 ", true),
    // Negatives.
    ("invalid_phone_number", false),
    ("HELLO WORLD", false),
    ("", false),
    ("+", false),
    ("12", false),
    ("+1 (234) 567-890", false),
    ("+1 (234 567-8990", false),
    ("1+2025550173", false),
    ("+11111111111", false),
    ("+12021110173", false),
    ("+9999999999999", false),
    ("+1234567890123456", false),
    ("0000000000+12025550173", false),
    ("00000000001202555017", false),
];

/// Helper: resolve a number to its ISO code, panicking with a useful message.
#[track_caller]
fn iso(number: &str) -> &'static str {
    extract_country(number)
        .unwrap_or_else(|| panic!("{number} did not resolve to a country"))
        .code
}

/// Helper: the parse error for an input that must fail.
#[track_caller]
fn err(input: &str) -> ParseError {
    try_normalize_phone_number(input).expect_err(&format!("{input:?} unexpectedly parsed"))
}

// ============================================================================
// Validation
// ============================================================================

mod validation {
    use super::*;

    #[test]
    fn every_fixture_row_resolves_to_its_country() {
        for &(expected, number) in COUNTRY_FIXTURE {
            assert_eq!(
                extract_country(number).map(|c| c.code),
                Some(expected),
                "{number} must resolve to {expected}"
            );
        }
    }

    #[test]
    fn every_fixture_row_is_valid_and_already_canonical() {
        for &(expected, number) in COUNTRY_FIXTURE {
            assert!(is_valid_phone_number(number), "{number} must be valid");
            assert_eq!(
                normalize_phone_number(number).as_deref(),
                Some(number),
                "{number} must already be canonical E.164"
            );

            let parsed =
                PhoneNumber::parse(number).unwrap_or_else(|| panic!("{number} must parse"));
            assert_eq!(parsed.e164(), number);
            assert_eq!(parsed.country().code, expected);
            assert_eq!(parsed.format(PhoneFormat::E164), number);
            assert_eq!(
                normalize_phone_number(&parsed.format(PhoneFormat::RFC3966)).as_deref(),
                Some(number),
                "RFC 3966 output of {number} must re-parse"
            );
        }
    }

    #[test]
    fn every_fixture_row_round_trips_through_national_format() {
        for &(_, number) in COUNTRY_FIXTURE {
            let parsed = PhoneNumber::parse(number).unwrap();
            let national = parsed.format(PhoneFormat::National);
            let back = PhoneNumber::parse_with_country(&national, parsed.country().code)
                .unwrap_or_else(|| {
                    panic!(
                        "national form {national:?} of {number} must re-parse as {}",
                        parsed.country().code
                    )
                });
            assert_eq!(back.e164(), number, "national round trip changed {number}");
        }
    }

    #[test]
    fn shared_calling_codes_resolve_to_the_documented_primary() {
        for &(number, primary, also) in SHARED_CALLING_CODE {
            assert_eq!(iso(number), primary, "{number} must resolve to {primary}");

            let candidates: Vec<&str> = extract_countries(number).iter().map(|c| c.code).collect();
            assert_eq!(
                candidates.first(),
                Some(&primary),
                "{number}: the primary must come first"
            );
            assert!(
                candidates.contains(&also),
                "{number}: {also} must be listed as a candidate, got {candidates:?}"
            );
        }
    }

    #[test]
    fn validation_table() {
        for &(input, expected) in VALIDATION_CASES {
            assert_eq!(
                is_valid_phone_number(input),
                expected,
                "is_valid_phone_number({input:?})"
            );
        }
    }

    #[test]
    fn invalid_inputs_report_the_right_reason() {
        for &(input, expected) in INVALID_CASES {
            assert!(!is_valid_phone_number(input), "{input:?} must not be valid");
            assert_eq!(err(input), expected, "error for {input:?}");
        }
    }

    #[test]
    fn every_parse_error_variant_is_reachable() {
        use std::collections::BTreeSet;

        let mut seen = BTreeSet::new();
        for &(input, _) in INVALID_CASES {
            seen.insert(format!("{:?}", err(input)));
        }
        seen.insert(format!(
            "{:?}",
            PhoneNumber::try_parse_with_country("020 7946 0958", "ZZ").unwrap_err()
        ));

        for variant in [
            "Empty",
            "InvalidCharacter",
            "TooLong",
            "TooShort",
            "UnknownCountryCode",
            "InvalidLength",
            "InvalidAreaCode",
            "InvalidExchange",
            "UnknownCountry",
        ] {
            assert!(
                seen.contains(variant),
                "ParseError::{variant} is never produced by the suite; reachable: {seen:?}"
            );
        }
    }

    #[test]
    fn is_valid_never_disagrees_with_normalize() {
        let corpus: Vec<&str> = VALIDATION_CASES
            .iter()
            .map(|&(s, _)| s)
            .chain(INVALID_CASES.iter().map(|&(s, _)| s))
            .chain(COUNTRY_FIXTURE.iter().map(|&(_, s)| s))
            .collect();

        for input in corpus {
            assert_eq!(
                is_valid_phone_number(input),
                normalize_phone_number(input).is_some(),
                "is_valid_phone_number and normalize_phone_number disagree on {input:?}"
            );
        }
    }

    #[test]
    fn error_messages_are_distinct_and_non_empty() {
        use std::collections::BTreeSet;

        let errors = [
            ParseError::Empty,
            ParseError::InvalidCharacter,
            ParseError::TooLong,
            ParseError::TooShort,
            ParseError::UnknownCountryCode,
            ParseError::InvalidLength,
            ParseError::InvalidAreaCode,
            ParseError::InvalidExchange,
            ParseError::UnknownCountry,
        ];
        let messages: BTreeSet<String> = errors.iter().map(ToString::to_string).collect();
        assert_eq!(messages.len(), errors.len(), "Display must be injective");
        assert!(messages.iter().all(|m| !m.is_empty()));

        // ParseError is a real error type, not an opaque &'static str.
        let boxed: Box<dyn std::error::Error> = Box::new(ParseError::TooShort);
        assert_eq!(
            boxed.to_string(),
            "fewer digits than any numbering plan accepts"
        );
    }
}

// ============================================================================
// Normalization
// ============================================================================

mod normalization {
    use super::*;

    #[test]
    fn normalizes_every_written_form_of_one_number() {
        for input in [
            "+12025550173",
            "+1 202 555 0173",
            "+1 (202) 555-0173",
            "+1.202.555.0173",
            "+1-202-555-0173",
            "+1/202/555/0173",
            "12025550173",
            " +12025550173 ",
            "tel:+12025550173",
            "+1\u{00A0}202\u{2013}555\u{2013}0173",
            "+1 202-555-0173 ext. 42",
        ] {
            assert_eq!(
                normalize_phone_number(input).as_deref(),
                Some("+12025550173"),
                "normalize_phone_number({input:?})"
            );
        }
    }

    #[test]
    fn is_idempotent() {
        for &(_, number) in COUNTRY_FIXTURE {
            let once = normalize_phone_number(number).unwrap();
            let twice = normalize_phone_number(&once).unwrap();
            assert_eq!(once, twice, "normalization of {number} is not idempotent");
        }
    }

    #[test]
    fn strips_idd_and_trunk_prefixes_before_the_calling_code() {
        for input in [
            "00442079460958",
            "0000000000442079460958",
            "+44 (0)20 7946 0958",
            "+44 0 20 7946 0958",
        ] {
            assert_eq!(
                normalize_phone_number(input).as_deref(),
                Some("+442079460958"),
                "normalize_phone_number({input:?})"
            );
        }

        // A *single* leading zero is a national trunk prefix, not the `00`
        // international access code, so it is not a marker on its own and the
        // number stays ambiguous.
        assert_eq!(
            try_normalize_phone_number("0442079460958").unwrap_err(),
            ParseError::MissingCallingCode
        );
        assert_eq!(
            PhoneNumber::parse_with_country("0442079460958", "GB")
                .unwrap()
                .e164(),
            "+442079460958"
        );
    }

    #[test]
    fn preserves_a_significant_italian_leading_zero() {
        assert_eq!(
            normalize_phone_number("+390698123456").as_deref(),
            Some("+390698123456")
        );
        assert_eq!(
            normalize_phone_number("+39 06 6982 1234").as_deref(),
            Some("+390669821234")
        );
        // Italian mobiles carry no leading zero and must stay distinct.
        assert_eq!(
            normalize_phone_number("+393331234567").as_deref(),
            Some("+393331234567")
        );
    }

    #[test]
    fn in_place_leaves_the_normalized_value_in_the_buffer() {
        let mut buffer = "+1 (202) 555-0173".to_string();
        let returned = normalize_phone_number_in_place(&mut buffer);
        assert_eq!(returned.as_deref(), Some("+12025550173"));
        assert_eq!(buffer, "+12025550173");

        // A second call is therefore a no-op rather than a failure.
        let again = normalize_phone_number_in_place(&mut buffer);
        assert_eq!(again.as_deref(), Some("+12025550173"));
        assert_eq!(buffer, "+12025550173");
    }

    #[test]
    fn in_place_leaves_the_buffer_untouched_on_failure() {
        for original in ["nonsense", "garbage", "hello ext. 5", "", "+"] {
            let mut buffer = original.to_string();
            assert_eq!(normalize_phone_number_in_place(&mut buffer), None);
            assert_eq!(buffer, original, "buffer was mutated for {original:?}");
        }
    }

    #[test]
    fn in_place_agrees_with_the_borrowing_function() {
        let corpus: Vec<&str> = VALIDATION_CASES
            .iter()
            .map(|&(s, _)| s)
            .chain(INVALID_CASES.iter().map(|&(s, _)| s))
            .chain([
                "000000000012025550173",
                "0000000000000000000012025550173",
                "0000000000+12025550173",
                "+0000000000442079460958",
                "00000000001202555017",
                "000 000 000 44 20 7946 0958",
            ])
            .collect();

        for input in corpus {
            let borrowed = normalize_phone_number(input);
            let mut buffer = input.to_string();
            let in_place = normalize_phone_number_in_place(&mut buffer);
            assert_eq!(
                borrowed, in_place,
                "normalize_phone_number and _in_place disagree on {input:?}"
            );
        }
    }

    #[test]
    fn try_variant_reports_the_reason() {
        assert_eq!(
            try_normalize_phone_number("+12025550173"),
            Ok("+12025550173".to_string())
        );
        assert_eq!(
            try_normalize_phone_number("+9999999999999"),
            Err(ParseError::UnknownCountryCode)
        );
        assert_eq!(try_normalize_phone_number(""), Err(ParseError::Empty));
    }

    #[test]
    fn extensions_are_never_folded_into_the_subscriber_number() {
        for (input, expected) in [
            ("+1 202-555-0173 ext. 42", "+12025550173"),
            ("+1 202-555-0173 x42", "+12025550173"),
            ("+43 1 2345678 x1", "+4312345678"),
            ("+43 1 2345678 ext-1", "+4312345678"),
            ("+39 06 9812345 x9", "+39069812345"),
        ] {
            assert_eq!(
                normalize_phone_number(input).as_deref(),
                Some(expected),
                "normalize_phone_number({input:?})"
            );
        }
    }

    #[test]
    fn fullwidth_forms_normalize_like_their_ascii_equivalents() {
        assert_eq!(
            normalize_phone_number("\u{FF0B}\u{FF18}\u{FF11}\u{FF13}\u{FF11}\u{FF12}\u{FF13}\u{FF14}\u{FF15}\u{FF16}\u{FF17}\u{FF18}"),
            normalize_phone_number("+81312345678")
        );
    }
}

// ============================================================================
// Country resolution
// ============================================================================

mod country_resolution {
    use super::*;

    #[test]
    fn calling_codes_shared_with_a_territory_resolve_to_the_sovereign_state() {
        // Each of these used to be hijacked by a shadowing table entry.
        assert_eq!(iso("+442079460958"), "GB");
        assert_eq!(iso("+441481123456"), "GB");
        assert_eq!(iso("+441534123456"), "GB");
        assert_eq!(iso("+79161234567"), "RU");
        assert_eq!(iso("+212612345678"), "MA");
        assert_eq!(iso("+358401234567"), "FI");
        assert_eq!(iso("+590590123456"), "GP");
        assert_eq!(iso("+672312345"), "NF");
        assert_eq!(iso("+4791234567"), "NO");
        assert_eq!(iso("+262639123456"), "RE");
    }

    #[test]
    fn nanp_area_codes_attribute_exactly() {
        for (number, expected) in [
            ("+12025550173", "US"),
            ("+14155552671", "US"),
            ("+14165550198", "CA"),
            ("+15145550199", "CA"),
            ("+16045550199", "CA"),
            ("+16135550199", "CA"),
            ("+12685550173", "AG"),
            ("+12465550173", "BB"),
            ("+18765550123", "JM"),
            ("+16585550123", "JM"),
            ("+18095550123", "DO"),
            ("+17875550123", "PR"),
        ] {
            assert_eq!(iso(number), expected, "{number}");
        }
    }

    #[test]
    fn nanp_rejects_structurally_impossible_numbers() {
        assert_eq!(err("+11111111111"), ParseError::InvalidAreaCode);
        assert_eq!(err("+11231231234"), ParseError::InvalidAreaCode);
        assert_eq!(err("+10005550199"), ParseError::InvalidAreaCode);
        assert_eq!(err("+12021110173"), ParseError::InvalidExchange);
        assert_eq!(err("+12020120173"), ParseError::InvalidExchange);
    }

    #[test]
    fn bare_national_numbers_do_not_default_to_the_united_states() {
        // "2025550173" is ten digits with no calling code. Reading it as US was
        // a bug; so is reading it as the Egyptian +20 2555 0173 that its digits
        // also spell. Without a marker there is no way to choose, so it is
        // rejected rather than guessed.
        for bare in [
            "2025550173",
            "(202) 555-0173",
            "202-555-0173",
            "020 7946 0958",
        ] {
            assert_eq!(
                try_normalize_phone_number(bare).unwrap_err(),
                ParseError::MissingCallingCode,
                "{bare} must not resolve to any country"
            );
        }
        // The explicit forms both work and are the documented way to get US.
        assert_eq!(iso("+12025550173"), "US");
        assert_eq!(
            PhoneNumber::parse_with_country("2025550173", "US")
                .unwrap()
                .e164(),
            "+12025550173"
        );
    }

    #[test]
    fn extract_countries_lists_every_candidate_primary_first() {
        assert_eq!(
            extract_countries("+79161234567")
                .iter()
                .map(|c| c.code)
                .collect::<Vec<_>>(),
            ["RU", "KZ"]
        );
        assert_eq!(
            extract_countries("+390669821234")
                .iter()
                .map(|c| c.code)
                .collect::<Vec<_>>(),
            ["IT", "VA"]
        );
        assert_eq!(
            extract_countries("+442079460958")
                .iter()
                .map(|c| c.code)
                .collect::<Vec<_>>(),
            ["GB", "GG", "IM", "JE"]
        );
        // Unambiguous numbers yield exactly one candidate.
        assert_eq!(
            extract_countries("+12025550173")
                .iter()
                .map(|c| c.code)
                .collect::<Vec<_>>(),
            ["US"]
        );
    }

    #[test]
    fn extract_countries_is_empty_for_unparseable_input() {
        assert!(extract_countries("nonsense").is_empty());
        assert!(extract_countries("").is_empty());
        assert!(extract_countries("+2126123456").is_empty());
    }

    #[test]
    fn guess_agrees_with_extract_for_valid_numbers() {
        for &(_, number) in COUNTRY_FIXTURE {
            assert_eq!(
                guess_country_from_number(number).map(|c| c.code),
                extract_country(number).map(|c| c.code),
                "guess and extract disagree on {number}"
            );
        }
    }

    #[test]
    fn guess_falls_back_to_the_leading_calling_code() {
        assert_eq!(
            guess_country_from_number("442079460958").map(|c| c.code),
            Some("GB")
        );
        assert_eq!(
            guess_country_from_number("79161234567").map(|c| c.code),
            Some("RU")
        );
        assert_eq!(
            guess_country_from_number("493012345678").map(|c| c.code),
            Some("DE")
        );
        assert_eq!(
            guess_country_from_number("33123456789").map(|c| c.code),
            Some("FR")
        );
        assert_eq!(guess_country_from_number("").map(|c| c.code), None);
    }

    /// The fallback scan tries the longest calling code first, so a NANP
    /// territory's four-digit code beats the bare `1`. Every input here is
    /// *invalid* — that is what forces the fallback rather than
    /// [`extract_country`] — and every one of them resolves to the plain `US`
    /// if the scan is run shortest-first instead.
    #[test]
    fn guess_prefers_the_longest_calling_code() {
        for (digits, expected, calling_code) in [
            ("1268555017", "AG", 1268u32),
            ("18095550", "DO", 1809),
            ("17875550", "PR", 1787),
            ("1441555017", "BM", 1441),
        ] {
            assert!(
                !is_valid_phone_number(digits),
                "{digits} must be invalid, or it never reaches the fallback scan"
            );
            let guessed = guess_country_from_number(digits)
                .unwrap_or_else(|| panic!("{digits} must still be guessable"));
            assert_eq!(guessed.code, expected, "{digits}");
            assert_eq!(guessed.prefix, calling_code, "{digits}");
        }
    }
}

// ============================================================================
// Country table
// ============================================================================

mod country_table {
    use super::*;

    #[test]
    fn table_is_structurally_sound() {
        assert!(
            countries().len() > 200,
            "the table lost most of its entries"
        );

        for country in countries() {
            assert!(!country.name.is_empty(), "{:?} has no name", country.code);
            assert_eq!(
                country.code.len(),
                2,
                "{:?} is not an ISO 3166-1 alpha-2 code",
                country.code
            );
            assert!(
                country.code.bytes().all(|b| b.is_ascii_uppercase()),
                "{:?} is not upper-case ASCII",
                country.code
            );
            assert!(
                !country.phone_lengths.is_empty(),
                "{} has no accepted lengths",
                country.code
            );

            let prefix_digits = country.prefix.to_string().len();
            assert!(
                (1..=4).contains(&prefix_digits),
                "{} has a {prefix_digits}-digit calling code",
                country.code
            );
            for &length in country.phone_lengths {
                assert!(
                    prefix_digits + length as usize <= 15,
                    "{} accepts a national length of {length}, which exceeds E.164 with a {prefix_digits}-digit calling code",
                    country.code
                );
                assert!(
                    length >= 4,
                    "{} accepts an implausible length {length}",
                    country.code
                );
            }
        }
    }

    #[test]
    fn table_contains_no_pseudo_countries() {
        for fake in ["GB-CYM", "XX", "ZZ"] {
            assert!(
                country_by_code(fake).is_none(),
                "{fake} must not be in the country table"
            );
        }
        // Uninhabited territories with fabricated numbering plans are gone.
        for uninhabited in ["BV", "HM", "AQ", "TF"] {
            assert!(
                country_by_code(uninhabited).is_none(),
                "{uninhabited} has no numbering plan and must not be in the table"
            );
        }
    }

    #[test]
    fn country_by_code_lookup() {
        let japan = country_by_code("JP").expect("JP must be in the table");
        assert_eq!(japan.prefix, 81);
        assert_eq!(japan.name, "Japan");
        assert!(
            japan.phone_lengths.contains(&9),
            "JP must accept 9-digit nationals"
        );

        assert!(country_by_code("ZZ").is_none());
        assert!(country_by_code("").is_none());
        // Casing is not meaningful in an ISO 3166-1 code, so lookup ignores it.
        // A code that is not in the table is still rejected.
        assert_eq!(country_by_code("jp"), country_by_code("JP"));
        assert_eq!(country_by_code("Jp"), country_by_code("JP"));
        assert!(country_by_code("JPN").is_none());
    }

    #[test]
    fn countries_by_code_returns_every_overlay() {
        let dominican: Vec<u32> = countries_by_code("DO").iter().map(|c| c.prefix).collect();
        assert_eq!(dominican, [1809, 1829, 1849]);

        let puerto_rico: Vec<u32> = countries_by_code("PR").iter().map(|c| c.prefix).collect();
        assert_eq!(puerto_rico, [1787, 1939]);

        let jamaica: Vec<u32> = countries_by_code("JM").iter().map(|c| c.prefix).collect();
        assert!(
            jamaica.contains(&1876) && jamaica.contains(&1658),
            "{jamaica:?}"
        );

        assert!(countries_by_code("ZZ").is_empty());
    }

    /// A copy of `PRIMARY_FOR_PREFIX` (src/lib.rs): `(calling code, sample
    /// number, every claimant in the order the crate reports them)`.
    ///
    /// The first ISO code of each row is the designated primary. Without the
    /// table the winner would be an accident of declaration order in
    /// `src/constants.rs`, which is how `+7` once resolved to Kazakhstan and
    /// `+212` to Western Sahara.
    const SHARED_CALLING_CODES: &[(u32, &str, &[&str])] = &[
        (1, "+12025550173", &["US", "CA"]),
        (7, "+79161234567", &["RU", "KZ"]),
        (39, "+390669821234", &["IT", "VA"]),
        (44, "+442079460958", &["GB", "GG", "IM", "JE"]),
        (47, "+4791234567", &["NO", "SJ"]),
        (61, "+61412345678", &["AU", "CC", "CX"]),
        (212, "+212612345678", &["MA", "EH"]),
        (262, "+262692691234", &["RE", "YT"]),
        (358, "+358401234567", &["FI", "AX"]),
        (500, "+50051234", &["FK", "GS"]),
        (590, "+590590123456", &["GP", "BL", "MF"]),
        (599, "+59995181234", &["CW", "BQ"]),
    ];

    #[test]
    fn countries_by_calling_code_puts_the_primary_first() {
        for &(calling_code, sample, expected) in SHARED_CALLING_CODES {
            let entries: Vec<&str> = countries_by_calling_code(calling_code)
                .iter()
                .map(|c| c.code)
                .collect();
            assert_eq!(
                entries, expected,
                "+{calling_code} must list exactly {expected:?}, primary first"
            );
            // The same winner has to come out of the single-answer APIs.
            assert_eq!(
                extract_country(sample).map(|c| c.code),
                Some(expected[0]),
                "{sample} must resolve to {}",
                expected[0]
            );
            assert_eq!(
                extract_countries(sample).first().map(|c| c.code),
                Some(expected[0]),
                "{sample}: the primary must come first"
            );
        }
        assert!(countries_by_calling_code(9999).is_empty());
    }

    /// Every calling code claimed by more than one entry must have a pinned
    /// primary above. A new shared code added to the table without a decision
    /// about who owns it would otherwise be resolved by declaration order.
    #[test]
    fn every_shared_calling_code_has_a_designated_primary() {
        use std::collections::{BTreeMap, BTreeSet};

        let mut by_prefix: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
        for country in countries() {
            by_prefix
                .entry(country.prefix)
                .or_default()
                .push(country.code);
        }

        let pinned: BTreeSet<u32> = SHARED_CALLING_CODES.iter().map(|&(p, _, _)| p).collect();
        let shared: BTreeSet<u32> = by_prefix
            .iter()
            .filter(|(_, codes)| codes.len() > 1)
            .map(|(&prefix, _)| prefix)
            .collect();
        assert_eq!(
            shared, pinned,
            "the set of shared calling codes changed; decide who owns the new one"
        );
    }

    #[test]
    fn recently_added_entries_are_present() {
        assert_eq!(country_by_code("XK").map(|c| c.prefix), Some(383), "Kosovo");
        assert_eq!(
            country_by_code("BQ").map(|c| c.prefix),
            Some(599),
            "Caribbean Netherlands"
        );
        assert_eq!(iso("+38344123456"), "XK");
        assert_eq!(iso("+5997001234"), "BQ");
    }

    #[test]
    fn country_display_is_human_readable() {
        assert_eq!(country_by_code("JP").unwrap().to_string(), "Japan (+81)");
    }

    #[test]
    fn every_country_referenced_by_the_fixture_is_in_the_table() {
        for &(code, _) in COUNTRY_FIXTURE {
            assert!(
                country_by_code(code).is_some(),
                "fixture references unknown country {code}"
            );
        }
    }
}

// ============================================================================
// Formatting
// ============================================================================

mod formatting {
    use super::*;

    /// `(E.164 input, E164, International, National, RFC3966)`.
    const FORMAT_CASES: &[(&str, &str, &str, &str, &str)] = &[
        (
            "+12025550173",
            "+12025550173",
            "+1 202 555 0173",
            "(202) 555-0173",
            "tel:+12025550173",
        ),
        (
            "+12685550173",
            "+12685550173",
            "+1 268 555 0173",
            "(268) 555-0173",
            "tel:+12685550173",
        ),
        (
            "+442079460958",
            "+442079460958",
            "+44 20 7946 0958",
            "020 7946 0958",
            "tel:+442079460958",
        ),
        (
            "+390669821234",
            "+390669821234",
            "+39 06 6982 1234",
            "06 6982 1234",
            "tel:+390669821234",
        ),
        (
            "+33142685300",
            "+33142685300",
            "+33 1 42 68 53 00",
            "01 42 68 53 00",
            "tel:+33142685300",
        ),
        (
            "+493012345678",
            "+493012345678",
            "+49 30 1234 5678",
            "030 1234 5678",
            "tel:+493012345678",
        ),
        (
            "+819012345678",
            "+819012345678",
            "+81 90 1234 5678",
            "090 1234 5678",
            "tel:+819012345678",
        ),
    ];

    #[test]
    fn formats_match_the_expected_shape() {
        for &(input, e164, international, national, rfc3966) in FORMAT_CASES {
            assert_eq!(
                format_phone_number(input, PhoneFormat::E164).as_deref(),
                Some(e164),
                "E164 of {input}"
            );
            assert_eq!(
                format_phone_number(input, PhoneFormat::International).as_deref(),
                Some(international),
                "International of {input}"
            );
            assert_eq!(
                format_phone_number(input, PhoneFormat::National).as_deref(),
                Some(national),
                "National of {input}"
            );
            assert_eq!(
                format_phone_number(input, PhoneFormat::RFC3966).as_deref(),
                Some(rfc3966),
                "RFC3966 of {input}"
            );
        }
    }

    #[test]
    fn national_format_carries_the_trunk_prefix() {
        // The trunk prefix is what makes the output dialable domestically and
        // what lets `parse_with_country` recover the original number.
        for (input, national, trunk) in [
            ("+442079460958", "020 7946 0958", "0"),
            ("+493012345678", "030 1234 5678", "0"),
            ("+61412345678", "04 1234 5678", "0"),
            // Russia writes the trunk `8` as its own token; gluing it produces
            // "8916 …", which reads as a four-digit area code.
            ("+79161234567", "8 916 123 45 67", "8"),
        ] {
            let formatted = format_phone_number(input, PhoneFormat::National).unwrap();
            assert_eq!(formatted, national, "National of {input}");
            assert!(
                formatted.starts_with(trunk),
                "{formatted} must start with the trunk prefix"
            );
        }
        // Italy has no trunk prefix; its leading zero is part of the number.
        assert_eq!(
            format_phone_number("+390669821234", PhoneFormat::National).as_deref(),
            Some("06 6982 1234")
        );
    }

    /// `(E.164, national significant length, National, International)`.
    ///
    /// Every row belongs to a country with **no** entry in
    /// `national_group_sizes`, so the grouping comes from `default_group_sizes`
    /// alone. One row per length that table handles and the number plan can
    /// actually produce, plus the two short lengths it deliberately does not
    /// group. These pin the *default table*, not any local writing convention.
    const DEFAULT_GROUPING_CASES: &[(&str, usize, &str, &str)] = &[
        // Below 6 digits there is no rule, so the number is emitted ungrouped.
        ("+3527599", 4, "7599", "+352 7599"),    // Luxembourg
        ("+24778694", 5, "78694", "+247 78694"), // Ascension
        ("+376312345", 6, "312 345", "+376 312 345"), // Andorra, [3, 3]
        ("+3544101234", 7, "410 1234", "+354 410 1234"), // Iceland, [3, 4]
        ("+35679012345", 8, "7901 2345", "+356 7901 2345"), // Malta, [4, 4]
        ("+34912345678", 9, "912 345 678", "+34 912 345 678"), // Spain, [3, 3, 3]
        ("+541143214321", 10, "114 321 4321", "+54 114 321 4321"), // Argentina, [3, 3, 4]
        (
            "+5491143214321",
            11,
            "911 4321 4321",
            "+54 911 4321 4321", // Argentina, [3, 4, 4]
        ),
        (
            "+43783700628291",
            12,
            "0783 700 628 291", // Austria, [3, 3, 3, 3] behind the trunk `0`
            "+43 783 700 628 291",
        ),
        (
            "+438214630396663",
            13,
            "0821 463 039 6663", // Austria, [3, 3, 3, 4]
            "+43 821 463 039 6663",
        ),
    ];

    #[test]
    fn default_grouping_is_pinned_for_every_reachable_length() {
        for &(number, national_len, national, international) in DEFAULT_GROUPING_CASES {
            let parsed = PhoneNumber::parse(number)
                .unwrap_or_else(|| panic!("{number} must be a valid number"));
            assert_eq!(
                parsed.national_number().len(),
                national_len,
                "{number} does not have {national_len} national digits"
            );
            assert_eq!(
                format_phone_number(number, PhoneFormat::National).as_deref(),
                Some(national),
                "National of {number}"
            );
            assert_eq!(
                format_phone_number(number, PhoneFormat::International).as_deref(),
                Some(international),
                "International of {number}"
            );
        }
    }

    /// `default_group_sizes` also has 14- and 15-digit arms. No country in the
    /// table accepts a national significant number that long, so they are
    /// unreachable — which is why the table above stops at 13.
    #[test]
    fn no_numbering_plan_reaches_the_longest_default_grouping_arms() {
        let longest = countries()
            .iter()
            .flat_map(|c| c.phone_lengths.iter().copied())
            .max()
            .expect("the country table is not empty");
        assert_eq!(
            longest, 13,
            "a plan now accepts {longest} national digits; extend DEFAULT_GROUPING_CASES"
        );
    }

    #[test]
    fn rfc3966_output_re_parses() {
        for &(_, number) in COUNTRY_FIXTURE {
            let uri = format_phone_number(number, PhoneFormat::RFC3966).unwrap();
            assert!(uri.starts_with("tel:+"), "{uri} is not a tel: URI");
            assert!(
                is_valid_phone_number(&uri),
                "the crate must accept its own RFC 3966 output {uri}"
            );
            assert_eq!(
                normalize_phone_number(&uri).as_deref(),
                Some(number),
                "{uri} must re-parse to {number}"
            );
        }
    }

    #[test]
    fn rfc3966_output_carries_the_extension_and_re_parses_with_it() {
        let parsed = PhoneNumber::parse("+1 202-555-0173 ext. 42").unwrap();
        let uri = parsed.format(PhoneFormat::RFC3966);
        assert_eq!(uri, "tel:+12025550173;ext=42");

        let back = PhoneNumber::parse(&uri).unwrap();
        assert_eq!(back.e164(), "+12025550173");
        assert_eq!(back.extension(), Some("42"));
    }

    #[test]
    fn formatting_an_invalid_number_returns_none() {
        for format in [
            PhoneFormat::E164,
            PhoneFormat::International,
            PhoneFormat::National,
            PhoneFormat::RFC3966,
        ] {
            assert_eq!(format_phone_number("nonsense", format), None);
            assert_eq!(format_phone_number("", format), None);
            assert_eq!(format_phone_number("+11111111111", format), None);
        }
    }

    #[test]
    fn phone_format_variants_are_distinct() {
        // `#[non_exhaustive]` only forces a wildcard arm outside the defining
        // crate, so that half of the contract is proved from `tests/`; here we
        // pin that the four variants exist and compare by value.
        let described = |format: PhoneFormat| match format {
            PhoneFormat::E164 => "e164",
            PhoneFormat::International => "international",
            PhoneFormat::National => "national",
            PhoneFormat::RFC3966 => "rfc3966",
        };
        assert_eq!(described(PhoneFormat::E164), "e164");
        assert_eq!(described(PhoneFormat::RFC3966), "rfc3966");
        assert_eq!(PhoneFormat::E164, PhoneFormat::E164);
        assert_ne!(PhoneFormat::E164, PhoneFormat::National);
    }
}

// ============================================================================
// Type classification
// ============================================================================

mod classification {
    use super::*;
    use PhoneNumberType::*;

    const TYPE_CASES: &[(&str, PhoneNumberType)] = &[
        // United Kingdom.
        ("+448001234567", TollFree),
        ("+448081234567", TollFree),
        ("+448451234567", SharedCost),
        ("+449091234567", PremiumRate),
        ("+443001234567", Uan),
        ("+445612345678", Voip),
        ("+447911123456", Mobile),
        ("+442079460958", FixedLine),
        ("+447000123456", PersonalNumber),
        ("+447600123456", Pager),
        // Germany.
        ("+498001234567", TollFree),
        ("+499001234567", PremiumRate),
        ("+491801234567", SharedCost),
        ("+497001234567", PersonalNumber),
        ("+491641234567", Pager),
        ("+491701234567", Mobile),
        ("+493012345678", FixedLine),
        // France.
        ("+33612345678", Mobile),
        ("+33912345678", Voip),
        ("+33142685300", FixedLine),
        ("+33810123456", SharedCost),
        ("+33891123456", PremiumRate),
        // Italy — the leading zero is part of the number.
        ("+390669821234", FixedLine),
        ("+393331234567", Mobile),
        ("+398001234567", TollFree),
        // North America: geographic ranges are not split by service.
        ("+12025550173", FixedLineOrMobile),
        ("+14165550198", FixedLineOrMobile),
        ("+12685550173", FixedLineOrMobile),
        ("+18005550199", TollFree),
        ("+18885550199", TollFree),
        ("+19005550199", PremiumRate),
        ("+15005550199", PersonalNumber),
        ("+17005550199", Uan),
        // Elsewhere.
        ("+8613800138000", Mobile),
        ("+862112345678", FixedLine),
        ("+5511987654321", Mobile),
        ("+551133334444", FixedLine),
        ("+919876543210", Mobile),
        ("+911123456789", FixedLine),
        ("+81120123456", TollFree),
        ("+819012345678", Mobile),
        ("+81312345678", FixedLine),
        ("+34612345678", Mobile),
        ("+34912345678", FixedLine),
        ("+27821234567", Mobile),
        ("+61412345678", Mobile),
        ("+611800123456", TollFree),
        ("+611300123456", SharedCost),
        ("+611900123456", PremiumRate),
        ("+79161234567", Mobile),
        ("+74951234567", FixedLine),
        ("+905321234567", Mobile),
        ("+353861234567", Mobile),
        ("+35312345678", FixedLine),
        // No range table: an honest Unknown rather than a first-digit guess.
        ("+96179123123", Unknown),
    ];

    #[test]
    fn type_table() {
        for &(number, expected) in TYPE_CASES {
            assert_eq!(
                detect_phone_number_type(number),
                Some(expected),
                "detect_phone_number_type({number})"
            );
        }
    }

    #[test]
    fn detection_returns_none_for_invalid_numbers() {
        assert_eq!(detect_phone_number_type("nonsense"), None);
        assert_eq!(detect_phone_number_type(""), None);
        assert_eq!(detect_phone_number_type("+11111111111"), None);
    }

    #[test]
    fn helper_predicates_agree_with_detection() {
        for &(number, expected) in TYPE_CASES {
            assert_eq!(
                is_mobile_number(number),
                expected == Mobile,
                "is_mobile_number({number})"
            );
            assert_eq!(
                is_landline_number(number),
                expected == FixedLine,
                "is_landline_number({number})"
            );
            assert_eq!(
                is_toll_free_number(number),
                expected == TollFree,
                "is_toll_free_number({number})"
            );
        }
        // Invalid input is never any of them.
        assert!(!is_mobile_number("nonsense"));
        assert!(!is_landline_number("nonsense"));
        assert!(!is_toll_free_number("nonsense"));
    }

    #[test]
    fn north_american_numbers_are_not_claimed_to_be_landlines() {
        // Number portability decoupled the prefix from the technology, so a
        // NANP geographic number is FixedLineOrMobile and neither predicate
        // may claim it.
        assert!(!is_landline_number("+12025550173"));
        assert!(!is_mobile_number("+12025550173"));
        assert_eq!(
            detect_phone_number_type("+12025550173"),
            Some(FixedLineOrMobile)
        );
    }

    #[test]
    fn every_phone_number_type_variant_is_reachable() {
        use std::collections::BTreeSet;

        let reachable: BTreeSet<String> =
            TYPE_CASES.iter().map(|&(_, t)| format!("{t:?}")).collect();
        for variant in [
            "Mobile",
            "FixedLine",
            "FixedLineOrMobile",
            "TollFree",
            "PremiumRate",
            "SharedCost",
            "Voip",
            "PersonalNumber",
            "Pager",
            "Uan",
            "Unknown",
        ] {
            assert!(
                reachable.contains(variant),
                "PhoneNumberType::{variant} is unreachable dead API"
            );
        }
    }

    #[test]
    fn phone_number_type_is_non_exhaustive_and_displays() {
        let describe = |t: PhoneNumberType| match t {
            Mobile => "m",
            FixedLine => "f",
            _ => "other",
        };
        assert_eq!(describe(Mobile), "m");
        assert_eq!(describe(TollFree), "other");

        assert_eq!(Mobile.to_string(), "mobile");
        assert_eq!(FixedLineOrMobile.to_string(), "fixed line or mobile");
        assert_eq!(Unknown.to_string(), "unknown");
    }
}

// ============================================================================
// Generation
// ============================================================================

mod generation {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn generated_numbers_round_trip_for_every_country_that_can_be_generated() {
        for country in countries() {
            let Some(number) = generate_random_phone_number(country.code) else {
                continue;
            };
            assert!(
                is_valid_phone_number(&number),
                "{} generated an invalid number {number}",
                country.code
            );
            assert_eq!(
                normalize_phone_number(&number).as_deref(),
                Some(number.as_str()),
                "{} generated a non-canonical number {number}",
                country.code
            );
            assert_eq!(
                extract_country(&number).map(|c| c.code),
                Some(country.code),
                "{} generated {number}, which resolves elsewhere",
                country.code
            );
        }
    }

    /// The exact set of countries that cannot be generated.
    ///
    /// Every one of them shares a calling code with an entry whose accepted
    /// lengths cover its own, so no number can be minted that resolves back to
    /// it. A country *leaving* this set is a fix; a country *joining* it is a
    /// resolution regression, and either way the list in the crate docs has to
    /// be updated — so the set is pinned exactly rather than bounded.
    const UNGENERATABLE: &[&str] = &[
        "AX", "BL", "CC", "CX", "EH", "GG", "GS", "IM", "JE", "KZ", "MF", "SJ", "VA", "YT",
    ];

    #[test]
    fn generation_returns_none_exactly_for_the_shadowed_countries() {
        let ungeneratable: BTreeSet<&str> = countries()
            .iter()
            .filter(|c| generate_random_phone_number(c.code).is_none())
            .map(|c| c.code)
            .collect();

        assert_eq!(
            ungeneratable,
            UNGENERATABLE.iter().copied().collect::<BTreeSet<_>>()
        );

        // Every one of them really is shadowed: another entry on the same
        // calling code is the designated primary.
        for &code in UNGENERATABLE {
            let country =
                country_by_code(code).unwrap_or_else(|| panic!("{code} is not a country"));
            assert!(
                countries_by_calling_code(country.prefix).len() > 1,
                "{code} cannot be generated but has +{} to itself",
                country.prefix
            );
            assert_ne!(
                countries_by_calling_code(country.prefix)[0].code,
                code,
                "{code} is the primary for +{} and must be generatable",
                country.prefix
            );
        }

        // Everything with an unambiguous plan must be generatable.
        for expected in [
            "US", "CA", "GB", "DE", "FR", "IT", "JP", "AU", "BR", "IN", "AG", "DO",
        ] {
            assert!(
                !ungeneratable.contains(expected),
                "{expected} must be generatable"
            );
        }
    }

    #[test]
    fn nanp_generation_draws_from_the_countrys_own_area_codes() {
        let numbers = generate_random_phone_numbers("CA", 200);
        assert_eq!(numbers.len(), 200);
        for number in &numbers {
            assert_eq!(
                extract_country(number).map(|c| c.code),
                Some("CA"),
                "{number} was generated for CA but resolves elsewhere"
            );
        }

        let us = generate_random_phone_numbers("US", 300);
        assert_eq!(us.len(), 300);
        for number in &us {
            assert_eq!(
                extract_country(number).map(|c| c.code),
                Some("US"),
                "{number}"
            );
            // Area codes start 2-9; a clamped rather than resampled leading
            // digit used to produce impossible 0/1 area codes.
            let area_first = number.as_bytes()[2];
            assert!(
                (b'2'..=b'9').contains(&area_first),
                "{number} has an impossible area code"
            );
        }
    }

    #[test]
    fn generation_samples_the_whole_length_range() {
        let austria = country_by_code("AT").unwrap();
        assert!(
            austria.phone_lengths.len() > 3,
            "this test needs a country with several accepted lengths"
        );

        let lengths: BTreeSet<usize> = generate_random_phone_numbers("AT", 400)
            .iter()
            .map(|n| n.len())
            .collect();
        assert!(
            lengths.len() >= 3,
            "generation always used the same length: {lengths:?}"
        );
    }

    #[test]
    fn generation_is_not_periodic() {
        // A generator that takes its digits from a single bit range repeats
        // with a short period; 300 draws from a healthy one collide rarely.
        let numbers = generate_random_phone_numbers("US", 300);
        let unique: BTreeSet<&String> = numbers.iter().collect();
        assert!(
            unique.len() >= 295,
            "only {} of 300 generated numbers were unique",
            unique.len()
        );

        let mut histogram = [0usize; 10];
        for number in &numbers {
            for byte in &number.as_bytes()[2..] {
                histogram[(byte - b'0') as usize] += 1;
            }
        }
        let total: usize = histogram.iter().sum();
        for (digit, &count) in histogram.iter().enumerate() {
            assert!(
                count * 10 > total / 3,
                "digit {digit} appears {count} times out of {total}; the distribution is skewed"
            );
        }
    }

    #[test]
    fn generation_rejects_an_unknown_country() {
        assert_eq!(generate_random_phone_number("ZZ"), None);
        assert_eq!(generate_random_phone_number(""), None);
        assert!(generate_random_phone_numbers("ZZ", 10).is_empty());
        // A caller-controlled count must not be used as an allocation hint.
        assert!(generate_random_phone_numbers("ZZ", usize::MAX).is_empty());
    }

    #[test]
    fn batch_generation_returns_the_requested_count() {
        assert_eq!(generate_random_phone_numbers("US", 5).len(), 5);
        assert_eq!(generate_random_phone_numbers("US", 0).len(), 0);
        assert!(generate_random_phone_numbers("US", 5)
            .iter()
            .all(|n| is_valid_phone_number(n)));
    }
}

// ============================================================================
// Comparison and grouping
// ============================================================================

mod comparison {
    use super::*;

    #[test]
    fn equality_ignores_formatting() {
        assert!(are_phone_numbers_equal(
            "+1 202-555-0173",
            "+1 (202) 555-0173"
        ));
        assert!(are_phone_numbers_equal("+12025550173", "tel:+12025550173"));
        assert!(are_phone_numbers_equal(
            "+442079460958",
            "+44 (0)20 7946 0958"
        ));
    }

    #[test]
    fn different_numbers_are_not_equal() {
        assert!(!are_phone_numbers_equal("+12025550173", "+12025550174"));
        assert!(!are_phone_numbers_equal("+12025550173", "+442079460958"));
        // A significant leading zero makes a genuinely different number.
        assert!(!are_phone_numbers_equal("+390698123456", "+39698123456"));
    }

    #[test]
    fn invalid_numbers_are_never_equal_even_to_themselves() {
        assert!(!are_phone_numbers_equal("nonsense", "nonsense"));
        assert!(!are_phone_numbers_equal("", ""));
        assert!(!are_phone_numbers_equal("+12025550173", "nonsense"));
    }

    #[test]
    fn grouping_collects_equivalent_spellings() {
        let numbers = ["+1 202-555-0173", "+1 (202) 555-0173", "+442079460958"];
        let groups = group_equivalent_phone_numbers(&numbers);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], ["+1 202-555-0173", "+1 (202) 555-0173"]);
        assert_eq!(groups[1], ["+442079460958"]);
    }

    #[test]
    fn grouping_preserves_every_input_exactly_once() {
        let numbers = [
            "+12025550173",
            "not a number",
            "not a number",
            "+1 202 555 0173",
            "",
        ];
        let groups = group_equivalent_phone_numbers(&numbers);
        let flattened: Vec<&String> = groups.iter().flatten().collect();
        assert_eq!(flattened.len(), numbers.len());
        for input in numbers {
            assert!(
                flattened.iter().any(|s| s.as_str() == input),
                "{input:?} was dropped"
            );
        }
        // Unnormalizable inputs each form their own group.
        assert_eq!(groups.len(), 4);
    }

    #[test]
    fn grouping_an_empty_slice_yields_no_groups() {
        let empty: [&str; 0] = [];
        assert!(group_equivalent_phone_numbers(&empty).is_empty());
    }
}

// ============================================================================
// Batch APIs
// ============================================================================

mod batch {
    use super::*;

    #[test]
    fn validate_batch() {
        assert_eq!(
            validate_phone_numbers_batch(&["+12025550173", "invalid", "+442079460958"]),
            vec![true, false, true]
        );
        assert!(validate_phone_numbers_batch::<&str>(&[]).is_empty());
    }

    #[test]
    fn normalize_batch() {
        assert_eq!(
            normalize_phone_numbers_batch(&["+12025550173", "+1 (202) 555-0173", "nonsense"]),
            vec![
                Some("+12025550173".to_string()),
                Some("+12025550173".to_string()),
                None
            ]
        );
    }

    #[test]
    fn extract_countries_batch_matches_the_scalar_function() {
        let out = extract_countries_batch(&["+12025550173", "+442079460958", "nonsense"]);
        assert_eq!(out[0].map(|c| c.code), Some("US"));
        assert_eq!(out[1].map(|c| c.code), Some("GB"));
        assert_eq!(out[2].map(|c| c.code), None);
    }

    #[test]
    fn detect_types_batch_matches_the_scalar_function() {
        let out = detect_phone_number_types_batch(&["+18005550199", "+447911123456", "nonsense"]);
        assert_eq!(out[0], Some(PhoneNumberType::TollFree));
        assert_eq!(out[1], Some(PhoneNumberType::Mobile));
        assert_eq!(out[2], None);
    }

    #[test]
    fn analysis_reports_every_field() {
        let out = analyze_phone_numbers_batch(&["+1 (202) 555-0173"]);
        assert_eq!(out.len(), 1);
        let analysis = &out[0];
        assert_eq!(analysis.original, "+1 (202) 555-0173");
        assert!(analysis.is_valid);
        assert_eq!(analysis.normalized.as_deref(), Some("+12025550173"));
        assert_eq!(analysis.country.map(|c| c.code), Some("US"));
        assert_eq!(
            analysis.phone_type,
            Some(PhoneNumberType::FixedLineOrMobile)
        );
        assert_eq!(analysis.error, None);
    }

    #[test]
    fn analysis_reports_the_rejection_reason() {
        for (input, expected) in [
            ("nonsense", ParseError::InvalidCharacter),
            ("+11111111111", ParseError::InvalidAreaCode),
            ("+12021110173", ParseError::InvalidExchange),
            ("+1202555", ParseError::InvalidLength),
            ("", ParseError::Empty),
        ] {
            let analysis = &analyze_phone_numbers_batch(&[input])[0];
            assert!(!analysis.is_valid, "{input:?}");
            assert_eq!(analysis.normalized, None);
            assert_eq!(analysis.country, None);
            assert_eq!(analysis.phone_type, None);
            assert_eq!(analysis.error, Some(expected), "error for {input:?}");
        }
    }

    #[test]
    fn analysis_never_contradicts_the_other_batch_apis() {
        let corpus: Vec<&str> = VALIDATION_CASES
            .iter()
            .map(|&(s, _)| s)
            .chain(INVALID_CASES.iter().map(|&(s, _)| s))
            .chain(COUNTRY_FIXTURE.iter().map(|&(_, s)| s))
            .collect();

        let analyses = analyze_phone_numbers_batch(&corpus);
        let validity = validate_phone_numbers_batch(&corpus);
        let normalized = normalize_phone_numbers_batch(&corpus);
        let resolved = extract_countries_batch(&corpus);
        let types = detect_phone_number_types_batch(&corpus);

        for (index, input) in corpus.iter().enumerate() {
            assert_eq!(
                analyses[index].is_valid, validity[index],
                "analyze and validate disagree on {input:?}"
            );
            assert_eq!(
                analyses[index].normalized, normalized[index],
                "analyze and normalize disagree on {input:?}"
            );
            assert_eq!(
                analyses[index].country.map(|c| c.code),
                resolved[index].map(|c| c.code),
                "analyze and extract_country disagree on {input:?}"
            );
            assert_eq!(
                analyses[index].phone_type, types[index],
                "analyze and detect_type disagree on {input:?}"
            );
            assert_eq!(analyses[index].original, *input);
        }
    }

    #[test]
    fn batches_accept_owned_strings_too() {
        let owned = vec!["+12025550173".to_string(), "nonsense".to_string()];
        assert_eq!(validate_phone_numbers_batch(&owned), vec![true, false]);
    }
}

// ============================================================================
// Correction suggestions
// ============================================================================

mod suggestions {
    use super::*;

    #[test]
    fn an_already_valid_number_is_returned_unchanged() {
        assert_eq!(
            suggest_phone_number_corrections("+12025550173", None),
            vec!["+12025550173".to_string()]
        );
        assert_eq!(
            suggest_phone_number_corrections("  +1 (202) 555-0173  ", None),
            vec!["+12025550173".to_string()]
        );
    }

    #[test]
    fn an_idd_prefix_is_repaired_into_a_plus() {
        assert_eq!(
            suggest_phone_number_corrections("00442079460958", None),
            vec!["+442079460958".to_string()]
        );
        assert_eq!(
            suggest_phone_number_corrections("011442079460958", None),
            vec!["+442079460958".to_string()]
        );
    }

    #[test]
    fn a_national_number_is_repaired_against_the_hint() {
        assert_eq!(
            suggest_phone_number_corrections("07911 123456", Some("GB")),
            vec!["+447911123456".to_string()]
        );
    }

    /// `(input, hint, the exact suggestions, in order)`.
    ///
    /// Asserting the exact vector rather than "non-empty and all valid" is the
    /// point: the loose form let the two `Some("US")` rows come back with five
    /// numbers from five unrelated countries and nobody noticed. That defect is
    /// fixed — every hinted row below is now entirely in the hinted country —
    /// and [`suggestions_for_a_hinted_input_reach_the_hint_country`] pins the
    /// property so it cannot come back.
    const SUGGESTION_CASES: &[(&str, Option<&str>, &[&str])] = &[
        // Already valid: returned unchanged, and nothing else is offered.
        ("+12025550173", None, &["+12025550173"]),
        // The hint reading wins over whatever the bare digits spell.
        ("2025550173", Some("US"), &["+12025550173"]),
        ("020 7946 0958", Some("GB"), &["+442079460958"]),
        ("07911 123456", Some("GB"), &["+447911123456"]),
        // An IDD prefix instead of '+'.
        ("00442079460958", None, &["+442079460958"]),
        // One digit too many: single deletions, from the end backwards, so the
        // obvious repair — drop the trailing digit — is offered first. Every
        // suggestion stays in the hinted country; the global reading of the
        // same digits is only offered when no hinted repair exists at all.
        (
            "+1 202 555 01730",
            Some("US"),
            &[
                "+12025550173",
                "+12025550170",
                "+12025550130",
                "+12025550730",
                "+12025551730",
            ],
        ),
        (
            "+44 20 7946 09580",
            Some("GB"),
            &[
                "+442079460958",
                "+442079460950",
                "+442079460980",
                "+442079460580",
                "+442079469580",
            ],
        ),
        // One digit missing. Insertions are tried from the end backwards,
        // because a truncated number lost its last digit, so the ten
        // end-insertions come first — and the real number is among them.
        (
            "+1 202 555 017",
            Some("US"),
            &[
                "+12025550170",
                "+12025550171",
                "+12025550172",
                "+12025550173",
                "+12025550174",
            ],
        ),
        // No hint: the candidate countries are tried in their documented order,
        // and the list is cut at five.
        (
            "07911123456",
            None,
            &[
                "+447911123456",
                "+497911123456",
                "+917911123456",
                "+617911123456",
                "+557911123456",
            ],
        ),
    ];

    #[test]
    fn every_suggestion_is_a_valid_number_and_there_are_at_most_five() {
        for &(input, hint, expected) in SUGGESTION_CASES {
            let suggestions = suggest_phone_number_corrections(input, hint);
            assert_eq!(
                suggestions, expected,
                "suggestions for {input:?} with hint {hint:?}"
            );
            assert!(
                suggestions.len() <= 5,
                "{input:?} produced {} suggestions",
                suggestions.len()
            );
            for suggestion in &suggestions {
                assert!(
                    is_valid_phone_number(suggestion),
                    "{suggestion} was suggested for {input:?} but is not valid"
                );
                assert_eq!(
                    normalize_phone_number(suggestion).as_deref(),
                    Some(suggestion.as_str()),
                    "{suggestion} is not canonical"
                );
            }
            let mut deduped = suggestions.clone();
            deduped.sort();
            deduped.dedup();
            assert_eq!(
                deduped.len(),
                suggestions.len(),
                "duplicate suggestions for {input:?}"
            );
        }
    }

    /// The documented five-suggestion cap, pinned by an input that overflows it.
    /// `found.truncate(5)` can otherwise be changed to any smaller number
    /// without a test noticing.
    #[test]
    fn the_list_is_cut_at_exactly_five() {
        for (input, hint) in [
            ("07911123456", None),
            ("+1 202 555 017", Some("US")),
            ("+44 20 7946 09580", Some("GB")),
        ] {
            assert_eq!(
                suggest_phone_number_corrections(input, hint).len(),
                5,
                "{input:?} must saturate the cap"
            );
        }
    }

    /// An unrecognised hint fails closed, exactly like
    /// `PhoneNumber::parse_with_country` and
    /// `extract_phone_numbers_with_country_hint`. Falling back to "no hint"
    /// would answer with a number from a country the caller never named — for
    /// `"2025550173"` that is the Egyptian `+2025550173`.
    #[test]
    fn an_unrecognised_hint_yields_nothing() {
        for hint in ["ZZ", "USA", "", "1", "XX", "gbr"] {
            assert!(
                suggest_phone_number_corrections("2025550173", Some(hint)).is_empty(),
                "hint {hint:?} must yield no suggestions"
            );
        }
        // Without a hint the same digits do produce a suggestion, so the empty
        // result above is the hint failing closed rather than the input being
        // hopeless.
        assert_eq!(
            suggest_phone_number_corrections("2025550173", None),
            vec!["+2025550173".to_string()]
        );
        // A differently-cased code is not a typo; it is the same country.
        assert_eq!(
            suggest_phone_number_corrections("2025550173", Some("us")),
            vec!["+12025550173".to_string()]
        );
    }

    /// Regression: tiers 3 and 4 of `suggest_phone_number_corrections` used to
    /// generate the bare global reading `format!("+{}", edited)` before the
    /// hinted one for every edit position, then truncate to five. A global
    /// reading found at the first edit position filled the whole list, so
    /// `("+1 202 555 017", Some("US"))` answered with five numbers from the
    /// Netherlands, Switzerland, Peru, Australia and Russia — contradicting
    /// both the function's doc and its own doctest.
    #[test]
    fn suggestions_for_a_hinted_input_reach_the_hint_country() {
        for (input, hint) in [
            ("+1 202 555 017", "US"),
            ("+1 202 555 01730", "US"),
            ("+44 20 7946 09580", "GB"),
        ] {
            let suggestions = suggest_phone_number_corrections(input, Some(hint));
            assert!(
                suggestions
                    .iter()
                    .any(|s| extract_country(s).map(|c| c.code) == Some(hint)),
                "no suggestion for {input:?} is in the hinted country {hint}: {suggestions:?}"
            );
        }
        // The obvious single deletion of the trailing digit.
        assert!(
            suggest_phone_number_corrections("+44 20 7946 09580", Some("GB"))
                .contains(&"+442079460958".to_string())
        );
    }

    #[test]
    fn hopeless_input_yields_no_suggestions() {
        for input in ["", "nonsense", "9876", "+442079460958123456789012"] {
            assert!(
                suggest_phone_number_corrections(input, None).is_empty(),
                "{input:?} should not be repairable"
            );
        }
    }

    #[test]
    fn potentially_valid_is_implied_by_valid() {
        let corpus: Vec<&str> = VALIDATION_CASES
            .iter()
            .map(|&(s, _)| s)
            .chain(COUNTRY_FIXTURE.iter().map(|&(_, s)| s))
            .collect();

        for input in corpus {
            if is_valid_phone_number(input) {
                assert!(
                    is_potentially_valid_phone_number(input),
                    "{input:?} is valid but not potentially valid"
                );
            }
        }
    }

    #[test]
    fn potentially_valid_rejects_the_hopeless() {
        assert!(!is_potentially_valid_phone_number("12"));
        assert!(!is_potentially_valid_phone_number("000000000"));
        assert!(!is_potentially_valid_phone_number(""));
        assert!(!is_potentially_valid_phone_number("+4312345678901234"));
        assert!(is_potentially_valid_phone_number("202-555-0173"));
    }
}

// ============================================================================
// Text API wrappers
// ============================================================================

mod text_api {
    use super::*;

    /// Every reported span must slice back to exactly the reported `raw`, and
    /// spans must not overlap.
    #[track_caller]
    fn checked(text: &str) -> Vec<ExtractedPhoneNumber> {
        let numbers = extract_phone_numbers_from_text(text);
        let mut previous_end = 0;
        for number in &numbers {
            assert!(
                number.start >= previous_end,
                "overlapping spans in {text:?}"
            );
            assert!(number.end > number.start, "empty span in {text:?}");
            assert_eq!(
                &text[number.start..number.end],
                number.raw,
                "span/raw mismatch"
            );
            assert_eq!(number.is_valid, number.normalized.is_some());
            previous_end = number.end;
        }
        numbers
    }

    #[test]
    fn extracts_international_numbers_with_exact_offsets() {
        let text = "Call me at +1-202-555-0173 or (415) 555-2671";
        let numbers = checked(text);
        assert_eq!(numbers.len(), 2);

        assert_eq!(numbers[0].raw, "+1-202-555-0173");
        assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
        assert!(numbers[0].is_valid);
        assert_eq!(numbers[0].start, 11);
        assert_eq!(numbers[0].end, 26);

        assert_eq!(numbers[1].raw, "(415) 555-2671");
        assert!(!numbers[1].is_valid);
    }

    #[test]
    fn adjacent_numbers_are_not_merged() {
        let numbers = checked("Call 2025550173 4155552671 now");
        assert_eq!(
            numbers.iter().map(|n| n.raw.as_str()).collect::<Vec<_>>(),
            ["2025550173", "4155552671"]
        );
    }

    #[test]
    fn separators_inside_a_number_do_not_break_extraction() {
        for text in [
            "Call +1 202\u{00A0}555\u{00A0}0173 now",
            "Call +1 202\u{2013}555\u{2013}0173 now",
            "Call +1 202\t555\t0173 now",
            "Call +1 202  555  0173 now",
            "Call +1 202 - 555 - 0173 now",
        ] {
            let numbers = checked(text);
            assert_eq!(numbers.len(), 1, "no match in {text:?}");
            assert_eq!(
                numbers[0].normalized.as_deref(),
                Some("+12025550173"),
                "{text:?}"
            );
        }
    }

    #[test]
    fn fullwidth_numbers_are_extracted() {
        let text = "Call \u{FF0B}\u{FF18}\u{FF11}\u{FF13}\u{FF11}\u{FF12}\u{FF13}\u{FF14}\u{FF15}\u{FF16}\u{FF17}\u{FF18} now";
        let numbers = checked(text);
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+81312345678"));
    }

    #[test]
    fn a_plus_inside_a_word_is_not_a_number_start() {
        assert!(checked("C++12025550173").is_empty());
        assert!(checked("email me at bob+12025550173@example.com").is_empty());
        // "Zip+4" must not become a country code.
        for number in checked("Zip+4 90210-1234") {
            assert!(!number.raw.starts_with('+'), "{:?}", number.raw);
        }
    }

    #[test]
    fn structured_numeric_data_is_not_mistaken_for_a_phone_number() {
        for text in [
            "Version 3.141592653589793 here",
            "IP 192.168.1.100 and date 2024-01-15",
            "ISBN 978-3-16-148410-0 here",
        ] {
            assert!(checked(text).is_empty(), "false positive in {text:?}");
        }
    }

    #[test]
    fn digit_runs_longer_than_e164_are_rejected_not_truncated() {
        for text in [
            "digits 0123456789012345 here",
            "digits 01234567890123456 here",
            "digits 012345678901234567890123456789 here",
        ] {
            assert!(
                checked(text).is_empty(),
                "{text:?} produced a truncated candidate"
            );
        }
    }

    #[test]
    fn balanced_parentheses_belong_to_the_number() {
        let numbers = checked("(2025550173)");
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].raw, "(2025550173)");
        assert_eq!(numbers[0].start, 0);
        assert_eq!(numbers[0].end, 12);
    }

    #[test]
    fn a_following_parenthesised_digit_is_not_swallowed() {
        let text = "Call 202-555-0173 (3 lines available)";
        let numbers = checked(text);
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].raw, "202-555-0173");
        assert_eq!(
            redact_phone_numbers(text, 0),
            "Call [PHONE] (3 lines available)"
        );
    }

    #[test]
    fn extract_valid_keeps_only_parseable_numbers() {
        let numbers = extract_valid_phone_numbers_from_text("Call +12025550173 or 123");
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
        assert!(numbers.iter().all(|n| n.is_valid));

        assert!(extract_valid_phone_numbers_from_text("nothing here").is_empty());
    }

    #[test]
    fn count_matches_the_extractor() {
        for text in [
            "+12025550173 and +442079460958",
            "no numbers here",
            "",
            "Call me at +1-202-555-0173 or (415) 555-2671",
        ] {
            assert_eq!(
                count_phone_numbers_in_text(text),
                extract_phone_numbers_from_text(text).len(),
                "{text:?}"
            );
        }
        assert_eq!(
            count_phone_numbers_in_text("+12025550173 and +442079460958"),
            2
        );
    }

    /// `(text, the number as written inside it, hint, expected E.164)`.
    ///
    /// The Italian rows are the ones that matter most: `src/text.rs::validate`
    /// used to strip a leading `0` unconditionally on the hint path, so it
    /// disagreed with `PhoneNumber::parse_with_country` for every country whose
    /// leading zero is significant — and returned a *different, still valid*
    /// number rather than failing. `011 123 4567` is Turin, and doubles as the
    /// case `src/text.rs`'s "an IDD prefix only counts when the remainder
    /// parses" comment cites.
    const HINTED_TEXT_CASES: &[(&str, &str, &str, &str)] = &[
        (
            "Call (202) 555-0173",
            "(202) 555-0173",
            "US",
            "+12025550173",
        ),
        (
            "Rufen Sie 030 12345678 an",
            "030 12345678",
            "DE",
            "+493012345678",
        ),
        (
            "Appelez 01 42 68 53 00",
            "01 42 68 53 00",
            "FR",
            "+33142685300",
        ),
        ("Call 020 7946 0958", "020 7946 0958", "GB", "+442079460958"),
        ("Call 0412 345 678", "0412 345 678", "AU", "+61412345678"),
        // A national number shorter than the no-hint floor is reachable
        // once the hint supplies the country's own length table.
        ("call 849338 now", "849338", "AD", "+376849338"),
        // Italy keeps its leading zero: Rome, Turin and Milan.
        (
            "chiama 06 6982 1234 oggi",
            "06 6982 1234",
            "IT",
            "+390669821234",
        ),
        (
            "chiama 011 123 4567 oggi",
            "011 123 4567",
            "IT",
            "+390111234567",
        ),
        (
            "chiama 02 1234 5678 oggi",
            "02 1234 5678",
            "IT",
            "+390212345678",
        ),
    ];

    #[test]
    fn country_hint_resolves_national_formats() {
        for &(text, written, hint, expected) in HINTED_TEXT_CASES {
            let numbers = extract_phone_numbers_with_country_hint(text, hint);
            assert_eq!(numbers.len(), 1, "{text:?} with hint {hint}");
            assert_eq!(numbers[0].raw, written, "{text:?} with hint {hint}");
            assert_eq!(
                numbers[0].normalized.as_deref(),
                Some(expected),
                "{text:?} with hint {hint}"
            );
        }
    }

    /// The two hinted-parse entry points must never disagree: the text
    /// extractor delegates to `PhoneNumber::parse_with_country` rather than
    /// reimplementing trunk-prefix handling.
    #[test]
    fn the_text_hint_agrees_with_parse_with_country() {
        for &(text, written, hint, expected) in HINTED_TEXT_CASES {
            let extracted = extract_phone_numbers_with_country_hint(text, hint);
            let parsed = PhoneNumber::parse_with_country(written, hint)
                .unwrap_or_else(|| panic!("{written:?} with hint {hint} must parse"));
            assert_eq!(parsed.e164(), expected, "{written:?} with hint {hint}");
            assert_eq!(
                extracted.first().and_then(|n| n.normalized.as_deref()),
                Some(parsed.e164()),
                "the extractor and parse_with_country disagree on {written:?} / {hint}"
            );
        }
    }

    #[test]
    fn an_explicit_country_code_wins_over_the_hint() {
        let numbers = extract_phone_numbers_with_country_hint("Call +12025550173", "DE");
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
        assert!(numbers[0].is_valid);
    }

    #[test]
    fn hint_extraction_preserves_raw_and_offsets() {
        let text = "Ring 020 7946 0958 today";
        let numbers = extract_phone_numbers_with_country_hint(text, "GB");
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].raw, "020 7946 0958");
        assert_eq!(&text[numbers[0].start..numbers[0].end], numbers[0].raw);
    }

    #[test]
    fn replace_rewrites_every_match_and_leaves_the_rest_alone() {
        assert_eq!(
            replace_phone_numbers_in_text("A +12025550173 B +442079460958 C", |n| format!(
                "<{}>",
                n.normalized.as_deref().unwrap_or("?")
            )),
            "A <+12025550173> B <+442079460958> C"
        );
        // Zero matches: the text comes back byte-for-byte.
        assert_eq!(
            replace_phone_numbers_in_text("nothing here", |_| "X".to_string()),
            "nothing here"
        );
        // The closure sees the raw text and the span it came from.
        let text = "Call +12025550173 now";
        assert_eq!(
            replace_phone_numbers_in_text(text, |n| format!("{}@{}", n.raw, n.start)),
            "Call +12025550173@5 now"
        );
    }

    #[test]
    fn replace_handles_non_ascii_surroundings() {
        assert_eq!(
            replace_phone_numbers_in_text("Ruf \u{00E4}n +12025550173 \u{1F600}", |_| "X"
                .to_string()),
            "Ruf \u{00E4}n X \u{1F600}"
        );
    }

    #[test]
    fn redaction_masks_all_but_the_visible_digits() {
        assert_eq!(
            redact_phone_numbers("Call +12025550173", 4),
            "Call *******0173"
        );
        assert_eq!(
            redact_phone_numbers("Call +12025550173", 1),
            "Call **********3"
        );
    }

    #[test]
    fn redaction_edge_branches() {
        // Zero visible digits replaces the whole number.
        assert_eq!(redact_phone_numbers("Call +12025550173", 0), "Call [PHONE]");
        // Asking for at least as many digits as the number has must not
        // reveal it.
        assert_eq!(
            redact_phone_numbers("Call +12025550173", 11),
            "Call [PHONE]"
        );
        assert_eq!(
            redact_phone_numbers("Call +12025550173", 12),
            "Call [PHONE]"
        );
        assert_eq!(
            redact_phone_numbers("Call +12025550173", usize::MAX),
            "Call [PHONE]"
        );
        // No numbers: unchanged.
        assert_eq!(redact_phone_numbers("no numbers", 4), "no numbers");
        assert_eq!(redact_phone_numbers("", 4), "");
    }

    #[test]
    fn redaction_never_leaks_digits_of_an_adjacent_number() {
        assert_eq!(
            redact_phone_numbers("Call 2025550173 4155552671 now", 0),
            "Call [PHONE] [PHONE] now"
        );
        assert_eq!(
            redact_phone_numbers("Numbers: 2025550173 4155552671", 0),
            "Numbers: [PHONE] [PHONE]"
        );
    }

    #[test]
    fn extraction_handles_empty_and_number_free_text() {
        assert!(checked("").is_empty());
        assert!(checked("no numbers here").is_empty());
        assert!(extract_phone_numbers_with_country_hint("", "US").is_empty());
        assert_eq!(count_phone_numbers_in_text(""), 0);
    }
}

// ============================================================================
// PhoneNumber
// ============================================================================

mod phone_number {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::str::FromStr;

    #[test]
    fn accessors_expose_every_part_of_the_number() {
        let parsed = PhoneNumber::parse("+1 (202) 555-0173 ext. 42").unwrap();
        assert_eq!(parsed.original(), "+1 (202) 555-0173 ext. 42");
        assert_eq!(parsed.e164(), "+12025550173");
        assert_eq!(parsed.national_number(), "2025550173");
        assert_eq!(parsed.country().code, "US");
        assert_eq!(parsed.country().name, "United States");
        assert_eq!(parsed.country_code(), 1);
        assert_eq!(parsed.phone_type(), PhoneNumberType::FixedLineOrMobile);
        assert_eq!(parsed.extension(), Some("42"));
    }

    #[test]
    fn national_number_never_returns_the_e164_string() {
        for &(_, number) in COUNTRY_FIXTURE {
            let parsed = PhoneNumber::parse(number).unwrap();
            let national = parsed.national_number();
            assert!(!national.starts_with('+'), "{number} -> {national}");
            assert!(
                national.bytes().all(|b| b.is_ascii_digit()),
                "{number} -> {national}"
            );
            assert!(
                number.ends_with(national),
                "{number} does not end with {national}"
            );
        }
    }

    #[test]
    fn a_territorys_area_code_belongs_to_its_calling_code() {
        let antigua = PhoneNumber::parse("+12685550173").unwrap();
        assert_eq!(antigua.country().code, "AG");
        assert_eq!(antigua.country_code(), 1268);
        assert_eq!(antigua.national_number(), "5550173");
        // The written international form still shows the NANP shape.
        assert_eq!(
            antigua.format(PhoneFormat::International),
            "+1 268 555 0173"
        );
        assert_eq!(antigua.format(PhoneFormat::National), "(268) 555-0173");
    }

    #[test]
    fn parse_rejects_invalid_input() {
        assert!(PhoneNumber::parse("nonsense").is_none());
        assert!(PhoneNumber::parse("").is_none());
        assert!(PhoneNumber::parse("+11111111111").is_none());
    }

    #[test]
    fn try_parse_reports_the_reason() {
        assert_eq!(
            PhoneNumber::try_parse("+1202555").unwrap_err(),
            ParseError::InvalidLength
        );
        assert_eq!(
            PhoneNumber::try_parse("HELLO WORLD").unwrap_err(),
            ParseError::InvalidCharacter
        );
        assert_eq!(PhoneNumber::try_parse("").unwrap_err(), ParseError::Empty);
        assert_eq!(
            PhoneNumber::try_parse("+12025550173").unwrap().e164(),
            "+12025550173"
        );
    }

    #[test]
    fn parse_with_country_resolves_national_formats() {
        for (input, hint, expected) in [
            ("020 7946 0958", "GB", "+442079460958"),
            ("2025550173", "US", "+12025550173"),
            ("030 12345678", "DE", "+493012345678"),
            ("01 42 68 53 00", "FR", "+33142685300"),
            ("0412 345 678", "AU", "+61412345678"),
            ("8 916 123-45-67", "RU", "+79161234567"),
            ("090-1234-5678", "JP", "+819012345678"),
            // Italy keeps its leading zero.
            ("06 9812 3456", "IT", "+390698123456"),
        ] {
            let parsed = PhoneNumber::parse_with_country(input, hint)
                .unwrap_or_else(|| panic!("{input:?} with hint {hint} must parse"));
            assert_eq!(parsed.e164(), expected, "{input:?} with hint {hint}");
            assert_eq!(parsed.original(), input, "original must be preserved");
        }
    }

    #[test]
    fn parse_with_country_takes_the_idd_path_when_nothing_else_fits() {
        // "0011" is the Australian IDD prefix; the digits after it spell a UK
        // number, and no Australian reading works.
        let parsed = PhoneNumber::parse_with_country("0011442079460958", "AU").unwrap();
        assert_eq!(parsed.e164(), "+442079460958");
        assert_eq!(parsed.country().code, "GB");
        assert_eq!(parsed.original(), "0011442079460958");
    }

    #[test]
    fn parse_with_country_never_lets_the_hint_contradict_the_number() {
        // An explicit '+' wins over the hint.
        let parsed = PhoneNumber::parse_with_country("+12025550173", "DE").unwrap();
        assert_eq!(parsed.country().code, "US");
        assert_eq!(parsed.country_code(), 1);

        // Under calling code 1 the area code, not the hint, picks the country,
        // and every derived value follows the country that was resolved.
        for (input, hint, expected_country, expected_e164, expected_national) in [
            ("2685550173", "US", "AG", "+12685550173", "5550173"),
            ("4165550198", "US", "CA", "+14165550198", "4165550198"),
            ("2025550173", "CA", "US", "+12025550173", "2025550173"),
        ] {
            let parsed = PhoneNumber::parse_with_country(input, hint).unwrap();
            assert_eq!(parsed.country().code, expected_country, "{input} / {hint}");
            assert_eq!(parsed.e164(), expected_e164);
            assert_eq!(parsed.national_number(), expected_national);
            // The type must be the one the resolved country implies.
            assert_eq!(
                parsed.phone_type(),
                detect_phone_number_type(expected_e164).unwrap(),
                "{input} / {hint}: stale phone_type"
            );
        }
    }

    #[test]
    fn parse_with_country_rejects_an_unknown_hint() {
        assert!(PhoneNumber::parse_with_country("020 7946 0958", "ZZ").is_none());
        assert_eq!(
            PhoneNumber::try_parse_with_country("020 7946 0958", "ZZ").unwrap_err(),
            ParseError::UnknownCountry
        );
        // A typo'd hint must never be more permissive than a correct one.
        assert!(PhoneNumber::parse_with_country("020 7946 0958", "GBR").is_none());
        // Differently-cased is not a typo — it is the same country.
        assert_eq!(
            PhoneNumber::parse_with_country("020 7946 0958", "gb").map(|p| p.e164().to_string()),
            Some("+442079460958".to_string())
        );
        assert!(PhoneNumber::parse_with_country("", "US").is_none());
        assert!(PhoneNumber::parse_with_country("nonsense", "US").is_none());
    }

    #[test]
    fn parse_with_country_keeps_the_extension() {
        let parsed = PhoneNumber::parse_with_country("(202) 555-0173 ext. 42", "US").unwrap();
        assert_eq!(parsed.e164(), "+12025550173");
        assert_eq!(parsed.extension(), Some("42"));
    }

    #[test]
    fn extension_markers() {
        for (input, extension) in [
            ("+1 202-555-0173 ext. 42", Some("42")),
            ("+1 202-555-0173 ext 42", Some("42")),
            ("+1 202-555-0173 EXT:42", Some("42")),
            ("+1 202-555-0173 ext-42", Some("42")),
            ("+1 202-555-0173 extn 42", Some("42")),
            ("+1 202-555-0173 extension 42", Some("42")),
            ("+1 202-555-0173 x42", Some("42")),
            ("+1 202-555-0173 #42", Some("42")),
            ("+1 202-555-0173,42", Some("42")),
            ("+1 202-555-0173;42", Some("42")),
            ("tel:+12025550173;ext=42", Some("42")),
            ("+1 202 555 0173 p42", Some("42")),
            ("+1 202 555 0173 w42", Some("42")),
            // No extension at all.
            ("+1 202-555-0173", None),
            ("+1 202-555-0173 ext.", None),
        ] {
            let parsed =
                PhoneNumber::parse(input).unwrap_or_else(|| panic!("{input:?} must parse"));
            assert_eq!(parsed.extension(), extension, "extension of {input:?}");
            assert_eq!(parsed.e164(), "+12025550173", "e164 of {input:?}");
        }
    }

    #[test]
    fn vanity_letters_convert_but_prose_does_not() {
        let flowers = PhoneNumber::parse("1-800-FLOWERS").unwrap();
        assert_eq!(flowers.e164(), "+18003569377");
        assert_eq!(flowers.extension(), None);

        assert!(PhoneNumber::parse("HELLO WORLD").is_none());
        assert!(PhoneNumber::parse("ILOVEYOUXX").is_none());
    }

    #[test]
    fn equality_and_ordering_follow_the_normalized_form() {
        let a = PhoneNumber::parse("+1 202-555-0173").unwrap();
        let b = PhoneNumber::parse("+1 (202) 555-0173").unwrap();
        let c = PhoneNumber::parse("+12025550174").unwrap();

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a < c);
        assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
        assert_eq!(a.partial_cmp(&c), Some(std::cmp::Ordering::Less));

        // The original spelling is retained even though it does not affect
        // equality.
        assert_eq!(a.original(), "+1 202-555-0173");
        assert_eq!(b.original(), "+1 (202) 555-0173");

        let mut sorted = [c.clone(), a.clone()];
        sorted.sort();
        assert_eq!(sorted[0].e164(), "+12025550173");
    }

    #[test]
    fn equal_numbers_hash_equally() {
        fn hash_of(value: &PhoneNumber) -> u64 {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        }

        let a = PhoneNumber::parse("+1 202-555-0173").unwrap();
        let b = PhoneNumber::parse("+1 (202) 555-0173").unwrap();
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));

        let set: std::collections::HashSet<PhoneNumber> = [a, b].into_iter().collect();
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn standard_trait_impls() {
        let parsed = PhoneNumber::parse("+1 (202) 555-0173").unwrap();

        assert_eq!(parsed.to_string(), "+12025550173");
        assert_eq!(format!("{parsed}"), "+12025550173");
        assert!(format!("{parsed:?}").contains("+12025550173"));
        assert_eq!(<PhoneNumber as AsRef<str>>::as_ref(&parsed), "+12025550173");
        assert_eq!(parsed.clone(), parsed);

        let from_str = PhoneNumber::from_str("+12025550173").unwrap();
        assert_eq!(from_str.e164(), "+12025550173");
        assert_eq!(
            "nonsense".parse::<PhoneNumber>().unwrap_err(),
            ParseError::InvalidCharacter
        );

        let try_from_ref = PhoneNumber::try_from("+12025550173").unwrap();
        assert_eq!(try_from_ref.e164(), "+12025550173");
        let try_from_owned = PhoneNumber::try_from("+12025550173".to_string()).unwrap();
        assert_eq!(try_from_owned.e164(), "+12025550173");
        assert_eq!(
            PhoneNumber::try_from("nonsense").unwrap_err(),
            ParseError::InvalidCharacter
        );
    }

    #[test]
    fn is_predicates_return_the_right_answer_not_just_a_value() {
        let mobile = PhoneNumber::parse("+447911123456").unwrap();
        assert!(mobile.is_mobile());
        assert!(!mobile.is_landline());
        assert!(!mobile.is_toll_free());

        let landline = PhoneNumber::parse("+442079460958").unwrap();
        assert!(!landline.is_mobile());
        assert!(landline.is_landline());
        assert!(!landline.is_toll_free());

        let toll_free = PhoneNumber::parse("+448001234567").unwrap();
        assert!(!toll_free.is_mobile());
        assert!(!toll_free.is_landline());
        assert!(toll_free.is_toll_free());

        // A NANP geographic number is neither, because its range carries both.
        let nanp = PhoneNumber::parse("+12025550173").unwrap();
        assert!(!nanp.is_mobile());
        assert!(!nanp.is_landline());
        assert!(!nanp.is_toll_free());
    }

    #[test]
    fn format_agrees_with_the_free_function() {
        for &(_, number) in COUNTRY_FIXTURE {
            let parsed = PhoneNumber::parse(number).unwrap();
            for format in [
                PhoneFormat::E164,
                PhoneFormat::International,
                PhoneFormat::National,
                PhoneFormat::RFC3966,
            ] {
                assert_eq!(
                    Some(parsed.format(format)),
                    format_phone_number(number, format),
                    "{number} / {format:?}"
                );
            }
        }
    }
}

// ============================================================================
// PhoneNumberSet
// ============================================================================

mod phone_number_set {
    use super::*;

    #[test]
    fn add_deduplicates_by_normalized_form() {
        let mut set = PhoneNumberSet::new();
        assert!(set.add("+1-202-555-0173"));
        assert!(!set.add("+1 (202) 555-0173"));
        assert_eq!(set.len(), 1);
        assert!(!set.is_empty());
    }

    #[test]
    fn try_add_distinguishes_a_duplicate_from_a_rejection() {
        let mut set = PhoneNumberSet::new();
        assert_eq!(set.try_add("+12025550173"), Ok(true));
        assert_eq!(set.try_add("+1 202-555-0173"), Ok(false));
        assert_eq!(set.try_add("nonsense"), Err(ParseError::InvalidCharacter));
        assert_eq!(set.try_add(""), Err(ParseError::Empty));
        assert_eq!(set.len(), 1);

        // `add` collapses both failure modes to `false`, as documented.
        assert!(!set.add("nonsense"));
        assert!(!set.add("+1 202-555-0173"));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn contains_and_get_accept_any_formatting() {
        let mut set = PhoneNumberSet::new();
        assert!(set.add("+1-202-555-0173"));

        assert!(set.contains("+12025550173"));
        assert!(set.contains("+1 (202) 555-0173"));
        assert!(set.contains("tel:+12025550173"));
        assert!(!set.contains("+12025550174"));
        assert!(!set.contains("nonsense"));

        let stored = set.get("+1 (202) 555-0173").expect("must be found");
        assert_eq!(stored.e164(), "+12025550173");
        // `get` returns the value as it was stored, not the lookup spelling.
        assert_eq!(stored.original(), "+1-202-555-0173");
        assert!(set.get("+12025550174").is_none());
        assert!(set.get("nonsense").is_none());
    }

    #[test]
    fn remove_accepts_any_formatting() {
        let mut set = PhoneNumberSet::new();
        assert!(set.add("+12025550173"));
        assert!(set.remove("12025550173"));
        assert!(set.is_empty());
        assert!(!set.remove("12025550173"));
        assert!(!set.remove("nonsense"));
    }

    #[test]
    fn normalized_numbers_lists_every_stored_key() {
        let mut set = PhoneNumberSet::new();
        assert!(set.add("+1-202-555-0173"));
        assert!(set.add("+44 20 7946 0958"));

        let mut normalized = set.normalized_numbers();
        normalized.sort_unstable();
        assert_eq!(normalized, ["+12025550173", "+442079460958"]);
        assert_eq!(normalized.len(), set.len());

        assert!(PhoneNumberSet::new().normalized_numbers().is_empty());
    }

    #[test]
    fn iteration_by_reference_and_by_value() {
        let mut set = PhoneNumberSet::new();
        assert!(set.add("+12025550173"));
        assert!(set.add("+442079460958"));

        assert_eq!(set.iter().count(), 2);

        let mut by_ref: Vec<&str> = (&set).into_iter().map(|p| p.e164()).collect();
        by_ref.sort_unstable();
        assert_eq!(by_ref, ["+12025550173", "+442079460958"]);

        let mut by_value: Vec<String> = set
            .clone()
            .into_iter()
            .map(|p| p.e164().to_string())
            .collect();
        by_value.sort();
        assert_eq!(by_value, ["+12025550173", "+442079460958"]);

        // `for n in &set` must compile.
        let mut seen = 0;
        for number in &set {
            assert!(number.e164().starts_with('+'));
            seen += 1;
        }
        assert_eq!(seen, 2);
    }

    #[test]
    fn default_is_an_empty_set() {
        let set = PhoneNumberSet::default();
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
        assert!(set.normalized_numbers().is_empty());
        assert_eq!(set, PhoneNumberSet::new());
    }

    #[test]
    fn extend_and_from_iterator() {
        let mut set = PhoneNumberSet::new();
        set.extend(["+12025550173", "+442079460958", "nonsense"]);
        assert_eq!(set.len(), 2, "invalid input must be skipped, not stored");

        let from_str_refs: PhoneNumberSet = ["+12025550173", "12025550173", "+442079460958"]
            .into_iter()
            .collect();
        assert_eq!(from_str_refs.len(), 2);

        let from_owned: PhoneNumberSet = vec![
            "+12025550173".to_string(),
            "+1 (202) 555-0173".to_string(),
            "+442079460958".to_string(),
        ]
        .into_iter()
        .collect();
        assert_eq!(from_owned.len(), 2);

        assert_eq!(from_str_refs, from_owned);
    }

    #[test]
    fn equality_and_clone() {
        let mut a = PhoneNumberSet::new();
        assert!(a.add("+12025550173"));
        let b = a.clone();
        assert_eq!(a, b);

        let mut c = PhoneNumberSet::new();
        assert!(c.add("+442079460958"));
        assert_ne!(a, c);

        assert!(format!("{a:?}").contains("+12025550173"));
    }
}

// ============================================================================
// Regressions
//
// One test per fixed audit finding. Each names the defect it pins; every one
// of them fails against the pre-rewrite code.
// ============================================================================

mod regressions {
    use super::*;

    // --- parsing and normalization -----------------------------------------

    /// Trunk-zero stripping used to corrupt canonical Italian E.164 numbers
    /// into different, valid numbers, so `+390698123456` became
    /// `+39698123456` and the two compared equal.
    #[test]
    fn italian_leading_zero_is_significant() {
        assert_eq!(
            normalize_phone_number("+390698123456").as_deref(),
            Some("+390698123456")
        );
        assert!(!are_phone_numbers_equal("+390698123456", "+39698123456"));
        assert_eq!(
            PhoneNumber::parse_with_country("+39 06 9812 3456", "IT")
                .unwrap()
                .e164(),
            "+390698123456"
        );
        // The Vatican shares the Italian plan, and `LEADING_ZERO_SIGNIFICANT`
        // lists "VA" — but VA's `[10]` is wholly inside IT's `[6..=11]`, so no
        // number ever resolves to VA and the entry is belt-and-braces. A VA hint
        // returns an Italian number; that is what is actually observable.
        let vatican = PhoneNumber::parse_with_country("06 6982 1234", "VA")
            .expect("a VA hint must still parse");
        assert_eq!(vatican.country().code, "IT");
        assert_eq!(vatican.e164(), "+390669821234");
        assert_eq!(
            extract_countries("+390669821234")
                .iter()
                .map(|c| c.code)
                .collect::<Vec<_>>(),
            ["IT", "VA"],
            "VA is only ever reachable as a secondary candidate"
        );
    }

    /// Benin, Côte d'Ivoire and San Marino also keep a significant leading zero,
    /// and used to have it stripped: the shortened number then failed the length
    /// check, so every one of these came back as *invalid* rather than as a
    /// different number.
    #[test]
    fn other_plans_that_keep_a_significant_leading_zero() {
        for (number, expected_country) in [
            // Benin's 2024 renumbering prepended `01` to every number.
            ("+2290197123456", "BJ"),
            ("+2290112345678", "BJ"),
            // Côte d'Ivoire's flat 10-digit plan: `01`, `05` and `07` mobiles.
            ("+2250712345678", "CI"),
            ("+2250123456789", "CI"),
            // San Marino's fixed-line plan is `0549 xxxxxx`.
            ("+3780549123456", "SM"),
        ] {
            assert!(is_valid_phone_number(number), "{number} must be valid");
            assert_eq!(
                normalize_phone_number(number).as_deref(),
                Some(number),
                "{number} lost its leading zero"
            );
            assert_eq!(iso(number), expected_country, "{number}");
        }
        // San Marino mobiles carry no leading zero and are eight digits.
        assert_eq!(iso("+37866123456"), "SM");
    }

    /// Gabon is deliberately *not* on that list. It writes `0XX XX XX XX`
    /// nationally, but the ITU is explicit that international callers dial the
    /// eight digits after the zero, so the zero is an ordinary trunk prefix.
    /// Adding `"GA"` to `LEADING_ZERO_SIGNIFICANT` makes the second number here
    /// validate as a nine-digit Gabonese number that does not exist.
    #[test]
    fn a_trunk_zero_is_not_promoted_to_a_significant_one() {
        assert_eq!(country_by_code("GA").unwrap().phone_lengths, &[8]);
        assert!(is_valid_phone_number("+24111441234"));
        assert_eq!(iso("+24111441234"), "GA");
        assert!(
            !is_valid_phone_number("+24101441234"),
            "Gabon's leading zero is a trunk prefix, not part of the number"
        );
    }

    /// Everywhere else the leading zero is a national trunk prefix and is not
    /// part of the E.164 number.
    #[test]
    fn trunk_zeros_are_stripped_outside_italy() {
        assert_eq!(
            normalize_phone_number("+44 (0)20 7946 0958").as_deref(),
            Some("+442079460958")
        );
        assert_eq!(
            normalize_phone_number("+49 (0)30 12345678").as_deref(),
            Some("+493012345678")
        );
        assert_eq!(
            normalize_phone_number("+33 (0)1 42 68 53 00").as_deref(),
            Some("+33142685300")
        );
    }

    /// Unhandled extension markers used to be vanity-converted into digits and
    /// glued onto the subscriber number, silently producing a different valid
    /// number.
    #[test]
    fn extension_markers_are_never_vanity_converted() {
        assert_eq!(
            normalize_phone_number("+43 1 2345678 x1").as_deref(),
            Some("+4312345678")
        );
        assert_eq!(
            normalize_phone_number("+43 1 2345678 ext-1").as_deref(),
            Some("+4312345678")
        );
        assert_eq!(
            normalize_phone_number("+39 06 9812345 x").as_deref(),
            Some("+39069812345")
        );
        // A trailing letter that is not an extension marker is not silently
        // absorbed either; the number is simply the wrong length.
        assert_eq!(err("+1202555017x"), ParseError::InvalidLength);
    }

    /// Only `ext.` and `ext ` used to be recognised; every other spelling was
    /// rejected or mis-parsed.
    #[test]
    fn all_documented_extension_markers_parse() {
        for input in [
            "+1 202-555-0173 ext. 42",
            "+1 202-555-0173 ext 42",
            "+1 202-555-0173 ext:42",
            "+1 202-555-0173 ext-42",
            "+1 202-555-0173 extn 42",
            "+1 202-555-0173 extension 42",
            "+1 202-555-0173 EXT-42",
            "+1 202-555-0173 x42",
            "+1 202-555-0173 #42",
            "+1 202-555-0173,42",
            "+1 202-555-0173;42",
            "tel:+12025550173;ext=42",
        ] {
            let parsed =
                PhoneNumber::parse(input).unwrap_or_else(|| panic!("{input:?} must parse"));
            assert_eq!(parsed.e164(), "+12025550173", "{input:?}");
            assert_eq!(parsed.extension(), Some("42"), "{input:?}");
        }
    }

    /// `contains_invalid_character` used to allow letters anywhere, so
    /// arbitrary prose validated as a phone number.
    #[test]
    fn prose_is_not_a_phone_number() {
        for prose in [
            "HELLO WORLD",
            "CALL ME NOW PLZ",
            "ILOVEYOUXX",
            "hello 12025550173",
        ] {
            assert!(!is_valid_phone_number(prose), "{prose:?} must be rejected");
            assert_eq!(normalize_phone_number(prose), None, "{prose:?}");
        }
        // Vanity conversion still works once the input looks like a number.
        assert_eq!(
            normalize_phone_number("1-800-FLOWERS").as_deref(),
            Some("+18003569377")
        );
        assert_eq!(
            normalize_phone_number("1-800-GOT-JUNK").as_deref(),
            Some("+18004685865")
        );
    }

    /// The prose guard is a threshold: `extract_digits` converts vanity letters
    /// only once at least three digits precede the first letter
    /// (`VANITY_MIN_LEADING_DIGITS`). Every other negative case in this suite
    /// has *zero* leading digits and every positive one has four, so the
    /// boundary itself needs pinning from both sides — loosening the constant to
    /// 2 or 1 would otherwise ship undetected.
    #[test]
    fn vanity_conversion_needs_three_leading_digits() {
        // One and two leading digits are prose, and are rejected before any
        // numbering plan is consulted.
        assert_eq!(err("1CALLNOWXXX"), ParseError::InvalidCharacter);
        assert_eq!(err("12FLOWERSXX"), ParseError::InvalidCharacter);
        assert_eq!(err("12FLOWERS"), ParseError::InvalidCharacter);

        // Exactly three gets past the character check and is judged on its
        // digits alone: this one converts to +1 233 569 3779, whose area code
        // 233 is not assignable.
        assert_eq!(err("123FLOWERSX"), ParseError::InvalidAreaCode);

        // …and when the digits do describe a real number, it parses.
        assert_eq!(
            normalize_phone_number("+493AAJJMMWW").as_deref(),
            Some("+49322556699")
        );
        assert_eq!(
            normalize_phone_number("+49322556699").as_deref(),
            Some("+49322556699"),
            "the vanity spelling and the digits must agree"
        );

        // Four or more is the ordinary case.
        assert!(is_valid_phone_number("1-800-FLOWERS"));
    }

    /// Non-ASCII digits used to be silently deleted, changing which number was
    /// parsed. They are now rejected, and only fullwidth forms are folded.
    #[test]
    fn stray_unicode_digits_are_rejected_not_deleted() {
        for input in [
            "+1\u{0660}2025550173",
            "+1202555017\u{0663}3",
            "+43\u{0661}12345678",
        ] {
            assert_eq!(
                try_normalize_phone_number(input).err(),
                Some(ParseError::InvalidCharacter),
                "{input:?}"
            );
        }
        // Fullwidth digits are a formatting variant, not a foreign script.
        assert_eq!(
            normalize_phone_number("\u{FF0B}\u{FF18}\u{FF11}\u{FF13}\u{FF11}\u{FF12}\u{FF13}\u{FF14}\u{FF15}\u{FF16}\u{FF17}\u{FF18}").as_deref(),
            Some("+81312345678")
        );
    }

    /// `is_valid_phone_number` and `normalize_phone_number` used to disagree on
    /// a large class of inputs. They now run the same parser.
    #[test]
    fn validation_and_normalization_never_disagree() {
        let corpus = [
            "+1/202/555/0173",
            "+1_202_555_0173",
            "+1@202#555$0173",
            "+1\n202\t555 0173",
            "000000000012025550173",
            "0000000000+12025550173",
            "+0000000000442079460958",
            "0000000000000000000012025550173",
            "00000000001202555017",
            "1-800-FLOWERS",
            "HELLO WORLD",
            "+1 (234 567-8990",
            "(((((",
            "",
        ];
        for input in corpus {
            assert_eq!(
                is_valid_phone_number(input),
                normalize_phone_number(input).is_some(),
                "is_valid_phone_number and normalize_phone_number disagree on {input:?}"
            );
        }
    }

    /// `normalize_phone_number` and `normalize_phone_number_in_place` used to
    /// diverge once the input exceeded the internal 20-digit buffer, because
    /// one counted leading zeros against the cap and the other did not.
    #[test]
    fn in_place_agrees_with_the_borrowing_form_on_long_zero_runs() {
        for input in [
            "000000000012025550173",
            "0000000000000000000012025550173",
            "+0000000000442079460958",
            "000 000 000 44 20 7946 0958",
            "00000000001202555017",
        ] {
            let borrowed = normalize_phone_number(input);
            let mut buffer = input.to_string();
            let in_place = normalize_phone_number_in_place(&mut buffer);
            assert_eq!(borrowed, in_place, "{input:?}");
            assert_eq!(
                is_valid_phone_number(input),
                borrowed.is_some(),
                "is_valid disagrees on {input:?}"
            );
        }
    }

    /// `normalize_phone_number_in_place` used to leave the bare national number
    /// in the caller's buffer on success and vanity-converted garbage on
    /// failure, and was not idempotent.
    #[test]
    fn in_place_buffer_semantics() {
        let mut buffer = "+1 (202) 555-0173".to_string();
        assert_eq!(
            normalize_phone_number_in_place(&mut buffer).as_deref(),
            Some("+12025550173")
        );
        assert_eq!(buffer, "+12025550173");
        assert_eq!(
            normalize_phone_number_in_place(&mut buffer).as_deref(),
            Some("+12025550173"),
            "a second call must be a no-op"
        );
        assert_eq!(buffer, "+12025550173");

        let mut failed = "hello ext. 5".to_string();
        assert_eq!(normalize_phone_number_in_place(&mut failed), None);
        assert_eq!(failed, "hello ext. 5", "the buffer must survive a failure");
    }

    /// A `+` used to be accepted at any position by the digit scanner while the
    /// character validator rejected it, so the two disagreed.
    #[test]
    fn a_plus_is_only_accepted_as_the_first_character() {
        assert!(is_valid_phone_number("+12025550173"));
        assert_eq!(err("1+2025550173"), ParseError::InvalidCharacter);
        assert_eq!(err("12025550173+"), ParseError::InvalidCharacter);
        assert_eq!(err("++12025550173"), ParseError::InvalidCharacter);
    }

    /// A parenthesised calling code — the form used by every North American
    /// style guide for an international number — ended the candidate at the
    /// `'('`, so the `+44` was dropped and the national digits were
    /// re-attributed to whatever country they happened to fit.
    #[test]
    fn a_parenthesised_calling_code_is_part_of_the_number() {
        for (input, expected) in [
            ("(+44) 20 7946 0958", "+442079460958"),
            ("(+1) 202 555 0173", "+12025550173"),
            ("(+49) 30 12345678", "+493012345678"),
        ] {
            assert_eq!(
                normalize_phone_number(input).as_deref(),
                Some(expected),
                "{input:?}"
            );
            let numbers = extract_phone_numbers_from_text(input);
            assert_eq!(numbers.len(), 1, "{input:?}");
            assert_eq!(numbers[0].raw, input, "{input:?}");
            assert_eq!(
                numbers[0].normalized.as_deref(),
                Some(expected),
                "{input:?}"
            );
        }
        // A `+` anywhere else is still rejected.
        assert_eq!(err("+1 (+202) 555 0173"), ParseError::InvalidCharacter);
    }

    /// A NANP hint used to accept only the bare ten digits. All four written
    /// forms a North American caller actually uses now resolve, and the area
    /// code — not the hint — still decides which territory the number belongs
    /// to.
    #[test]
    fn nanp_hints_accept_every_written_form() {
        for (input, hint, expected_e164, expected_country) in [
            // Bare ten digits.
            ("202 555 0173", "US", "+12025550173", "US"),
            // The long-distance trunk `1`.
            ("1-202-555-0173", "US", "+12025550173", "US"),
            ("1 (416) 555-0198", "CA", "+14165550198", "CA"),
            // A territory hint plus a local seven-digit number: the hint's own
            // area code is what completes it.
            ("555-0173", "AG", "+12685550173", "AG"),
            ("555-0123", "JM", "+18765550123", "JM"),
            // The NANP IDD prefix `011` in front of a foreign number.
            ("011 44 20 7946 0958", "US", "+442079460958", "GB"),
        ] {
            let parsed = PhoneNumber::parse_with_country(input, hint)
                .unwrap_or_else(|| panic!("{input:?} with hint {hint} must parse"));
            assert_eq!(parsed.e164(), expected_e164, "{input:?} / {hint}");
            assert_eq!(
                parsed.country().code,
                expected_country,
                "{input:?} / {hint}"
            );
        }
    }

    /// A leading `00` or `0011` is unambiguously an international prefix — no
    /// national number starts with two zeros — so it is honoured before any
    /// national reading. `011` is not: it is a real area code in several plans,
    /// so it is only tried once every national reading has failed. Japan's
    /// `0114…` is Sapporo, not `011` + Austria.
    #[test]
    fn idd_prefixes_are_ordered_against_the_national_readings() {
        for (input, hint, expected, expected_country) in [
            ("00 44 20 7946 0958", "DE", "+442079460958", "GB"),
            ("0011 44 20 7946 0958", "AU", "+442079460958", "GB"),
            // `011` after the national readings: both of these are Japanese.
            ("0114567890", "JP", "+81114567890", "JP"),
            ("011 123 4567", "JP", "+81111234567", "JP"),
            // …but it is still tried when nothing national fits.
            ("011 44 20 7946 0958", "JP", "+442079460958", "GB"),
        ] {
            let parsed = PhoneNumber::parse_with_country(input, hint)
                .unwrap_or_else(|| panic!("{input:?} with hint {hint} must parse"));
            assert_eq!(parsed.e164(), expected, "{input:?} / {hint}");
            assert_eq!(
                parsed.country().code,
                expected_country,
                "{input:?} / {hint}"
            );
        }
    }

    /// `original()` is the input exactly as it was supplied. The hinted parser
    /// works on a trimmed, scheme-stripped copy and used to store *that* copy,
    /// so surrounding whitespace and a `tel:` scheme were silently lost.
    #[test]
    fn original_is_preserved_verbatim() {
        for (input, hint) in [
            ("  +1 (202) 555-0173  ", "US"),
            ("\t020 7946 0958\n", "GB"),
            ("tel:+12025550173", "DE"),
            ("1-202-555-0173", "US"),
            ("0011442079460958", "AU"),
        ] {
            let parsed = PhoneNumber::parse_with_country(input, hint)
                .unwrap_or_else(|| panic!("{input:?} with hint {hint} must parse"));
            assert_eq!(parsed.original(), input, "hinted parse of {input:?}");
        }
        for input in [
            "  +1 (202) 555-0173  ",
            "+1 202-555-0173\n",
            "tel:+12025550173",
        ] {
            let parsed =
                PhoneNumber::parse(input).unwrap_or_else(|| panic!("{input:?} must parse"));
            assert_eq!(parsed.original(), input, "plain parse of {input:?}");
        }
    }

    /// A redundant trunk zero after an explicit country code used to work only
    /// for countries whose length list happened to contain both L and L+1.
    #[test]
    fn a_redundant_trunk_zero_after_the_calling_code_is_handled_uniformly() {
        for (input, expected) in [
            ("+49 0 30 12345678", "+493012345678"),
            ("+44 0 20 7946 0958", "+442079460958"),
            ("+33 0 1 42 68 53 00", "+33142685300"),
        ] {
            assert_eq!(
                normalize_phone_number(input).as_deref(),
                Some(expected),
                "{input:?}"
            );
        }
    }

    // --- country resolution -------------------------------------------------

    /// Thirteen countries — including CA, GB, RU and MA — used to be
    /// unreachable because another entry sharing their calling code shadowed
    /// them.
    #[test]
    fn shadowed_countries_are_reachable_again() {
        for (number, expected) in [
            ("+442079460958", "GB"),
            ("+79161234567", "RU"),
            ("+212612345678", "MA"),
            ("+358401234567", "FI"),
            ("+14165550198", "CA"),
            ("+590590123456", "GP"),
            ("+672312345", "NF"),
            ("+262639123456", "RE"),
            ("+4791234567", "NO"),
        ] {
            assert_eq!(iso(number), expected, "{number}");
        }
    }

    /// The fabricated `GB-CYM` / "Cymru" entry hijacked every UK number and
    /// made the GB classification and formatting tables dead code.
    #[test]
    fn the_cymru_pseudo_country_is_gone() {
        assert!(country_by_code("GB-CYM").is_none());
        assert!(countries().iter().all(|c| c.code != "GB-CYM"));
        assert!(countries().iter().all(|c| c.name != "Cymru"));

        assert_eq!(iso("+442079460958"), "GB");
        assert_eq!(
            generate_random_phone_number("GB")
                .and_then(|n| extract_country(&n))
                .map(|c| c.code),
            Some("GB")
        );
        // The GB range table is live again.
        assert_eq!(
            detect_phone_number_type("+448001234567"),
            Some(PhoneNumberType::TollFree)
        );
        assert!(is_toll_free_number("+448001234567"));
        assert_eq!(
            detect_phone_number_type("+443001234567"),
            Some(PhoneNumberType::Uan)
        );
        assert_eq!(
            detect_phone_number_type("+445612345678"),
            Some(PhoneNumberType::Voip)
        );
        // And so is the GB format table.
        assert_eq!(
            format_phone_number("+442079460958", PhoneFormat::National).as_deref(),
            Some("020 7946 0958")
        );
    }

    /// The Åland Islands entry used to swallow every Finnish number of length
    /// 5-10.
    #[test]
    fn aland_no_longer_swallows_finnish_numbers() {
        assert_eq!(iso("+358401234567"), "FI");
        assert_eq!(iso("+35891234567"), "FI");
        assert_eq!(
            generate_random_phone_number("FI")
                .and_then(|n| extract_country(&n))
                .map(|c| c.code),
            Some("FI")
        );
    }

    /// Guernsey and Jersey carried `phone_lengths: [6]`; real GG/JE numbers
    /// have a 10-digit national significant number.
    #[test]
    fn crown_dependency_lengths_are_realistic() {
        for code in ["GG", "JE", "IM"] {
            let country = country_by_code(code).unwrap_or_else(|| panic!("{code} missing"));
            assert_eq!(country.prefix, 44, "{code}");
            assert_eq!(country.phone_lengths, &[10], "{code}");
        }
        // Their numbers parse, even though +44 resolves to GB by default.
        assert!(is_valid_phone_number("+441481123456"));
        assert!(is_valid_phone_number("+441534123456"));
        assert!(extract_countries("+441481123456")
            .iter()
            .any(|c| c.code == "GG"));
    }

    /// Uninhabited territories with fabricated numbering plans produced pure
    /// false accepts on +47, +672 and +262.
    #[test]
    fn uninhabited_territories_no_longer_accept_numbers() {
        for input in [
            "+471234567890",
            "+2621234567890",
            "+6721234567890",
            "+6721234",
        ] {
            assert!(!is_valid_phone_number(input), "{input} must be rejected");
        }
        for code in ["BV", "TF", "HM", "AQ"] {
            assert!(
                country_by_code(code).is_none(),
                "{code} must not be in the table"
            );
        }
    }

    /// Japan rejected every 9-digit landline and freephone number.
    #[test]
    fn japanese_nine_digit_numbers_are_accepted() {
        assert_eq!(country_by_code("JP").unwrap().phone_lengths, &[9, 10, 11]);
        assert_eq!(iso("+81312345678"), "JP");
        assert_eq!(iso("+81120123456"), "JP");
        assert!(is_valid_phone_number("+819012345678"));
    }

    /// Romania's `phone_lengths: [10]` rejected every real Romanian number.
    #[test]
    fn romanian_numbers_are_nine_digits() {
        assert_eq!(country_by_code("RO").unwrap().phone_lengths, &[9]);
        assert_eq!(iso("+40712345678"), "RO");
        assert_eq!(iso("+40212345678"), "RO");
    }

    /// Egypt, the UAE, Thailand and the Philippines omitted their landline
    /// national lengths.
    #[test]
    fn landline_lengths_are_present_for_eg_ae_th_ph() {
        for (number, expected) in [
            ("+201012345678", "EG"),
            ("+20212345678", "EG"),
            ("+971501234567", "AE"),
            ("+97142345678", "AE"),
            ("+66812345678", "TH"),
            ("+6621234567", "TH"),
            ("+639171234567", "PH"),
            ("+6321234567", "PH"),
        ] {
            assert_eq!(iso(number), expected, "{number}");
        }
    }

    /// Jamaica's 658 overlay was missing, so `+1 658` numbers were attributed
    /// to the United States.
    #[test]
    fn jamaicas_658_overlay_is_present() {
        assert_eq!(iso("+18765550123"), "JM");
        assert_eq!(iso("+16585550123"), "JM");
    }

    /// Kosovo was absent entirely and Bonaire was folded into Curaçao.
    #[test]
    fn kosovo_and_bonaire_are_modelled() {
        assert_eq!(country_by_code("XK").map(|c| c.prefix), Some(383));
        assert_eq!(iso("+38344123456"), "XK");
        assert_eq!(country_by_code("BQ").map(|c| c.prefix), Some(599));
        assert_eq!(iso("+5997001234"), "BQ");
    }

    /// Austria's maximum national length let a 16-digit number exceed the
    /// E.164 ceiling.
    #[test]
    fn no_country_can_exceed_the_e164_ceiling() {
        assert_eq!(err("+4312345678901234"), ParseError::TooLong);
        for country in countries() {
            let prefix_digits = country.prefix.to_string().len();
            let longest = *country.phone_lengths.iter().max().unwrap() as usize;
            assert!(
                prefix_digits + longest <= 15,
                "{} can produce a {}-digit E.164 number",
                country.code,
                prefix_digits + longest
            );
        }
    }

    /// Pitcairn was assigned +870, the Inmarsat SNAC, which is not a country
    /// calling code.
    #[test]
    fn the_inmarsat_snac_is_not_a_country() {
        assert_eq!(err("+8709123456"), ParseError::UnknownCountryCode);
        assert!(countries().iter().all(|c| c.prefix != 870));
    }

    /// The Dominican Republic and Puerto Rico could only ever reach one of
    /// their overlay calling codes.
    #[test]
    fn nanp_overlay_calling_codes_are_all_present() {
        assert_eq!(
            countries_by_code("DO")
                .iter()
                .map(|c| c.prefix)
                .collect::<Vec<_>>(),
            [1809, 1829, 1849]
        );
        assert_eq!(
            countries_by_code("PR")
                .iter()
                .map(|c| c.prefix)
                .collect::<Vec<_>>(),
            [1787, 1939]
        );
        for number in ["+18095550123", "+18295550123", "+18495550123"] {
            assert_eq!(iso(number), "DO", "{number}");
        }
        for number in ["+17875550123", "+19395550123"] {
            assert_eq!(iso(number), "PR", "{number}");
        }
    }

    /// Only the national number's *length* was validated, so 93% of random
    /// digit strings of a plausible length were accepted.
    #[test]
    fn nanp_numbers_are_range_validated_not_just_length_validated() {
        assert_eq!(err("+11111111111"), ParseError::InvalidAreaCode);
        assert_eq!(err("+11231231234"), ParseError::InvalidAreaCode);
        assert_eq!(err("+10005550199"), ParseError::InvalidAreaCode);
        assert_eq!(err("+12021110173"), ParseError::InvalidExchange);
        assert_eq!(err("+12020000173"), ParseError::InvalidExchange);
        assert!(is_valid_phone_number("+12025550173"));
    }

    /// NANP tests used to pin Canadian area codes to the United States and
    /// accepted the non-assignable area code 123.
    #[test]
    fn canadian_area_codes_resolve_to_canada() {
        for number in [
            "+14165551234",
            "+15145551234",
            "+16045551234",
            "+16135551234",
        ] {
            assert_eq!(iso(number), "CA", "{number}");
        }
        assert_eq!(err("+11231231234"), ParseError::InvalidAreaCode);
    }

    /// `guess_country_from_number` inherited the shadowing bug and hard-coded a
    /// US/GB/DE fallback that made it never return `None`.
    #[test]
    fn guessing_uses_the_real_calling_code_table() {
        assert_eq!(
            guess_country_from_number("442079460958").map(|c| c.code),
            Some("GB")
        );
        assert_eq!(
            guess_country_from_number("79161234567").map(|c| c.code),
            Some("RU")
        );
        assert_eq!(guess_country_from_number("").map(|c| c.code), None);

        // It never contradicts extract_country for a valid number.
        for &(_, number) in COUNTRY_FIXTURE {
            assert_eq!(
                guess_country_from_number(number).map(|c| c.code),
                extract_country(number).map(|c| c.code),
                "{number}"
            );
        }
    }

    /// Two invariants of the data tables are enforced by `const _: () = { … }`
    /// blocks in src/lib.rs and so are checked at *compile* time: every
    /// `PRIMARY_FOR_PREFIX` entry must name a country that exists in
    /// `COUNTRIES` under that calling code, and every `1XXX` entry in
    /// `COUNTRIES` must have a matching `nanp::NANP_AREA_CODES` row. A `const`
    /// assert cannot be observed from a runtime test — breaking either one stops
    /// the crate compiling, so this suite never gets to run — which is why there
    /// is no runtime check for them here and none should be added. What *is*
    /// worth testing is the behaviour they protect, and that is what
    /// `countries_by_calling_code_puts_the_primary_first` and
    /// `nanp_overlay_calling_codes_are_all_present` do.
    #[test]
    fn the_country_table_is_publicly_accessible() {
        assert!(countries().len() > 200);
        assert!(countries().iter().any(|c| c.code == "JP"));
        assert_eq!(country_by_code("DE").map(|c| c.prefix), Some(49));
        assert!(!countries_by_code("DO").is_empty());
        assert!(!countries_by_calling_code(44).is_empty());
        assert!(!extract_countries("+79161234567").is_empty());
    }

    // --- classification -----------------------------------------------------

    /// The UK branch of `classify_phone_number_type` was dead code, so 0800
    /// numbers came back as `Mobile`.
    #[test]
    fn uk_service_numbers_classify_correctly() {
        for (number, expected) in [
            ("+448001234567", PhoneNumberType::TollFree),
            ("+448451234567", PhoneNumberType::SharedCost),
            ("+443001234567", PhoneNumberType::Uan),
            ("+445612345678", PhoneNumberType::Voip),
            ("+449091234567", PhoneNumberType::PremiumRate),
            ("+447911123456", PhoneNumberType::Mobile),
            ("+442079460958", PhoneNumberType::FixedLine),
        ] {
            assert_eq!(detect_phone_number_type(number), Some(expected), "{number}");
        }
        assert!(is_toll_free_number("+448001234567"));
    }

    /// Germany's toll-free arm was unreachable because leading zeros were
    /// stripped before classification, so +49 800 came back as `FixedLine`.
    #[test]
    fn german_toll_free_numbers_classify_as_toll_free() {
        assert_eq!(
            detect_phone_number_type("+498001234567"),
            Some(PhoneNumberType::TollFree)
        );
        assert_eq!(
            detect_phone_number_type("+491801234567"),
            Some(PhoneNumberType::SharedCost)
        );
        assert_eq!(
            detect_phone_number_type("+499001234567"),
            Some(PhoneNumberType::PremiumRate)
        );
    }

    /// Only six countries had effective type tables; everything else fell
    /// through to a single global first-digit guess that inverted the answer for
    /// CN, IT, BR and IN.
    ///
    /// Each country now has its own arm. Most are range tables keyed on the
    /// first two to four digits (CN, IT, BR); the fall-through inside the
    /// `IN`/`CN`/`ZA`/`NG` arms is still a deliberately coarse first-digit
    /// heuristic once the named ranges have been tried, so the rows below use
    /// numbers whose answer does not depend on which of the two decided it.
    /// [`indian_metro_landlines_are_not_classified_as_mobile`] covers the one
    /// place where that coarseness needed an explicit exception.
    #[test]
    fn each_country_has_its_own_classification_arm() {
        for (number, expected) in [
            ("+8613800138000", PhoneNumberType::Mobile),
            ("+862112345678", PhoneNumberType::FixedLine),
            ("+390669821234", PhoneNumberType::FixedLine),
            ("+393331234567", PhoneNumberType::Mobile),
            ("+5511987654321", PhoneNumberType::Mobile),
            ("+551133334444", PhoneNumberType::FixedLine),
            ("+919876543210", PhoneNumberType::Mobile),
            ("+911123456789", PhoneNumberType::FixedLine),
        ] {
            assert_eq!(detect_phone_number_type(number), Some(expected), "{number}");
        }
        // A country with no range table says so instead of guessing.
        assert_eq!(
            detect_phone_number_type("+96179123123"),
            Some(PhoneNumberType::Unknown)
        );
    }

    /// Regression: the `"IN"` arm was a bare first-digit guess
    /// (`b'6'..=b'9' => Mobile`). Indian landlines carry their STD code in the
    /// national significant number, so the two-digit metro codes that start
    /// with 7 or 8 — Ahmedabad `079` and Bengaluru `080` — came back `Mobile`.
    /// The other two-digit codes (11, 20, 22, 33, 40, 44) already fell in the
    /// fixed-line range, so those two are the whole collision.
    #[test]
    fn indian_metro_landlines_are_not_classified_as_mobile() {
        for (number, expected, what) in [
            ("+918022221234", PhoneNumberType::FixedLine, "Bengaluru 080"),
            ("+917912345678", PhoneNumberType::FixedLine, "Ahmedabad 079"),
            ("+911123456789", PhoneNumberType::FixedLine, "Delhi 011"),
            ("+912212345678", PhoneNumberType::FixedLine, "Mumbai 022"),
            ("+919876543210", PhoneNumberType::Mobile, "a real mobile"),
            (
                "+918123456789",
                PhoneNumberType::Mobile,
                "mobile starting 81",
            ),
        ] {
            assert_eq!(
                detect_phone_number_type(number),
                Some(expected),
                "{number} is {what}"
            );
        }
    }

    /// Australian 1300 local-rate numbers came back `Unknown` and a nonexistent
    /// 188 range was mapped to `TollFree`.
    #[test]
    fn australian_service_ranges_classify_correctly() {
        assert_eq!(
            detect_phone_number_type("+611800123456"),
            Some(PhoneNumberType::TollFree)
        );
        assert_eq!(
            detect_phone_number_type("+611300123456"),
            Some(PhoneNumberType::SharedCost)
        );
        assert_eq!(
            detect_phone_number_type("+611900123456"),
            Some(PhoneNumberType::PremiumRate)
        );
        assert_eq!(
            detect_phone_number_type("+61412345678"),
            Some(PhoneNumberType::Mobile)
        );
    }

    /// French 089x is premium rate, not toll-free, and 09xx is VoIP rather
    /// than a fixed line.
    #[test]
    fn french_service_ranges_classify_correctly() {
        assert_eq!(
            detect_phone_number_type("+33891123456"),
            Some(PhoneNumberType::PremiumRate)
        );
        assert_eq!(
            detect_phone_number_type("+33912345678"),
            Some(PhoneNumberType::Voip)
        );
        assert_eq!(
            detect_phone_number_type("+33810123456"),
            Some(PhoneNumberType::SharedCost)
        );
    }

    /// Four `PhoneNumberType` variants were unreachable dead public API. The
    /// two that could never describe an E.164 number were removed; the other
    /// two are now produced.
    #[test]
    fn personal_number_and_pager_are_reachable() {
        assert_eq!(
            detect_phone_number_type("+447000123456"),
            Some(PhoneNumberType::PersonalNumber)
        );
        assert_eq!(
            detect_phone_number_type("+447600123456"),
            Some(PhoneNumberType::Pager)
        );
        assert_eq!(
            detect_phone_number_type("+497001234567"),
            Some(PhoneNumberType::PersonalNumber)
        );
        assert_eq!(
            detect_phone_number_type("+491641234567"),
            Some(PhoneNumberType::Pager)
        );
        assert_eq!(
            detect_phone_number_type("+15005550199"),
            Some(PhoneNumberType::PersonalNumber)
        );
    }

    // --- formatting ---------------------------------------------------------

    /// National format omitted every country's trunk prefix and RFC 3966 output
    /// could not be re-parsed by the crate that produced it.
    #[test]
    fn formatted_output_round_trips() {
        for &(_, number) in COUNTRY_FIXTURE {
            let parsed = PhoneNumber::parse(number).unwrap();

            let rfc = parsed.format(PhoneFormat::RFC3966);
            assert!(
                is_valid_phone_number(&rfc),
                "the crate rejects its own RFC 3966 output {rfc}"
            );
            assert_eq!(normalize_phone_number(&rfc).as_deref(), Some(number));

            let national = parsed.format(PhoneFormat::National);
            let back = PhoneNumber::parse_with_country(&national, parsed.country().code)
                .unwrap_or_else(|| {
                    panic!("{national:?} must re-parse as {}", parsed.country().code)
                });
            assert_eq!(back.e164(), number, "national round trip changed {number}");
        }
    }

    /// The RFC 3966 renderer dropped the extension.
    #[test]
    fn rfc3966_keeps_the_extension() {
        assert_eq!(
            format_phone_number("+1 202-555-0173 ext. 42", PhoneFormat::RFC3966).as_deref(),
            Some("tel:+12025550173;ext=42")
        );
        let back = PhoneNumber::parse("tel:+12025550173;ext=42").unwrap();
        assert_eq!(back.extension(), Some("42"));
    }

    /// `PhoneFormat` variant docs described output the crate never produced.
    #[test]
    fn documented_format_shapes_are_what_is_emitted() {
        assert_eq!(
            format_phone_number("+12025550173", PhoneFormat::International).as_deref(),
            Some("+1 202 555 0173")
        );
        assert_eq!(
            format_phone_number("+12025550173", PhoneFormat::National).as_deref(),
            Some("(202) 555-0173")
        );
        assert_eq!(
            format_phone_number("+12025550173", PhoneFormat::RFC3966).as_deref(),
            Some("tel:+12025550173")
        );
        assert_eq!(
            format_phone_number("+442079460958", PhoneFormat::National).as_deref(),
            Some("020 7946 0958")
        );
    }

    // --- PhoneNumber and PhoneNumberSet -------------------------------------

    /// `PhoneNumber`'s fields were public with no enforced invariant, so
    /// `format()` and `national_number()` could be made to panic. The fields
    /// are private now, and every accessor holds for every valid number.
    #[test]
    fn phone_number_accessors_never_panic() {
        for &(_, number) in COUNTRY_FIXTURE {
            let parsed = PhoneNumber::parse(number).unwrap();
            assert!(!parsed.national_number().is_empty(), "{number}");
            assert!(parsed.country_code() > 0);
            for format in [
                PhoneFormat::E164,
                PhoneFormat::International,
                PhoneFormat::National,
                PhoneFormat::RFC3966,
            ] {
                assert!(!parsed.format(format).is_empty(), "{number} / {format:?}");
            }
        }
    }

    /// `parse_with_country` used to force the hint country onto a number whose
    /// normalized form, type and national number had been computed for a
    /// different country.
    #[test]
    fn parse_with_country_keeps_country_and_number_consistent() {
        for (input, hint) in [
            ("2685550173", "US"),
            ("2645550173", "US"),
            ("8095550173", "US"),
            ("4165550198", "CA"),
            ("2025550173", "CA"),
        ] {
            let hinted = PhoneNumber::parse_with_country(input, hint)
                .unwrap_or_else(|| panic!("{input} / {hint} must parse"));
            let direct = PhoneNumber::parse(hinted.e164()).unwrap();

            assert_eq!(
                hinted.country().code,
                direct.country().code,
                "{input} / {hint}"
            );
            assert_eq!(hinted.national_number(), direct.national_number());
            assert_eq!(hinted.country_code(), direct.country_code());
            assert_eq!(hinted.phone_type(), direct.phone_type());
            assert_eq!(
                hinted.format(PhoneFormat::RFC3966),
                direct.format(PhoneFormat::RFC3966)
            );
        }
    }

    /// `parse_with_country` silently ignored an unrecognised country code, so a
    /// typo'd hint was more permissive than a correct one.
    #[test]
    fn an_unrecognised_hint_is_an_error_not_a_free_pass() {
        assert_eq!(
            PhoneNumber::try_parse_with_country("020 7946 0958", "ZZ").unwrap_err(),
            ParseError::UnknownCountry
        );
        for hint in ["ZZ", "GBR", "", "XX", "44"] {
            assert!(
                PhoneNumber::parse_with_country("020 7946 0958", hint).is_none(),
                "hint {hint:?} must be rejected"
            );
        }
        // The correct hint still works, in any casing.
        for hint in ["GB", "gb", "Gb"] {
            assert_eq!(
                PhoneNumber::parse_with_country("020 7946 0958", hint)
                    .unwrap()
                    .e164(),
                "+442079460958",
                "hint {hint:?} must resolve"
            );
        }
    }

    /// `national_number()` silently returned the full E.164 string when
    /// `country` was `None`. The field is no longer optional.
    #[test]
    fn national_number_is_always_a_national_number() {
        let parsed = PhoneNumber::parse("+442079460958").unwrap();
        assert_eq!(parsed.national_number(), "2079460958");
        assert_eq!(parsed.format(PhoneFormat::National), "020 7946 0958");
        assert!(!parsed.national_number().starts_with('+'));
    }

    /// `PhoneNumberSet::add` could not distinguish a rejected input from a
    /// duplicate.
    #[test]
    fn set_reports_rejection_separately_from_duplication() {
        let mut set = PhoneNumberSet::new();
        assert_eq!(set.try_add("+12025550173"), Ok(true));
        assert_eq!(set.try_add("+1 202-555-0173"), Ok(false));
        assert!(set.try_add("nonsense").is_err());
        assert_eq!(set.len(), 1);
    }

    /// `PhoneNumberSet` lacked `IntoIterator`, `Extend` and `PartialEq`.
    #[test]
    fn set_implements_the_standard_collection_traits() {
        let mut set = PhoneNumberSet::new();
        set.extend(["+12025550173", "+442079460958"]);
        assert_eq!(set.len(), 2);

        let by_ref: Vec<&PhoneNumber> = (&set).into_iter().collect();
        assert_eq!(by_ref.len(), 2);
        let owned: Vec<PhoneNumber> = set.clone().into_iter().collect();
        assert_eq!(owned.len(), 2);

        let other: PhoneNumberSet = ["+442079460958", "+12025550173"].into_iter().collect();
        assert_eq!(set, other);
        assert_eq!(set, set.clone());
    }

    /// `find_duplicates` was a misnomer that returned at most one element; it
    /// is now `get`, which is what it always did.
    #[test]
    fn set_lookup_is_named_for_what_it_does() {
        let mut set = PhoneNumberSet::new();
        assert!(set.add("+1-202-555-0173"));
        let found = set.get("+1 (202) 555-0173").expect("must be found");
        assert_eq!(found.e164(), "+12025550173");
        assert!(set.get("+12025550174").is_none());
    }

    /// `FromStr::Err` was `&'static str` and every fallible function returned a
    /// bare `Option`.
    #[test]
    fn every_fallible_entry_point_has_a_try_variant() {
        assert_eq!(
            try_normalize_phone_number("nonsense"),
            Err(ParseError::InvalidCharacter)
        );
        assert_eq!(
            PhoneNumber::try_parse("nonsense").unwrap_err(),
            ParseError::InvalidCharacter
        );
        assert_eq!(
            PhoneNumber::try_parse_with_country("x", "ZZ").unwrap_err(),
            ParseError::UnknownCountry
        );
        assert_eq!(
            PhoneNumberSet::new().try_add("nonsense").unwrap_err(),
            ParseError::InvalidCharacter
        );
        let from_str: Result<PhoneNumber, ParseError> = "nonsense".parse();
        assert_eq!(from_str.unwrap_err(), ParseError::InvalidCharacter);
    }

    // --- text extraction ----------------------------------------------------

    /// Two whitespace-separated numbers merged into one bogus candidate,
    /// silently dropping the second.
    #[test]
    fn whitespace_separated_numbers_stay_separate() {
        let numbers = extract_phone_numbers_from_text("Call 2025550173 4155552671 now");
        assert_eq!(
            numbers.iter().map(|n| n.raw.as_str()).collect::<Vec<_>>(),
            ["2025550173", "4155552671"]
        );
    }

    /// Redaction leaked the digits of a second number when two numbers were
    /// separated by a single space.
    #[test]
    fn redaction_does_not_leak_an_adjacent_number() {
        assert_eq!(
            redact_phone_numbers("Call 2025550173 4155552671 now", 0),
            "Call [PHONE] [PHONE] now"
        );
        assert_eq!(
            redact_phone_numbers("Call 2025550173 4155552671 now", 4),
            "Call ******0173 ******2671 now"
        );
        assert_eq!(
            redact_phone_numbers("Numbers: 2025550173 4155552671", 0),
            "Numbers: [PHONE] [PHONE]"
        );
    }

    /// A number followed by " (<digit>" swallowed the paren and digit, so
    /// replacement destroyed surrounding words and left a dangling ')'.
    #[test]
    fn a_following_parenthesised_digit_survives_redaction() {
        assert_eq!(
            redact_phone_numbers("Call 202-555-0173 (3 lines available)", 0),
            "Call [PHONE] (3 lines available)"
        );
    }

    /// An unmatched '(' was included in `raw` while the ')' was not, so
    /// redacting "(2025550173)" left a stray ')'.
    #[test]
    fn parentheses_are_balanced_in_the_reported_span() {
        let numbers = extract_phone_numbers_from_text("Call (2025550173) now");
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].raw, "(2025550173)");
        assert_eq!(
            redact_phone_numbers("Call (2025550173) now", 0),
            "Call [PHONE] now"
        );
    }

    /// '+' and '(' candidate starts skipped the left-boundary check, producing
    /// valid-looking false positives from a '+' embedded in a word.
    #[test]
    fn a_plus_inside_a_word_is_not_a_candidate_start() {
        assert!(extract_phone_numbers_from_text("C++12025550173").is_empty());
        assert!(
            extract_phone_numbers_from_text("email me at bob+12025550173@example.com").is_empty()
        );
        for number in extract_phone_numbers_from_text("Zip+4 90210-1234") {
            assert!(!number.raw.starts_with('+'), "{:?}", number.raw);
            assert!(!number.is_valid, "{:?} must not validate", number.raw);
        }
        for number in extract_phone_numbers_from_text("score 1e+12345678901") {
            assert!(!number.raw.starts_with('+'), "{:?}", number.raw);
        }
    }

    /// The digit cap admitted 16 digits and truncated longer runs into a bogus
    /// 16-digit candidate.
    #[test]
    fn over_length_digit_runs_are_rejected_not_truncated() {
        for text in [
            "digits 0123456789012345 here",
            "digits 01234567890123456 here",
            "digits 012345678901234567890123456789 here",
        ] {
            assert!(
                extract_phone_numbers_from_text(text).is_empty(),
                "{text:?} produced a candidate"
            );
        }
    }

    /// NBSP, en dash, tab, double space and spaced hyphens inside a number
    /// caused total extraction failure.
    #[test]
    fn unicode_and_wide_separators_do_not_break_extraction() {
        for text in [
            "Call +1 202\u{00A0}555\u{00A0}0173 now",
            "Call +1 202\u{2013}555\u{2013}0173 now",
            "Call +1 202\t555\t0173 now",
            "Call +1 202  555  0173 now",
            "Call +1 202 - 555 - 0173 now",
        ] {
            let numbers = extract_phone_numbers_from_text(text);
            assert_eq!(numbers.len(), 1, "{text:?}");
            assert_eq!(
                numbers[0].normalized.as_deref(),
                Some("+12025550173"),
                "{text:?}"
            );
        }
    }

    /// Fullwidth digits and the fullwidth plus were never extracted, making the
    /// API blind to CJK-formatted numbers.
    #[test]
    fn fullwidth_numbers_are_extracted_from_text() {
        let text = "\u{7DCA}\u{6025}: \u{FF0B}\u{FF18}\u{FF11}\u{FF13}\u{FF11}\u{FF12}\u{FF13}\u{FF14}\u{FF15}\u{FF16}\u{FF17}\u{FF18}";
        let numbers = extract_phone_numbers_from_text(text);
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+81312345678"));
        assert_eq!(&text[numbers[0].start..numbers[0].end], numbers[0].raw);
    }

    /// Dates, IPv4 addresses and ISBNs were all treated as phone numbers.
    #[test]
    fn structured_numeric_data_is_not_extracted() {
        for text in [
            "Version 3.141592653589793 here",
            "IP 192.168.1.100 and date 2024-01-15",
            "ISBN 978-3-16-148410-0 here",
        ] {
            assert!(
                extract_phone_numbers_from_text(text).is_empty(),
                "false positive in {text:?}"
            );
        }
        // A dotted run that is not a quad is still reported as a candidate,
        // but it must never come back as a valid phone number.
        for number in extract_phone_numbers_from_text("1.2.3.4567890") {
            assert!(!number.is_valid, "{:?} must not validate", number.raw);
            assert_eq!(number.normalized, None);
        }
    }

    /// `extract_phone_numbers_with_country_hint` re-prefixed numbers that
    /// already carried a '+' country code, returning a different real number
    /// with `is_valid = true`.
    #[test]
    fn the_hint_never_re_prefixes_an_explicit_country_code() {
        for hint in ["DE", "GB", "FR", "US"] {
            let numbers = extract_phone_numbers_with_country_hint("Call +12025550173", hint);
            assert_eq!(numbers.len(), 1, "hint {hint}");
            assert_eq!(
                numbers[0].normalized.as_deref(),
                Some("+12025550173"),
                "hint {hint} re-prefixed an explicit country code"
            );
        }
    }

    /// Text extraction had a hard 7-digit floor, making the national numbers of
    /// 17 countries structurally invisible.
    #[test]
    fn short_national_numbers_are_reachable_with_a_hint() {
        let numbers = extract_phone_numbers_with_country_hint("call 849338 now", "AD");
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+376849338"));
    }

    /// Every reported span used to be computed by counting characters, which
    /// was both quadratic and wrong for multi-byte input.
    #[test]
    fn reported_spans_are_exact_byte_offsets() {
        let text = "\u{1F4DE} Call +1-202-555-0173 or \u{00E4}+442079460958 today";
        let numbers = extract_phone_numbers_from_text(text);
        assert_eq!(numbers.len(), 2);
        let mut previous_end = 0;
        for number in &numbers {
            assert_eq!(&text[number.start..number.end], number.raw);
            assert!(number.start >= previous_end);
            previous_end = number.end;
        }
    }

    /// `redact_phone_numbers` documented separators it never emitted, and
    /// `visible_digits >= total` inverted into full redaction without saying so.
    #[test]
    fn redaction_matches_its_documented_contract() {
        assert_eq!(
            redact_phone_numbers("Call +12025550173", 4),
            "Call *******0173"
        );
        assert_eq!(redact_phone_numbers("Call +12025550173", 0), "Call [PHONE]");
        assert_eq!(
            redact_phone_numbers("Call +12025550173", 11),
            "Call [PHONE]"
        );
        assert_eq!(
            redact_phone_numbers("Call +12025550173", 99),
            "Call [PHONE]"
        );
    }

    /// `.` and `,` used to count as word characters for the left-boundary
    /// check, so a number in a comma-separated list or a CSV field was never
    /// even considered — and therefore never redacted. The two inputs that the
    /// boundary check exists for must still be skipped.
    #[test]
    fn numbers_in_comma_separated_text_are_found_and_redacted() {
        for (text, expected) in [
            ("name,+12025550173,x", "+12025550173"),
            ("a,b,+442079460958,c", "+442079460958"),
            ("first,2025550173,last", "2025550173"),
        ] {
            let numbers = extract_phone_numbers_from_text(text);
            assert_eq!(numbers.len(), 1, "{text:?}");
            assert_eq!(numbers[0].raw, expected, "{text:?}");
        }
        assert_eq!(
            redact_phone_numbers("name,+12025550173,x", 0),
            "name,[PHONE],x"
        );

        // A digit before the comma makes it part of one number, not a list
        // separator; and a '+' inside an address is still not a number start.
        assert!(extract_phone_numbers_from_text("1,2025550173").is_empty());
        assert!(extract_phone_numbers_from_text("user+12025550173@example.com").is_empty());
        assert!(extract_phone_numbers_from_text("v1.2025550173").is_empty());
    }

    /// The cap on how many separated groups form one candidate is `+`-aware:
    /// eight groups when the candidate carries an explicit `+`, five without.
    /// A flat cap of eight glued the six groups of `06 45 34 25 45 06` into a
    /// twelve-digit run that normalized to a *fabricated* valid number.
    #[test]
    fn the_group_cap_depends_on_an_explicit_plus() {
        // Six unprefixed groups: only the first five are taken, so the trailing
        // "06" is left out rather than folded in.
        let numbers = extract_phone_numbers_from_text("06 45 34 25 45 06");
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].raw, "06 45 34 25 45");
        assert_eq!(numbers[0].normalized, None);
        assert!(!numbers[0].is_valid);
        assert_eq!(
            extract_phone_numbers_with_country_hint("06 45 34 25 45 06", "FR")[0]
                .normalized
                .as_deref(),
            Some("+33645342545")
        );

        // Five unprefixed groups still form one candidate.
        let french = extract_phone_numbers_with_country_hint("06 12 34 56 78", "FR");
        assert_eq!(french.len(), 1);
        assert_eq!(french[0].raw, "06 12 34 56 78");
        assert_eq!(french[0].normalized.as_deref(), Some("+33612345678"));

        // With a '+' the author has said it is one number, so eight groups are
        // allowed.
        let german = extract_phone_numbers_from_text("+49 (0) 30 12 34 56 78");
        assert_eq!(german.len(), 1);
        assert_eq!(german[0].raw, "+49 (0) 30 12 34 56 78");
        assert_eq!(german[0].normalized.as_deref(), Some("+493012345678"));
    }

    /// Dot-separated numbers are written that way across half of Europe, so the
    /// shape alone cannot reject them. Only the precise dotted-quad and ISO-date
    /// shapes are filtered.
    #[test]
    fn dotted_numbers_survive_but_addresses_and_dates_do_not() {
        let numbers = extract_phone_numbers_from_text("02.345.67.89");
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].raw, "02.345.67.89");
        assert_eq!(
            extract_phone_numbers_with_country_hint("02.345.67.89", "BE")[0]
                .normalized
                .as_deref(),
            Some("+3223456789")
        );
        assert_eq!(
            extract_phone_numbers_from_text("202.555.0173")[0].raw,
            "202.555.0173"
        );

        for text in ["192.168.1.1", "10.0.0.138", "2024-01-15", "15-01-2024"] {
            assert!(
                extract_phone_numbers_from_text(text).is_empty(),
                "false positive in {text:?}"
            );
        }

        // A date is rejected on its *shape*, before normalization is attempted,
        // so a hint cannot resurrect it. A dotted quad is not: the shape is far
        // less discriminating than a date's, so it is only applied to a
        // candidate that already failed to normalize — and under a hint
        // `192.168.1.1` does normalize. That asymmetry is deliberate, and this
        // pins it so it cannot be inverted by accident.
        for text in ["2024-01-15", "15-01-2024", "1999-12-31"] {
            assert!(
                extract_phone_numbers_with_country_hint(text, "DE").is_empty(),
                "a date must stay rejected under a hint: {text:?}"
            );
        }
        assert_eq!(
            extract_phone_numbers_with_country_hint("192.168.1.1", "DE")
                .first()
                .and_then(|n| n.normalized.as_deref()),
            Some("+4919216811"),
            "an address that is also a valid national number is not separable"
        );
    }

    /// An IDD prefix only counts when what follows it actually parses. When it
    /// does, the number behind it is the answer; when it does not, the digits
    /// stay a national number rather than being mangled.
    #[test]
    fn an_idd_prefix_only_counts_when_the_remainder_parses() {
        for (text, expected) in [
            ("0012025550173", "+12025550173"),
            ("011442079460958", "+442079460958"),
            ("00442079460958", "+442079460958"),
        ] {
            let numbers = extract_phone_numbers_from_text(text);
            assert_eq!(numbers.len(), 1, "{text:?}");
            assert_eq!(numbers[0].normalized.as_deref(), Some(expected), "{text:?}");
        }
        // `011 123 4567` is the Turin area code, and `011` + `1234567` is not a
        // number anywhere, so nothing is claimed without a hint…
        for text in ["0111234567", "011 123 4567"] {
            let numbers = extract_phone_numbers_from_text(text);
            assert_eq!(numbers.len(), 1, "{text:?}");
            assert_eq!(numbers[0].normalized, None, "{text:?}");
        }
        // …and with the Italian hint it is Turin, zero and all.
        assert_eq!(
            extract_phone_numbers_with_country_hint("011 123 4567", "IT")[0]
                .normalized
                .as_deref(),
            Some("+390111234567")
        );
    }

    // --- generation ---------------------------------------------------------

    /// `generate_random_phone_number("CA")` never produced a number that
    /// resolved back to CA, and "US" emitted ~2.3% Caribbean numbers.
    #[test]
    fn generated_numbers_resolve_back_to_the_requested_country() {
        for code in [
            "US", "CA", "GB", "DE", "FR", "IT", "JP", "AU", "BR", "IN", "AG", "DO", "PR", "JM",
        ] {
            for number in generate_random_phone_numbers(code, 50) {
                assert_eq!(
                    extract_country(&number).map(|c| c.code),
                    Some(code),
                    "{code} generated {number}"
                );
            }
        }
    }

    /// A zero leading digit was clamped to '1' rather than resampled, doubling
    /// P(lead = '1') and producing structurally impossible NANP area codes.
    #[test]
    fn generation_resamples_rather_than_clamps_the_leading_digit() {
        let numbers = generate_random_phone_numbers("US", 500);
        assert_eq!(numbers.len(), 500);
        for number in &numbers {
            let area = &number[2..5];
            assert!(
                (b'2'..=b'9').contains(&area.as_bytes()[0]),
                "{number} has area code {area}"
            );
            assert!(!area.ends_with("11"), "{number} has an N11 area code");
        }
    }

    /// Generation always used `phone_lengths.first()`, so generated corpora had
    /// zero length diversity.
    #[test]
    fn generation_samples_more_than_one_length() {
        use std::collections::BTreeSet;
        let lengths: BTreeSet<usize> = generate_random_phone_numbers("AT", 400)
            .iter()
            .map(|n| n.len())
            .collect();
        assert!(
            lengths.len() >= 3,
            "only these lengths were generated: {lengths:?}"
        );
    }

    /// The PRNG took its digit from a single bit range, making digit parity
    /// perfectly periodic with period 131072.
    #[test]
    fn generation_is_not_trivially_periodic() {
        use std::collections::BTreeSet;
        let numbers = generate_random_phone_numbers("US", 300);
        let unique: BTreeSet<&String> = numbers.iter().collect();
        assert!(unique.len() >= 295, "{} unique of 300", unique.len());
    }

    /// `count` went straight to `Vec::with_capacity`, so a large value aborted
    /// the process.
    #[test]
    fn a_huge_count_is_not_used_as_an_allocation_hint() {
        assert!(generate_random_phone_numbers("ZZ", usize::MAX).is_empty());
        // A country that cannot be generated stops at the first failure rather
        // than trying `count` times.
        assert!(generate_random_phone_numbers("GG", usize::MAX).is_empty());
    }

    // --- suggestions and batch APIs -----------------------------------------

    /// Length-repair branches were gated so tightly that a single-digit typo
    /// produced an empty list. The exact suggestions for these three inputs are
    /// pinned by `SUGGESTION_CASES` in `mod suggestions`; what this test adds is
    /// that every one of them is canonical and really does come back valid.
    #[test]
    fn single_digit_typos_are_repairable() {
        for (input, hint) in [
            ("+1 202 555 01730", Some("US")),
            ("+1 202 555 017", Some("US")),
            ("+44 20 7946 09580", Some("GB")),
        ] {
            let suggestions = suggest_phone_number_corrections(input, hint);
            assert!(!suggestions.is_empty(), "no suggestion for {input:?}");
            for suggestion in &suggestions {
                assert!(
                    is_valid_phone_number(suggestion),
                    "{suggestion} suggested for {input:?} is not valid"
                );
                assert_eq!(
                    normalize_phone_number(suggestion).as_deref(),
                    Some(suggestion.as_str()),
                    "{suggestion} is not canonical"
                );
            }
        }
    }

    /// Candidates were ranked by byte order before `truncate(5)`, so the
    /// zero-edit suggestion was discarded whenever the input sorted high.
    #[test]
    fn the_minimal_edit_suggestion_is_never_truncated_away() {
        assert_eq!(
            suggest_phone_number_corrections("+12025550173", None),
            vec!["+12025550173".to_string()]
        );
        assert_eq!(
            suggest_phone_number_corrections("  +1 (202) 555-0173  ", None),
            vec!["+12025550173".to_string()]
        );
        assert_eq!(
            suggest_phone_number_corrections("00442079460958", None),
            vec!["+442079460958".to_string()]
        );
    }

    /// `is_potentially_valid_phone_number` used a hardcoded digit window that
    /// contradicted the country table and rejected vanity numbers the
    /// validator accepted.
    #[test]
    fn potentially_valid_is_implied_by_valid() {
        for input in [
            "1-800-FLOWERS",
            "1-800-GOT-JUNK",
            "+12025550173",
            "+442079460958",
            "+390698123456",
        ] {
            assert!(is_valid_phone_number(input), "{input:?}");
            assert!(
                is_potentially_valid_phone_number(input),
                "{input:?} is valid but not potentially valid"
            );
        }
        for &(_, number) in COUNTRY_FIXTURE {
            assert!(is_potentially_valid_phone_number(number), "{number}");
        }
    }

    /// `analyze_phone_numbers_batch` and `validate_phone_numbers_batch`
    /// reported contradictory validity for the same slice.
    #[test]
    fn the_batch_apis_never_contradict_each_other() {
        let corpus = [
            "+12025550173",
            " +12025550173",
            "+12025550173\n",
            "+12025550173\t",
            "nonsense",
            "",
            "  ",
            "+44 (0)20 7946 0958",
            "1-800-FLOWERS",
        ];
        let analyses = analyze_phone_numbers_batch(&corpus);
        let validity = validate_phone_numbers_batch(&corpus);
        for (index, input) in corpus.iter().enumerate() {
            assert_eq!(analyses[index].is_valid, validity[index], "{input:?}");
        }
    }

    /// `group_equivalent_phone_numbers` was quadratic and not reflexive on
    /// inputs that failed to normalize.
    #[test]
    fn grouping_returns_every_input_exactly_once() {
        let numbers = [
            "not a number",
            "not a number",
            "+12025550173",
            "+1 202 555 0173",
        ];
        let groups = group_equivalent_phone_numbers(&numbers);
        let flattened: Vec<&String> = groups.iter().flatten().collect();
        assert_eq!(flattened.len(), numbers.len());
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[2], ["+12025550173", "+1 202 555 0173"]);
    }

    // --- API surface --------------------------------------------------------

    /// `Country` appeared in five public signatures but was not re-exported,
    /// so downstream crates could not name it (E0603). The type is now public;
    /// `tests/public_api.rs` proves it from outside the crate.
    #[test]
    fn country_is_a_first_class_public_type() {
        let country: &'static Country = country_by_code("JP").unwrap();
        assert_eq!(country.prefix, 81);
        assert_eq!(country.code, "JP");
        assert!(!country.phone_lengths.is_empty());
        assert_eq!(country.to_string(), "Japan (+81)");

        fn takes_country(c: &'static Country) -> u32 {
            c.prefix
        }
        assert_eq!(takes_country(country), 81);
    }

    /// `PhoneNumberAnalysis` had no way to report *why* a number was rejected.
    #[test]
    fn analysis_carries_the_parse_error() {
        let analysis = &analyze_phone_numbers_batch(&["+11111111111"])[0];
        assert!(!analysis.is_valid);
        assert_eq!(analysis.error, Some(ParseError::InvalidAreaCode));
        assert_eq!(analysis.normalized, None);

        let ok = &analyze_phone_numbers_batch(&["+12025550173"])[0];
        assert!(ok.is_valid);
        assert_eq!(ok.error, None);
    }

    // ========================================================================
    // Defects the test suite found in code it does not own, and that have since
    // been fixed.
    //
    // Each of these started life as an `#[ignore]`d test pinning a live bug.
    // They all run now; the doc comment on each says what was wrong, because
    // that is what makes the test worth keeping.
    //
    // Two `#[ignore]`d tests are still waiting on a fix, and their `#[ignore]`
    // reason names the defect: `indian_landlines_are_misclassified_as_mobile`
    // below, and `suggestions_for_a_hinted_input_reach_the_hint_country` in
    // `mod suggestions`. Nothing else in this file is ignored.
    // ========================================================================

    /// The French toll-free arm of `classify_phone_number_type` used to match
    /// `n4` — the first *four* digits — against `800..=805`. A French national
    /// significant number is nine digits, so `n4` for `0800 123 456` was `8001`,
    /// never `800`: the arm was unreachable, `PhoneNumberType::TollFree` could
    /// never be produced for FR, and 0800-0805 numbers fell through to
    /// `Unknown`. It matches `n3` now, the way the GB/IE/AU arms scale their own
    /// ranges.
    #[test]
    fn french_toll_free_numbers_classify_as_toll_free() {
        for number in [
            "+33800123456",
            "+33801123456",
            "+33802123456",
            "+33805123456",
        ] {
            assert_eq!(
                detect_phone_number_type(number),
                Some(PhoneNumberType::TollFree),
                "{number}"
            );
            assert!(is_toll_free_number(number), "{number}");
        }
    }

    /// The French arm was not the only place a service range was matched
    /// against the wrong number of digits. A range that is `N` digits wide
    /// after the trunk zero is stripped must be compared against the `N`-digit
    /// prefix; comparing `020` against a three-digit prefix, or `080` against a
    /// three-digit one, silently makes the whole arm unreachable. One case per
    /// range that was wrong, plus the neighbours that must NOT be caught by it.
    #[test]
    fn service_ranges_are_matched_at_their_real_width() {
        use PhoneNumberType::*;
        let cases: &[(&str, PhoneNumberType)] = &[
            // Sweden: freephone is 020, two digits after the trunk zero.
            ("+46201234567", TollFree),
            ("+46771234567", SharedCost),
            ("+46701234567", Mobile),
            ("+46812345678", FixedLine),
            // Korea: 080 freephone and 060 premium, likewise two digits.
            ("+82801234567", TollFree),
            ("+82601234567", PremiumRate),
            ("+825012345678", Voip),
            ("+821012345678", Mobile),
            ("+82212345678", FixedLine),
            // Korean "representative" numbers are 15xx / 16xx / 18xx and share
            // their leading pair with the mobile block, so only the eight-digit
            // length tells them apart. Matching the pair alone would swallow
            // every 15x/16x/18x mobile.
            ("+8215881234", Uan),
            ("+8216441234", Uan),
            ("+8218991234", Uan),
            ("+82150123456", Mobile),
            ("+82160123456", Mobile),
            ("+82180123456", Mobile),
            // UK: all of 09 is premium, not only 090x.
            ("+449881234567", PremiumRate),
            ("+449091234567", PremiumRate),
            ("+448001234567", TollFree),
            ("+447000123456", PersonalNumber),
            ("+447600123456", Pager),
            // Ireland: 0818 is shared cost inside the 08 mobile block.
            ("+353818123456", SharedCost),
            ("+353861234567", Mobile),
        ];

        for (number, expected) in cases {
            assert_eq!(
                detect_phone_number_type(number),
                Some(*expected),
                "{number} should classify as {expected}"
            );
        }
    }

    /// `suggest_phone_number_corrections` used to try `format!("+{digits}")` in
    /// tier 0 before the country hint was consulted at all. When a national
    /// number's digits happened to spell a valid foreign E.164 number the search
    /// stopped there and the caller got a country they had not asked about:
    /// `"020 7946 0958"` with hint `"GB"` yielded the Egyptian `+2079460958`,
    /// and `"2025550173"` with hint `"US"` the Egyptian `+2025550173`. Tier 0
    /// now reads through the hint when one is supplied.
    ///
    /// Tiers 3 and 4 still do not — see
    /// `suggestions_for_a_hinted_input_reach_the_hint_country`.
    #[test]
    fn suggestions_honour_the_country_hint() {
        assert_eq!(
            suggest_phone_number_corrections("020 7946 0958", Some("GB")),
            vec!["+442079460958".to_string()]
        );
        assert_eq!(
            suggest_phone_number_corrections("2025550173", Some("US")),
            vec!["+12025550173".to_string()]
        );
    }

    /// `extract_phone_numbers_with_country_hint` used to fail *open*: a code
    /// that `country_by_code` did not resolve was silently downgraded to "no
    /// hint", so the call returned a different number — `"Call 2025550173"` with
    /// hint `"us"` yielded the Egyptian `+2025550173` where `"US"` yielded
    /// `+12025550173`. It now fails closed, like
    /// `PhoneNumber::try_parse_with_country` and
    /// `suggest_phone_number_corrections`, and resolves the hint
    /// case-insensitively so `"us"` is not a typo in the first place.
    #[test]
    fn text_hint_does_not_silently_ignore_an_unrecognised_country() {
        let correct = extract_phone_numbers_with_country_hint("Call 2025550173", "US");
        assert_eq!(
            correct.first().and_then(|n| n.normalized.as_deref()),
            Some("+12025550173")
        );

        // ISO codes are canonically upper case, but a differently-cased one is
        // unambiguous and resolves to the same country.
        for cased in ["us", "Us", "uS"] {
            let got = extract_phone_numbers_with_country_hint("Call 2025550173", cased);
            assert_eq!(
                got.first().and_then(|n| n.normalized.as_deref()),
                Some("+12025550173"),
                "hint {cased:?} should resolve like \"US\""
            );
        }

        // An unrecognised code fails closed. The point of the defect was that
        // it used to fall back to "no hint" and return a *different* valid
        // number (+2025550173, Egypt); returning nothing is the safe answer.
        for unknown in ["USA", "XX", "", "1"] {
            let got = extract_phone_numbers_with_country_hint("Call 2025550173", unknown);
            assert!(
                got.is_empty(),
                "hint {unknown:?} should yield nothing, got {:?}",
                got.first().and_then(|n| n.normalized.as_deref())
            );
        }
    }
}
