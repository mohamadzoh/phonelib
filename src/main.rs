
mod constants;
mod definitions;
pub fn main() {
    let phone_number = "+1 (234) 567-890";
    println!("{}",phonelib::phone_number_cleaner(&mut "+96103146594".to_string()).unwrap());
    // println!("{}",phone_number_utils::extract_country("+96179146594".to_string()).unwrap().code)
    // print!("{}", phone_number_utils::is_valid_phone_number(phone_number).to_string());
}

