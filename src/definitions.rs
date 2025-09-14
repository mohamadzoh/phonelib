#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Country {
    pub name: &'static str,
    pub code: &'static str,
    pub phone_lengths: &'static [u8],
    pub prefix: u32,
}

/// Phone number types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhoneNumberType {
    /// Mobile/cellular phone
    Mobile,
    /// Fixed line (landline)
    FixedLine,
    /// Toll-free number
    TollFree,
    /// Premium rate number
    PremiumRate,
    /// Shared cost number
    SharedCost,
    /// Voice over IP
    Voip,
    /// Personal number
    PersonalNumber,
    /// Pager
    Pager,
    /// Universal Access Number
    Uan,
    /// Emergency services
    Emergency,
    /// Voicemail
    Voicemail,
    /// Unknown type
    Unknown,
}