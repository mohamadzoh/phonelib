use crate::definitions;

use definitions::Country;

/// Country calling-code table.
///
/// # Field semantics
///
/// * `prefix` — the E.164 country calling code. For NANP territories this is the
///   4-digit `1` + area-code form (e.g. `1268` = Antigua and Barbuda), because the
///   area code is what identifies the territory.
/// * `phone_lengths` — the valid lengths of the national significant number, i.e.
///   the digits that follow `prefix`. Ascending, no duplicates.
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
/// NSN length alone (only leading digits distinguish them). Those entries are kept
/// so that lookup-by-ISO-code works, but the main region wins prefix resolution:
/// `1` US/CA, `7` RU/KZ, `39` IT/VA, `44` GB/GG/IM/JE, `47` NO/SJ, `61` AU/CC/CX,
/// `212` MA/EH, `262` RE/YT, `358` FI/AX, `500` FK/GS, `590` GP/BL/MF, `599` CW/BQ.
pub const COUNTRIES: [Country; 249] = [
    Country {
        name: "Andorra",
        code: "AD",
        phone_lengths: &[6],
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
        // 8 = landline (e.g. +971 4 123 4567), 9 = mobile (+971 50 123 4567).
        name: "United Arab Emirates",
        code: "AE",
        phone_lengths: &[8, 9],
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
        // 8 = fixed line (04 222 2222 -> NSN 42222222), 9 = mobile.
        name: "Albania",
        code: "AL",
        phone_lengths: &[8, 9],
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
        // Austria's plan tops out at 13 NSN digits; 2 + 13 = 15, the E.164 ceiling.
        name: "Austria",
        code: "AT",
        phone_lengths: &[5, 6, 7, 8, 9, 10, 11, 12, 13],
        prefix: 43,
    },
    Country {
        name: "Australia",
        code: "AU",
        phone_lengths: &[9, 10],
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
        phone_lengths: &[8],
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
        phone_lengths: &[7, 8, 9],
        prefix: 359,
    },
    Country {
        name: "Bahrain",
        code: "BH",
        phone_lengths: &[8],
        prefix: 973,
    },
    Country {
        // 8 = fixed line (02 298 2000 -> NSN 22982000), 9 = mobile (59x ...).
        name: "Palestine",
        code: "PS",
        phone_lengths: &[8, 9],
        prefix: 970,
    },
    Country {
        name: "Israel",
        code: "IL",
        phone_lengths: &[8, 9],
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
        // 30 November 2024, taking the NSN from 8 to 10 digits; the 8-digit
        // plan was withdrawn a month later. NOTE: the leading zero is part of
        // the number here, not a trunk prefix — the ITU communication is
        // explicit that "it is mandatory to dial the leading zero digit `0`
        // when calling from abroad". `BJ` therefore has to be listed in
        // `LEADING_ZERO_SIGNIFICANT` in src/lib.rs for these to resolve.
        name: "Benin",
        code: "BJ",
        phone_lengths: &[10],
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
        phone_lengths: &[10, 11],
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
        phone_lengths: &[7, 8],
        prefix: 267,
    },
    Country {
        name: "Belarus",
        code: "BY",
        phone_lengths: &[9],
        prefix: 375,
    },
    Country {
        name: "Belize",
        code: "BZ",
        phone_lengths: &[7],
        prefix: 501,
    },
    Country {
        // Cocos (Keeling) Islands sit inside the Australian plan (+61 8 9162 xxxx),
        // so the NSN is always 9 digits, exactly like a mainland Australian number.
        name: "Cocos (Keeling) Islands",
        code: "CC",
        phone_lengths: &[9],
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
        phone_lengths: &[9],
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
        phone_lengths: &[9],
        prefix: 56,
    },
    Country {
        name: "Cameroon",
        code: "CM",
        phone_lengths: &[9],
        prefix: 237,
    },
    Country {
        // Mainland China is always dialled with an area code internationally, so
        // the NSN is 9-12 digits (mobile is 11). The 7- and 8-digit forms are
        // subscriber-only and are not valid E.164 numbers.
        name: "China",
        code: "CN",
        phone_lengths: &[9, 10, 11, 12],
        prefix: 86,
    },
    Country {
        name: "Colombia",
        code: "CO",
        phone_lengths: &[10],
        prefix: 57,
    },
    Country {
        name: "Costa Rica",
        code: "CR",
        phone_lengths: &[8],
        prefix: 506,
    },
    Country {
        name: "Cuba",
        code: "CU",
        phone_lengths: &[8],
        prefix: 53,
    },
    Country {
        name: "Cape Verde",
        code: "CV",
        phone_lengths: &[7],
        prefix: 238,
    },
    Country {
        // Curacao is the main region for +599 and its numbers are always 8 digits
        // (9 xxx xxxx), so it is listed ahead of BQ below.
        name: "Curacao",
        code: "CW",
        phone_lengths: &[8],
        prefix: 599,
    },
    Country {
        // Bonaire, Sint Eustatius and Saba stayed on +599 after the dissolution
        // of the Netherlands Antilles. Every number here is 7 digits — Bonaire
        // 7xx xxxx, Saba/Sint Eustatius 4xx xxxx, mobile 3xx xxxx — so the
        // previous `8` was a length the territory has no numbers of. Only
        // leading digits separate BQ from Curacao, so the split that actually
        // works is by length: 8-digit +599 numbers resolve to CW (the main
        // region) and BQ owns the 7-digit range.
        name: "Bonaire, Sint Eustatius and Saba",
        code: "BQ",
        phone_lengths: &[7],
        prefix: 599,
    },
    Country {
        // Christmas Island is served from the Australian plan (+61 8 9164 xxxx):
        // a 9-digit NSN, same as the mainland.
        name: "Christmas Island",
        code: "CX",
        phone_lengths: &[9],
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
        phone_lengths: &[9],
        prefix: 420,
    },
    Country {
        name: "Germany",
        code: "DE",
        phone_lengths: &[6, 7, 8, 9, 10, 11],
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
        name: "Ecuador",
        // 8 = fixed line (02 299 3200 -> NSN 22993200), 9 = mobile.
        code: "EC",
        phone_lengths: &[8, 9],
        prefix: 593,
    },
    Country {
        name: "Estonia",
        // 7 = fixed line (631 3000) and short mobile, 8 = mobile (5123 4567).
        code: "EE",
        phone_lengths: &[7, 8],
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
        // ranges. No NSN length can separate Aland from mainland Finland, so this
        // entry exists for lookup-by-ISO-code only and is deliberately listed
        // *after* FI so that every +358 number resolves to Finland. The lengths
        // are Aland's own: the 018 fixed-line range is `18[1-8]` plus 3 to 6 more
        // digits (NSN 6-9), and Aland subscribers carry Finnish mobiles at 6-10.
        // A previous revision narrowed this to a flat 7, which is narrower than
        // the 018 range itself and is not a length rule Aland uses.
        name: "Åland Islands",
        code: "AX",
        phone_lengths: &[6, 7, 8, 9, 10],
        prefix: 358,
    },
    Country {
        name: "Fiji",
        code: "FJ",
        phone_lengths: &[7],
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
        // Exactly 8, per ARCEP Gabon: the 2019-2024 migration took the *national*
        // format to nine digits, `0XX XX XX XX`, but the leading `0` is a trunk
        // prefix that is dropped internationally — "for international calls to
        // Gabon, users will have to dial `+241` followed by the eight (8) digits
        // after the first digit `0`". So the NSN is 8 (min and max), beginning
        // 11 (fixed), 62/65/66/74/77 (mobile), 8 or 9. The retired 7-digit plan
        // and the `9` that used to be listed here are both unreachable.
        name: "Gabon",
        code: "GA",
        phone_lengths: &[8],
        prefix: 241,
    },
    Country {
        // 10 = the common case (20 xxxx xxxx, 7xxx xxxxxx); 9 covers the four-digit
        // area codes with five-digit subscriber numbers (e.g. +44 1697 73xxx) and
        // the 0800/0500 ranges; 7 is the short freephone block, of which the live
        // example is 0800 1111 (Childline). GB is the main region for +44 and is
        // listed ahead of GG/IM/JE, which share the code but cannot be told apart
        // by length.
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
        // Guernsey is +44 1481 xxxxxx / 7781 xxxxxx: a 10-digit NSN like the rest
        // of the UK plan, not 6.
        name: "Guernsey",
        code: "GG",
        phone_lengths: &[10],
        prefix: 44,
    },
    Country {
        name: "Ghana",
        code: "GH",
        phone_lengths: &[9],
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
        phone_lengths: &[7],
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
        // Guadeloupe is the main region for +590 and must precede BL and MF, which
        // share the code.
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
        phone_lengths: &[10],
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
        phone_lengths: &[8],
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
        phone_lengths: &[9],
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
        phone_lengths: &[8],
        prefix: 852,
    },
    Country {
        name: "Honduras",
        code: "HN",
        phone_lengths: &[8],
        prefix: 504,
    },
    Country {
        name: "Croatia",
        // 8 = Zagreb and the 2-digit area codes (01 4569 964 -> NSN 14569964),
        // 9 = the longer fixed subscriber ranges and mobile.
        code: "HR",
        phone_lengths: &[8, 9],
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
        phone_lengths: &[9, 10, 11, 12],
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
        phone_lengths: &[10],
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
        phone_lengths: &[10],
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
        // the lengths here include it. Main region for +39; VA follows.
        name: "Italy",
        code: "IT",
        phone_lengths: &[6, 7, 8, 9, 10, 11],
        prefix: 39,
    },
    Country {
        // +379 is reserved for the Holy See but has never been brought into
        // service: Vatican numbers live in the Italian plan as 06 698 xxxxx. The
        // old 379 entry accepted numbers that cannot be dialled anywhere, so this
        // entry now carries Italy's code and is listed after IT, which wins +39.
        name: "Holy See (Vatican City State)",
        code: "VA",
        phone_lengths: &[10],
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
        // (+81 90 1234 5678), 11 = the longer 0120/0570-style service numbers.
        name: "Japan",
        code: "JP",
        phone_lengths: &[9, 10, 11],
        prefix: 81,
    },
    Country {
        name: "Kenya",
        // Fixed line is variable at 7-9 (20 201 2345 and shorter county ranges),
        // mobile is 9.
        code: "KE",
        phone_lengths: &[7, 8, 9],
        prefix: 254,
    },
    Country {
        name: "Kyrgyzstan",
        code: "KG",
        phone_lengths: &[9],
        prefix: 996,
    },
    Country {
        name: "Cambodia",
        code: "KH",
        phone_lengths: &[8, 9],
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
        name: "Korea, Republic of",
        // 8-10 = fixed line and mobile, 11 = personal/toll-free ranges. No Korean
        // number type has a 7-digit NSN.
        code: "KR",
        phone_lengths: &[8, 9, 10, 11],
        prefix: 82,
    },
    Country {
        name: "Kuwait",
        code: "KW",
        phone_lengths: &[8],
        prefix: 965,
    },
    Country {
        name: "Cayman Islands",
        code: "KY",
        phone_lengths: &[7],
        prefix: 1345,
    },
    Country {
        // Russia and Kazakhstan share +7 and both use a 10-digit NSN, so length
        // cannot separate them. Russia wins prefix resolution because
        // `PRIMARY_FOR_PREFIX` in src/lib.rs names it for +7 — not because of
        // where it sits in this array. It is kept immediately before KZ only so
        // that the two entries read together.
        name: "Russian Federation",
        code: "RU",
        phone_lengths: &[10],
        prefix: 7,
    },
    Country {
        name: "Kazakhstan",
        code: "KZ",
        phone_lengths: &[10],
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
        phone_lengths: &[8, 9],
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
        phone_lengths: &[4, 5, 6, 7, 8, 9],
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
        // Morocco is the main region for +212 and must precede EH, which shares it.
        name: "Morocco",
        code: "MA",
        phone_lengths: &[9],
        prefix: 212,
    },
    Country {
        // Western Sahara has no plan of its own; it is served entirely from
        // Morocco's +212 ranges (5288/5289), so it is listed immediately after MA.
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
        phone_lengths: &[8],
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
        phone_lengths: &[8],
        prefix: 976,
    },
    Country {
        name: "Macao",
        code: "MO",
        phone_lengths: &[8],
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
        name: "Mauritius",
        // 7 = fixed line (208 1234), 8 = the longer fixed range and mobile.
        code: "MU",
        phone_lengths: &[7, 8],
        prefix: 230,
    },
    Country {
        name: "Maldives",
        code: "MV",
        phone_lengths: &[7],
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
        name: "Nigeria",
        // Uniformly 10. The NCC moved fixed lines from 8 to 10 digits on
        // 1 January 2024 by prefixing `02` (old 01 463 0643 -> 020 1463 0643,
        // NSN 2014630643); mobiles were already 10. 7, 8 and 9 are all retired.
        code: "NG",
        phone_lengths: &[10],
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
        phone_lengths: &[9],
        prefix: 31,
    },
    Country {
        name: "Norway",
        code: "NO",
        phone_lengths: &[8],
        prefix: 47,
    },
    Country {
        name: "Nepal",
        // 8 = fixed line (1 421 1000 in Kathmandu), 10 = mobile (98x xxx xxxx).
        code: "NP",
        phone_lengths: &[8, 10],
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
        phone_lengths: &[8, 9, 10],
        prefix: 64,
    },
    Country {
        name: "Oman",
        code: "OM",
        phone_lengths: &[8],
        prefix: 968,
    },
    Country {
        name: "Panama",
        // 7 = fixed line (all Panamanian landlines), 8 = mobile.
        code: "PA",
        phone_lengths: &[7, 8],
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
        phone_lengths: &[8],
        prefix: 689,
    },
    Country {
        name: "Papua New Guinea",
        code: "PG",
        phone_lengths: &[7, 8],
        prefix: 675,
    },
    Country {
        // 10 = mobile (9xx xxx xxxx); 8-9 = landline, including the Metro Manila
        // area code 2 with its post-2019 eight-digit subscriber numbers.
        name: "Philippines",
        code: "PH",
        phone_lengths: &[8, 9, 10],
        prefix: 63,
    },
    Country {
        name: "Pakistan",
        // 9-10 = fixed line (51 111 1234 is 9), 10 = mobile.
        code: "PK",
        phone_lengths: &[9, 10],
        prefix: 92,
    },
    Country {
        name: "Poland",
        code: "PL",
        phone_lengths: &[9],
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
        phone_lengths: &[9],
        prefix: 595,
    },
    Country {
        name: "Qatar",
        code: "QA",
        phone_lengths: &[8],
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
        // Romania is a uniform 9-digit plan (21 xxx xxxx fixed, 7xx xxx xxx mobile).
        name: "Romania",
        code: "RO",
        phone_lengths: &[9],
        prefix: 40,
    },
    Country {
        name: "Serbia",
        // Serbian numbers are genuinely variable-length: fixed line runs 7-12
        // (Belgrade 11 + 6 or 7, e.g. 11 234567 -> NSN 11234567) and mobile
        // 8-10. This is a wide set, but narrowing it to 9 rejected every
        // 8-digit Serbian number.
        code: "RS",
        phone_lengths: &[7, 8, 9, 10, 11, 12],
        prefix: 381,
    },
    Country {
        name: "Rwanda",
        code: "RW",
        phone_lengths: &[9],
        prefix: 250,
    },
    Country {
        name: "Saudi Arabia",
        code: "SA",
        phone_lengths: &[9],
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
        phone_lengths: &[7, 8, 9, 10],
        prefix: 46,
    },
    Country {
        name: "Singapore",
        code: "SG",
        phone_lengths: &[8],
        prefix: 65,
    },
    Country {
        name: "Saint Helena",
        code: "SH",
        phone_lengths: &[4, 5],
        prefix: 290,
    },
    Country {
        name: "Slovenia",
        code: "SI",
        phone_lengths: &[8],
        prefix: 386,
    },
    Country {
        // Svalbard and Jan Mayen use Norwegian numbers (+47 79 xx xx xx); NO is the
        // main region for +47 and is listed above.
        name: "Svalbard and Jan Mayen",
        code: "SJ",
        phone_lengths: &[8],
        prefix: 47,
    },
    Country {
        name: "Slovakia",
        code: "SK",
        phone_lengths: &[9],
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
        phone_lengths: &[8],
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
        phone_lengths: &[8],
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
        // 9 = mobile (8x/9x xxx xxxx); 8 = landline (Bangkok 2 xxx xxxx).
        name: "Thailand",
        code: "TH",
        phone_lengths: &[8, 9],
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
        phone_lengths: &[4, 5],
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
        phone_lengths: &[10],
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
        phone_lengths: &[8, 9],
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
        phone_lengths: &[9],
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
        phone_lengths: &[8, 9],
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
        phone_lengths: &[9, 10],
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
        phone_lengths: &[6],
        prefix: 681,
    },
    Country {
        name: "Samoa",
        code: "WS",
        phone_lengths: &[5, 6, 7],
        prefix: 685,
    },
    Country {
        // Kosovo took over +383 from the ITU in 2016; 8 = fixed (38 xxx xxx) and
        // mobile (4x xxx xxx), 9 covers the longer mobile ranges.
        name: "Kosovo",
        code: "XK",
        phone_lengths: &[8, 9],
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
        // Mayotte shares +262 with Reunion, which is listed first.
        name: "Mayotte",
        code: "YT",
        phone_lengths: &[9],
        prefix: 262,
    },
    Country {
        name: "South Africa",
        code: "ZA",
        phone_lengths: &[9],
        prefix: 27,
    },
    Country {
        name: "Zambia",
        code: "ZM",
        phone_lengths: &[9],
        prefix: 260,
    },
    Country {
        name: "Zimbabwe",
        // Zimbabwean fixed lines are genuinely variable at 5-10 digits (Harare
        // 4 + 6 or 7, small exchanges much shorter); mobile is 9. This is a wide
        // set, but a flat 9 rejected every fixed line in the country.
        code: "ZW",
        phone_lengths: &[5, 6, 7, 8, 9, 10],
        prefix: 263,
    },
];
