//! Property and differential tests, run against the crate from outside.
//!
//! These do not pin individual values — `tests/public_api.rs` and the in-crate
//! unit tests do that. What they check is that the *relationships* between the
//! public functions hold for a large randomised corpus:
//!
//! * generation round-trips, or honestly returns `None`;
//! * `normalize` and `normalize_in_place` agree, always;
//! * `is_valid` and `normalize` agree, always;
//! * normalization is idempotent;
//! * E.164, RFC 3966 and National output all round-trip;
//! * the batch APIs agree with their scalar counterparts;
//! * `format_phone_number` agrees with `PhoneNumber::format`;
//! * text extraction reports exact, non-overlapping byte spans;
//! * `replace` with an identity closure is the identity on the text;
//! * nothing panics on hostile input;
//! * the `Eq`/`Hash` contract holds.

use phonelib::*;
use std::collections::{BTreeSet, HashSet};

/// Deterministic PRNG, so a failure is always reproducible.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

/// Collects violations so one run reports all of them rather than the first.
#[derive(Default)]
struct Violations(Vec<String>);

impl Violations {
    fn check(&mut self, condition: bool, message: impl FnOnce() -> String) {
        if !condition {
            self.0.push(message());
        }
    }

    #[track_caller]
    fn assert_none(&self) {
        assert!(
            self.0.is_empty(),
            "{} property violation(s):\n{}",
            self.0.len(),
            self.0.join("\n")
        );
    }
}

/// Random already-canonical-looking input: a real calling code followed by a
/// length that country accepts. About half of these are genuinely valid, which
/// is what exercises both sides of every agreement property.
fn random_candidate(rng: &mut Rng) -> String {
    let country = &countries()[rng.below(countries().len() as u64) as usize];
    let length = country.phone_lengths[rng.below(country.phone_lengths.len() as u64) as usize];

    let mut candidate = String::with_capacity(length as usize + 5);
    candidate.push('+');
    candidate.push_str(&country.prefix.to_string());
    for _ in 0..length {
        candidate.push(char::from(b'0' + rng.below(10) as u8));
    }
    candidate
}

/// Written forms of `canonical` that the crate documents itself as accepting,
/// each of which must normalize straight back to it.
///
/// The generated corpus is exclusively canonical `+CC<digits>`, so without this
/// every deep property — round trips, `normalize` vs `normalize_in_place`,
/// scalar vs batch, `Eq`/`Hash` — only ever sees input that is already in the
/// output format, and a parser regression that only affects *formatted* input
/// would have to be caught by the fixed tables alone.
fn accepted_spellings(parsed: &PhoneNumber) -> Vec<String> {
    let canonical = parsed.e164();
    let digits = &canonical[1..];

    let mut spaced = String::from("+");
    for digit in digits.chars() {
        spaced.push(digit);
        spaced.push(' ');
    }

    vec![
        // The crate's own renderings.
        parsed.format(PhoneFormat::E164),
        parsed.format(PhoneFormat::International),
        parsed.format(PhoneFormat::RFC3966),
        // Written forms a caller might paste in.
        format!("{canonical} ext. 42"),
        format!("  {canonical}  "),
        // An IDD prefix instead of the '+'.
        format!("00{digits}"),
        // Separators and parentheses scattered through the digits.
        format!("+{}-{}", &digits[..2], &digits[2..]),
        format!("+{}({}){}", &digits[..1], &digits[1..3], &digits[3..]),
        spaced.trim_end().to_string(),
    ]
}

#[test]
fn generation_round_trips_or_returns_none() {
    let mut ungeneratable = BTreeSet::new();

    for country in countries() {
        match generate_random_phone_number(country.code) {
            Some(number) => {
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
            None => {
                ungeneratable.insert(country.code);
            }
        }
    }

    // Only entries shadowed by another country on the same calling code may be
    // ungeneratable, and the set is documented in the crate docs, so it is
    // pinned exactly: a country joining it is a resolution regression and a
    // country leaving it is a fix, and either way the docs have to change.
    let expected: BTreeSet<&str> = [
        "AX", "BL", "CC", "CX", "EH", "GG", "GS", "IM", "JE", "KZ", "MF", "SJ", "VA", "YT",
    ]
    .into_iter()
    .collect();
    assert_eq!(ungeneratable, expected);
    for reachable in ["US", "CA", "GB", "DE", "FR", "IT", "JP", "AU", "IN", "BR"] {
        assert!(
            !ungeneratable.contains(reachable),
            "{reachable} must be generatable"
        );
    }
}

#[test]
fn parsing_relationships_hold_over_a_random_corpus() {
    let mut rng = Rng::new(0x1234_5678_9abc_def0);
    let mut violations = Violations::default();

    for iteration in 0..40_000 {
        let input = random_candidate(&mut rng);
        let normalized = normalize_phone_number(&input);

        // --- normalize vs normalize_in_place ---
        let mut buffer = input.clone();
        let in_place = normalize_phone_number_in_place(&mut buffer);
        violations.check(normalized == in_place, || {
            format!("normalize vs in_place on {input:?}: {normalized:?} vs {in_place:?}")
        });
        match &in_place {
            Some(value) => violations.check(&buffer == value, || {
                format!("in_place buffer {buffer:?} != return {value:?} for {input:?}")
            }),
            None => violations.check(buffer == input, || {
                format!("in_place mutated the buffer on failure for {input:?}: {buffer:?}")
            }),
        }

        // --- is_valid vs normalize ---
        violations.check(
            is_valid_phone_number(&input) == normalized.is_some(),
            || format!("is_valid vs normalize on {input:?}"),
        );

        // --- try_ variants vs Option variants ---
        violations.check(
            try_normalize_phone_number(&input).ok() == normalized,
            || format!("try_normalize vs normalize on {input:?}"),
        );
        violations.check(
            PhoneNumber::try_parse(&input).is_ok() == PhoneNumber::parse(&input).is_some(),
            || format!("try_parse vs parse on {input:?}"),
        );

        let Some(number) = normalized else {
            continue;
        };

        // --- idempotency ---
        violations.check(
            normalize_phone_number(&number).as_deref() == Some(number.as_str()),
            || format!("not idempotent: {input:?} -> {number:?}"),
        );

        let parsed = PhoneNumber::parse(&number).expect("a normalized number must re-parse");

        // --- format round trips ---
        for format in [PhoneFormat::E164, PhoneFormat::RFC3966] {
            let rendered = parsed.format(format);
            violations.check(
                normalize_phone_number(&rendered).as_deref() == Some(number.as_str()),
                || format!("{format:?} does not round-trip: {number} -> {rendered}"),
            );
        }

        let national = parsed.format(PhoneFormat::National);
        let recovered = PhoneNumber::parse_with_country(&national, parsed.country().code)
            .map(|p| p.e164().to_string());
        violations.check(recovered.as_deref() == Some(number.as_str()), || {
            format!(
                "National does not round-trip for {}: {number} -> {national:?} -> {recovered:?}",
                parsed.country().code
            )
        });

        // --- every documented written form re-parses to the same number ---
        //
        // Nine spellings per number times five checks each is the most expensive
        // block in the file, so it runs on a fixed quarter of the corpus —
        // still some ten thousand distinct numbers from the same distribution.
        let spellings = if iteration % 4 == 0 {
            accepted_spellings(&parsed)
        } else {
            Vec::new()
        };
        for spelling in spellings {
            violations.check(
                normalize_phone_number(&spelling).as_deref() == Some(number.as_str()),
                || {
                    format!(
                        "{spelling:?} must normalize to {number}, got {:?}",
                        normalize_phone_number(&spelling)
                    )
                },
            );
            violations.check(is_valid_phone_number(&spelling), || {
                format!("{spelling:?} is a written form of {number} but is not valid")
            });
            // The formatted spellings must go through the batch and in-place
            // paths identically, not only through `normalize_phone_number`.
            let mut buffer = spelling.clone();
            let in_place = normalize_phone_number_in_place(&mut buffer);
            violations.check(in_place.as_deref() == Some(number.as_str()), || {
                format!("in_place disagrees on the written form {spelling:?}")
            });
            let analysis = &analyze_phone_numbers_batch(&[spelling.as_str()])[0];
            violations.check(
                analysis.normalized.as_deref() == Some(number.as_str()),
                || format!("analysis disagrees on the written form {spelling:?}"),
            );
            violations.check(are_phone_numbers_equal(&spelling, &number), || {
                format!("{spelling:?} is not equal to {number}")
            });
        }

        // --- free function vs method ---
        for format in [
            PhoneFormat::E164,
            PhoneFormat::International,
            PhoneFormat::National,
            PhoneFormat::RFC3966,
        ] {
            violations.check(
                format_phone_number(&number, format).as_deref() == Some(parsed.format(format).as_str()),
                || format!("format_phone_number disagrees with PhoneNumber::format on {number} / {format:?}"),
            );
        }

        // --- accessors are consistent with the E.164 string ---
        violations.check(number.ends_with(parsed.national_number()), || {
            format!(
                "{number} does not end with its national number {:?}",
                parsed.national_number()
            )
        });
        violations.check(parsed.country_code() == parsed.country().prefix, || {
            format!("country_code disagrees with country().prefix on {number}")
        });

        // --- scalar vs batch ---
        let analysis = &analyze_phone_numbers_batch(&[number.as_str()])[0];
        violations.check(analysis.is_valid, || format!("analysis rejected {number}"));
        violations.check(
            analysis.normalized.as_deref() == Some(number.as_str()),
            || format!("analysis normalized {number} differently"),
        );
        violations.check(
            analysis.country.map(|c| c.code) == extract_country(&number).map(|c| c.code),
            || format!("analysis country disagrees on {number}"),
        );
        violations.check(
            analysis.phone_type == detect_phone_number_type(&number),
            || format!("analysis type disagrees on {number}"),
        );
        violations.check(analysis.error.is_none(), || {
            format!("a valid number carries an error: {number}")
        });

        // --- resolution consistency ---
        let candidates = extract_countries(&number);
        violations.check(
            candidates.first().map(|c| c.code) == extract_country(&number).map(|c| c.code),
            || format!("extract_countries[0] disagrees with extract_country on {number}"),
        );

        // --- validity implies potential validity ---
        violations.check(is_potentially_valid_phone_number(&number), || {
            format!("{number} is valid but not potentially valid")
        });

        // --- equality is reflexive and formatting-insensitive ---
        violations.check(are_phone_numbers_equal(&number, &number), || {
            format!("{number} is not equal to itself")
        });
        violations.check(
            are_phone_numbers_equal(&number, &parsed.format(PhoneFormat::E164)),
            || format!("{number} is not equal to its own E.164 rendering"),
        );
    }

    violations.assert_none();
}

#[test]
fn batch_apis_agree_with_their_scalar_counterparts() {
    let mut rng = Rng::new(0xdead_beef_cafe_f00d);
    let corpus: Vec<String> = (0..2_000)
        .map(|index| {
            if index % 5 == 0 {
                // Deliberately unparseable inputs, so the negative side of every
                // agreement is exercised too.
                format!("junk-{index}")
            } else {
                random_candidate(&mut rng)
            }
        })
        .collect();

    let validity = validate_phone_numbers_batch(&corpus);
    let normalized = normalize_phone_numbers_batch(&corpus);
    let resolved = extract_countries_batch(&corpus);
    let types = detect_phone_number_types_batch(&corpus);
    let analyses = analyze_phone_numbers_batch(&corpus);

    assert_eq!(validity.len(), corpus.len());
    assert_eq!(normalized.len(), corpus.len());
    assert_eq!(resolved.len(), corpus.len());
    assert_eq!(types.len(), corpus.len());
    assert_eq!(analyses.len(), corpus.len());

    for (index, input) in corpus.iter().enumerate() {
        assert_eq!(validity[index], is_valid_phone_number(input), "{input:?}");
        assert_eq!(
            normalized[index],
            normalize_phone_number(input),
            "{input:?}"
        );
        assert_eq!(
            resolved[index].map(|c| c.code),
            extract_country(input).map(|c| c.code),
            "{input:?}"
        );
        assert_eq!(types[index], detect_phone_number_type(input), "{input:?}");

        assert_eq!(analyses[index].original, *input);
        assert_eq!(analyses[index].is_valid, validity[index], "{input:?}");
        assert_eq!(analyses[index].normalized, normalized[index], "{input:?}");
        assert_eq!(analyses[index].phone_type, types[index], "{input:?}");
        assert_eq!(
            analyses[index].error.is_none(),
            analyses[index].is_valid,
            "{input:?}: error and is_valid disagree"
        );
    }

    // Grouping never loses or duplicates an input.
    let groups = group_equivalent_phone_numbers(&corpus);
    let total: usize = groups.iter().map(Vec::len).sum();
    assert_eq!(total, corpus.len(), "grouping changed the input count");
}

#[test]
fn text_extraction_reports_exact_non_overlapping_spans() {
    let mut rng = Rng::new(0x0bad_c0de_1234_5678);

    for _ in 0..3_000 {
        // Build prose out of words, numbers and multi-byte characters so the
        // byte offsets are non-trivial. Several of the branches emit *formatted*
        // numbers — the crate's own renderings and the written forms it accepts
        // — so the scanner is not only ever fed canonical E.164 strings.
        let mut text = String::new();
        for _ in 0..rng.below(8) + 1 {
            match rng.below(10) {
                0 => text.push_str("call "),
                1 => text.push_str(&random_candidate(&mut rng)),
                2 => text.push_str("(202) 555-0173"),
                3 => text.push_str("caf\u{00E9} \u{1F4DE} \u{7DCA}\u{6025}"),
                4 => text.push_str("v1.2.3 build 2024-01-15"),
                5 => text.push_str("name,+12025550173,x"),
                6..=8 => {
                    let candidate = random_candidate(&mut rng);
                    match PhoneNumber::parse(&candidate) {
                        Some(parsed) => {
                            let spellings = accepted_spellings(&parsed);
                            text.push_str(&spellings[rng.below(spellings.len() as u64) as usize]);
                        }
                        None => text.push_str(&candidate),
                    }
                }
                _ => text.push_str(" \u{00A0}\u{2013} "),
            }
            text.push(' ');
        }

        let numbers = extract_phone_numbers_from_text(&text);
        let mut previous_end = 0;
        for number in &numbers {
            assert!(number.end > number.start, "empty span in {text:?}");
            assert!(
                number.start >= previous_end,
                "overlapping spans in {text:?}"
            );
            assert_eq!(
                &text[number.start..number.end],
                number.raw,
                "span does not slice back to raw in {text:?}"
            );
            assert_eq!(
                number.is_valid,
                number.normalized.is_some(),
                "is_valid disagrees with normalized in {text:?}"
            );
            if let Some(normalized) = &number.normalized {
                assert_eq!(
                    normalize_phone_number(normalized).as_deref(),
                    Some(normalized.as_str()),
                    "extractor emitted a non-canonical number in {text:?}"
                );
            }
            previous_end = number.end;
        }

        assert_eq!(count_phone_numbers_in_text(&text), numbers.len());
        assert_eq!(
            extract_valid_phone_numbers_from_text(&text).len(),
            numbers.iter().filter(|n| n.is_valid).count()
        );

        // Replacing each match with itself is the identity on the text.
        assert_eq!(
            replace_phone_numbers_in_text(&text, |n| n.raw.clone()),
            text,
            "identity replacement changed the text"
        );

        // Redaction never leaves a digit of a redacted number behind.
        let redacted = redact_phone_numbers(&text, 0);
        for number in &numbers {
            assert!(
                !redacted.contains(&number.raw),
                "redaction left {:?} intact in {text:?}",
                number.raw
            );
        }
    }
}

/// The two hinted-parse entry points must agree.
///
/// `extract_phone_numbers_with_country_hint` used to reimplement trunk-prefix
/// handling — it stripped a leading `0` unconditionally — so it disagreed with
/// `PhoneNumber::parse_with_country` for every plan whose leading zero is
/// significant, and answered with a *different, still valid* number rather than
/// failing. Nothing in the suite compared the two, and the fixed tables happened
/// to contain no Italian row.
#[test]
fn the_text_hint_agrees_with_parse_with_country() {
    let mut exercised = 0usize;
    let mut violations = Violations::default();

    for country in countries() {
        for _ in 0..8 {
            let Some(number) = generate_random_phone_number(country.code) else {
                // The 14 shadowed countries; `generation_round_trips_or_returns_none`
                // pins exactly which ones those are.
                break;
            };
            let parsed = PhoneNumber::parse(&number).expect("a generated number must parse");
            for format in [
                PhoneFormat::National,
                PhoneFormat::International,
                PhoneFormat::E164,
                PhoneFormat::RFC3966,
            ] {
                let written = parsed.format(format);
                let extracted = extract_phone_numbers_with_country_hint(&written, country.code);
                let direct = PhoneNumber::parse_with_country(&written, country.code)
                    .map(|p| p.e164().to_string());

                violations.check(extracted.len() == 1, || {
                    format!(
                        "{} {format:?} {written:?} produced {} spans, not 1",
                        country.code,
                        extracted.len()
                    )
                });
                let got = extracted.first().and_then(|n| n.normalized.clone());
                violations.check(got == direct, || {
                    format!(
                        "{} {format:?} {written:?}: extractor says {got:?}, \
                         parse_with_country says {direct:?}",
                        country.code
                    )
                });
                violations.check(direct.as_deref() == Some(parsed.e164()), || {
                    format!(
                        "{} {format:?} {written:?} does not round-trip to {}",
                        country.code,
                        parsed.e164()
                    )
                });
                exercised += 1;
            }
        }
    }

    violations.assert_none();
    assert!(
        exercised > 3_000,
        "only {exercised} hinted parses were exercised"
    );
}

#[test]
fn hostile_input_never_panics() {
    let mut hostile: Vec<String> = vec![
        String::new(),
        "+".into(),
        "+++".into(),
        "0".into(),
        "0000000000000".into(),
        "   ".into(),
        "\u{0}\u{1}".into(),
        "(((((((((".into(),
        ")))".into(),
        "+1(202)555-0173)".into(),
        "9".repeat(1000),
        "1".repeat(100_000),
        "+1 202 555 0173 ext. ".into(),
        "ext. 1234".into(),
        "tel:".into(),
        "tel:;ext=".into(),
        "\u{FF0B}\u{FF18}\u{FF11}\u{FF13}\u{FF11}\u{FF12}\u{FF13}\u{FF14}\u{FF15}\u{FF16}\u{FF17}\u{FF18}".into(),
        "\u{202F}\u{00A0}\u{2013}".into(),
        "\u{1F600}+12025550173\u{1F600}".into(),
        "\u{0661}\u{0662}\u{0663}\u{0664}\u{0665}\u{0666}\u{0667}\u{0668}\u{0669}\u{0660}".into(),
        "\u{00E9}".repeat(64),
    ];

    // Random junk from an alphabet chosen to hit every branch of the scanner.
    let mut rng = Rng::new(0xfeed_face_dead_babe);
    for _ in 0..20_000 {
        let length = rng.below(24) as usize;
        hostile.push(
            (0..length)
                .map(|_| {
                    const ALPHABET: &[u8] = b"0123456789+-(). abcXYZ#,;*/\t";
                    char::from(ALPHABET[rng.below(ALPHABET.len() as u64) as usize])
                })
                .collect(),
        );
    }

    for input in &hostile {
        let _ = is_valid_phone_number(input);
        let _ = extract_country(input);
        let _ = extract_countries(input);
        let _ = detect_phone_number_type(input);
        let _ = guess_country_from_number(input);
        let _ = is_potentially_valid_phone_number(input);
        let _ = is_mobile_number(input);
        let _ = is_landline_number(input);
        let _ = is_toll_free_number(input);
        let _ = suggest_phone_number_corrections(input, Some("US"));
        let _ = suggest_phone_number_corrections(input, None);
        let _ = PhoneNumber::parse(input);
        let _ = PhoneNumber::parse_with_country(input, "US");
        let _ = count_phone_numbers_in_text(input);
        let _ = redact_phone_numbers(input, 4);
        let _ = replace_phone_numbers_in_text(input, |n| n.raw.clone());
        let _ = extract_phone_numbers_with_country_hint(input, "GB");

        let mut owned = input.clone();
        let _ = normalize_phone_number_in_place(&mut owned);

        for format in [
            PhoneFormat::E164,
            PhoneFormat::International,
            PhoneFormat::National,
            PhoneFormat::RFC3966,
        ] {
            let _ = format_phone_number(input, format);
        }

        // Whatever survives normalization must be a well-formed E.164 string.
        if let Some(number) = normalize_phone_number(input) {
            assert!(number.starts_with('+'), "{number:?} from {input:?}");
            assert!(
                number[1..].bytes().all(|b| b.is_ascii_digit()),
                "{number:?} from {input:?}"
            );
            assert!(number.len() <= 16, "{number:?} exceeds the E.164 ceiling");
            assert_eq!(
                normalize_phone_number(&number).as_deref(),
                Some(number.as_str())
            );
        }
    }
}

#[test]
fn equality_hashing_and_ordering_are_consistent() {
    let mut rng = Rng::new(0x5eed_0001_0002_0003);
    let mut parsed: Vec<PhoneNumber> = Vec::new();

    for _ in 0..2_000 {
        if let Some(number) = PhoneNumber::parse(&random_candidate(&mut rng)) {
            parsed.push(number);
        }
    }
    assert!(
        parsed.len() > 100,
        "the corpus produced too few valid numbers"
    );

    // Equal values hash equally, so a HashSet collapses exactly the duplicates.
    let unique_e164: BTreeSet<&str> = parsed.iter().map(PhoneNumber::e164).collect();
    let hashed: HashSet<PhoneNumber> = parsed.iter().cloned().collect();
    assert_eq!(
        hashed.len(),
        unique_e164.len(),
        "the Eq/Hash contract is broken"
    );

    // Ordering agrees with the E.164 string ordering.
    let mut sorted = parsed.clone();
    sorted.sort();
    let mut expected: Vec<&str> = parsed.iter().map(PhoneNumber::e164).collect();
    expected.sort_unstable();
    assert_eq!(
        sorted.iter().map(PhoneNumber::e164).collect::<Vec<_>>(),
        expected
    );

    // Different spellings of the same number are equal, hash equally and are
    // interchangeable as set keys.
    let a = PhoneNumber::parse("+1 202-555-0173").unwrap();
    let b = PhoneNumber::parse("+1 (202) 555-0173").unwrap();
    let c = PhoneNumber::parse("tel:+12025550173").unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
    let set: HashSet<PhoneNumber> = [a, b, c].into_iter().collect();
    assert_eq!(set.len(), 1);
}

#[test]
fn phone_number_set_deduplicates_exactly_like_normalization() {
    let mut rng = Rng::new(0xabcd_0000_1111_2222);
    let corpus: Vec<String> = (0..3_000).map(|_| random_candidate(&mut rng)).collect();

    let mut set = PhoneNumberSet::new();
    for number in &corpus {
        set.add(number);
    }

    let expected: BTreeSet<String> = corpus
        .iter()
        .filter_map(|n| normalize_phone_number(n))
        .collect();
    assert_eq!(set.len(), expected.len());

    let stored: BTreeSet<&str> = set.normalized_numbers().into_iter().collect();
    assert_eq!(
        stored,
        expected.iter().map(String::as_str).collect::<BTreeSet<_>>()
    );

    for number in &corpus {
        assert_eq!(
            set.contains(number),
            normalize_phone_number(number).is_some(),
            "{number}"
        );
        assert_eq!(
            set.get(number).map(|p| p.e164().to_string()),
            normalize_phone_number(number),
            "{number}"
        );
    }

    // Removing every stored key empties the set.
    for number in set
        .normalized_numbers()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
    {
        assert!(set.remove(&number));
    }
    assert!(set.is_empty());
}

#[test]
fn suggestions_are_always_valid_and_bounded() {
    let mut rng = Rng::new(0x9999_8888_7777_6666);

    for _ in 0..2_000 {
        let mut input = random_candidate(&mut rng);
        // Damage the number so the repair tiers are actually exercised.
        match rng.below(3) {
            0 => {
                input.pop();
            }
            1 => input.push('7'),
            _ => {
                input = input.trim_start_matches('+').to_string();
            }
        }

        for hint in [None, Some("US"), Some("GB")] {
            let suggestions = suggest_phone_number_corrections(&input, hint);
            assert!(
                suggestions.len() <= 5,
                "{input:?} with hint {hint:?} returned {} suggestions",
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
}
