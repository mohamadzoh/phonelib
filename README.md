# Phonelib

Phonelib is a Rust library for handling phone numbers. It provides functions for validation, cleaning, and extracting country information from phone numbers.

## Usage

Add this to your `Cargo.toml`:

```
[dependencies]
phonelib = "0.1.0"
```

## Struct

```
pub struct Country {
pub name: &'static str,
pub code: &'static str,
pub phone_lengths: &'static [u8],
pub prefix: u32,
}

```

## USAGE

```
extern crate phonelib
```

## EXAMPLE

- validating if phone number is valid

```
let  phone_number  =  "+96179123123".to_string();

if  phonelib::is_valid_phone_number(&phone_number) {
	println!("{} is a valid phone number.", phone_number);
} else {
	println!("{} is not a valid phone number.", phone_number);
}
```

- extracting country code and information about phone number

```
let  phone_number  =  "+11231231232".to_string();
match  phonelib::extract_country(&phone_number) {
Some(country) => {
println!("Country code for {} is: {}", phone_number, country.code);
}
None => println!("Unable to extract country information for {}", phone_number),
}
```

- get santinized phone number

```
let  phone_number  =  "+096179123123".to_string();
match  phonelib::normalize_phone_number(&phone_number) {
Some(cleaned) => println!("Cleaned phone number: {}", cleaned),
None => println!("Invalid characters in the phone number."),
}
```

- santinize phone number in place (while affect the value of the passed string)

```
// Example 3: Cleaning a phone number in place

let  mut  phone_number  =  "+0012345678912".to_string();
match  phonelib::normalize_phone_number_in_place(&mut  phone_number) {
Some(_) => println!("Cleaned phone number in place: {}", phone_number),
None => println!("Invalid characters in the phone number."),
}
```

## Rusty Rails Project

Rusty Rails is a larger project aiming to bridge the gap between Rust and Ruby/Ruby on Rails. We are actively working on recreating ruby library into rust that seamlessly make working in rust more easy and fun for new developers.

### Contributing

Contributions to the Phonelib library are welcome! Feel free to open issues, submit pull requests, or provide feedback to help improve this library.
