//! Formatting benchmarks.
//!
//! As in `parsing_benchmark.rs`, the corpus is fixed and asserted at setup
//! rather than filtered down to whatever happens to parse today.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use phonelib::{format_phone_number, PhoneFormat, PhoneNumber};
use std::hint::black_box;

/// `(E.164 number, expected ISO country code)`.
///
/// Chosen to exercise every branch of the formatter: the NANP `(AAA) NNN-NNNN`
/// shape, a four-digit territory calling code, countries with a trunk prefix
/// (`GB`, `DE`, `AU`, `JP`, `RU`), countries without one (`IT`, `ES`, `PT`),
/// Italy's significant leading zero, the per-country grouping tables, and the
/// length-based fallback grouping.
const CORPUS: &[(&str, &str)] = &[
    ("+12025550173", "US"),
    ("+14155552671", "US"),
    ("+18005550199", "US"),
    ("+14165550198", "CA"),
    ("+18764561234", "JM"),
    ("+12684641234", "AG"),
    ("+442079460958", "GB"),
    ("+447911123456", "GB"),
    ("+493012345678", "DE"),
    ("+4915112345678", "DE"),
    ("+33123456789", "FR"),
    ("+33612345678", "FR"),
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
    ("+82221234567", "KR"),
    ("+6591234567", "SG"),
    ("+85212345678", "HK"),
    ("+5511987654321", "BR"),
    ("+551123456789", "BR"),
    ("+525512345678", "MX"),
    ("+56912345678", "CL"),
    ("+971501234567", "AE"),
    ("+972541234567", "IL"),
    ("+27211234567", "ZA"),
    ("+2348012345678", "NG"),
    ("+79161234567", "RU"),
    ("+6491234567", "NZ"),
];

const FORMATS: &[(&str, PhoneFormat)] = &[
    ("e164", PhoneFormat::E164),
    ("international", PhoneFormat::International),
    ("national", PhoneFormat::National),
    ("rfc3966", PhoneFormat::RFC3966),
];

fn parse_corpus() -> Vec<PhoneNumber> {
    CORPUS
        .iter()
        .map(|(number, expected)| {
            let parsed = PhoneNumber::try_parse(number)
                .unwrap_or_else(|e| panic!("corpus entry {number:?} no longer parses: {e}"));
            assert_eq!(
                parsed.country().code,
                *expected,
                "corpus entry {number:?} now resolves to {}",
                parsed.country().code
            );
            parsed
        })
        .collect()
}

fn formatting_benchmark(c: &mut Criterion) {
    let numbers = parse_corpus();

    // Formatting an already-parsed number.
    let mut group = c.benchmark_group("formatting/parsed");
    group.throughput(Throughput::Elements(numbers.len() as u64));
    for (label, format) in FORMATS {
        group.bench_function(*label, |b| {
            b.iter(|| {
                for number in &numbers {
                    black_box(number.format(black_box(*format)));
                }
            })
        });
    }
    group.finish();

    // Single-number latency for the same work.
    let one = PhoneNumber::parse("+12025550173").unwrap();
    let mut group = c.benchmark_group("formatting/single");
    for (label, format) in FORMATS {
        group.bench_function(*label, |b| {
            b.iter(|| black_box(one.format(black_box(*format))))
        });
    }
    group.finish();

    // The free function, which parses and formats in one call — the path most
    // callers take when they only need a string out.
    let mut group = c.benchmark_group("formatting/parse_and_format");
    group.throughput(Throughput::Elements(CORPUS.len() as u64));
    for (label, format) in FORMATS {
        group.bench_function(*label, |b| {
            b.iter(|| {
                for (number, _) in CORPUS {
                    black_box(format_phone_number(black_box(number), black_box(*format)));
                }
            })
        });
    }
    group.finish();
}

criterion_group!(benches, formatting_benchmark);
criterion_main!(benches);
