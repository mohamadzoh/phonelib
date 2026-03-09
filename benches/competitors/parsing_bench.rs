use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use phonenumber::{self as rlp, country::Id};
use rlibphonenumber::PHONE_NUMBER_UTIL;

type TestEntry = (&'static str, &'static str, Id);

/// Comprehensive phone number set covering 40+ countries across 6 continents.
/// Each number is validated against phonelib's COUNTRIES data for correct digit counts.
fn setup_all_numbers() -> Vec<TestEntry> {
    vec![
        // =================================================================
        // E.164 Format — Clean international numbers
        // =================================================================

        // North America (NANP — prefix 1, 10 national digits)
        ("+12025550173", "US", Id::US),       // Washington DC
        ("+14155552671", "US", Id::US),        // San Francisco
        ("+18005551234", "US", Id::US),        // Toll-free
        ("+12125551234", "US", Id::US),        // New York
        ("+13055551234", "US", Id::US),        // Miami
        ("+14165550198", "CA", Id::CA),        // Toronto
        ("+15145551234", "CA", Id::CA),        // Montreal
        ("+16045551234", "CA", Id::CA),        // Vancouver

        // United Kingdom (prefix 44, 10 national digits)
        ("+442087654321", "GB", Id::GB),       // London landline
        ("+447911123456", "GB", Id::GB),       // UK mobile
        ("+441234567890", "GB", Id::GB),       // Regional

        // Western Europe
        ("+493012345678", "DE", Id::DE),       // Germany Berlin (10 digits)
        ("+4915112345678", "DE", Id::DE),      // Germany mobile (11 digits)
        ("+494012345678", "DE", Id::DE),       // Germany Hamburg (10 digits)
        ("+33123456789", "FR", Id::FR),        // France landline (9 digits)
        ("+33612345678", "FR", Id::FR),        // France mobile (9 digits)
        ("+34612345678", "ES", Id::ES),        // Spain (9 digits)
        ("+34912345678", "ES", Id::ES),        // Spain landline (9 digits)
        ("+41446681800", "CH", Id::CH),        // Switzerland (9 digits)
        ("+31201234567", "NL", Id::NL),        // Netherlands (9 digits)
        ("+32234567890", "BE", Id::BE),        // Belgium (9 digits)

        // Northern Europe
        ("+4721234567", "NO", Id::NO),         // Norway (8 digits)
        ("+46812345678", "SE", Id::SE),        // Sweden (9 digits)
        ("+4512345678", "DK", Id::DK),         // Denmark (8 digits)

        // Eastern / Southern Europe
        ("+48221234567", "PL", Id::PL),        // Poland (9 digits)
        ("+351212345678", "PT", Id::PT),       // Portugal (9 digits)
        ("+353123456789", "IE", Id::IE),       // Ireland (9 digits)
        ("+302101234567", "GR", Id::GR),       // Greece (10 digits)
        ("+36123456789", "HU", Id::HU),        // Hungary (9 digits)
        ("+402112345678", "RO", Id::RO),       // Romania (10 digits)
        ("+905321234567", "TR", Id::TR),       // Turkey (10 digits)
        ("+390212345678", "IT", Id::IT),       // Italy (10 digits)

        // Asia-Pacific
        ("+61412345678", "AU", Id::AU),        // Australia mobile (9 digits)
        ("+61291234567", "AU", Id::AU),        // Australia landline (9 digits)
        ("+819012345678", "JP", Id::JP),       // Japan mobile (10 digits)
        ("+81312345678", "JP", Id::JP),        // Japan Tokyo (10 digits but verify)
        ("+8613800138000", "CN", Id::CN),      // China mobile (11 digits)
        ("+862112345678", "CN", Id::CN),       // China Shanghai (10 digits)
        ("+919876543210", "IN", Id::IN),       // India (10 digits)
        ("+916123456789", "IN", Id::IN),       // India mobile (10 digits)
        ("+82221234567", "KR", Id::KR),        // South Korea (9 digits)
        ("+6591234567", "SG", Id::SG),         // Singapore (8 digits)
        ("+66812345678", "TH", Id::TH),        // Thailand (9 digits)
        ("+85212345678", "HK", Id::HK),        // Hong Kong (8 digits)
        ("+639171234567", "PH", Id::PH),       // Philippines (10 digits)

        // Americas (non-NANP)
        ("+5511987654321", "BR", Id::BR),      // Brazil mobile (11 digits) — breaks rlp
        ("+551123456789", "BR", Id::BR),       // Brazil landline (10 digits)
        ("+525512345678", "MX", Id::MX),       // Mexico (10 digits)
        ("+541112345678", "AR", Id::AR),       // Argentina (10 digits)
        ("+56912345678", "CL", Id::CL),        // Chile (9 digits)
        ("+571234567890", "CO", Id::CO),       // Colombia (10 digits)

        // Middle East
        ("+971501234567", "AE", Id::AE),       // UAE (9 digits)
        ("+972541234567", "IL", Id::IL),       // Israel (9 digits)
        ("+966501234567", "SA", Id::SA),       // Saudi Arabia (9 digits)

        // Africa
        ("+27211234567", "ZA", Id::ZA),        // South Africa (9 digits)
        ("+2348012345678", "NG", Id::NG),      // Nigeria (10 digits)
        ("+254712345678", "KE", Id::KE),       // Kenya (9 digits)
        ("+201012345678", "EG", Id::EG),       // Egypt (10 digits)

        // Russia / CIS (prefix 7, 10 national digits)
        ("+79161234567", "RU", Id::RU),        // Russia Moscow
        ("+79031234567", "RU", Id::RU),        // Russia mobile
        ("+77272501234", "KZ", Id::KZ),        // Kazakhstan

        // Oceania
        ("+6491234567", "NZ", Id::NZ),         // New Zealand (8 digits)
        ("+64211234567", "NZ", Id::NZ),        // New Zealand mobile (9 digits)

        // =================================================================
        // Formatted / National — Various real-world formatting styles
        // =================================================================
        ("+44 20 8765 4321", "GB", Id::GB),
        ("(650) 253-0000", "US", Id::US),
        ("(202) 555-0173", "US", Id::US),
        ("+1 (646) 222-3333", "US", Id::US),
        ("416-555-0198", "CA", Id::CA),
        (" + 49 (0) 30 123456-78 ", "DE", Id::DE),
        ("++41-44-668-18-00", "CH", Id::CH),
        ("+61 4 1234 5678", "AU", Id::AU),
        ("+91 98765 43210", "IN", Id::IN),
        ("+86 138 0013 8000", "CN", Id::CN),
        ("+7 916 123 4567", "RU", Id::RU),
        ("+7 727 250 1234", "KZ", Id::KZ),
        ("+65 9123 4567", "SG", Id::SG),
        ("+52 55 1234 5678", "MX", Id::MX),
        ("+972 54 123 4567", "IL", Id::IL),
        ("+27 21 123 4567", "ZA", Id::ZA),
        ("020 8765 4321", "GB", Id::GB),
        ("02 12345678", "IT", Id::IT),
        ("(03) 1234 5678", "JP", Id::JP),
        ("02-2123-4567", "KR", Id::KR),
        ("0800 83 83 83", "NZ", Id::NZ),

        // =================================================================
        // Special formats
        // =================================================================
        ("+1 (646) 222-3333 ext. 987", "US", Id::US),
        ("0011 54 9 11 8765 4321 ext. 1234", "AU", Id::AU),
        ("011 15-1234-5678", "AR", Id::AR),
        ("1-800-FLOWERS", "US", Id::US),
        ("1300 FLIGHT", "AU", Id::AU),
    ]
}

pub fn parsing_benchmark(c: &mut Criterion) {
    let all_numbers = setup_all_numbers();
    let mut group = c.benchmark_group("Parsing Comparison");

    // Pre-filter numbers for each library at setup time
    let rlib_numbers: Vec<&TestEntry> = all_numbers
        .iter()
        .filter(|(s, r, _)| {
            PHONE_NUMBER_UTIL
                .parse_with_default_region(s, r)
                .is_ok()
        })
        .collect();

    let rlp_numbers: Vec<&TestEntry> = all_numbers
        .iter()
        .filter(|(s, _, id)| rlp::parse(Some(*id), s).is_ok())
        .collect();

    let phonelib_numbers: Vec<&TestEntry> = all_numbers
        .iter()
        .filter(|(s, r, _)| phonelib::PhoneNumber::parse_with_country(s, r).is_some())
        .collect();

    eprintln!(
        "\n=== Parsing Compatibility ===\n\
         Total test numbers:    {}\n\
         rlibphonenumber:       {}/{} parseable\n\
         rust-phonenumber:      {}/{} parseable\n\
         phonelib:              {}/{} parseable\n",
        all_numbers.len(),
        rlib_numbers.len(),
        all_numbers.len(),
        rlp_numbers.len(),
        all_numbers.len(),
        phonelib_numbers.len(),
        all_numbers.len(),
    );

    // 1. rlibphonenumber
    group.bench_function("rlibphonenumber: parse()", |b| {
        let mut iter = rlib_numbers.iter().cycle();
        b.iter_batched(
            || iter.next().unwrap(),
            |(number_str, region, _)| {
                let _ = PHONE_NUMBER_UTIL
                    .parse_with_default_region(black_box(number_str), black_box(region))
                    .unwrap();
            },
            BatchSize::SmallInput,
        )
    });

    // 2. rust-phonenumber
    group.bench_function("rust-phonenumber: parse()", |b| {
        let mut iter = rlp_numbers.iter().cycle();
        b.iter_batched(
            || iter.next().unwrap(),
            |(number_str, _, region_id)| {
                let _ = rlp::parse(black_box(Some(*region_id)), black_box(number_str))
                    .expect(number_str);
            },
            BatchSize::SmallInput,
        )
    });

    // 3. phonelib
    group.bench_function("phonelib: parse()", |b| {
        let mut iter = phonelib_numbers.iter().cycle();
        b.iter_batched(
            || iter.next().unwrap(),
            |(number_str, region, _)| {
                let _ = phonelib::PhoneNumber::parse_with_country(
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
