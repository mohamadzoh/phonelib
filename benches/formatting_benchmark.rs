use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use phonelib::{PhoneFormat, PhoneNumber};
use std::hint::black_box;

type TestEntry = (&'static str, &'static str);

fn setup_numbers() -> Vec<TestEntry> {
    vec![
        ("+12025550173", "US"),
        ("+14155552671", "US"),
        ("+18005551234", "US"),
        ("+12125551234", "US"),
        ("+13055551234", "US"),
        ("+14165550198", "CA"),
        ("+15145551234", "CA"),
        ("+16045551234", "CA"),
        ("+442087654321", "GB"),
        ("+447911123456", "GB"),
        ("+441234567890", "GB"),
        ("+493012345678", "DE"),
        ("+4915112345678", "DE"),
        ("+494012345678", "DE"),
        ("+33123456789", "FR"),
        ("+33612345678", "FR"),
        ("+34612345678", "ES"),
        ("+34912345678", "ES"),
        ("+41446681800", "CH"),
        ("+31201234567", "NL"),
        ("+32234567890", "BE"),
        ("+4721234567", "NO"),
        ("+46812345678", "SE"),
        ("+4512345678", "DK"),
        ("+48221234567", "PL"),
        ("+351212345678", "PT"),
        ("+353123456789", "IE"),
        ("+302101234567", "GR"),
        ("+36123456789", "HU"),
        ("+402112345678", "RO"),
        ("+905321234567", "TR"),
        ("+390212345678", "IT"),
        ("+61412345678", "AU"),
        ("+61291234567", "AU"),
        ("+819012345678", "JP"),
        ("+8613800138000", "CN"),
        ("+862112345678", "CN"),
        ("+919876543210", "IN"),
        ("+82221234567", "KR"),
        ("+6591234567", "SG"),
        ("+66812345678", "TH"),
        ("+85212345678", "HK"),
        ("+639171234567", "PH"),
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
        ("+77272501234", "KZ"),
        ("+6491234567", "NZ"),
        ("+64211234567", "NZ"),
    ]
}

fn convert_to_phonelib_numbers(numbers: &[TestEntry]) -> Vec<PhoneNumber> {
    numbers
        .iter()
        .filter_map(|(number, region)| PhoneNumber::parse_with_country(number, region))
        .collect()
}

fn formatting_benchmark(c: &mut Criterion) {
    let numbers_data = setup_numbers();
    let phonelib_numbers = convert_to_phonelib_numbers(&numbers_data);

    eprintln!(
        "\n=== Phonelib Formatting Compatibility ===\nTotal test numbers: {}\nparsed numbers: {}\n",
        numbers_data.len(),
        phonelib_numbers.len(),
    );

    let mut group = c.benchmark_group("Phonelib Formatting");

    let mut bench_format = |label: &str, format: PhoneFormat| {
        group.bench_function(label, |b| {
            let mut iter = phonelib_numbers.iter().cycle();
            b.iter_batched(
                || iter.next().unwrap(),
                |number| number.format(black_box(format)),
                BatchSize::SmallInput,
            )
        });
    };

    bench_format("format(E164)", PhoneFormat::E164);
    bench_format("format(International)", PhoneFormat::International);
    bench_format("format(National)", PhoneFormat::National);
    bench_format("format(RFC3966)", PhoneFormat::RFC3966);

    group.finish();
}

criterion_group!(benches, formatting_benchmark);
criterion_main!(benches);