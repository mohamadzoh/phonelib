use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use phonelib::{is_valid_phone_number, normalize_phone_number, extract_country};

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

fn bench_validation(c: &mut Criterion) {
    c.bench_function("is_valid_phone_number", |b| {
        b.iter(|| {
            for phone in TEST_PHONES {
                black_box(is_valid_phone_number(black_box(phone.to_string())));
            }
        })
    });
}

fn bench_normalization(c: &mut Criterion) {
    c.bench_function("normalize_phone_number", |b| {
        b.iter(|| {
            for phone in TEST_PHONES {
                black_box(normalize_phone_number(black_box(phone.to_string())));
            }
        })
    });
}

fn bench_extraction(c: &mut Criterion) {
    c.bench_function("extract_country", |b| {
        b.iter(|| {
            for phone in TEST_PHONES {
                black_box(extract_country(black_box(phone.to_string())));
            }
        })
    });
}

criterion_group!(benches, bench_validation, bench_normalization, bench_extraction);
criterion_main!(benches);