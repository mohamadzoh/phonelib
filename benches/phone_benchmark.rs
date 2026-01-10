use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::hint::black_box;
use phonelib::{
    is_valid_phone_number, normalize_phone_number, extract_country,
    format_phone_number, detect_phone_number_type, are_phone_numbers_equal,
    validate_phone_numbers_batch, normalize_phone_numbers_batch,
    extract_phone_numbers_from_text, PhoneFormat, PhoneNumber,
};

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

// ============================================================================
// Core Function Benchmarks
// ============================================================================

fn bench_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");
    
    // Single number validation
    group.bench_function("single_valid", |b| {
        b.iter(|| {
            black_box(is_valid_phone_number(black_box("+12025550173")))
        })
    });
    
    group.bench_function("single_invalid", |b| {
        b.iter(|| {
            black_box(is_valid_phone_number(black_box("invalid_number")))
        })
    });
    
    // Multiple numbers
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
        b.iter(|| {
            black_box(normalize_phone_number(black_box("+12025550173")))
        })
    });
    
    group.bench_function("dirty_number", |b| {
        b.iter(|| {
            black_box(normalize_phone_number(black_box("+1 (202) 555-0173")))
        })
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
        b.iter(|| {
            black_box(extract_country(black_box("+12025550173")))
        })
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
    
    group.bench_function("e164", |b| {
        b.iter(|| {
            black_box(format_phone_number(black_box("+12025550173"), PhoneFormat::E164))
        })
    });
    
    group.bench_function("international", |b| {
        b.iter(|| {
            black_box(format_phone_number(black_box("+12025550173"), PhoneFormat::International))
        })
    });
    
    group.bench_function("national", |b| {
        b.iter(|| {
            black_box(format_phone_number(black_box("+12025550173"), PhoneFormat::National))
        })
    });
    
    group.bench_function("rfc3966", |b| {
        b.iter(|| {
            black_box(format_phone_number(black_box("+12025550173"), PhoneFormat::RFC3966))
        })
    });
    
    group.finish();
}

// ============================================================================
// Type Detection Benchmarks
// ============================================================================

fn bench_type_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_detection");
    
    group.bench_function("single", |b| {
        b.iter(|| {
            black_box(detect_phone_number_type(black_box("+12025550173")))
        })
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
    
    // Batch validation
    group.throughput(Throughput::Elements(TEST_PHONES.len() as u64));
    group.bench_function("validate_batch_10", |b| {
        b.iter(|| {
            black_box(validate_phone_numbers_batch(black_box(TEST_PHONES)))
        })
    });
    
    // Batch normalization
    group.bench_function("normalize_batch_10", |b| {
        b.iter(|| {
            black_box(normalize_phone_numbers_batch(black_box(TEST_PHONES)))
        })
    });
    
    // Compare batch vs sequential
    group.bench_function("validate_sequential_10", |b| {
        b.iter(|| {
            let results: Vec<bool> = TEST_PHONES.iter()
                .map(|p| is_valid_phone_number(*p))
                .collect();
            black_box(results)
        })
    });
    
    group.finish();
}

// ============================================================================
// Text Extraction Benchmarks
// ============================================================================

fn bench_text_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_extraction");
    
    group.throughput(Throughput::Bytes(TEST_TEXT.len() as u64));
    group.bench_function("extract_from_text", |b| {
        b.iter(|| {
            black_box(extract_phone_numbers_from_text(black_box(TEST_TEXT)))
        })
    });
    
    // Longer text
    let long_text = TEST_TEXT.repeat(10);
    group.throughput(Throughput::Bytes(long_text.len() as u64));
    group.bench_function("extract_from_long_text", |b| {
        b.iter(|| {
            black_box(extract_phone_numbers_from_text(black_box(&long_text)))
        })
    });
    
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
                black_box("12025550173")
            ))
        })
    });
    
    group.bench_function("are_equal_different", |b| {
        b.iter(|| {
            black_box(are_phone_numbers_equal(
                black_box("+12025550173"),
                black_box("+442079460958")
            ))
        })
    });
    
    group.finish();
}

// ============================================================================
// PhoneNumber Struct Benchmarks
// ============================================================================

fn bench_phone_number_struct(c: &mut Criterion) {
    let mut group = c.benchmark_group("phone_number_struct");
    
    group.bench_function("parse", |b| {
        b.iter(|| {
            black_box(PhoneNumber::parse(black_box("+12025550173")))
        })
    });
    
    let phone = PhoneNumber::parse("+12025550173").unwrap();
    
    group.bench_function("e164", |b| {
        b.iter(|| {
            black_box(phone.e164())
        })
    });
    
    group.bench_function("national_number", |b| {
        b.iter(|| {
            black_box(phone.national_number())
        })
    });
    
    group.bench_function("format_international", |b| {
        b.iter(|| {
            black_box(phone.format(PhoneFormat::International))
        })
    });
    
    group.finish();
}

// ============================================================================
// Scaling Benchmarks
// ============================================================================

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");
    
    for size in [10, 100, 1000].iter() {
        let phones: Vec<&str> = TEST_PHONES.iter()
            .cycle()
            .take(*size)
            .copied()
            .collect();
        
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("validate_batch", size),
            &phones,
            |b, phones| {
                b.iter(|| {
                    black_box(validate_phone_numbers_batch(black_box(phones.as_slice())))
                })
            }
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
    bench_comparison,
    bench_phone_number_struct,
    bench_scaling,
);
criterion_main!(benches);