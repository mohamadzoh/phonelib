//! The public API exercised the way a downstream crate would use it.
//!
//! The crate previously had no integration tests at all, so nothing caught an
//! item that was accidentally left private — `Country` shipped for four
//! releases as an unnameable type that five public signatures returned.
//!
//! Everything here is deliberately named through an explicit `use`, and the
//! signature of every public function is pinned by coercing it to a function
//! pointer. Making an item private, renaming it, or changing its signature
//! breaks this file at compile time.

use phonelib::{
    analyze_phone_numbers_batch, are_phone_numbers_equal, count_phone_numbers_in_text, countries,
    countries_by_calling_code, countries_by_code, country_by_code, detect_phone_number_type,
    detect_phone_number_types_batch, extract_countries, extract_countries_batch, extract_country,
    extract_phone_numbers_from_text, extract_phone_numbers_with_country_hint,
    extract_valid_phone_numbers_from_text, format_phone_number, generate_random_phone_number,
    generate_random_phone_numbers, group_equivalent_phone_numbers, guess_country_from_number,
    is_landline_number, is_mobile_number, is_potentially_valid_phone_number, is_toll_free_number,
    is_valid_phone_number, normalize_phone_number, normalize_phone_number_in_place,
    normalize_phone_numbers_batch, redact_phone_numbers, replace_phone_numbers_in_text,
    suggest_phone_number_corrections, try_normalize_phone_number, validate_phone_numbers_batch,
    Country, ExtractedPhoneNumber, ParseError, PhoneFormat, PhoneNumber, PhoneNumberAnalysis,
    PhoneNumberSet, PhoneNumberType,
};
use std::collections::HashSet;
use std::str::FromStr;

// ============================================================================
// Reachability and signatures
// ============================================================================

/// Coerces every public free function to a function pointer. If a signature
/// changes, or an item stops being public, this stops compiling.
#[test]
fn every_public_function_is_reachable_with_the_documented_signature() {
    let _: fn(&str) -> bool = is_valid_phone_number;
    let _: fn(&str) -> Option<String> = normalize_phone_number;
    let _: fn(&str) -> Result<String, ParseError> = try_normalize_phone_number;
    let _: fn(&mut String) -> Option<String> = normalize_phone_number_in_place;
    let _: fn(&str) -> Option<&'static Country> = extract_country;
    let _: fn(&str) -> Vec<&'static Country> = extract_countries;
    let _: fn() -> &'static [Country] = countries;
    let _: fn(&str) -> Option<&'static Country> = country_by_code;
    let _: fn(&str) -> Vec<&'static Country> = countries_by_code;
    let _: fn(u32) -> Vec<&'static Country> = countries_by_calling_code;
    let _: fn(&str, PhoneFormat) -> Option<String> = format_phone_number;
    let _: fn(&str) -> Option<PhoneNumberType> = detect_phone_number_type;
    let _: fn(&str) -> bool = is_mobile_number;
    let _: fn(&str) -> bool = is_landline_number;
    let _: fn(&str) -> bool = is_toll_free_number;
    let _: fn(&str) -> Option<String> = generate_random_phone_number;
    let _: fn(&str, usize) -> Vec<String> = generate_random_phone_numbers;
    let _: fn(&str, &str) -> bool = are_phone_numbers_equal;
    // The batch APIs are generic over `T: AsRef<str>`; pinning them at a
    // concrete `T` is what makes them nameable as a function pointer.
    let _: fn(&[String]) -> Vec<Vec<String>> = group_equivalent_phone_numbers::<String>;
    let _: fn(&[String]) -> Vec<bool> = validate_phone_numbers_batch::<String>;
    let _: fn(&[String]) -> Vec<Option<String>> = normalize_phone_numbers_batch::<String>;
    let _: fn(&[String]) -> Vec<Option<&'static Country>> = extract_countries_batch::<String>;
    let _: fn(&[String]) -> Vec<Option<PhoneNumberType>> =
        detect_phone_number_types_batch::<String>;
    let _: fn(&[String]) -> Vec<PhoneNumberAnalysis> = analyze_phone_numbers_batch::<String>;
    let _: fn(&str, Option<&str>) -> Vec<String> = suggest_phone_number_corrections;
    let _: fn(&str) -> bool = is_potentially_valid_phone_number;
    let _: fn(&str) -> Option<&'static Country> = guess_country_from_number;
    let _: fn(&str) -> Vec<ExtractedPhoneNumber> = extract_phone_numbers_from_text;
    let _: fn(&str) -> Vec<ExtractedPhoneNumber> = extract_valid_phone_numbers_from_text;
    let _: fn(&str, &str) -> Vec<ExtractedPhoneNumber> = extract_phone_numbers_with_country_hint;
    let _: fn(&str) -> usize = count_phone_numbers_in_text;
    let _: fn(&str, usize) -> String = redact_phone_numbers;

    // Generic functions need a concrete instantiation to be named.
    type Replacement = fn(&ExtractedPhoneNumber) -> String;
    let _: fn(&str, Replacement) -> String = replace_phone_numbers_in_text::<Replacement>;

    // Inherent methods.
    let _: fn(&str) -> Option<PhoneNumber> = PhoneNumber::parse;
    let _: fn(&str) -> Result<PhoneNumber, ParseError> = PhoneNumber::try_parse;
    let _: fn(&str, &str) -> Option<PhoneNumber> = PhoneNumber::parse_with_country;
    let _: fn(&str, &str) -> Result<PhoneNumber, ParseError> = PhoneNumber::try_parse_with_country;
    let _: fn(&PhoneNumber) -> &str = PhoneNumber::original;
    let _: fn(&PhoneNumber) -> &str = PhoneNumber::e164;
    let _: fn(&PhoneNumber) -> &str = PhoneNumber::national_number;
    let _: fn(&PhoneNumber) -> &'static Country = PhoneNumber::country;
    let _: fn(&PhoneNumber) -> u32 = PhoneNumber::country_code;
    let _: fn(&PhoneNumber) -> PhoneNumberType = PhoneNumber::phone_type;
    let _: fn(&PhoneNumber) -> Option<&str> = PhoneNumber::extension;
    let _: fn(&PhoneNumber, PhoneFormat) -> String = PhoneNumber::format;
    let _: fn(&PhoneNumber) -> bool = PhoneNumber::is_mobile;
    let _: fn(&PhoneNumber) -> bool = PhoneNumber::is_landline;
    let _: fn(&PhoneNumber) -> bool = PhoneNumber::is_toll_free;

    let _: fn() -> PhoneNumberSet = PhoneNumberSet::new;
    let _: fn(&mut PhoneNumberSet, &str) -> bool = PhoneNumberSet::add;
    let _: fn(&mut PhoneNumberSet, &str) -> Result<bool, ParseError> = PhoneNumberSet::try_add;
    let _: fn(&PhoneNumberSet, &str) -> bool = PhoneNumberSet::contains;
    let _: for<'a> fn(&'a PhoneNumberSet, &str) -> Option<&'a PhoneNumber> = PhoneNumberSet::get;
    let _: fn(&PhoneNumberSet) -> usize = PhoneNumberSet::len;
    let _: fn(&PhoneNumberSet) -> bool = PhoneNumberSet::is_empty;
    let _: fn(&PhoneNumberSet) -> Vec<&str> = PhoneNumberSet::normalized_numbers;
    let _: fn(&mut PhoneNumberSet, &str) -> bool = PhoneNumberSet::remove;

    // The pinned pointers are also callable, so this is not purely a
    // compile-time check.
    let validate: fn(&str) -> bool = is_valid_phone_number;
    let normalize: fn(&str) -> Option<String> = normalize_phone_number;
    assert!(validate("+12025550173"));
    assert_eq!(normalize("+12025550173").as_deref(), Some("+12025550173"));
}

#[test]
fn country_is_a_nameable_public_type_with_public_fields() {
    // This is the shape that used to fail with E0603.
    fn calling_code_of(country: &'static Country) -> u32 {
        country.prefix
    }

    let japan: &'static Country = country_by_code("JP").expect("JP must be in the table");
    assert_eq!(calling_code_of(japan), 81);
    assert_eq!(japan.code, "JP");
    assert_eq!(japan.name, "Japan");
    assert!(japan.phone_lengths.contains(&10));

    // Copy, Clone, Debug, PartialEq, Eq and Hash are all available downstream.
    let copied: Country = *japan;
    assert_eq!(copied, *japan);
    assert!(format!("{copied:?}").contains("JP"));
    assert_eq!(copied.to_string(), "Japan (+81)");
    let set: HashSet<Country> = [copied, *japan].into_iter().collect();
    assert_eq!(set.len(), 1);

    // The type flows out of the resolution APIs too.
    let resolved: Option<&'static Country> = extract_country("+81312345678");
    assert_eq!(resolved.map(|c| c.code), Some("JP"));
    let table: &'static [Country] = countries();
    assert!(table.iter().any(|c| c.code == "JP"));
}

#[test]
fn non_exhaustive_enums_require_a_wildcard_arm_downstream() {
    // Outside the defining crate these matches do not compile without `_`,
    // which is exactly what `#[non_exhaustive]` buys. If the attribute were
    // dropped, the wildcards below would become unreachable-pattern warnings —
    // and CI denies warnings.
    fn describe_type(value: PhoneNumberType) -> &'static str {
        match value {
            PhoneNumberType::Mobile => "mobile",
            PhoneNumberType::FixedLine => "fixed",
            PhoneNumberType::FixedLineOrMobile => "either",
            PhoneNumberType::TollFree => "free",
            _ => "other",
        }
    }

    fn describe_format(value: PhoneFormat) -> &'static str {
        match value {
            PhoneFormat::E164 => "e164",
            PhoneFormat::National => "national",
            _ => "other",
        }
    }

    fn describe_error(value: ParseError) -> &'static str {
        match value {
            ParseError::Empty => "empty",
            ParseError::InvalidCharacter => "character",
            _ => "other",
        }
    }

    assert_eq!(describe_type(PhoneNumberType::Mobile), "mobile");
    assert_eq!(describe_type(PhoneNumberType::Pager), "other");
    assert_eq!(describe_format(PhoneFormat::E164), "e164");
    assert_eq!(describe_format(PhoneFormat::RFC3966), "other");
    assert_eq!(describe_error(ParseError::Empty), "empty");
    assert_eq!(describe_error(ParseError::TooLong), "other");

    // `PhoneNumberAnalysis` is non-exhaustive too: it can be read field by
    // field but not destructured exhaustively or built with a struct literal.
    let analysis: &PhoneNumberAnalysis = &analyze_phone_numbers_batch(&["+12025550173"])[0];
    assert_eq!(analysis.original, "+12025550173");
    assert!(analysis.is_valid);
    assert_eq!(analysis.normalized.as_deref(), Some("+12025550173"));
    assert_eq!(analysis.country.map(|c| c.code), Some("US"));
    assert_eq!(
        analysis.phone_type,
        Some(PhoneNumberType::FixedLineOrMobile)
    );
    assert_eq!(analysis.error, None);
}

// ============================================================================
// A downstream consumer's happy path
// ============================================================================

#[test]
fn validating_and_normalizing() {
    assert!(is_valid_phone_number("+12025550173"));
    assert!(is_valid_phone_number("+1 (202) 555-0173"));
    assert!(!is_valid_phone_number("not a phone number"));

    assert_eq!(
        normalize_phone_number("+1 (202) 555-0173"),
        Some("+12025550173".to_string())
    );
    assert_eq!(normalize_phone_number("nonsense"), None);

    assert_eq!(
        try_normalize_phone_number("+11111111111"),
        Err(ParseError::InvalidAreaCode)
    );
    assert_eq!(
        try_normalize_phone_number("+11111111111")
            .unwrap_err()
            .to_string(),
        "unassigned or invalid NANP area code"
    );

    let mut buffer = "+1 (202) 555-0173".to_string();
    assert_eq!(
        normalize_phone_number_in_place(&mut buffer),
        Some("+12025550173".to_string())
    );
    assert_eq!(buffer, "+12025550173");
}

#[test]
fn resolving_countries() {
    assert_eq!(extract_country("+12025550173").unwrap().code, "US");
    assert_eq!(extract_country("+14165550198").unwrap().code, "CA");
    assert_eq!(extract_country("+442079460958").unwrap().code, "GB");
    assert_eq!(extract_country("+79161234567").unwrap().code, "RU");
    assert_eq!(extract_country("+12685550173").unwrap().code, "AG");
    assert!(extract_country("nonsense").is_none());

    let candidates: Vec<&str> = extract_countries("+79161234567")
        .iter()
        .map(|c| c.code)
        .collect();
    assert_eq!(candidates, ["RU", "KZ"]);

    assert_eq!(country_by_code("DE").unwrap().prefix, 49);
    assert!(country_by_code("ZZ").is_none());
    assert_eq!(countries_by_code("PR").len(), 2);
    assert_eq!(countries_by_calling_code(44)[0].code, "GB");
    assert!(countries_by_calling_code(9999).is_empty());
    assert!(countries().len() > 200);

    assert_eq!(
        guess_country_from_number("442079460958").unwrap().code,
        "GB"
    );
    assert!(guess_country_from_number("").is_none());
}

#[test]
fn formatting_and_round_tripping() {
    for (input, format, expected) in [
        ("+12025550173", PhoneFormat::E164, "+12025550173"),
        (
            "+12025550173",
            PhoneFormat::International,
            "+1 202 555 0173",
        ),
        ("+12025550173", PhoneFormat::National, "(202) 555-0173"),
        ("+12025550173", PhoneFormat::RFC3966, "tel:+12025550173"),
        ("+442079460958", PhoneFormat::National, "020 7946 0958"),
        (
            "+442079460958",
            PhoneFormat::International,
            "+44 20 7946 0958",
        ),
    ] {
        assert_eq!(
            format_phone_number(input, format).as_deref(),
            Some(expected),
            "{input} / {format:?}"
        );
    }
    assert_eq!(format_phone_number("nonsense", PhoneFormat::E164), None);

    // The crate accepts its own RFC 3966 output.
    let uri = format_phone_number("+442079460958", PhoneFormat::RFC3966).unwrap();
    assert!(is_valid_phone_number(&uri));
    assert_eq!(
        normalize_phone_number(&uri).as_deref(),
        Some("+442079460958")
    );

    // National output round-trips through the country hint.
    let national = format_phone_number("+442079460958", PhoneFormat::National).unwrap();
    assert_eq!(
        PhoneNumber::parse_with_country(&national, "GB")
            .unwrap()
            .e164(),
        "+442079460958"
    );
}

#[test]
fn classifying_numbers() {
    assert_eq!(
        detect_phone_number_type("+18005550199"),
        Some(PhoneNumberType::TollFree)
    );
    assert_eq!(
        detect_phone_number_type("+447911123456"),
        Some(PhoneNumberType::Mobile)
    );
    assert_eq!(
        detect_phone_number_type("+442079460958"),
        Some(PhoneNumberType::FixedLine)
    );
    assert_eq!(
        detect_phone_number_type("+12025550173"),
        Some(PhoneNumberType::FixedLineOrMobile)
    );
    assert_eq!(detect_phone_number_type("nonsense"), None);

    assert!(is_mobile_number("+447911123456"));
    assert!(!is_mobile_number("+442079460958"));
    assert!(is_landline_number("+442079460958"));
    assert!(!is_landline_number("+447911123456"));
    assert!(is_toll_free_number("+18005550199"));
    assert!(!is_toll_free_number("+12025550173"));

    assert_eq!(PhoneNumberType::Mobile.to_string(), "mobile");
}

#[test]
fn comparing_and_grouping() {
    assert!(are_phone_numbers_equal(
        "+1 202-555-0173",
        "+1 (202) 555-0173"
    ));
    assert!(!are_phone_numbers_equal("+12025550173", "+12025550174"));
    assert!(!are_phone_numbers_equal("nonsense", "nonsense"));

    let groups =
        group_equivalent_phone_numbers(&["+1 202-555-0173", "+1 (202) 555-0173", "+442079460958"]);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 2);
    assert_eq!(groups[1], ["+442079460958"]);
}

#[test]
fn batch_processing() {
    let inputs = ["+12025550173", "invalid", "+442079460958"];

    assert_eq!(validate_phone_numbers_batch(&inputs), [true, false, true]);
    assert_eq!(
        normalize_phone_numbers_batch(&inputs),
        [
            Some("+12025550173".to_string()),
            None,
            Some("+442079460958".to_string())
        ]
    );
    assert_eq!(
        extract_countries_batch(&inputs)
            .iter()
            .map(|c| c.map(|c| c.code))
            .collect::<Vec<_>>(),
        [Some("US"), None, Some("GB")]
    );
    assert_eq!(
        detect_phone_number_types_batch(&inputs),
        [
            Some(PhoneNumberType::FixedLineOrMobile),
            None,
            Some(PhoneNumberType::FixedLine)
        ]
    );

    let analyses = analyze_phone_numbers_batch(&inputs);
    assert_eq!(analyses.len(), 3);
    assert!(analyses[0].is_valid);
    assert!(!analyses[1].is_valid);
    assert_eq!(analyses[1].error, Some(ParseError::InvalidCharacter));

    // The batch APIs also accept owned strings.
    let owned = vec!["+12025550173".to_string(), "invalid".to_string()];
    assert_eq!(validate_phone_numbers_batch(&owned), [true, false]);
}

#[test]
fn generating_sample_numbers() {
    let number = generate_random_phone_number("CA").expect("CA must be generatable");
    assert!(is_valid_phone_number(&number));
    assert_eq!(extract_country(&number).unwrap().code, "CA");

    let batch = generate_random_phone_numbers("US", 10);
    assert_eq!(batch.len(), 10);
    assert!(batch.iter().all(|n| is_valid_phone_number(n)));
    assert!(batch
        .iter()
        .all(|n| extract_country(n).map(|c| c.code) == Some("US")));

    assert!(generate_random_phone_number("ZZ").is_none());
    assert!(generate_random_phone_numbers("ZZ", 10).is_empty());
}

#[test]
fn suggesting_corrections() {
    assert_eq!(
        suggest_phone_number_corrections("+12025550173", None),
        ["+12025550173".to_string()]
    );
    assert_eq!(
        suggest_phone_number_corrections("00442079460958", None),
        ["+442079460958".to_string()]
    );
    assert_eq!(
        suggest_phone_number_corrections("07911 123456", Some("GB")),
        ["+447911123456".to_string()]
    );
    assert!(suggest_phone_number_corrections("nonsense", None).is_empty());

    assert!(is_potentially_valid_phone_number("202-555-0173"));
    assert!(!is_potentially_valid_phone_number("12"));
}

#[test]
fn working_with_text() {
    let text = "Call me at +1-202-555-0173 or +442079460958 today";

    let numbers = extract_phone_numbers_from_text(text);
    assert_eq!(numbers.len(), 2);
    assert_eq!(numbers[0].raw, "+1-202-555-0173");
    assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
    assert!(numbers[0].is_valid);
    assert_eq!(&text[numbers[0].start..numbers[0].end], numbers[0].raw);

    // ExtractedPhoneNumber is a public, comparable, printable type.
    let first: &ExtractedPhoneNumber = &numbers[0];
    assert_eq!(first.clone(), numbers[0]);
    assert!(format!("{first:?}").contains("+12025550173"));

    assert_eq!(count_phone_numbers_in_text(text), 2);
    assert_eq!(extract_valid_phone_numbers_from_text(text).len(), 2);

    assert_eq!(
        extract_phone_numbers_with_country_hint("Call (202) 555-0173", "US")[0]
            .normalized
            .as_deref(),
        Some("+12025550173")
    );

    assert_eq!(
        replace_phone_numbers_in_text(text, |n| format!("<{}>", n.raw)),
        "Call me at <+1-202-555-0173> or <+442079460958> today"
    );
    assert_eq!(
        redact_phone_numbers("Call +12025550173", 4),
        "Call *******0173"
    );
    assert_eq!(redact_phone_numbers("Call +12025550173", 0), "Call [PHONE]");
}

#[test]
fn using_the_phone_number_type() {
    let parsed = PhoneNumber::parse("+1 (202) 555-0173 ext. 42").expect("must parse");
    assert_eq!(parsed.original(), "+1 (202) 555-0173 ext. 42");
    assert_eq!(parsed.e164(), "+12025550173");
    assert_eq!(parsed.national_number(), "2025550173");
    assert_eq!(parsed.country().code, "US");
    assert_eq!(parsed.country_code(), 1);
    assert_eq!(parsed.phone_type(), PhoneNumberType::FixedLineOrMobile);
    assert_eq!(parsed.extension(), Some("42"));
    assert_eq!(
        parsed.format(PhoneFormat::RFC3966),
        "tel:+12025550173;ext=42"
    );
    assert!(!parsed.is_mobile());
    assert!(!parsed.is_landline());
    assert!(!parsed.is_toll_free());

    assert_eq!(
        PhoneNumber::try_parse("nonsense").unwrap_err(),
        ParseError::InvalidCharacter
    );
    assert_eq!(
        PhoneNumber::try_parse_with_country("020 7946 0958", "ZZ").unwrap_err(),
        ParseError::UnknownCountry
    );
    assert_eq!(
        PhoneNumber::parse_with_country("020 7946 0958", "GB")
            .unwrap()
            .e164(),
        "+442079460958"
    );

    // Standard traits, used the way a downstream crate would.
    let from_str = PhoneNumber::from_str("+12025550173").unwrap();
    let parsed_via_turbofish = "+12025550173".parse::<PhoneNumber>().unwrap();
    let via_try_from = PhoneNumber::try_from("+12025550173").unwrap();
    let via_try_from_string = PhoneNumber::try_from("+12025550173".to_string()).unwrap();
    assert_eq!(from_str, parsed_via_turbofish);
    assert_eq!(from_str, via_try_from);
    assert_eq!(from_str, via_try_from_string);

    assert_eq!(from_str.to_string(), "+12025550173");
    let as_ref: &str = from_str.as_ref();
    assert_eq!(as_ref, "+12025550173");

    let mut sorted = [
        PhoneNumber::parse("+12025550174").unwrap(),
        PhoneNumber::parse("+12025550173").unwrap(),
    ];
    sorted.sort();
    assert_eq!(sorted[0].e164(), "+12025550173");

    let unique: HashSet<PhoneNumber> = [
        PhoneNumber::parse("+1 202-555-0173").unwrap(),
        PhoneNumber::parse("+1 (202) 555-0173").unwrap(),
    ]
    .into_iter()
    .collect();
    assert_eq!(unique.len(), 1);
}

#[test]
fn using_the_phone_number_set() {
    let mut set = PhoneNumberSet::new();
    assert!(set.is_empty());
    assert!(set.add("+1-202-555-0173"));
    assert!(!set.add("+1 (202) 555-0173"));
    assert_eq!(set.len(), 1);

    assert_eq!(set.try_add("+442079460958"), Ok(true));
    assert_eq!(set.try_add("+44 20 7946 0958"), Ok(false));
    assert_eq!(set.try_add("nonsense"), Err(ParseError::InvalidCharacter));
    assert_eq!(set.len(), 2);

    assert!(set.contains("12025550173"));
    assert_eq!(set.get("+1 (202) 555-0173").unwrap().e164(), "+12025550173");
    assert!(set.get("nonsense").is_none());

    let mut normalized = set.normalized_numbers();
    normalized.sort_unstable();
    assert_eq!(normalized, ["+12025550173", "+442079460958"]);
    assert_eq!(set.iter().count(), 2);

    let mut by_ref: Vec<&str> = (&set).into_iter().map(PhoneNumber::e164).collect();
    by_ref.sort_unstable();
    assert_eq!(by_ref, ["+12025550173", "+442079460958"]);

    let mut extended = PhoneNumberSet::default();
    extended.extend(["+12025550173", "+442079460958", "nonsense"]);
    assert_eq!(extended, set);

    let from_refs: PhoneNumberSet = ["+12025550173", "+442079460958"].into_iter().collect();
    let from_owned: PhoneNumberSet = vec!["+12025550173".to_string(), "+442079460958".to_string()]
        .into_iter()
        .collect();
    assert_eq!(from_refs, from_owned);
    assert_eq!(from_refs, set);

    let mut owned: Vec<String> = set
        .clone()
        .into_iter()
        .map(|p| p.e164().to_string())
        .collect();
    owned.sort();
    assert_eq!(owned, ["+12025550173", "+442079460958"]);

    // Removing something that is not in the set is a no-op.
    assert!(!set.remove("+12025550174"));
    assert!(!set.remove("nonsense"));
    assert_eq!(set.len(), 2);

    // Removing accepts any formatting of a stored number.
    assert!(set.remove("12025550173"));
    assert_eq!(set.len(), 1);
    assert!(!set.remove("+1 (202) 555-0173"));
}

// ============================================================================
// Behaviour a downstream consumer would be surprised by if it regressed
// ============================================================================

#[test]
fn a_bare_national_number_is_not_assumed_to_be_american() {
    // Ten digits with no calling code are ambiguous: they read as a US national
    // number, and equally as the Egyptian +20 2555 0173. Guessing either way
    // turns one country's number into another's, so neither is offered.
    assert_eq!(normalize_phone_number("2025550173"), None);
    assert_eq!(extract_country("2025550173"), None);
    assert_eq!(
        try_normalize_phone_number("2025550173").unwrap_err(),
        ParseError::MissingCallingCode
    );

    // The two explicit ways to get a US number both work.
    assert_eq!(
        normalize_phone_number("+12025550173").as_deref(),
        Some("+12025550173")
    );
    assert_eq!(
        PhoneNumber::parse_with_country("2025550173", "US")
            .unwrap()
            .e164(),
        "+12025550173"
    );
}

#[test]
fn prose_is_rejected_but_vanity_numbers_still_work() {
    assert!(!is_valid_phone_number("HELLO WORLD"));
    assert!(!is_valid_phone_number("CALL ME NOW PLZ"));
    assert_eq!(
        normalize_phone_number("1-800-FLOWERS").as_deref(),
        Some("+18003569377")
    );
}

#[test]
fn nanp_numbers_are_range_validated() {
    assert!(!is_valid_phone_number("+11111111111"));
    assert_eq!(
        try_normalize_phone_number("+11111111111"),
        Err(ParseError::InvalidAreaCode)
    );
    assert_eq!(
        try_normalize_phone_number("+12021110173"),
        Err(ParseError::InvalidExchange)
    );
    assert!(is_valid_phone_number("+12025550173"));
}

#[test]
fn a_significant_leading_zero_is_preserved() {
    assert_eq!(
        normalize_phone_number("+390698123456").as_deref(),
        Some("+390698123456")
    );
    assert!(!are_phone_numbers_equal("+390698123456", "+39698123456"));
    // Everywhere else the leading zero is a trunk prefix.
    assert_eq!(
        normalize_phone_number("+44 (0)20 7946 0958").as_deref(),
        Some("+442079460958")
    );
}

#[test]
fn parse_errors_are_usable_as_std_errors() {
    fn fallible(input: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(try_normalize_phone_number(input)?)
    }

    assert_eq!(fallible("+12025550173").unwrap(), "+12025550173");
    let message = fallible("nonsense").unwrap_err().to_string();
    assert_eq!(message, "input contains an invalid character");
}
