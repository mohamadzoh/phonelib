#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Country {
    pub name: &'static str,
    pub code: &'static str,
    pub phone_lengths: &'static [u8],
    pub prefix: u32,
}