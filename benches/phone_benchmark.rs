use phonelib::{is_valid_phone_number, normalize_phone_number, extract_country};
use std::time::Instant;

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

fn main() {
    let iterations = 10000;
    
    println!("Running performance benchmark with {} iterations...", iterations);
    
    // Benchmark is_valid_phone_number
    let start = Instant::now();
    for _ in 0..iterations {
        for phone in TEST_PHONES {
            let _ = is_valid_phone_number(phone.to_string());
        }
    }
    let validation_duration = start.elapsed();
    
    // Benchmark normalize_phone_number
    let start = Instant::now();
    for _ in 0..iterations {
        for phone in TEST_PHONES {
            let _ = normalize_phone_number(phone.to_string());
        }
    }
    let normalization_duration = start.elapsed();
    
    // Benchmark extract_country
    let start = Instant::now();
    for _ in 0..iterations {
        for phone in TEST_PHONES {
            let _ = extract_country(phone.to_string());
        }
    }
    let extraction_duration = start.elapsed();
    
    println!("Validation: {:?} ({:.2} ops/sec)", 
             validation_duration, 
             (iterations * TEST_PHONES.len()) as f64 / validation_duration.as_secs_f64());
    println!("Normalization: {:?} ({:.2} ops/sec)", 
             normalization_duration,
             (iterations * TEST_PHONES.len()) as f64 / normalization_duration.as_secs_f64());
    println!("Extraction: {:?} ({:.2} ops/sec)", 
             extraction_duration,
             (iterations * TEST_PHONES.len()) as f64 / extraction_duration.as_secs_f64());
}