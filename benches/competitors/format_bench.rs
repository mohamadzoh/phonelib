use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use rlibphonenumber::{PHONE_NUMBER_UTIL, PhoneNumberFormat};

use phonenumber::{
    self as rlp, Mode,
    country::Id::{
        self, AE, AR, AU, BE, BR, CA, CH, CL, CN, CO, DE, DK, EG, ES, FR, GB, GR, HK, HU, IE,
        IL, IN, IT, JP, KE, KR, KZ, MX, NG, NL, NO, NZ, PH, PL, PT, RO, RU, SA, SE, SG, TH,
        TR, US, ZA,
    },
};

use phonelib::{PhoneFormat, PhoneNumber as PhonelibNumber};

type TestEntry = (&'static str, &'static str, Id);

/// Comprehensive set of E.164 numbers for formatting benchmarks.
/// All numbers are pre-validated for correct digit counts per country.
fn setup_numbers() -> Vec<TestEntry> {
    vec![
        // North America (NANP — prefix 1, 10 national digits)
        ("+12025550173", "US", US),
        ("+14155552671", "US", US),
        ("+18005551234", "US", US),
        ("+12125551234", "US", US),
        ("+13055551234", "US", US),
        ("+14165550198", "CA", CA),
        ("+15145551234", "CA", CA),
        ("+16045551234", "CA", CA),

        // United Kingdom (prefix 44, 10 national digits)
        ("+442087654321", "GB", GB),
        ("+447911123456", "GB", GB),
        ("+441234567890", "GB", GB),

        // Western Europe
        ("+493012345678", "DE", DE),       // 10 digits
        ("+4915112345678", "DE", DE),      // 11 digits (mobile)
        ("+494012345678", "DE", DE),       // 10 digits
        ("+33123456789", "FR", FR),        // 9 digits
        ("+33612345678", "FR", FR),        // 9 digits (mobile)
        ("+34612345678", "ES", ES),        // 9 digits
        ("+34912345678", "ES", ES),        // 9 digits
        ("+41446681800", "CH", CH),        // 9 digits
        ("+31201234567", "NL", NL),        // 9 digits
        ("+32234567890", "BE", BE),        // 9 digits

        // Northern Europe
        ("+4721234567", "NO", NO),         // 8 digits
        ("+46812345678", "SE", SE),        // 9 digits
        ("+4512345678", "DK", DK),         // 8 digits

        // Eastern / Southern Europe
        ("+48221234567", "PL", PL),        // 9 digits
        ("+351212345678", "PT", PT),       // 9 digits
        ("+353123456789", "IE", IE),       // 9 digits
        ("+302101234567", "GR", GR),       // 10 digits
        ("+36123456789", "HU", HU),        // 9 digits
        ("+402112345678", "RO", RO),       // 10 digits
        ("+905321234567", "TR", TR),       // 10 digits
        ("+390212345678", "IT", IT),       // 10 digits

        // Asia-Pacific
        ("+61412345678", "AU", AU),        // 9 digits
        ("+61291234567", "AU", AU),        // 9 digits
        ("+819012345678", "JP", JP),       // 10 digits
        ("+8613800138000", "CN", CN),      // 11 digits
        ("+862112345678", "CN", CN),       // 10 digits
        ("+919876543210", "IN", IN),       // 10 digits
        ("+82221234567", "KR", KR),        // 9 digits
        ("+6591234567", "SG", SG),         // 8 digits
        ("+66812345678", "TH", TH),        // 9 digits
        ("+85212345678", "HK", HK),        // 8 digits
        ("+639171234567", "PH", PH),       // 10 digits

        // Americas (non-NANP)
        ("+5511987654321", "BR", BR),      // 11 digits — breaks rlp
        ("+551123456789", "BR", BR),       // 10 digits
        ("+525512345678", "MX", MX),       // 10 digits
        ("+541112345678", "AR", AR),       // 10 digits
        ("+56912345678", "CL", CL),        // 9 digits
        ("+571234567890", "CO", CO),       // 10 digits

        // Middle East
        ("+971501234567", "AE", AE),       // 9 digits
        ("+972541234567", "IL", IL),       // 9 digits
        ("+966501234567", "SA", SA),       // 9 digits

        // Africa
        ("+27211234567", "ZA", ZA),        // 9 digits
        ("+2348012345678", "NG", NG),      // 10 digits
        ("+254712345678", "KE", KE),       // 9 digits
        ("+201012345678", "EG", EG),       // 10 digits

        // Russia / CIS (prefix 7, 10 national digits)
        ("+79161234567", "RU", RU),
        ("+79031234567", "RU", RU),
        ("+77272501234", "KZ", KZ),

        // Oceania
        ("+6491234567", "NZ", NZ),         // 8 digits
        ("+64211234567", "NZ", NZ),        // 9 digits
    ]
}

fn convert_to_rlp_numbers(numbers: &[TestEntry]) -> Vec<rlp::PhoneNumber> {
    numbers
        .iter()
        .filter_map(|s| rlp::parse(Some(s.2), s.0).ok())
        .collect()
}

fn convert_to_rlibphonenumber_numbers(numbers: &[TestEntry]) -> Vec<rlibphonenumber::PhoneNumber> {
    numbers
        .iter()
        .filter_map(|s| {
            PHONE_NUMBER_UTIL
                .parse_with_default_region(s.0, s.1)
                .ok()
        })
        .collect()
}

fn convert_to_phonelib_numbers(numbers: &[TestEntry]) -> Vec<PhonelibNumber> {
    numbers
        .iter()
        .filter_map(|s| PhonelibNumber::parse_with_country(s.0, s.1))
        .collect()
}

fn formatting_benchmark(c: &mut Criterion) {
    let numbers_data = setup_numbers();

    let rlib_numbers = convert_to_rlibphonenumber_numbers(&numbers_data);
    let rlp_numbers = convert_to_rlp_numbers(&numbers_data);
    let phonelib_numbers = convert_to_phonelib_numbers(&numbers_data);

    eprintln!(
        "\n=== Formatting Compatibility ===\n\
         Total test numbers:    {}\n\
         rlibphonenumber:       {} parsed\n\
         rust-phonenumber:      {} parsed\n\
         phonelib:              {} parsed\n",
        numbers_data.len(),
        rlib_numbers.len(),
        rlp_numbers.len(),
        phonelib_numbers.len(),
    );

    let mut group = c.benchmark_group("Formatting Comparison");

    // Validate E164 output matches between rlp and rlibphonenumber
    // (only for numbers both can parse)
    for (number_a, number_b) in rlp_numbers.iter().zip(rlib_numbers.iter()) {
        assert_eq!(
            rlp::format(number_a).mode(Mode::E164).to_string(),
            PHONE_NUMBER_UTIL.format(number_b, PhoneNumberFormat::E164)
        );
    }

    let mut test = |format_a: PhoneNumberFormat, format_b: Mode, format_c: PhoneFormat| {
        // 1. rlibphonenumber
        group.bench_function(format!("rlibphonenumber: format({:?})", format_a), |b| {
            let mut iter = rlib_numbers.iter().cycle();
            b.iter_batched(
                || iter.next().unwrap(),
                |number| PHONE_NUMBER_UTIL.format(black_box(number), black_box(format_a)),
                BatchSize::SmallInput,
            )
        });

        // 2. rust-phonenumber (rlp)
        group.bench_function(format!("rust-phonenumber: format({:?})", format_b), |b| {
            let mut iter = rlp_numbers.iter().cycle();
            b.iter_batched(
                || iter.next().unwrap(),
                |number| {
                    rlp::format(black_box(number))
                        .mode(black_box(format_b))
                        .to_string()
                },
                BatchSize::SmallInput,
            )
        });

        // 3. phonelib
        group.bench_function(format!("phonelib: format({:?})", format_a), |b| {
            let mut iter = phonelib_numbers.iter().cycle();
            b.iter_batched(
                || iter.next().unwrap(),
                |number| number.format(black_box(format_c)),
                BatchSize::SmallInput,
            )
        });
    };

    test(PhoneNumberFormat::E164, Mode::E164, PhoneFormat::E164);
    test(
        PhoneNumberFormat::International,
        Mode::International,
        PhoneFormat::International,
    );
    test(
        PhoneNumberFormat::National,
        Mode::National,
        PhoneFormat::National,
    );
    test(
        PhoneNumberFormat::RFC3966,
        Mode::Rfc3966,
        PhoneFormat::RFC3966,
    );

    group.finish();
}

criterion_group!(benches, formatting_benchmark);
criterion_main!(benches);
