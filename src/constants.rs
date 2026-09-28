use crate::definitions;

use definitions::Country;

/// Country calling-code table.
///
/// # Field semantics
///
/// * `prefix` — the E.164 country calling code. For NANP territories this is the
///   4-digit `1` + area-code form (e.g. `1268` = Antigua and Barbuda), because the
///   area code is what identifies the territory.
/// * `phone_lengths` — every length the national significant number can have,
///   i.e. the digits that follow `prefix`, across **all** number types: fixed
///   line, mobile, freephone, premium rate, shared cost, UAN, VoIP and so on.
///   Ascending, no duplicates.
///
/// # What this table is, and is not
///
/// The crate validates the *structure* of a numbering plan — calling code and
/// national number length — and deliberately does not track which prefixes an
/// operator currently has in service. Range allocations change every month;
/// a plan's lengths change once in a decade. A number in a range that opens
/// after this table was written is therefore still accepted, as long as its
/// length is one the plan uses.
///
/// The lengths are the dialable-from-abroad lengths published in Google's
/// libphonenumber metadata (`possibleLengths`, `national` only, never
/// `localOnly`), taken as the union over every number type, last reconciled
/// against release v9.0.40. A length that only a number longer than the E.164
/// maximum could use is left out, because such a number cannot be dialled
/// internationally (see invariant 1).
///
/// # Invariants
///
/// 1. **E.164 ceiling.** `digits(prefix) + max(phone_lengths) <= 15` for every
///    entry, so no entry can accept a number longer than E.164 permits.
/// 2. **Ordering.** Entries are alphabetical by ISO 3166-1 alpha-2 `code`. Array
///    order does *not* decide which territory wins a shared calling code: the
///    resolver walks a prefix bucket sorted by `PRIMARY_FOR_PREFIX`
///    (`src/lib.rs`) first, and falls back to declaration order only to break
///    ties among entries that table does not name. So adding a second territory
///    to a calling code means editing `PRIMARY_FOR_PREFIX` — moving the entry to
///    the front of this array silently does nothing. The compile-time assert in
///    `src/lib.rs` only checks that each named primary exists, not that array
///    order agrees with it. Where an entry is nevertheless kept out of
///    alphabetical order for readability, it carries a comment.
/// 3. **Real administrations only.** Every entry corresponds to a territory with an
///    actual numbering plan. Uninhabited territories (Antarctica, Bouvet Island,
///    Heard & McDonald Islands, French Southern Territories) are intentionally
///    absent — they have no numbers to validate, so listing them only produces
///    false accepts.
///
/// # Shared prefixes
///
/// Some calling codes genuinely serve several territories and cannot be split by
/// NSN length alone (only leading digits distinguish them): `1` US/CA and the
/// Caribbean, `7` RU/KZ, `39` IT/VA, `44` GB/GG/IM/JE, `47` NO/SJ, `61` AU/CC/CX,
/// `212` MA/EH, `262` RE/YT, `290` SH/TA, `358` FI/AX, `500` FK/GS, `590`
/// GP/BL/MF, `599` CW/BQ. Calling code 1 is split by the NANP area-code table
/// in `src/nanp.rs`; the others by [`TERRITORY_PREFIXES`], with the main region
/// taking every number no rule claims. Those prefixes only ever decide *which*
/// territory is reported, never whether a number is valid.
///
/// # Non-geographic entities
///
/// The ITU assigns a few calling codes to global services rather than to a
/// country — international freephone (`+800`), Inmarsat (`+870`), satellite and
/// international networks (`+881`/`+882`/`+883`) and others. They carry the
/// code `"001"`, the UN M.49 code for the world, which is also what
/// libphonenumber uses for them.
pub const COUNTRIES: [Country; 259] = [
    Country {
        name: "Andorra",
        code: "AD",
        phone_lengths: &[6, 8, 9],
        prefix: 376,
    },
    Country {
        // Ascension migrated from 4- to 5-digit subscriber numbers in 2015;
        // 4-digit numbers no longer route.
        name: "Ascension Island",
        code: "AC",
        phone_lengths: &[5, 6],
        prefix: 247,
    },
    Country {
        // 8 = landline (+971 4 123 4567), 9 = mobile (+971 50 123 4567) and the
        // 600/700/900 service ranges; 800 freephone numbers run anywhere from 5 to
        // 12 digits.
        name: "United Arab Emirates",
        code: "AE",
        phone_lengths: &[5, 6, 7, 8, 9, 10, 11, 12],
        prefix: 971,
    },
    Country {
        name: "Afghanistan",
        code: "AF",
        phone_lengths: &[9],
        prefix: 93,
    },
    Country {
        name: "Antigua and Barbuda",
        code: "AG",
        phone_lengths: &[7],
        prefix: 1268,
    },
    Country {
        name: "Anguilla",
        code: "AI",
        phone_lengths: &[7],
        prefix: 1264,
    },
    Country {
        // 8 = fixed line (04 222 2222 -> NSN 42222222) and personal numbering,
        // 9 = mobile, 7 = 800 freephone, 6 = the 808 shared-cost and 900
        // premium-rate ranges.
        name: "Albania",
        code: "AL",
        phone_lengths: &[6, 7, 8, 9],
        prefix: 355,
    },
    Country {
        // Uniformly 8 digits across every number type. The 6- and 7-digit forms
        // that used to be listed here are local-only dialling, never an NSN.
        name: "Armenia",
        code: "AM",
        phone_lengths: &[8],
        prefix: 374,
    },
    Country {
        name: "Angola",
        code: "AO",
        phone_lengths: &[9],
        prefix: 244,
    },
    Country {
        // 10 = geographic (11 4321 1234), 11 = mobile with the `9` prefix
        // (9 11 4321 1234). 6-8 digits are subscriber-only forms that are never
        // dialable internationally.
        name: "Argentina",
        code: "AR",
        phone_lengths: &[10, 11],
        prefix: 54,
    },
    Country {
        name: "American Samoa",
        code: "AS",
        phone_lengths: &[7],
        prefix: 1684,
    },
    Country {
        // Austria has an open numbering plan: a fixed-line subscriber number grows
        // with the exchange's size, from 4 digits up to 13; 2 + 13 = 15, the E.164
        // ceiling.
        name: "Austria",
        code: "AT",
        phone_lengths: &[4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
        prefix: 43,
    },
    Country {
        name: "Australia",
        code: "AU",
        phone_lengths: &[5, 6, 7, 8, 9, 10, 12],
        prefix: 61,
    },
    Country {
        name: "Aruba",
        code: "AW",
        phone_lengths: &[7],
        prefix: 297,
    },
    Country {
        name: "Azerbaijan",
        code: "AZ",
        phone_lengths: &[9],
        prefix: 994,
    },
    Country {
        name: "Bosnia and Herzegovina",
        code: "BA",
        phone_lengths: &[8, 9],
        prefix: 387,
    },
    Country {
        name: "Barbados",
        code: "BB",
        phone_lengths: &[7],
        prefix: 1246,
    },
    Country {
        name: "Bangladesh",
        code: "BD",
        phone_lengths: &[6, 7, 8, 9, 10],
        prefix: 880,
    },
    Country {
        // 8 = landline (2 xxx xx xx), 9 = mobile (4xx xx xx xx).
        name: "Belgium",
        code: "BE",
        phone_lengths: &[8, 9],
        prefix: 32,
    },
    Country {
        name: "Burkina Faso",
        code: "BF",
        phone_lengths: &[8],
        prefix: 226,
    },
    Country {
        name: "Bulgaria",
        code: "BG",
        phone_lengths: &[6, 7, 8, 9, 12],
        prefix: 359,
    },
    Country {
        name: "Bahrain",
        code: "BH",
        phone_lengths: &[8],
        prefix: 973,
    },
    Country {
        // 8 = fixed line (02 298 2000 -> NSN 22982000), 9 = mobile (59x ...),
        // 10 = the 1700/1800 non-geographic ranges.
        name: "Palestine",
        code: "PS",
        phone_lengths: &[8, 9, 10],
        prefix: 970,
    },
    Country {
        name: "Israel",
        code: "IL",
        phone_lengths: &[7, 8, 9, 10, 11, 12],
        prefix: 972,
    },
    Country {
        name: "Burundi",
        code: "BI",
        phone_lengths: &[8],
        prefix: 257,
    },
    Country {
        // ARCEP-Benin prepended `01` to every fixed and mobile number on
        // 30 November 2024, taking those NSNs from 8 to 10 digits. NOTE: that
        // leading zero is part of the number here, not a trunk prefix - the ITU
        // communication is explicit that "it is mandatory to dial the leading zero
        // digit `0` when calling from abroad" (see `ZERO_LED` below). The 8-digit
        // length survives for the 81 UAN and 85 VoIP ranges.
        name: "Benin",
        code: "BJ",
        phone_lengths: &[8, 10],
        prefix: 229,
    },
    Country {
        name: "Bermuda",
        code: "BM",
        phone_lengths: &[7],
        prefix: 1441,
    },
    Country {
        name: "Brunei Darussalam",
        code: "BN",
        phone_lengths: &[7],
        prefix: 673,
    },
    Country {
        name: "Bolivia",
        code: "BO",
        phone_lengths: &[8, 9],
        prefix: 591,
    },
    Country {
        name: "Brazil",
        code: "BR",
        phone_lengths: &[8, 9, 10, 11],
        prefix: 55,
    },
    Country {
        name: "Bahamas",
        code: "BS",
        phone_lengths: &[7],
        prefix: 1242,
    },
    Country {
        name: "Bhutan",
        code: "BT",
        phone_lengths: &[7, 8],
        prefix: 975,
    },
    Country {
        name: "Botswana",
        code: "BW",
        phone_lengths: &[7, 8, 10],
        prefix: 267,
    },
    Country {
        name: "Belarus",
        code: "BY",
        phone_lengths: &[6, 7, 8, 9, 10, 11],
        prefix: 375,
    },
    Country {
        name: "Belize",
        code: "BZ",
        phone_lengths: &[7, 11],
        prefix: 501,
    },
    Country {
        // Cocos (Keeling) Islands sit inside the Australian plan (+61 8 9162 xxxx),
        // so they share every Australian length: 9 for fixed and mobile, and the
        // 13/1300/1800/1900 non-geographic ranges at 6, 7, 8, 10 and 12.
        name: "Cocos (Keeling) Islands",
        code: "CC",
        phone_lengths: &[6, 7, 8, 9, 10, 12],
        prefix: 61,
    },
    Country {
        name: "Congo, Democratic Republic of the",
        // Fixed line is variable, 7-10 digits; mobile is 7 or 9.
        code: "CD",
        phone_lengths: &[7, 8, 9, 10],
        prefix: 243,
    },
    Country {
        name: "Central African Republic",
        code: "CF",
        phone_lengths: &[8],
        prefix: 236,
    },
    Country {
        name: "Congo, Republic of the",
        code: "CG",
        phone_lengths: &[9],
        prefix: 242,
    },
    Country {
        name: "Switzerland",
        code: "CH",
        phone_lengths: &[9, 12],
        prefix: 41,
    },
    Country {
        // Cote d'Ivoire moved to a uniform 10-digit plan in January 2021; the old
        // 8-digit numbers were withdrawn.
        name: "Cote d'Ivoire",
        code: "CI",
        phone_lengths: &[10],
        prefix: 225,
    },
    Country {
        name: "Cook Islands",
        // Both fixed and mobile are 5 digits; nothing in the plan is 7.
        code: "CK",
        phone_lengths: &[5],
        prefix: 682,
    },
    Country {
        name: "Chile",
        code: "CL",
        phone_lengths: &[9, 10, 11],
        prefix: 56,
    },
    Country {
        name: "Cameroon",
        code: "CM",
        phone_lengths: &[8, 9],
        prefix: 237,
    },
    Country {
        // Mainland China is dialled with an area code internationally. Fixed lines
        // run 7-11 digits with it (2- or 3-digit area code plus a 5- to 8-digit
        // subscriber number), mobile is 11, and 400/800 service numbers 10 or 12.
        name: "China",
        code: "CN",
        phone_lengths: &[7, 8, 9, 10, 11, 12],
        prefix: 86,
    },
    Country {
        name: "Colombia",
        code: "CO",
        phone_lengths: &[8, 10, 11],
        prefix: 57,
    },
    Country {
        name: "Costa Rica",
        code: "CR",
        phone_lengths: &[8, 10],
        prefix: 506,
    },
    Country {
        name: "Cuba",
        code: "CU",
        phone_lengths: &[6, 7, 8, 10],
        prefix: 53,
    },
    Country {
        name: "Cape Verde",
        code: "CV",
        phone_lengths: &[7],
        prefix: 238,
    },
    Country {
        // Curacao is the main region for +599. Its fixed and mobile numbers are
        // 7 or 8 digits (9 xxx xxxx); only the leading digit separates it from BQ,
        // which shares the calling code (see `TERRITORY_PREFIXES` below).
        name: "Curacao",
        code: "CW",
        phone_lengths: &[7, 8],
        prefix: 599,
    },
    Country {
        // Bonaire, Sint Eustatius and Saba stayed on +599 after the dissolution
        // of the Netherlands Antilles. Every number here is 7 digits — Bonaire
        // 7xx xxxx, Saba/Sint Eustatius 4xx xxxx, mobile 3xx xxxx. Curacao has
        // 7-digit numbers too, so length cannot split the two; the leading 3, 4
        // or 7 does (see `TERRITORY_PREFIXES` below).
        name: "Bonaire, Sint Eustatius and Saba",
        code: "BQ",
        phone_lengths: &[7],
        prefix: 599,
    },
    Country {
        // Christmas Island is served from the Australian plan (+61 8 9164 xxxx)
        // and shares every Australian length; see CC above.
        name: "Christmas Island",
        code: "CX",
        phone_lengths: &[6, 7, 8, 9, 10, 12],
        prefix: 61,
    },
    Country {
        name: "Cyprus",
        code: "CY",
        phone_lengths: &[8],
        prefix: 357,
    },
    Country {
        name: "Czech Republic",
        code: "CZ",
        phone_lengths: &[9, 10, 11, 12],
        prefix: 420,
    },
    Country {
        name: "Germany",
        code: "DE",
        phone_lengths: &[4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
        prefix: 49,
    },
    Country {
        name: "Djibouti",
        code: "DJ",
        phone_lengths: &[8],
        prefix: 253,
    },
    Country {
        name: "Denmark",
        code: "DK",
        phone_lengths: &[8],
        prefix: 45,
    },
    Country {
        name: "Dominica",
        code: "DM",
        phone_lengths: &[7],
        prefix: 1767,
    },
    // The Dominican Republic runs three overlay area codes (809/829/849) over one
    // territory. Each needs its own entry so that prefix resolution recognises all
    // three, which means the ISO code "DO" deliberately appears more than once.
    // Lookups of the form `COUNTRIES.iter().find(|c| c.code == "DO")` reach only
    // the first (809) entry; that is intentional and 809 is the original code.
    // The same pattern applies to PR (787/939) and JM (876/658) below.
    Country {
        name: "Dominican Republic",
        code: "DO",
        phone_lengths: &[7],
        prefix: 1809,
    },
    Country {
        name: "Dominican Republic",
        code: "DO",
        phone_lengths: &[7],
        prefix: 1829,
    },
    Country {
        name: "Dominican Republic",
        code: "DO",
        phone_lengths: &[7],
        prefix: 1849,
    },
    Country {
        name: "Algeria",
        // 8 = fixed line (021 71 15 00 -> NSN 21711500), 9 = fixed and mobile.
        code: "DZ",
        phone_lengths: &[8, 9],
        prefix: 213,
    },
    Country {
        // 8 = fixed line (02 299 3200 -> NSN 22993200), 9 = mobile, 10-11 = the
        // 1800 freephone range.
        name: "Ecuador",
        code: "EC",
        phone_lengths: &[8, 9, 10, 11],
        prefix: 593,
    },
    Country {
        // 7 = fixed line (631 3000) and short mobile, 8 = mobile (5123 4567),
        // 10 = the long 800 freephone range.
        name: "Estonia",
        code: "EE",
        phone_lengths: &[7, 8, 10],
        prefix: 372,
    },
    Country {
        // 10 = mobile (1x xxxx xxxx); 8-9 = landline (Cairo 2 xxxx xxxx, and the
        // shorter two-/three-digit area codes elsewhere).
        name: "Egypt",
        code: "EG",
        phone_lengths: &[8, 9, 10],
        prefix: 20,
    },
    Country {
        name: "Eritrea",
        code: "ER",
        phone_lengths: &[7],
        prefix: 291,
    },
    Country {
        name: "Spain",
        code: "ES",
        phone_lengths: &[9],
        prefix: 34,
    },
    Country {
        name: "Ethiopia",
        code: "ET",
        phone_lengths: &[9],
        prefix: 251,
    },
    Country {
        name: "Finland",
        code: "FI",
        phone_lengths: &[5, 6, 7, 8, 9, 10, 11, 12],
        prefix: 358,
    },
    Country {
        // The Aland Islands have no calling code of their own: they are area code
        // 18 inside the Finnish plan, and Aland mobiles use ordinary Finnish mobile
        // ranges. Finland is the main region for +358, and an Aland number is told
        // apart by its `18` area code (see `TERRITORY_PREFIXES` below). The lengths
        // are the Finnish plan's, which is genuinely variable: 5-12 across the
        // national service numbers.
        name: "Åland Islands",
        code: "AX",
        phone_lengths: &[5, 6, 7, 8, 9, 10, 11, 12],
        prefix: 358,
    },
    Country {
        name: "Fiji",
        code: "FJ",
        phone_lengths: &[7, 11],
        prefix: 679,
    },
    Country {
        name: "Falkland Islands (Malvinas)",
        code: "FK",
        phone_lengths: &[5],
        prefix: 500,
    },
    Country {
        name: "Micronesia, Federated States of",
        code: "FM",
        phone_lengths: &[7],
        prefix: 691,
    },
    Country {
        // Faroese numbers are uniformly 6 digits; 5 is not a length in the plan.
        name: "Faroe Islands",
        code: "FO",
        phone_lengths: &[6],
        prefix: 298,
    },
    Country {
        name: "France",
        code: "FR",
        phone_lengths: &[9],
        prefix: 33,
    },
    Country {
        // Gabon's 2019-2024 migration took the *national* format to nine digits,
        // `0XX XX XX XX`, but that leading `0` is a trunk prefix dropped
        // internationally, leaving 8 (11 fixed, 6x/7x mobile). Numbers from the
        // earlier plan keep a significant leading zero at 8 (`01 44 12 34`), and a
        // few mobile ranges are still 7.
        name: "Gabon",
        code: "GA",
        phone_lengths: &[7, 8],
        prefix: 241,
    },
    Country {
        // 10 = the common case (20 xxxx xxxx, 7xxx xxxxxx); 9 covers the four-digit
        // area codes with five-digit subscriber numbers (e.g. +44 1697 73xxx) and
        // the 0800/0500 ranges; 7 is the short freephone block, of which the live
        // example is 0800 1111 (Childline). GB is the main region for +44; GG, IM
        // and JE share the code and the lengths, and are told apart by prefix
        // (see `TERRITORY_PREFIXES` below).
        name: "United Kingdom",
        code: "GB",
        phone_lengths: &[7, 9, 10],
        prefix: 44,
    },
    Country {
        name: "Grenada",
        code: "GD",
        phone_lengths: &[7],
        prefix: 1473,
    },
    Country {
        name: "Georgia",
        code: "GE",
        phone_lengths: &[9],
        prefix: 995,
    },
    Country {
        name: "French Guiana",
        code: "GF",
        phone_lengths: &[9],
        prefix: 594,
    },
    Country {
        // Guernsey is +44 1481 xxxxxx / 7781 xxxxxx inside the UK plan, so it
        // shares the UK's lengths: 10, plus the short 7- and 9-digit freephone and
        // premium ranges. It is told apart from GB by prefix (see
        // `TERRITORY_PREFIXES` below).
        name: "Guernsey",
        code: "GG",
        phone_lengths: &[7, 9, 10],
        prefix: 44,
    },
    Country {
        name: "Ghana",
        code: "GH",
        phone_lengths: &[8, 9],
        prefix: 233,
    },
    Country {
        name: "Gibraltar",
        code: "GI",
        phone_lengths: &[8],
        prefix: 350,
    },
    Country {
        name: "Greenland",
        code: "GL",
        phone_lengths: &[6],
        prefix: 299,
    },
    Country {
        name: "Gambia",
        code: "GM",
        phone_lengths: &[7, 9],
        prefix: 220,
    },
    Country {
        name: "Guinea",
        // 8 = fixed line (30 41 15 00 in Conakry), 9 = mobile.
        code: "GN",
        phone_lengths: &[8, 9],
        prefix: 224,
    },
    Country {
        // Guadeloupe is the main region for +590. Saint Barthelemy and Saint
        // Martin share the code and the length; each has a few blocks of its own
        // (see `TERRITORY_PREFIXES` below).
        name: "Guadeloupe",
        code: "GP",
        phone_lengths: &[9],
        prefix: 590,
    },
    Country {
        name: "Saint Barthelemy",
        code: "BL",
        phone_lengths: &[9],
        prefix: 590,
    },
    Country {
        name: "Saint Martin (French part)",
        code: "MF",
        phone_lengths: &[9],
        prefix: 590,
    },
    Country {
        name: "Equatorial Guinea",
        code: "GQ",
        phone_lengths: &[9],
        prefix: 240,
    },
    Country {
        name: "Greece",
        code: "GR",
        phone_lengths: &[10, 11, 12],
        prefix: 30,
    },
    Country {
        name: "South Georgia and the South Sandwich Islands",
        code: "GS",
        phone_lengths: &[5],
        prefix: 500,
    },
    Country {
        name: "Guatemala",
        code: "GT",
        phone_lengths: &[8, 11],
        prefix: 502,
    },
    Country {
        name: "Guam",
        code: "GU",
        phone_lengths: &[7],
        prefix: 1671,
    },
    Country {
        name: "Guinea-Bissau",
        code: "GW",
        phone_lengths: &[7, 9],
        prefix: 245,
    },
    Country {
        name: "Guyana",
        code: "GY",
        phone_lengths: &[7],
        prefix: 592,
    },
    Country {
        name: "Hong Kong",
        code: "HK",
        phone_lengths: &[5, 6, 7, 8, 9, 11],
        prefix: 852,
    },
    Country {
        name: "Honduras",
        code: "HN",
        phone_lengths: &[8, 11],
        prefix: 504,
    },
    Country {
        // 8 = Zagreb and the 2-digit area codes (01 4569 964 -> NSN 14569964),
        // 9 = the longer fixed subscriber ranges and mobile, 7 = the short 060
        // premium and 0800 freephone ranges.
        name: "Croatia",
        code: "HR",
        phone_lengths: &[7, 8, 9],
        prefix: 385,
    },
    Country {
        name: "Haiti",
        code: "HT",
        phone_lengths: &[8],
        prefix: 509,
    },
    Country {
        name: "Hungary",
        // 8 = fixed line (Budapest 1 + 7 digits, elsewhere 2-digit area + 6),
        // 9 = mobile.
        code: "HU",
        phone_lengths: &[8, 9],
        prefix: 36,
    },
    Country {
        name: "Indonesia",
        code: "ID",
        phone_lengths: &[7, 8, 9, 10, 11, 12, 13],
        prefix: 62,
    },
    Country {
        name: "Ireland",
        // Fixed line is variable at 7-10; mobile is 9; 10 also carries the
        // 1800 xxxxxx freephone and 15xx premium ranges.
        code: "IE",
        phone_lengths: &[7, 8, 9, 10],
        prefix: 353,
    },
    Country {
        name: "Isle of Man",
        code: "IM",
        phone_lengths: &[10],
        prefix: 44,
    },
    Country {
        name: "India",
        code: "IN",
        phone_lengths: &[8, 9, 10, 11, 12, 13],
        prefix: 91,
    },
    Country {
        name: "British Indian Ocean Territory",
        code: "IO",
        phone_lengths: &[7],
        prefix: 246,
    },
    Country {
        name: "Iraq",
        // 8 and 9 = fixed line (Baghdad 1 + 7, other governorates 2 + 7 or 8),
        // 10 = mobile (7xx xxx xxxx).
        code: "IQ",
        phone_lengths: &[8, 9, 10],
        prefix: 964,
    },
    Country {
        name: "Iran",
        code: "IR",
        phone_lengths: &[4, 5, 6, 7, 10],
        prefix: 98,
    },
    Country {
        name: "Iceland",
        // 7 = the standard Icelandic number, 9 = the longer mobile range.
        code: "IS",
        phone_lengths: &[7, 9],
        prefix: 354,
    },
    Country {
        // Italy keeps the leading 0 of the area code in the international form, so
        // the lengths here include it (see `ZERO_LED` below). Fixed lines run 6-12
        // digits; mobiles are 9 or 10. Main region for +39; VA follows.
        name: "Italy",
        code: "IT",
        phone_lengths: &[6, 7, 8, 9, 10, 11, 12],
        prefix: 39,
    },
    Country {
        // +379 is reserved for the Holy See but has never been brought into
        // service: Vatican numbers live in the Italian plan as 06 698 xxxxx. This
        // entry therefore carries Italy's code and lengths, and IT wins +39 except
        // for the Vatican's own `06698` block (see `TERRITORY_PREFIXES` below).
        name: "Holy See (Vatican City State)",
        code: "VA",
        phone_lengths: &[6, 7, 8, 9, 10, 11, 12],
        prefix: 39,
    },
    Country {
        // Jersey is +44 1534 xxxxxx / 7797 xxxxxx: a 10-digit NSN, not 6.
        name: "Jersey",
        code: "JE",
        phone_lengths: &[10],
        prefix: 44,
    },
    Country {
        name: "Jamaica",
        code: "JM",
        phone_lengths: &[7],
        prefix: 1876,
    },
    Country {
        // 658 is Jamaica's overlay on 876, in service since 2018.
        name: "Jamaica",
        code: "JM",
        phone_lengths: &[7],
        prefix: 1658,
    },
    Country {
        name: "Jordan",
        code: "JO",
        phone_lengths: &[8, 9],
        prefix: 962,
    },
    Country {
        // 9 = landline (+81 3 1234 5678 -> NSN 312345678), 10 = mobile
        // (+81 90 1234 5678), and 8-13 for the 0120/0800 freephone and 0037/0066
        // carrier freephone ranges. Longer freephone forms exist but exceed the
        // E.164 ceiling and are not globally dialable.
        name: "Japan",
        code: "JP",
        phone_lengths: &[8, 9, 10, 11, 12, 13],
        prefix: 81,
    },
    Country {
        // Fixed line is variable at 7-9 (20 201 2345 and shorter county ranges),
        // mobile is 9, and 800 freephone is 9 or 10.
        name: "Kenya",
        code: "KE",
        phone_lengths: &[7, 8, 9, 10],
        prefix: 254,
    },
    Country {
        name: "Kyrgyzstan",
        code: "KG",
        phone_lengths: &[9, 10],
        prefix: 996,
    },
    Country {
        name: "Cambodia",
        code: "KH",
        phone_lengths: &[8, 9, 10],
        prefix: 855,
    },
    Country {
        name: "Kiribati",
        // Kiribati moved mobile from 5 to 8 digits between 2013 and 2018; the
        // outer-island fixed ranges are still 5. Both lengths are live.
        code: "KI",
        phone_lengths: &[5, 8],
        prefix: 686,
    },
    Country {
        name: "Comoros",
        code: "KM",
        phone_lengths: &[7],
        prefix: 269,
    },
    Country {
        name: "Saint Kitts and Nevis",
        code: "KN",
        phone_lengths: &[7],
        prefix: 1869,
    },
    Country {
        name: "Korea, Democratic People's Republic of",
        // 8 = fixed line (2 xxx xxxx), 10 = the longer fixed range and mobile
        // (191 xxx xxxx). 6, 7 and 11 are not lengths the plan produces.
        code: "KP",
        phone_lengths: &[8, 10],
        prefix: 850,
    },
    Country {
        // Fixed lines run 8-10 digits (5 and 6 for a few short service numbers),
        // mobile 9-10, personal 10-11, and the 00798 international freephone
        // ranges 11-13.
        name: "Korea, Republic of",
        code: "KR",
        phone_lengths: &[5, 6, 8, 9, 10, 11, 12, 13],
        prefix: 82,
    },
    Country {
        name: "Kuwait",
        code: "KW",
        phone_lengths: &[7, 8],
        prefix: 965,
    },
    Country {
        name: "Cayman Islands",
        code: "KY",
        phone_lengths: &[7],
        prefix: 1345,
    },
    Country {
        // Russia and Kazakhstan share +7 and the same lengths - 10, plus 14 for
        // the `8 10 8xx` freephone range - so length cannot separate them. Russia
        // wins prefix resolution because `PRIMARY_FOR_PREFIX` in src/lib.rs names
        // it for +7; a Kazakh number is told apart by its leading 7 (see
        // `TERRITORY_PREFIXES` below). It is kept immediately before KZ only so
        // that the two entries read together.
        name: "Russian Federation",
        code: "RU",
        phone_lengths: &[10, 14],
        prefix: 7,
    },
    Country {
        name: "Kazakhstan",
        code: "KZ",
        phone_lengths: &[10, 14],
        prefix: 7,
    },
    Country {
        name: "Lao People's Democratic Republic",
        // 8 = fixed line, 9-10 = mobile (20 xx xxx xxx is 10).
        code: "LA",
        phone_lengths: &[8, 9, 10],
        prefix: 856,
    },
    Country {
        name: "Lebanon",
        code: "LB",
        phone_lengths: &[7, 8],
        prefix: 961,
    },
    Country {
        name: "Saint Lucia",
        code: "LC",
        phone_lengths: &[7],
        prefix: 1758,
    },
    Country {
        name: "Liechtenstein",
        // 7 = fixed line and the short mobile range, 9 = the 66x mobile range.
        code: "LI",
        phone_lengths: &[7, 9],
        prefix: 423,
    },
    Country {
        // Sri Lanka is on a uniform 9-digit plan (7x xxx xxxx mobile,
        // 11 x xxx xxxx fixed). The old 7- and 10-digit entries were not dialable.
        name: "Sri Lanka",
        code: "LK",
        phone_lengths: &[9],
        prefix: 94,
    },
    Country {
        name: "Liberia",
        code: "LR",
        phone_lengths: &[7, 8, 9],
        prefix: 231,
    },
    Country {
        name: "Lesotho",
        code: "LS",
        phone_lengths: &[8],
        prefix: 266,
    },
    Country {
        name: "Lithuania",
        code: "LT",
        phone_lengths: &[8],
        prefix: 370,
    },
    Country {
        name: "Luxembourg",
        code: "LU",
        phone_lengths: &[4, 5, 6, 7, 8, 9, 10, 11],
        prefix: 352,
    },
    Country {
        name: "Latvia",
        code: "LV",
        phone_lengths: &[8],
        prefix: 371,
    },
    Country {
        name: "Libya",
        // Libyan numbers are uniformly 9 digits; 8 and 10 are not in the plan.
        code: "LY",
        phone_lengths: &[9],
        prefix: 218,
    },
    Country {
        // Morocco is the main region for +212 and shares it with EH.
        name: "Morocco",
        code: "MA",
        phone_lengths: &[9],
        prefix: 212,
    },
    Country {
        // Western Sahara has no plan of its own; it is served entirely from
        // Morocco's +212 ranges (5288/5289), which Morocco's own plan also
        // covers, so +212 numbers are reported as MA. This entry exists for
        // lookup by ISO code.
        name: "Western Sahara",
        code: "EH",
        phone_lengths: &[9],
        prefix: 212,
    },
    Country {
        name: "Monaco",
        // 8 = fixed line and the 4x mobile range, 9 = the 6x mobile range.
        code: "MC",
        phone_lengths: &[8, 9],
        prefix: 377,
    },
    Country {
        name: "Moldova, Republic of",
        code: "MD",
        phone_lengths: &[8],
        prefix: 373,
    },
    Country {
        name: "Montenegro",
        code: "ME",
        phone_lengths: &[8, 9],
        prefix: 382,
    },
    Country {
        name: "Madagascar",
        // Malagasy numbers are uniformly 9 digits; 7 and 8 are local-only forms.
        code: "MG",
        phone_lengths: &[9],
        prefix: 261,
    },
    Country {
        name: "Marshall Islands",
        code: "MH",
        phone_lengths: &[7],
        prefix: 692,
    },
    Country {
        name: "North Macedonia",
        code: "MK",
        phone_lengths: &[8],
        prefix: 389,
    },
    Country {
        name: "Mali",
        code: "ML",
        phone_lengths: &[8],
        prefix: 223,
    },
    Country {
        name: "Myanmar",
        // Fixed line is variable at 6-9, mobile at 7-10; most Burmese mobiles
        // are 8 or 9, which the old [7, 10] rejected outright.
        code: "MM",
        phone_lengths: &[6, 7, 8, 9, 10],
        prefix: 95,
    },
    Country {
        name: "Mongolia",
        code: "MN",
        phone_lengths: &[8, 9, 10],
        prefix: 976,
    },
    Country {
        name: "Macao",
        code: "MO",
        phone_lengths: &[7, 8],
        prefix: 853,
    },
    Country {
        name: "Northern Mariana Islands",
        code: "MP",
        phone_lengths: &[7],
        prefix: 1670,
    },
    Country {
        name: "Martinique",
        code: "MQ",
        phone_lengths: &[9],
        prefix: 596,
    },
    Country {
        name: "Mauritania",
        code: "MR",
        phone_lengths: &[8],
        prefix: 222,
    },
    Country {
        name: "Montserrat",
        code: "MS",
        phone_lengths: &[7],
        prefix: 1664,
    },
    Country {
        name: "Malta",
        code: "MT",
        phone_lengths: &[8],
        prefix: 356,
    },
    Country {
        // 7 = fixed line (208 1234) and service ranges, 8 = the longer fixed range
        // and mobile, 10 = long freephone numbers.
        name: "Mauritius",
        code: "MU",
        phone_lengths: &[7, 8, 10],
        prefix: 230,
    },
    Country {
        name: "Maldives",
        code: "MV",
        phone_lengths: &[7, 10],
        prefix: 960,
    },
    Country {
        name: "Malawi",
        // 7 = fixed line, 9 = mobile. Nothing in the plan is 8.
        code: "MW",
        phone_lengths: &[7, 9],
        prefix: 265,
    },
    Country {
        name: "Mexico",
        code: "MX",
        phone_lengths: &[10],
        prefix: 52,
    },
    Country {
        name: "Malaysia",
        // 8-9 = fixed line, 9-10 = mobile and the 1300/1600 ranges. 7 is a
        // local-only form, not an NSN.
        code: "MY",
        phone_lengths: &[8, 9, 10],
        prefix: 60,
    },
    Country {
        name: "Mozambique",
        code: "MZ",
        phone_lengths: &[8, 9],
        prefix: 258,
    },
    Country {
        name: "Namibia",
        // 8-9 = fixed line, 9 = mobile. 7 and 10 are not lengths in the plan.
        code: "NA",
        phone_lengths: &[8, 9],
        prefix: 264,
    },
    Country {
        name: "New Caledonia",
        code: "NC",
        phone_lengths: &[6],
        prefix: 687,
    },
    Country {
        name: "Niger",
        code: "NE",
        phone_lengths: &[8],
        prefix: 227,
    },
    Country {
        // Sole occupant of +672 now that the fictitious Antarctica and Heard &
        // McDonald Islands entries are gone.
        name: "Norfolk Island",
        code: "NF",
        phone_lengths: &[6],
        prefix: 672,
    },
    Country {
        // 10 for fixed and mobile: the NCC moved fixed lines from 8 to 10 digits
        // on 1 January 2024 by prefixing `02` (old 01 463 0643 -> 020 1463 0643,
        // NSN 2014630643). The 700/800 non-geographic ranges run to 12.
        name: "Nigeria",
        code: "NG",
        phone_lengths: &[10, 11, 12],
        prefix: 234,
    },
    Country {
        name: "Nicaragua",
        code: "NI",
        phone_lengths: &[8],
        prefix: 505,
    },
    Country {
        name: "Netherlands",
        code: "NL",
        phone_lengths: &[5, 6, 7, 8, 9, 10, 11],
        prefix: 31,
    },
    Country {
        name: "Norway",
        code: "NO",
        phone_lengths: &[5, 8],
        prefix: 47,
    },
    Country {
        // 8 = fixed line (1 421 1000 in Kathmandu), 10 = mobile (98x xxx xxxx),
        // 11 = the 1660 freephone range.
        name: "Nepal",
        code: "NP",
        phone_lengths: &[8, 10, 11],
        prefix: 977,
    },
    Country {
        name: "Nauru",
        code: "NR",
        phone_lengths: &[7],
        prefix: 674,
    },
    Country {
        name: "Niue",
        // 4 = the original fixed/mobile numbers, 7 = the later mobile range.
        code: "NU",
        phone_lengths: &[4, 7],
        prefix: 683,
    },
    Country {
        name: "New Zealand",
        code: "NZ",
        phone_lengths: &[5, 6, 7, 8, 9, 10],
        prefix: 64,
    },
    Country {
        name: "Oman",
        code: "OM",
        phone_lengths: &[7, 8, 9],
        prefix: 968,
    },
    Country {
        // 7 = fixed line (all Panamanian landlines), 8 = mobile, 10-11 = the
        // 800/00800 freephone ranges.
        name: "Panama",
        code: "PA",
        phone_lengths: &[7, 8, 10, 11],
        prefix: 507,
    },
    Country {
        // 9 = mobile (9xx xxx xxx); 8 = Lima landline (1 xxx xxxx).
        name: "Peru",
        code: "PE",
        phone_lengths: &[8, 9],
        prefix: 51,
    },
    Country {
        name: "French Polynesia",
        code: "PF",
        phone_lengths: &[6, 8, 9],
        prefix: 689,
    },
    Country {
        name: "Papua New Guinea",
        code: "PG",
        phone_lengths: &[7, 8],
        prefix: 675,
    },
    Country {
        // 10 = mobile (9xx xxx xxxx); 6 and 8-10 = landline, including the Metro
        // Manila area code 2 with its post-2019 eight-digit subscriber numbers;
        // 11-13 = the 1800 freephone ranges.
        name: "Philippines",
        code: "PH",
        phone_lengths: &[6, 8, 9, 10, 11, 12, 13],
        prefix: 63,
    },
    Country {
        // 9-10 = fixed line (51 111 1234 is 9), 10 = mobile, 8 = the 800/900
        // ranges, 11-12 = the 111 UAN ranges.
        name: "Pakistan",
        code: "PK",
        phone_lengths: &[8, 9, 10, 11, 12],
        prefix: 92,
    },
    Country {
        name: "Poland",
        code: "PL",
        phone_lengths: &[6, 7, 8, 9, 10],
        prefix: 48,
    },
    Country {
        name: "Saint Pierre and Miquelon",
        // 6 = the local fixed/mobile format, 9 = the longer ranges. Nothing is 8.
        code: "PM",
        phone_lengths: &[6, 9],
        prefix: 508,
    },
    // Puerto Rico runs two overlay area codes (787/939); see the note on the
    // Dominican Republic group above for why the ISO code repeats.
    Country {
        name: "Puerto Rico",
        code: "PR",
        phone_lengths: &[7],
        prefix: 1787,
    },
    Country {
        name: "Puerto Rico",
        code: "PR",
        phone_lengths: &[7],
        prefix: 1939,
    },
    Country {
        name: "Portugal",
        code: "PT",
        phone_lengths: &[9],
        prefix: 351,
    },
    Country {
        name: "Palau",
        code: "PW",
        phone_lengths: &[7],
        prefix: 680,
    },
    Country {
        name: "Paraguay",
        code: "PY",
        phone_lengths: &[6, 7, 8, 9, 10, 11],
        prefix: 595,
    },
    Country {
        name: "Qatar",
        code: "QA",
        phone_lengths: &[7, 8, 9, 11],
        prefix: 974,
    },
    Country {
        // Main region for +262; Mayotte (YT) shares the code and follows later.
        name: "Reunion",
        code: "RE",
        phone_lengths: &[9],
        prefix: 262,
    },
    Country {
        // Romania is a 9-digit plan (21 xxx xxxx fixed, 7xx xxx xxx mobile); the
        // only other length is 6, for a handful of short fixed-line service numbers.
        name: "Romania",
        code: "RO",
        phone_lengths: &[6, 9],
        prefix: 40,
    },
    Country {
        // Serbian numbers are genuinely variable-length: fixed line runs 7-12
        // (Belgrade 11 + 6 or 7, e.g. 11 234567 -> NSN 11234567), mobile 8-10, and
        // the freephone, premium and UAN ranges go down to 6.
        name: "Serbia",
        code: "RS",
        phone_lengths: &[6, 7, 8, 9, 10, 11, 12],
        prefix: 381,
    },
    Country {
        name: "Rwanda",
        code: "RW",
        phone_lengths: &[8, 9],
        prefix: 250,
    },
    Country {
        name: "Saudi Arabia",
        code: "SA",
        phone_lengths: &[9, 10],
        prefix: 966,
    },
    Country {
        name: "Solomon Islands",
        // 5 = fixed line and the short mobile range, 7 = the longer mobile range.
        code: "SB",
        phone_lengths: &[5, 7],
        prefix: 677,
    },
    Country {
        name: "Seychelles",
        code: "SC",
        phone_lengths: &[7],
        prefix: 248,
    },
    Country {
        name: "Sudan",
        // Sudanese numbers are uniformly 9 digits; 7 and 10 are not in the plan.
        code: "SD",
        phone_lengths: &[9],
        prefix: 249,
    },
    Country {
        name: "Sweden",
        code: "SE",
        phone_lengths: &[6, 7, 8, 9, 10, 12],
        prefix: 46,
    },
    Country {
        name: "Singapore",
        code: "SG",
        phone_lengths: &[8, 10, 11],
        prefix: 65,
    },
    Country {
        // Main region for +290; Tristan da Cunha shares the code (see TA).
        name: "Saint Helena",
        code: "SH",
        phone_lengths: &[4, 5],
        prefix: 290,
    },
    Country {
        // Tristan da Cunha is part of the Saint Helena plan: four-digit numbers
        // in the 8xxx block. Kept next to SH so that the two read together.
        name: "Tristan da Cunha",
        code: "TA",
        phone_lengths: &[4],
        prefix: 290,
    },
    Country {
        name: "Slovenia",
        code: "SI",
        phone_lengths: &[5, 6, 7, 8],
        prefix: 386,
    },
    Country {
        // Svalbard and Jan Mayen use Norwegian numbers (+47 79 xx xx xx) and share
        // Norway's lengths, 8 and the 5-digit `02xxx` UAN range. NO is the main
        // region for +47; the `79` block identifies SJ (see `TERRITORY_PREFIXES`).
        name: "Svalbard and Jan Mayen",
        code: "SJ",
        phone_lengths: &[5, 8],
        prefix: 47,
    },
    Country {
        name: "Slovakia",
        code: "SK",
        phone_lengths: &[6, 7, 9],
        prefix: 421,
    },
    Country {
        name: "Sierra Leone",
        code: "SL",
        phone_lengths: &[8],
        prefix: 232,
    },
    Country {
        name: "San Marino",
        // 8 = mobile (6x xxx xxx), 10 = fixed line (0549 88 63 77 — San Marino
        // fixed numbers keep the Italian-style leading zero and are reached as
        // +378 0549 xxxxxx). 6, 7 and 9 are not lengths the plan produces.
        code: "SM",
        phone_lengths: &[8, 10],
        prefix: 378,
    },
    Country {
        name: "Senegal",
        code: "SN",
        phone_lengths: &[9],
        prefix: 221,
    },
    Country {
        name: "Somalia",
        // 6-7 = fixed line, 7-9 = mobile.
        code: "SO",
        phone_lengths: &[6, 7, 8, 9],
        prefix: 252,
    },
    Country {
        name: "Suriname",
        code: "SR",
        phone_lengths: &[6, 7],
        prefix: 597,
    },
    Country {
        name: "South Sudan",
        // South Sudanese numbers are uniformly 9 digits; 7 is not in the plan.
        code: "SS",
        phone_lengths: &[9],
        prefix: 211,
    },
    Country {
        name: "Sao Tome and Principe",
        code: "ST",
        phone_lengths: &[7],
        prefix: 239,
    },
    Country {
        name: "El Salvador",
        code: "SV",
        phone_lengths: &[7, 8, 11],
        prefix: 503,
    },
    Country {
        name: "Sint Maarten (Dutch part)",
        code: "SX",
        phone_lengths: &[7],
        prefix: 1721,
    },
    Country {
        name: "Syrian Arab Republic",
        // 8-9 = fixed line, 9 = mobile. 7 and 10 are not lengths in the plan.
        code: "SY",
        phone_lengths: &[8, 9],
        prefix: 963,
    },
    Country {
        name: "Eswatini",
        code: "SZ",
        phone_lengths: &[8, 9],
        prefix: 268,
    },
    Country {
        name: "Turks and Caicos Islands",
        code: "TC",
        phone_lengths: &[7],
        prefix: 1649,
    },
    Country {
        name: "Chad",
        // Chadian numbers are uniformly 8 digits; 6 is not in the plan.
        code: "TD",
        phone_lengths: &[8],
        prefix: 235,
    },
    Country {
        name: "Togo",
        code: "TG",
        phone_lengths: &[8],
        prefix: 228,
    },
    Country {
        // 9 = mobile (8x/9x xxx xxxx); 8 = landline (Bangkok 2 xxx xxxx);
        // 10 = the 1800/1900 ranges; 13 = the 001800 international freephone range.
        name: "Thailand",
        code: "TH",
        phone_lengths: &[8, 9, 10, 13],
        prefix: 66,
    },
    Country {
        name: "Tajikistan",
        code: "TJ",
        phone_lengths: &[9],
        prefix: 992,
    },
    Country {
        name: "Tokelau",
        code: "TK",
        phone_lengths: &[4, 5, 6, 7],
        prefix: 690,
    },
    Country {
        name: "Timor-Leste",
        // 7 = fixed line (331 2345), 8 = mobile (77 21 2345).
        code: "TL",
        phone_lengths: &[7, 8],
        prefix: 670,
    },
    Country {
        name: "Turkmenistan",
        // Turkmen numbers are uniformly 8 digits; 9 is not in the plan.
        code: "TM",
        phone_lengths: &[8],
        prefix: 993,
    },
    Country {
        name: "Tunisia",
        code: "TN",
        phone_lengths: &[8],
        prefix: 216,
    },
    Country {
        name: "Tonga",
        // 5 = fixed line, 7 = mobile. 6 and 8 are not lengths in the plan.
        code: "TO",
        phone_lengths: &[5, 7],
        prefix: 676,
    },
    Country {
        name: "Turkey",
        code: "TR",
        phone_lengths: &[7, 10, 12, 13],
        prefix: 90,
    },
    Country {
        name: "Trinidad and Tobago",
        code: "TT",
        phone_lengths: &[7],
        prefix: 1868,
    },
    Country {
        name: "Tuvalu",
        code: "TV",
        phone_lengths: &[5, 6, 7],
        prefix: 688,
    },
    Country {
        name: "Taiwan",
        code: "TW",
        phone_lengths: &[7, 8, 9, 10, 11],
        prefix: 886,
    },
    Country {
        name: "Tanzania, United Republic of",
        code: "TZ",
        phone_lengths: &[9],
        prefix: 255,
    },
    Country {
        name: "Ukraine",
        code: "UA",
        phone_lengths: &[9, 10],
        prefix: 380,
    },
    Country {
        name: "Uganda",
        code: "UG",
        phone_lengths: &[9],
        prefix: 256,
    },
    Country {
        // US and Canada share the bare +1; the US is the main region and is listed
        // first. The 4-digit 1xxx entries above cover the Caribbean territories.
        name: "United States",
        code: "US",
        phone_lengths: &[10],
        prefix: 1,
    },
    Country {
        name: "Canada",
        code: "CA",
        phone_lengths: &[10],
        prefix: 1,
    },
    Country {
        name: "Uruguay",
        code: "UY",
        phone_lengths: &[4, 5, 6, 7, 8, 9, 10, 11, 12],
        prefix: 598,
    },
    Country {
        name: "Uzbekistan",
        code: "UZ",
        phone_lengths: &[9],
        prefix: 998,
    },
    Country {
        name: "Saint Vincent and the Grenadines",
        code: "VC",
        phone_lengths: &[7],
        prefix: 1784,
    },
    Country {
        name: "Venezuela",
        code: "VE",
        phone_lengths: &[10],
        prefix: 58,
    },
    Country {
        name: "Virgin Islands, British",
        code: "VG",
        phone_lengths: &[7],
        prefix: 1284,
    },
    Country {
        name: "Virgin Islands, U.S.",
        code: "VI",
        phone_lengths: &[7],
        prefix: 1340,
    },
    Country {
        name: "Vietnam",
        code: "VN",
        phone_lengths: &[7, 8, 9, 10],
        prefix: 84,
    },
    Country {
        name: "Vanuatu",
        // 5 = fixed line, 7 = mobile. Nothing in the plan is 6.
        code: "VU",
        phone_lengths: &[5, 7],
        prefix: 678,
    },
    Country {
        name: "Wallis and Futuna",
        code: "WF",
        phone_lengths: &[6, 9],
        prefix: 681,
    },
    Country {
        name: "Samoa",
        code: "WS",
        phone_lengths: &[5, 6, 7, 10],
        prefix: 685,
    },
    Country {
        // Kosovo took over +383 from the ITU in 2016; 8 = mobile (4x xxx xxx) and
        // short fixed lines (38 xxx xxx), with fixed lines running to 12.
        name: "Kosovo",
        code: "XK",
        phone_lengths: &[8, 9, 10, 11, 12],
        prefix: 383,
    },
    Country {
        name: "Yemen",
        // 7-8 = fixed line (1-digit area code + 6 or 7 subscriber digits),
        // 9 = mobile.
        code: "YE",
        phone_lengths: &[7, 8, 9],
        prefix: 967,
    },
    Country {
        // Mayotte shares +262 with Reunion, the main region; its 269 and 639
        // blocks identify it (see `TERRITORY_PREFIXES` below).
        name: "Mayotte",
        code: "YT",
        phone_lengths: &[9],
        prefix: 262,
    },
    Country {
        name: "South Africa",
        code: "ZA",
        phone_lengths: &[5, 6, 7, 8, 9, 10],
        prefix: 27,
    },
    Country {
        name: "Zambia",
        code: "ZM",
        phone_lengths: &[9],
        prefix: 260,
    },
    Country {
        // Fixed lines dialled from abroad are 7 or 9 digits; the 3- to 6-digit
        // forms are local-only dialling from the same exchange and never an NSN.
        // Mobile is 9, and VoIP 10.
        name: "Zimbabwe",
        code: "ZW",
        phone_lengths: &[7, 9, 10],
        prefix: 263,
    },
    // Non-geographic entities. The ITU assigns these calling codes to global
    // services, not to a country; they are kept together here, out of
    // alphabetical order, for readability.
    Country {
        name: "International Freephone Service",
        code: "001",
        phone_lengths: &[8],
        prefix: 800,
    },
    Country {
        name: "International Shared Cost Service",
        code: "001",
        phone_lengths: &[8],
        prefix: 808,
    },
    Country {
        name: "Inmarsat",
        code: "001",
        phone_lengths: &[9, 12],
        prefix: 870,
    },
    Country {
        name: "Universal Personal Telecommunications",
        code: "001",
        phone_lengths: &[12],
        prefix: 878,
    },
    Country {
        name: "Global Mobile Satellite System",
        code: "001",
        phone_lengths: &[9, 10],
        prefix: 881,
    },
    Country {
        name: "International Networks",
        code: "001",
        phone_lengths: &[7, 8, 9, 10, 11, 12],
        prefix: 882,
    },
    Country {
        name: "International Networks",
        code: "001",
        phone_lengths: &[8, 9, 10, 11, 12],
        prefix: 883,
    },
    Country {
        name: "Telecommunications for Disaster Relief",
        code: "001",
        phone_lengths: &[11],
        prefix: 888,
    },
    Country {
        name: "International Premium Rate Service",
        code: "001",
        phone_lengths: &[9],
        prefix: 979,
    },
];

/// A set of national significant number lengths.
pub(crate) type Lengths = &'static [u8];

/// Plans in which a national significant number may itself begin with `0`.
///
/// Everywhere else a `0` after the calling code can only be a national trunk
/// prefix written by mistake (`+44 (0)20 7946 0958`), and is dropped. Here it
/// may be part of the number: Italian fixed lines (`+39 06 …`), Congolese and
/// Ivorian mobiles, the `00800` freephone ranges of Panama or Qatar, Japanese
/// `0037` freephone numbers.
///
/// `(code, calling code, lengths of the NSNs that begin with 0, lengths of
/// every other NSN, whether some NSN begins with 00)`. The two sets of lengths
/// together are exactly that entry's `phone_lengths`, which a compile-time
/// check in `src/lib.rs` enforces. The last column matters because `00` is
/// otherwise an international access code: only in these plans can a number
/// written `00…` also be a national one.
pub(crate) const ZERO_LED: &[(&str, u32, Lengths, Lengths, bool)] = &[
    ("AC", 247, &[6], &[5, 6], false),
    ("BF", 226, &[8], &[8], false),
    ("BG", 359, &[12], &[6, 7, 8, 9], true),
    ("BJ", 229, &[10], &[8], false),
    ("BW", 267, &[10], &[7, 8, 10], false),
    ("BZ", 501, &[11], &[7], false),
    ("CG", 242, &[9], &[9], false),
    ("CI", 225, &[10], &[10], false),
    ("FJ", 679, &[11], &[7], false),
    ("GA", 241, &[8], &[7, 8], false),
    ("ID", 62, &[11, 12, 13], &[7, 8, 9, 10, 11, 12], true),
    ("IN", 91, &[13], &[8, 9, 10, 11, 12, 13], true),
    (
        "IT",
        39,
        &[6, 7, 8, 9, 10, 11, 12],
        &[6, 8, 9, 10, 11, 12],
        false,
    ),
    ("JP", 81, &[8, 9, 10, 11, 12, 13], &[9, 10], true),
    ("KR", 82, &[11, 12, 13], &[5, 6, 8, 9, 10, 11], true),
    ("MO", 853, &[7], &[8], false),
    ("NC", 687, &[6], &[6], false),
    ("NE", 227, &[8], &[8], false),
    ("NO", 47, &[5], &[8], false),
    ("PA", 507, &[11], &[7, 8, 10], true),
    ("QA", 974, &[11], &[7, 8, 9], true),
    ("RW", 250, &[8], &[9], false),
    ("SJ", 47, &[5], &[8], false),
    ("SM", 378, &[10], &[8], false),
    ("SZ", 268, &[8], &[8, 9], false),
    ("TH", 66, &[13], &[8, 9, 10], true),
    ("TJ", 992, &[9], &[9], true),
    ("TO", 676, &[7], &[5, 7], false),
    ("UY", 598, &[6, 7, 8, 9, 10, 11, 12], &[4, 5, 7, 8], true),
    (
        "VA",
        39,
        &[6, 7, 8, 9, 10, 11],
        &[6, 8, 9, 10, 11, 12],
        false,
    ),
    ("001", 800, &[8], &[8], true),
    ("001", 881, &[9], &[9, 10], true),
    ("001", 888, &[11], &[11], true),
];

/// National (trunk) prefix by ISO code, for plans that have one: the digits
/// dialled before a national number from inside the country. Calling code 1
/// is handled by the NANP code, and a plan absent here has no trunk prefix.
///
/// Most plans use `0`; Russia, Kazakhstan, Belarus and Turkmenistan still use
/// `8`, Hungary `06`, and the Marshall Islands `1`.
pub(crate) const NATIONAL_PREFIX: &[(&str, &str)] = &[
    ("AE", "0"),
    ("AF", "0"),
    ("AL", "0"),
    ("AM", "0"),
    ("AR", "0"),
    ("AT", "0"),
    ("AU", "0"),
    ("AX", "0"),
    ("AZ", "0"),
    ("BA", "0"),
    ("BD", "0"),
    ("BE", "0"),
    ("BG", "0"),
    ("BL", "0"),
    ("BO", "0"),
    ("BR", "0"),
    ("BY", "8"),
    ("CC", "0"),
    ("CD", "0"),
    ("CH", "0"),
    ("CN", "0"),
    ("CO", "0"),
    ("CU", "0"),
    ("CX", "0"),
    ("DE", "0"),
    ("DZ", "0"),
    ("EC", "0"),
    ("EG", "0"),
    ("EH", "0"),
    ("ER", "0"),
    ("ET", "0"),
    ("FI", "0"),
    ("FR", "0"),
    ("GB", "0"),
    ("GE", "0"),
    ("GF", "0"),
    ("GG", "0"),
    ("GH", "0"),
    ("GP", "0"),
    ("HR", "0"),
    ("HU", "06"),
    ("ID", "0"),
    ("IE", "0"),
    ("IL", "0"),
    ("IM", "0"),
    ("IN", "0"),
    ("IQ", "0"),
    ("IR", "0"),
    ("JE", "0"),
    ("JO", "0"),
    ("JP", "0"),
    ("KE", "0"),
    ("KG", "0"),
    ("KH", "0"),
    ("KI", "0"),
    ("KP", "0"),
    ("KR", "0"),
    ("KZ", "8"),
    ("LA", "0"),
    ("LB", "0"),
    ("LI", "0"),
    ("LK", "0"),
    ("LR", "0"),
    ("LT", "0"),
    ("LY", "0"),
    ("MA", "0"),
    ("MC", "0"),
    ("MD", "0"),
    ("ME", "0"),
    ("MF", "0"),
    ("MG", "0"),
    ("MH", "1"),
    ("MK", "0"),
    ("MM", "0"),
    ("MN", "0"),
    ("MQ", "0"),
    ("MW", "0"),
    ("MY", "0"),
    ("NA", "0"),
    ("NG", "0"),
    ("NL", "0"),
    ("NP", "0"),
    ("NZ", "0"),
    ("PE", "0"),
    ("PH", "0"),
    ("PK", "0"),
    ("PM", "0"),
    ("PS", "0"),
    ("PY", "0"),
    ("RE", "0"),
    ("RO", "0"),
    ("RS", "0"),
    ("RU", "8"),
    ("RW", "0"),
    ("SA", "0"),
    ("SD", "0"),
    ("SE", "0"),
    ("SI", "0"),
    ("SK", "0"),
    ("SL", "0"),
    ("SO", "0"),
    ("SS", "0"),
    ("SY", "0"),
    ("TH", "0"),
    ("TM", "8"),
    ("TR", "0"),
    ("TW", "0"),
    ("TZ", "0"),
    ("UA", "0"),
    ("UG", "0"),
    ("UY", "0"),
    ("VE", "0"),
    ("VN", "0"),
    ("XK", "0"),
    ("YE", "0"),
    ("YT", "0"),
    ("ZA", "0"),
    ("ZM", "0"),
    ("ZW", "0"),
];

/// Further trunk prefixes accepted when *parsing* a national number, beyond
/// the one in [`NATIONAL_PREFIX`]: Belarus also accepts the `80` and `0` forms,
/// and Lithuania the `8` it used before moving to `0`.
pub(crate) const EXTRA_NATIONAL_PREFIXES: &[(&str, &str)] =
    &[("BY", "80"), ("BY", "0"), ("LT", "8")];

/// `(national number prefix, what is written in front of such numbers)`.
pub(crate) type FormatExceptions = &'static [(&'static str, &'static str)];

/// What is written in front of a national number's first group, by calling
/// code: `(calling code, usual prefix, exceptions)`, sorted by calling code.
///
/// Usually that is the trunk prefix glued to the area code (`020 7946 0958`),
/// but not always: Chinese mobiles and the Australian `1800` freephone range
/// are dialled without it, and Russia writes its `8` as a token of its own
/// (`8 916 123-45-67`). An exception is `(national number prefix, written
/// prefix)`, and the longest matching one wins. A calling code absent here
/// writes nothing in front of its numbers.
///
/// Derived from the national format rules in libphonenumber's metadata. Where
/// those write a trunk `0` as a token of its own to suit a grouping of their
/// own (`0 800 …` in France), it is glued to the number here, since this crate
/// groups numbers its own way.
#[rustfmt::skip]
pub(crate) const NATIONAL_FORMAT_PREFIX: &[(u32, &str, FormatExceptions)] = &[
    (7, "8 ", &[]),
    (20, "0", &[]),
    (27, "0", &[]),
    (31, "0", &[("1", ""), ("10", "0"), ("11", "0"), ("13", "0"), ("15", "0"), ("16", "0"), ("17", "0"), ("18", "0")]),
    (32, "0", &[]),
    (33, "0", &[]),
    (36, "06 ", &[]),
    (40, "0", &[]),
    (41, "0", &[]),
    (43, "0", &[]),
    (44, "0", &[]),
    (46, "0", &[]),
    (49, "0", &[]),
    (51, "0", &[("9", "")]),
    (53, "0", &[]),
    (54, "0", &[]),
    (55, "", &[("303", "0"), ("5", "0"), ("51", ""), ("53", ""), ("54", ""), ("55", ""), ("8", "0"), ("81", ""), ("82", ""), ("83", ""), ("84", ""), ("85", ""), ("86", ""), ("88", ""), ("9", "0"), ("91", ""), ("92", ""), ("93", ""), ("94", ""), ("95", ""), ("96", ""), ("97", ""), ("98", ""), ("99", "")]),
    (57, "", &[("1", "0")]),
    (58, "0", &[]),
    (60, "0", &[("13", ""), ("131", "0"), ("132", "0"), ("134", "0"), ("136", "0"), ("16", ""), ("161", "0"), ("163", "0"), ("167", "0"), ("168", "0"), ("17", ""), ("171", "0"), ("173", "0"), ("177", "0"), ("179", "0"), ("18", ""), ("181", "0"), ("182", "0"), ("183", "0"), ("184", "0"), ("185", "0"), ("186", "0"), ("187", "0"), ("188", "0"), ("189", "0")]),
    (61, "0", &[("1", ""), ("14", "0"), ("16", "0")]),
    (62, "0", &[("0", ""), ("1", ""), ("17", "0")]),
    (63, "0", &[("1", "")]),
    (64, "0", &[]),
    (66, "0", &[("1", ""), ("10", "0"), ("14", "0"), ("16", "0"), ("181", "0"), ("182", "0"), ("184", "0"), ("185", "0"), ("186", "0"), ("187", "0"), ("188", "0"), ("189", "0"), ("192", "0"), ("194", "0"), ("198", "0"), ("199", "0")]),
    (81, "0", &[("0", "")]),
    (82, "0", &[("0", ""), ("16", ""), ("161", "0"), ("162", "0"), ("169", "0"), ("18", ""), ("181", "0"), ("182", "0"), ("184", "0"), ("186", "0"), ("188", "0")]),
    (84, "0", &[("1", ""), ("121", "0"), ("14", "0"), ("16", "0"), ("199", "0")]),
    (86, "", &[("10", "0"), ("2", "0"), ("3", "0"), ("41", "0"), ("43", "0"), ("45", "0"), ("46", "0"), ("47", "0"), ("5", "0"), ("6", "0"), ("7", "0"), ("807", "0"), ("81", "0"), ("82", "0"), ("85", "0"), ("87", "0"), ("88", "0"), ("89", "0"), ("90", "0"), ("93", "0"), ("951", "0"), ("953", "0"), ("955", "0"), ("97", "0"), ("99", "0")]),
    (90, "0", &[("4", ""), ("41", "0"), ("42", "0"), ("43", "0"), ("442", "0"), ("446", "0"), ("45", "0"), ("46", "0"), ("47", "0"), ("48", "0")]),
    (91, "", &[("11", "0"), ("12", "0"), ("15", "0"), ("16", "0"), ("17", "0"), ("189", "0"), ("2", "0"), ("3", "0"), ("4", "0"), ("5", "0"), ("6", "0"), ("7", "0"), ("8", "0"), ("9", "0")]),
    (92, "0", &[("1", "")]),
    (93, "0", &[]),
    (94, "0", &[]),
    (95, "0", &[]),
    (98, "0", &[]),
    (211, "0", &[]),
    (212, "0", &[]),
    (213, "0", &[]),
    (218, "0", &[]),
    (231, "0", &[]),
    (232, "0", &[]),
    (233, "0", &[]),
    (234, "0", &[]),
    (241, "0", &[("0", "")]),
    (243, "0", &[]),
    (249, "0", &[]),
    (250, "0", &[("0", ""), ("2", "")]),
    (251, "0", &[]),
    (254, "0", &[]),
    (255, "0", &[]),
    (256, "0", &[]),
    (260, "0", &[]),
    (261, "0", &[]),
    (262, "0", &[]),
    (263, "0", &[]),
    (264, "0", &[]),
    (265, "0", &[]),
    (291, "0", &[]),
    (353, "0", &[("1", ""), ("10", "0"), ("11", "0"), ("12", "0"), ("13", "0"), ("14", "0"), ("150", "0"), ("16", "0"), ("17", "0"), ("181", "0"), ("182", "0"), ("183", "0"), ("184", "0"), ("187", "0"), ("188", "0"), ("19", "0")]),
    (355, "0", &[]),
    (358, "0", &[]),
    (359, "0", &[]),
    (370, "0", &[]),
    (373, "0", &[]),
    (374, "0", &[]),
    (375, "8 ", &[("1", "8 0"), ("2", "8 0"), ("3", "8 0"), ("4", "8 0")]),
    (377, "", &[("4", "0"), ("6", "0"), ("7", "0")]),
    (380, "0", &[]),
    (381, "0", &[]),
    (382, "0", &[]),
    (383, "0", &[]),
    (385, "0", &[]),
    (386, "0", &[]),
    (387, "0", &[]),
    (389, "0", &[]),
    (421, "0", &[]),
    (508, "0", &[("7", ""), ("705", "0"), ("71", "0"), ("72", "0"), ("73", "0"), ("74", "0"), ("78", "0"), ("79", "0")]),
    (590, "0", &[]),
    (593, "0", &[("1", "")]),
    (594, "0", &[]),
    (595, "0", &[("8", ""), ("80", "0"), ("81", "0"), ("82", "0"), ("83", "0"), ("85", "0"), ("86", "0"), ("98", ""), ("981", "0"), ("982", "0"), ("983", "0"), ("984", "0"), ("985", "0"), ("986", "0"), ("987", "0")]),
    (596, "0", &[]),
    (598, "", &[("40", "0"), ("8", "0"), ("9", "0")]),
    (850, "0", &[]),
    (855, "0", &[("1", ""), ("10", "0"), ("11", "0"), ("12", "0"), ("13", "0"), ("14", "0"), ("15", "0"), ("16", "0"), ("17", "0"), ("181", "0"), ("182", "0"), ("184", "0"), ("185", "0"), ("186", "0"), ("187", "0"), ("189", "0"), ("193", "0"), ("198", "0"), ("199", "0")]),
    (856, "0", &[]),
    (880, "0", &[]),
    (886, "0", &[]),
    (961, "", &[("1", "0"), ("3", "0"), ("4", "0"), ("5", "0"), ("6", "0"), ("7", "0"), ("70", ""), ("71", ""), ("760", ""), ("761", ""), ("763", ""), ("764", ""), ("765", ""), ("766", ""), ("767", ""), ("768", ""), ("769", ""), ("787", ""), ("788", ""), ("789", ""), ("791", ""), ("792", ""), ("793", ""), ("82", "0"), ("83", "0"), ("84", "0"), ("85", "0"), ("86", "0"), ("87", "0"), ("88", "0"), ("89", "0"), ("92", "0"), ("93", "0"), ("94", "0"), ("97", "0")]),
    (962, "0", &[]),
    (963, "0", &[]),
    (964, "0", &[]),
    (966, "", &[("1", "0"), ("5", "0")]),
    (967, "0", &[]),
    (970, "0", &[("1", "")]),
    (971, "", &[("2", "0"), ("3", "0"), ("42", "0"), ("43", "0"), ("44", "0"), ("45", "0"), ("46", "0"), ("47", "0"), ("48", "0"), ("5", "0"), ("62", "0"), ("63", "0"), ("64", "0"), ("65", "0"), ("66", "0"), ("67", "0"), ("68", "0"), ("73", "0"), ("74", "0"), ("75", "0"), ("76", "0"), ("77", "0"), ("78", "0"), ("92", "0"), ("93", "0"), ("94", "0"), ("95", "0"), ("96", "0"), ("97", "0"), ("98", "0")]),
    (972, "", &[("2", "0"), ("3", "0"), ("4", "0"), ("5", "0"), ("7", "0"), ("8", "0"), ("9", "0")]),
    (976, "", &[("1", "0"), ("2", "0")]),
    (977, "", &[("10", "0"), ("11", "0"), ("12", "0"), ("13", "0"), ("14", "0"), ("15", "0"), ("161", "0"), ("162", "0"), ("164", "0"), ("168", "0"), ("2", "0"), ("3", "0"), ("4", "0"), ("5", "0"), ("6", "0"), ("7", "0"), ("8", "0"), ("91", "0"), ("92", "0"), ("93", "0"), ("94", "0"), ("95", "0"), ("965", "0"), ("966", "0"), ("99", "0")]),
    (993, "8 ", &[]),
    (994, "0", &[]),
    (995, "0", &[("5", ""), ("79", "")]),
    (996, "0", &[]),
];

/// International access codes that a plan uses besides `00`, `0011` and
/// `011`, which are recognised for every country: `(code, early, late)`.
/// Only used to read a number written as it would be dialled *from* that
/// country — `8 10 44 20 7946 0958` from Russia, `010 44 …` from Japan.
///
/// An *early* code cannot be mistaken for a national number, because no
/// number of the plan is dialled that way — Russia's `8 10` is its trunk `8`
/// followed by `10`, and no Russian number continues with `10` — so it is
/// tried first. A *late* code can (Thailand's `001` is also the start of its
/// `001 800` freephone numbers), so it is only tried when the digits do not
/// read as a national number. Each list is longest first, so that no code is
/// shadowed by a shorter one it starts with.
#[rustfmt::skip]
pub(crate) const INTERNATIONAL_PREFIXES: &[(&str, &[&str], &[&str])] = &[
    ("AU", &["0014", "0015", "0016", "0018", "0019"], &[]),
    ("BO", &["0010", "0012", "0013", "0014", "0015", "0016", "0017", "0018", "0019"], &[]),
    ("BR", &["0012", "0014", "0015", "0021", "0022", "0023", "0025", "0031", "0041", "0043", "0055", "0065", "0099"], &[]),
    ("BY", &["810"], &[]),
    ("CC", &["0014", "0015", "0016", "0018", "0019"], &[]),
    ("CO", &["00414", "00444", "00456", "005", "007", "009"], &[]),
    ("CU", &["119"], &[]),
    ("CV", &["0"], &[]),
    ("CX", &["0014", "0015", "0016", "0018", "0019"], &[]),
    ("FJ", &["052"], &[]),
    ("GY", &["001"], &[]),
    ("HK", &["0030", "0050", "0059", "001", "002", "006", "007", "008", "009"], &[]),
    ("ID", &["008", "009"], &[]),
    ("IL", &["0105", "013", "014", "016"], &["012", "015", "017", "018", "019"]),
    ("JP", &["010"], &[]),
    ("KE", &["000"], &[]),
    ("KH", &["001", "004", "005", "006", "007", "008", "009"], &[]),
    ("KP", &["99"], &[]),
    ("KR", &["00345", "00365", "00391", "00700", "00727", "00755", "00761", "00762", "00766", "0073", "001", "002", "005", "006", "008", "009"], &[]),
    ("KZ", &["810"], &[]),
    ("MN", &["001"], &[]),
    ("MU", &["020", "030", "033", "040", "050", "060", "070"], &[]),
    ("MV", &["019"], &[]),
    ("MX", &["09"], &[]),
    ("NG", &["009"], &[]),
    ("NZ", &["0161"], &[]),
    ("PW", &["012"], &[]),
    ("RU", &["810"], &[]),
    ("SB", &["01"], &[]),
    ("SC", &["010", "01", "02"], &[]),
    ("SG", &["000", "001", "002", "003", "004", "005", "006", "007", "008", "009", "010", "012", "013", "014", "015", "016", "017", "018", "019", "020", "021", "022", "023", "024", "025", "026", "027", "028", "029", "030", "031", "032", "033", "034", "035", "036", "037", "038", "039"], &[]),
    ("TD", &["16"], &[]),
    ("TH", &["002", "003", "004", "005", "006", "007", "008", "009"], &["001"]),
    ("TJ", &["810"], &[]),
    ("TM", &["810"], &[]),
    ("TW", &["002", "005", "006", "007", "009", "019"], &[]),
    ("TZ", &["000", "005", "006"], &[]),
    ("UG", &["000", "005", "007"], &[]),
    ("WS", &["0"], &[]),
];

/// National number prefixes that identify a secondary territory on a shared
/// calling code. A number whose calling code several entries share belongs to
/// the main region (see `PRIMARY_FOR_PREFIX` in `src/lib.rs`) unless it starts
/// with one of these.
///
/// This is the one place the crate looks at prefixes, and it only chooses
/// *which* territory a number is reported as — it never makes a number
/// invalid. A number in a block opened after this table was written is still
/// accepted, and reported as the main region.
pub(crate) const TERRITORY_PREFIXES: &[(&str, &[&str])] = &[
    ("KZ", &["7"]),
    ("VA", &["06698"]),
    (
        "GG",
        &["980", "981", "1481", "7781", "7839", "79111", "79117"],
    ),
    ("IM", &["1624", "7524", "7624", "74576"]),
    (
        "JE",
        &[
            "1534", "7509", "7797", "7829", "7937", "77003", "77007", "77008",
        ],
    ),
    ("SJ", &["79"]),
    (
        "CC",
        &[
            "89162", "851002", "851031", "851060", "851089", "851118", "851176", "851223", "891703",
        ],
    ),
    (
        "CX",
        &[
            "89164", "851001", "851030", "851059", "851088", "851117", "851146", "851175",
            "851222", "851235", "891958",
        ],
    ),
    ("YT", &["63", "269", "2689", "7093", "9398", "9478", "9769"]),
    ("TA", &["8"]),
    ("AX", &["18"]),
    ("BL", &["59027", "59029", "59051", "59052", "59087"]),
    (
        "MF",
        &[
            "59000", "59007", "59009", "59013", "59030", "59043", "59050", "59056", "59058",
            "59077", "59079",
        ],
    ),
    ("BQ", &["3", "4", "7"]),
];
