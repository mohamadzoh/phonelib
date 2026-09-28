//! Core public types: [`Country`], [`PhoneNumberType`] and [`ParseError`].

use std::fmt;

/// A country or territory in the international numbering plan.
///
/// Instances are static entries in the crate's country table; obtain one with
/// [`crate::extract_country`], [`crate::country_by_code`] or
/// [`crate::countries`].
///
/// This type is `#[non_exhaustive]`: fields may be added in future releases,
/// so it cannot be constructed or exhaustively destructured outside the crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Country {
    /// English name of the country or territory.
    pub name: &'static str,
    /// ISO 3166-1 alpha-2 code, or `"001"` — the UN M.49 code for the world,
    /// as libphonenumber uses — for the ITU's non-geographic calling codes
    /// (`+800` freephone, `+870` Inmarsat, …).
    ///
    /// Several entries may share a code when a country operates more than one
    /// calling code (for example the Dominican Republic's 1809/1829/1849
    /// overlays), and every non-geographic entry shares `"001"`.
    pub code: &'static str,
    /// Every length the national significant number can have, in digits,
    /// across all kinds of number — fixed line, mobile, freephone and so on.
    ///
    /// This excludes the country calling code and any national trunk prefix.
    pub phone_lengths: &'static [u8],
    /// E.164 country calling code.
    ///
    /// For NANP territories this is the four-digit `1` + area code form, e.g.
    /// `1268` for Antigua and Barbuda.
    pub prefix: u32,
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (+{})", self.name, self.prefix)
    }
}

/// The service class of a phone number.
///
/// Classification is metadata-driven and only as good as the ranges the crate
/// knows about. A number in a block no range table claims is
/// [`PhoneNumberType::Unknown`] rather than a guess; a number's type never
/// affects whether it is valid.
///
/// This enum is `#[non_exhaustive]`: new variants may be added in future
/// releases, so `match` on it must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PhoneNumberType {
    /// Mobile/cellular phone.
    Mobile,
    /// Fixed line (landline).
    FixedLine,
    /// A range that carries both fixed lines and mobiles, so the two cannot be
    /// told apart from the number alone. This is the norm in the North
    /// American Numbering Plan, where number portability has decoupled the
    /// prefix from the technology.
    FixedLineOrMobile,
    /// Toll-free number, paid for by the called party.
    TollFree,
    /// Premium rate number, charged above the standard rate.
    PremiumRate,
    /// Shared cost number, split between caller and called party.
    SharedCost,
    /// Voice over IP.
    Voip,
    /// Personal number that follows the subscriber between lines.
    PersonalNumber,
    /// Pager.
    Pager,
    /// Universal Access Number.
    Uan,
    /// The number is valid but its class is not known.
    Unknown,
}

impl fmt::Display for PhoneNumberType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PhoneNumberType::Mobile => "mobile",
            PhoneNumberType::FixedLine => "fixed line",
            PhoneNumberType::FixedLineOrMobile => "fixed line or mobile",
            PhoneNumberType::TollFree => "toll-free",
            PhoneNumberType::PremiumRate => "premium rate",
            PhoneNumberType::SharedCost => "shared cost",
            PhoneNumberType::Voip => "VoIP",
            PhoneNumberType::PersonalNumber => "personal number",
            PhoneNumberType::Pager => "pager",
            PhoneNumberType::Uan => "UAN",
            PhoneNumberType::Unknown => "unknown",
        };
        f.write_str(s)
    }
}

/// Why a phone number could not be parsed.
///
/// Every fallible entry point in the crate has a `try_` variant returning
/// `Result<_, ParseError>`; the `Option`-returning functions are thin wrappers
/// that discard this detail.
///
/// This enum is `#[non_exhaustive]`: new variants may be added in future
/// releases, so `match` on it must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ParseError {
    /// The input contained no digits at all.
    Empty,
    /// The input contained a character that cannot appear in a phone number.
    InvalidCharacter,
    /// The input held more than 15 significant digits, the E.164 maximum.
    TooLong,
    /// The input held fewer digits than any known numbering plan accepts.
    TooShort,
    /// The leading digits did not match any known country calling code.
    UnknownCountryCode,
    /// The input carried no `+` and no international access code, so which
    /// country it belongs to cannot be determined.
    ///
    /// The leading digits may well match some country's calling code, but
    /// without a marker there is no way to tell that reading apart from a
    /// national number — `(202) 555-0173` is a Washington DC number, not the
    /// Egyptian number `+20 2555 0173`. Supply the `+`, or use
    /// [`crate::PhoneNumber::parse_with_country`]. Numbers under calling code 1
    /// are exempt, because their area code is checked against the numbering
    /// plan and so is not a guess.
    MissingCallingCode,
    /// The country was identified but the national number's length is not
    /// valid for that country's numbering plan.
    InvalidLength,
    /// The number is in the North American Numbering Plan but its area code is
    /// unassigned or structurally invalid.
    InvalidAreaCode,
    /// The number is in the North American Numbering Plan but its central
    /// office (exchange) code is structurally invalid.
    InvalidExchange,
    /// The ISO 3166-1 alpha-2 code supplied as a hint is not in the country
    /// table.
    UnknownCountry,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ParseError::Empty => "input contains no digits",
            ParseError::InvalidCharacter => "input contains an invalid character",
            ParseError::TooLong => "more than 15 significant digits",
            ParseError::TooShort => "fewer digits than any numbering plan accepts",
            ParseError::UnknownCountryCode => "no known country calling code matches",
            ParseError::MissingCallingCode => {
                "no '+' or international access code, so the country is ambiguous"
            }
            ParseError::InvalidLength => "national number length invalid for this country",
            ParseError::InvalidAreaCode => "unassigned or invalid NANP area code",
            ParseError::InvalidExchange => "invalid NANP central office code",
            ParseError::UnknownCountry => "unknown ISO 3166-1 alpha-2 country code",
        };
        f.write_str(s)
    }
}

impl std::error::Error for ParseError {}
