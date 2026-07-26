//! End-to-end benchmarks for the crate's public surface.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use phonelib::{
    are_phone_numbers_equal, detect_phone_number_type, extract_country,
    extract_phone_numbers_from_text, extract_phone_numbers_with_country_hint, format_phone_number,
    group_equivalent_phone_numbers, is_valid_phone_number, normalize_phone_number,
    normalize_phone_numbers_batch, redact_phone_numbers, validate_phone_numbers_batch, PhoneFormat,
    PhoneNumber,
};
use std::hint::black_box;

const TEST_PHONES: &[&str] = &[
    "+12025550173",
    "+442079460958",
    "+919876543210",
    "+493012345678",
    "+61412345678",
    "+8613800138000",
    "+33123456789",
    "+5511912345678",
    "+819012345678",
    "+14155552671",
];

const TEST_TEXT: &str = "Contact us at +12025550173 or call our UK office at +442079460958. \
    For Asia, reach us at +919876543210 or +8613800138000. Our European numbers are \
    +493012345678 and +33123456789. Australian customers can call +61412345678.";

/// Digit-dense text of at least `target` bytes.
///
/// The unit carries four phone numbers per ~180 bytes, so the match count grows
/// with the input. That is deliberate: text extraction used to be quadratic in
/// (input size x match count), and only an input that scales both catches a
/// regression back to it.
fn text_of_size(target: usize) -> String {
    const UNIT: &str = "Contact +1 202-555-0173 or +44 20 7946 0958 for details; the Berlin \
        office is on +49 30 1234 5678 and Tokyo on +81 3 1234 5678. ";
    let mut out = String::with_capacity(target + UNIT.len());
    while out.len() < target {
        out.push_str(UNIT);
    }
    out
}

// ============================================================================
// Core Function Benchmarks
// ============================================================================

fn bench_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");

    group.bench_function("single_valid", |b| {
        b.iter(|| black_box(is_valid_phone_number(black_box("+12025550173"))))
    });

    group.bench_function("single_invalid", |b| {
        b.iter(|| black_box(is_valid_phone_number(black_box("invalid_number"))))
    });

    // A structurally plausible number rejected by the NANP area-code table —
    // the range check that plain length validation cannot do.
    group.bench_function("single_unassigned_nanp", |b| {
        b.iter(|| black_box(is_valid_phone_number(black_box("+11111111111"))))
    });

    group.throughput(Throughput::Elements(TEST_PHONES.len() as u64));
    group.bench_function("multiple_10", |b| {
        b.iter(|| {
            for phone in TEST_PHONES {
                black_box(is_valid_phone_number(black_box(*phone)));
            }
        })
    });

    group.finish();
}

fn bench_normalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("normalization");

    group.bench_function("clean_number", |b| {
        b.iter(|| black_box(normalize_phone_number(black_box("+12025550173"))))
    });

    group.bench_function("dirty_number", |b| {
        b.iter(|| black_box(normalize_phone_number(black_box("+1 (202) 555-0173"))))
    });

    group.throughput(Throughput::Elements(TEST_PHONES.len() as u64));
    group.bench_function("multiple_10", |b| {
        b.iter(|| {
            for phone in TEST_PHONES {
                black_box(normalize_phone_number(black_box(*phone)));
            }
        })
    });

    group.finish();
}

fn bench_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("country_extraction");

    group.bench_function("single", |b| {
        b.iter(|| black_box(extract_country(black_box("+12025550173"))))
    });

    // A four-digit territory calling code takes the longer resolution path.
    group.bench_function("single_territory", |b| {
        b.iter(|| black_box(extract_country(black_box("+18764561234"))))
    });

    group.throughput(Throughput::Elements(TEST_PHONES.len() as u64));
    group.bench_function("multiple_10", |b| {
        b.iter(|| {
            for phone in TEST_PHONES {
                black_box(extract_country(black_box(*phone)));
            }
        })
    });

    group.finish();
}

// ============================================================================
// Formatting Benchmarks
// ============================================================================

fn bench_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("formatting");

    for (label, format) in [
        ("e164", PhoneFormat::E164),
        ("international", PhoneFormat::International),
        ("national", PhoneFormat::National),
        ("rfc3966", PhoneFormat::RFC3966),
    ] {
        group.bench_function(label, |b| {
            b.iter(|| {
                black_box(format_phone_number(
                    black_box("+12025550173"),
                    black_box(format),
                ))
            })
        });
    }

    group.finish();
}

// ============================================================================
// Type Detection Benchmarks
// ============================================================================

fn bench_type_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_detection");

    group.bench_function("single", |b| {
        b.iter(|| black_box(detect_phone_number_type(black_box("+12025550173"))))
    });

    group.throughput(Throughput::Elements(TEST_PHONES.len() as u64));
    group.bench_function("multiple_10", |b| {
        b.iter(|| {
            for phone in TEST_PHONES {
                black_box(detect_phone_number_type(black_box(*phone)));
            }
        })
    });

    group.finish();
}

// ============================================================================
// Batch Processing Benchmarks
// ============================================================================

fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    group.throughput(Throughput::Elements(TEST_PHONES.len() as u64));
    group.bench_function("validate_batch_10", |b| {
        b.iter(|| black_box(validate_phone_numbers_batch(black_box(TEST_PHONES))))
    });

    group.bench_function("normalize_batch_10", |b| {
        b.iter(|| black_box(normalize_phone_numbers_batch(black_box(TEST_PHONES))))
    });

    group.bench_function("validate_sequential_10", |b| {
        b.iter(|| {
            let results: Vec<bool> = TEST_PHONES
                .iter()
                .map(|p| is_valid_phone_number(p))
                .collect();
            black_box(results)
        })
    });

    group.finish();
}

// ============================================================================
// Text Extraction Benchmarks
//
// Sizes run to 256 KB. The old suite stopped at 2.2 KB, where the then
// quadratic curve was still flat enough to look linear; anything short of tens
// of kilobytes cannot tell the two apart.
// ============================================================================

fn bench_text_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_extraction");
    group.sample_size(20);

    group.throughput(Throughput::Bytes(TEST_TEXT.len() as u64));
    group.bench_function("sentence", |b| {
        b.iter(|| black_box(extract_phone_numbers_from_text(black_box(TEST_TEXT))))
    });

    for kib in [2usize, 32, 64, 128, 256] {
        let text = text_of_size(kib * 1024);
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("scan", format!("{kib}KiB")),
            &text,
            |b, t| b.iter(|| black_box(extract_phone_numbers_from_text(black_box(t.as_str())))),
        );
    }

    // Text with no numbers at all: the scanner's reject path.
    let prose = "the quick brown fox jumps over the lazy dog ".repeat(1024);
    group.throughput(Throughput::Bytes(prose.len() as u64));
    group.bench_function("prose_no_matches_44KiB", |b| {
        b.iter(|| black_box(extract_phone_numbers_from_text(black_box(prose.as_str()))))
    });

    group.finish();
}

fn bench_text_country_hint(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_country_hint");

    const NATIONAL_TEXT: &str = "Reception is on (202) 555-0173, accounts on 415-555-2671, \
        and the warehouse on 305-555-0147. After hours try 212-555-0123.";
    group.throughput(Throughput::Bytes(NATIONAL_TEXT.len() as u64));
    group.bench_function("sentence_us", |b| {
        b.iter(|| {
            black_box(extract_phone_numbers_with_country_hint(
                black_box(NATIONAL_TEXT),
                black_box("US"),
            ))
        })
    });

    let text = NATIONAL_TEXT.repeat(512);
    group.throughput(Throughput::Bytes(text.len() as u64));
    group.bench_function("bulk_us_66KiB", |b| {
        b.iter(|| {
            black_box(extract_phone_numbers_with_country_hint(
                black_box(text.as_str()),
                black_box("US"),
            ))
        })
    });

    group.finish();
}

fn bench_redaction(c: &mut Criterion) {
    let mut group = c.benchmark_group("redaction");

    group.throughput(Throughput::Bytes(TEST_TEXT.len() as u64));
    group.bench_function("sentence_visible_4", |b| {
        b.iter(|| black_box(redact_phone_numbers(black_box(TEST_TEXT), black_box(4))))
    });
    group.bench_function("sentence_full", |b| {
        b.iter(|| black_box(redact_phone_numbers(black_box(TEST_TEXT), black_box(0))))
    });

    for kib in [32usize, 128] {
        let text = text_of_size(kib * 1024);
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("bulk_visible_4", format!("{kib}KiB")),
            &text,
            |b, t| b.iter(|| black_box(redact_phone_numbers(black_box(t.as_str()), black_box(4)))),
        );
    }

    group.finish();
}

// ============================================================================
// Comparison Benchmarks
// ============================================================================

fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");

    group.bench_function("are_equal_same", |b| {
        b.iter(|| {
            black_box(are_phone_numbers_equal(
                black_box("+12025550173"),
                black_box("12025550173"),
            ))
        })
    });

    group.bench_function("are_equal_different", |b| {
        b.iter(|| {
            black_box(are_phone_numbers_equal(
                black_box("+12025550173"),
                black_box("+442079460958"),
            ))
        })
    });

    group.finish();
}

/// Grouping used to be quadratic with two re-normalizations per comparison; it
/// is now a single hash pass. The sizes below make that difference visible.
fn bench_grouping(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouping");

    for size in [10usize, 100, 1000, 5000] {
        // A mix of duplicates, distinct numbers and unparseable junk, so both
        // the hit and the miss path are exercised.
        let numbers: Vec<String> = (0..size)
            .map(|i| match i % 4 {
                0 => "+1 202-555-0173".to_string(),
                1 => "+1 (202) 555-0173".to_string(),
                2 => format!("+4420794{:05}", i % 100_000),
                _ => format!("not-a-number-{i}"),
            })
            .collect();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("group_equivalent", size),
            &numbers,
            |b, numbers| {
                b.iter(|| {
                    black_box(group_equivalent_phone_numbers(black_box(
                        numbers.as_slice(),
                    )))
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// PhoneNumber Struct Benchmarks
//
// `e164()` is a plain field borrow and was previously benchmarked as if it were
// work; it is not measured here. What is measured is parsing, the error path,
// and formatting.
// ============================================================================

fn bench_phone_number_struct(c: &mut Criterion) {
    let mut group = c.benchmark_group("phone_number_struct");

    group.bench_function("parse", |b| {
        b.iter(|| black_box(PhoneNumber::parse(black_box("+12025550173"))))
    });

    group.bench_function("parse_with_extension", |b| {
        b.iter(|| black_box(PhoneNumber::parse(black_box("+1 202-555-0173 ext. 987"))))
    });

    group.bench_function("try_parse_rejected", |b| {
        b.iter(|| black_box(PhoneNumber::try_parse(black_box("+11111111111")).is_err()))
    });

    group.bench_function("parse_with_country", |b| {
        b.iter(|| {
            black_box(PhoneNumber::parse_with_country(
                black_box("020 7946 0958"),
                black_box("GB"),
            ))
        })
    });

    let phone = PhoneNumber::parse("+12025550173").unwrap();

    group.bench_function("format_international", |b| {
        b.iter(|| black_box(phone.format(black_box(PhoneFormat::International))))
    });

    group.finish();
}

// ============================================================================
// Scaling Benchmarks
// ============================================================================

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    for size in [10usize, 100, 1000] {
        let phones: Vec<&str> = TEST_PHONES.iter().cycle().take(size).copied().collect();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("validate_batch", size),
            &phones,
            |b, phones| {
                b.iter(|| black_box(validate_phone_numbers_batch(black_box(phones.as_slice()))))
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_validation,
    bench_normalization,
    bench_extraction,
    bench_formatting,
    bench_type_detection,
    bench_batch_processing,
    bench_text_extraction,
    bench_text_country_hint,
    bench_redaction,
    bench_comparison,
    bench_grouping,
    bench_phone_number_struct,
    bench_scaling,
);
criterion_main!(benches);
