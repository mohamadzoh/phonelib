use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use phonelib::PhoneNumber;
use std::hint::black_box;

type TestEntry = (&'static str, &'static str);

fn setup_all_numbers() -> Vec<TestEntry> {
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
        ("+77272501234", "KZ"),
        ("+6491234567", "NZ"),
        ("+64211234567", "NZ"),
        ("+44 20 8765 4321", "GB"),
        ("(650) 253-0000", "US"),
        ("(202) 555-0173", "US"),
        ("+1 (646) 222-3333", "US"),
        ("416-555-0198", "CA"),
        (" + 49 (0) 30 123456-78 ", "DE"),
        ("++41-44-668-18-00", "CH"),
        ("+61 4 1234 5678", "AU"),
        ("+91 98765 43210", "IN"),
        ("+86 138 0013 8000", "CN"),
        ("+7 916 123 4567", "RU"),
        ("+7 727 250 1234", "KZ"),
        ("+65 9123 4567", "SG"),
        ("+52 55 1234 5678", "MX"),
        ("+972 54 123 4567", "IL"),
        ("+27 21 123 4567", "ZA"),
        ("020 8765 4321", "GB"),
        ("02 12345678", "IT"),
        ("(03) 1234 5678", "JP"),
        ("02-2123-4567", "KR"),
        ("0800 83 83 83", "NZ"),
        ("+1 (646) 222-3333 ext. 987", "US"),
        ("0011 54 9 11 8765 4321 ext. 1234", "AU"),
        ("011 15-1234-5678", "AR"),
        ("1-800-FLOWERS", "US"),
        ("1300 FLIGHT", "AU"),
    ]
}

fn parsing_benchmark(c: &mut Criterion) {
    let all_numbers = setup_all_numbers();
    let parse_numbers: Vec<&TestEntry> = all_numbers
        .iter()
        .filter(|(number, _)| PhoneNumber::parse(number).is_some())
        .collect();
    let parse_with_country_numbers: Vec<&TestEntry> = all_numbers
        .iter()
        .filter(|(number, region)| PhoneNumber::parse_with_country(number, region).is_some())
        .collect();

    eprintln!(
        "\n=== Phonelib Parsing Compatibility ===\nTotal test numbers: {}\nparse(): {}\nparse_with_country(): {}\n",
        all_numbers.len(),
        parse_numbers.len(),
        parse_with_country_numbers.len(),
    );

    let mut group = c.benchmark_group("Phonelib Parsing");

    group.bench_function("parse()", |b| {
        let mut iter = parse_numbers.iter().cycle();
        b.iter_batched(
            || iter.next().unwrap(),
            |(number_str, _)| {
                let _ = PhoneNumber::parse(black_box(number_str)).unwrap();
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("parse_with_country()", |b| {
        let mut iter = parse_with_country_numbers.iter().cycle();
        b.iter_batched(
            || iter.next().unwrap(),
            |(number_str, region)| {
                let _ = PhoneNumber::parse_with_country(
                    black_box(number_str),
                    black_box(region),
                )
                .unwrap();
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, parsing_benchmark);
criterion_main!(benches);