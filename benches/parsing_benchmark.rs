//! Parsing benchmarks.
//!
//! The corpus below is **fixed and explicit**: every entry is a number that is
//! known to parse and to resolve to the country listed beside it. `setup()`
//! asserts that before any measurement runs, so a coverage regression fails the
//! benchmark loudly instead of silently shrinking the input set. Deriving the
//! inputs by filtering out whatever currently fails — which is what this file
//! used to do — hides exactly the regression a benchmark suite exists to catch,
//! and makes results incomparable across versions.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use phonelib::PhoneNumber;
use std::hint::black_box;

/// Canonical E.164 input: `(number, expected ISO country code)`.
const E164_CORPUS: &[(&str, &str)] = &[
    ("+12025550173", "US"),
    ("+14155552671", "US"),
    ("+18005550199", "US"),
    ("+12125550123", "US"),
    ("+13055550147", "US"),
    ("+19005550199", "US"),
    ("+14165550198", "CA"),
    ("+15145550123", "CA"),
    ("+16045550198", "CA"),
    ("+18764561234", "JM"),
    ("+12684641234", "AG"),
    ("+17875550123", "PR"),
    ("+442079460958", "GB"),
    ("+447911123456", "GB"),
    ("+441134960000", "GB"),
    ("+493012345678", "DE"),
    ("+4915112345678", "DE"),
    ("+494012345678", "DE"),
    ("+33123456789", "FR"),
    ("+33612345678", "FR"),
    ("+34612345678", "ES"),
    ("+34912345678", "ES"),
    ("+41446681800", "CH"),
    ("+31201234567", "NL"),
    ("+3222345678", "BE"),
    ("+4721234567", "NO"),
    ("+46812345678", "SE"),
    ("+4512345678", "DK"),
    ("+48221234567", "PL"),
    ("+351212345678", "PT"),
    ("+353123456789", "IE"),
    ("+302101234567", "GR"),
    ("+36123456789", "HU"),
    ("+40211234567", "RO"),
    ("+905321234567", "TR"),
    ("+390669821234", "IT"),
    ("+393331234567", "IT"),
    ("+61412345678", "AU"),
    ("+61291234567", "AU"),
    ("+819012345678", "JP"),
    ("+81312345678", "JP"),
    ("+8613800138000", "CN"),
    ("+862112345678", "CN"),
    ("+919876543210", "IN"),
    ("+916123456789", "IN"),
    ("+82221234567", "KR"),
    ("+6591234567", "SG"),
    ("+66812345678", "TH"),
    ("+85212345678", "HK"),
    ("+639171234567", "PH"),
    ("+5511987654321", "BR"),
    ("+551123456789", "BR"),
    ("+525512345678", "MX"),
    ("+541112345678", "AR"),
    ("+56912345678", "CL"),
    ("+571234567890", "CO"),
    ("+971501234567", "AE"),
    ("+972541234567", "IL"),
    ("+966501234567", "SA"),
    ("+27211234567", "ZA"),
    ("+2348012345678", "NG"),
    ("+254712345678", "KE"),
    ("+201012345678", "EG"),
    ("+79161234567", "RU"),
    ("+79031234567", "RU"),
    ("+6491234567", "NZ"),
    ("+64211234567", "NZ"),
];

/// Global numbers written the way humans actually write them: separators,
/// parentheses, surrounding whitespace, a `tel:` URI, an extension.
const MESSY_CORPUS: &[(&str, &str)] = &[
    ("+1 (202) 555-0173", "US"),
    ("+1-415-555-2671", "US"),
    ("+44 20 7946 0958", "GB"),
    (" +49 (0) 30 1234 5678 ", "DE"),
    ("+41-44-668-18-00", "CH"),
    ("+61 4 1234 5678", "AU"),
    ("+91 98765 43210", "IN"),
    ("+86 138 0013 8000", "CN"),
    ("+7 916 123 4567", "RU"),
    ("+65 9123 4567", "SG"),
    ("+52 55 1234 5678", "MX"),
    ("+972 54 123 4567", "IL"),
    ("+27 21 123 4567", "ZA"),
    ("tel:+12025550173", "US"),
    ("+1 202-555-0173 ext. 987", "US"),
    ("1-800-555-0199", "US"),
];

/// National-format input that only resolves through a country hint:
/// `(number, hint)`. Includes trunk prefixes and Italy's significant zero.
const NATIONAL_CORPUS: &[(&str, &str)] = &[
    ("(202) 555-0173", "US"),
    ("415-555-2671", "US"),
    ("416 555 0198", "CA"),
    ("020 7946 0958", "GB"),
    ("07911 123456", "GB"),
    ("030 12345678", "DE"),
    ("01 23 45 67 89", "FR"),
    ("06 6982 1234", "IT"),
    ("333 123 4567", "IT"),
    ("02 1234 5678", "AU"),
    ("03-1234-5678", "JP"),
    ("098765 43210", "IN"),
    ("021 123 4567", "NZ"),
    ("011 1234 5678", "BR"),
    ("0800 555 0199", "GB"),
    ("8 916 123 4567", "RU"),
];

/// Inputs that must be *rejected*. The error path is a real code path and is
/// measured too; asserting the rejection keeps a validation regression from
/// quietly turning this into a success-path benchmark.
const INVALID_CORPUS: &[&str] = &[
    "nonsense",
    "",
    "+11111111111",
    "+12021111111",
    "+9999999999999",
    "+1 202 555",
    "+1234567890123456",
    "(202 555-0173",
];

/// Fail before measuring if the corpus no longer matches the crate's behaviour.
fn assert_corpus_parses() {
    for (number, expected) in E164_CORPUS.iter().chain(MESSY_CORPUS.iter()) {
        let parsed = PhoneNumber::try_parse(number)
            .unwrap_or_else(|e| panic!("corpus entry {number:?} no longer parses: {e}"));
        assert_eq!(
            parsed.country().code,
            *expected,
            "corpus entry {number:?} now resolves to {}",
            parsed.country().code
        );
    }

    for (number, hint) in NATIONAL_CORPUS {
        let parsed = PhoneNumber::try_parse_with_country(number, hint).unwrap_or_else(|e| {
            panic!("corpus entry {number:?} no longer parses with hint {hint:?}: {e}")
        });
        assert_eq!(
            parsed.country().code,
            *hint,
            "corpus entry {number:?} now resolves to {}",
            parsed.country().code
        );
    }

    for number in INVALID_CORPUS {
        assert!(
            PhoneNumber::parse(number).is_none(),
            "corpus entry {number:?} is now accepted; it must stay invalid"
        );
    }
}

fn parsing_benchmark(c: &mut Criterion) {
    assert_corpus_parses();

    let mut group = c.benchmark_group("parsing");

    group.throughput(Throughput::Elements(E164_CORPUS.len() as u64));
    group.bench_function("parse/e164", |b| {
        b.iter(|| {
            for (number, _) in E164_CORPUS {
                black_box(PhoneNumber::parse(black_box(number)));
            }
        })
    });

    group.throughput(Throughput::Elements(MESSY_CORPUS.len() as u64));
    group.bench_function("parse/formatted", |b| {
        b.iter(|| {
            for (number, _) in MESSY_CORPUS {
                black_box(PhoneNumber::parse(black_box(number)));
            }
        })
    });

    group.throughput(Throughput::Elements(NATIONAL_CORPUS.len() as u64));
    group.bench_function("parse_with_country/national", |b| {
        b.iter(|| {
            for (number, hint) in NATIONAL_CORPUS {
                black_box(PhoneNumber::parse_with_country(
                    black_box(number),
                    black_box(hint),
                ));
            }
        })
    });

    group.throughput(Throughput::Elements(INVALID_CORPUS.len() as u64));
    group.bench_function("try_parse/rejected", |b| {
        b.iter(|| {
            for number in INVALID_CORPUS {
                black_box(PhoneNumber::try_parse(black_box(number)).is_err());
            }
        })
    });

    // Single-number latency, the figure most callers actually care about.
    group.throughput(Throughput::Elements(1));
    group.bench_function("parse/single", |b| {
        b.iter(|| black_box(PhoneNumber::parse(black_box("+12025550173"))))
    });
    group.bench_function("parse_with_country/single", |b| {
        b.iter(|| {
            black_box(PhoneNumber::parse_with_country(
                black_box("020 7946 0958"),
                black_box("GB"),
            ))
        })
    });

    group.finish();
}

criterion_group!(benches, parsing_benchmark);
criterion_main!(benches);
