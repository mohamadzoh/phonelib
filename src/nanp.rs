//! North American Numbering Plan (NANP) area codes.
//!
//! Most countries in this crate are validated by number length alone. That is
//! not enough for country code `+1`: twenty-five countries and territories
//! share it, so length reveals nothing about *which* one a number belongs to,
//! and every 10-digit string is accepted regardless of whether its area code
//! was ever assigned. This module carries the area-code table that makes `+1`
//! resolvable.
//!
//! # Source
//!
//! [`NANP_AREA_CODES`] is derived from the North American Numbering Plan
//! Administrator's public NPA database
//! (`reports.nanpa.com/public/npa_report.csv`), file date 2026-07-25. Every
//! NPA that report marks as assigned is present. Codes that are merely
//! *reserved* are deliberately absent, because no number can be dialled in
//! them yet: 387, 460, 487, 537 and 568 (held for future Canadian relief) and
//! 456 are the notable examples.
//!
//! # Non-geographic codes
//!
//! Service codes that no single country operates are tagged with ISO 3166-1
//! *user-assigned* alpha-2 elements. Those elements are permanently
//! unallocated by the standard, so a tag returned here can never be confused
//! with a real country code:
//!
//! - `XF` - toll-free (800, 822, 833, 844, 855, 866, 877, 888)
//! - `XP` - premium rate (900)
//! - `XN` - other NANP-wide non-geographic service: the 500-series personal
//!   communications codes, 700 (interexchange carrier services) and 710
//!   (US Government)
//!
//! Compare against [`TOLL_FREE`], [`PREMIUM_RATE`] and [`NON_GEOGRAPHIC`], or
//! call [`is_non_geographic`], rather than hard-coding the strings. Canada's
//! own non-geographic codes (600, 622 and 633) map to `CA` instead, because
//! NANPA assigns them to a single country and they are only reachable inside
//! it. Likewise the US territories keep their own ISO codes rather than `US`:
//! 340 `VI`, 670 `MP`, 671 `GU`, 684 `AS`, 787 and 939 `PR`.
//!
//! # Caveat
//!
//! This is area-code granularity **only**. A `+1` number whose area code
//! appears here is plausible, not proven: the seven-digit subscriber part is
//! not validated beyond the structural central-office rule in
//! [`is_valid_exchange_format`]. Establishing that a particular NXX block is
//! in service requires the full NPA-NXX assignment database, which is several
//! orders of magnitude larger and changes daily.

/// Tag for toll-free area codes, returned by [`country_for_area_code`].
///
/// `XF` is an ISO 3166-1 user-assigned element and will never be a real
/// country code.
pub(crate) const TOLL_FREE: &str = "XF";

/// Tag for premium-rate (900) area codes, returned by
/// [`country_for_area_code`].
pub(crate) const PREMIUM_RATE: &str = "XP";

/// Tag for NANP-wide non-geographic service codes, returned by
/// [`country_for_area_code`].
pub(crate) const NON_GEOGRAPHIC: &str = "XN";

/// True if `tag` is one of the non-geographic sentinels rather than an
/// ISO 3166-1 country code.
pub(crate) fn is_non_geographic(tag: &str) -> bool {
    matches!(tag, TOLL_FREE | PREMIUM_RATE | NON_GEOGRAPHIC)
}

/// Every assigned NANP area code paired with the ISO 3166-1 alpha-2 code of
/// the country or territory that operates it, or one of the [`TOLL_FREE`] /
/// [`PREMIUM_RATE`] / [`NON_GEOGRAPHIC`] sentinels.
///
/// Sorted ascending by area code so [`country_for_area_code`] can
/// binary-search it. The ordering is enforced at compile time by the `const`
/// block below this table, which also rules out duplicates.
pub(crate) const NANP_AREA_CODES: &[(u16, &str)] = &[
    (201, "US"),
    (202, "US"),
    (203, "US"),
    (204, "CA"),
    (205, "US"),
    (206, "US"),
    (207, "US"),
    (208, "US"),
    (209, "US"),
    (210, "US"),
    (212, "US"),
    (213, "US"),
    (214, "US"),
    (215, "US"),
    (216, "US"),
    (217, "US"),
    (218, "US"),
    (219, "US"),
    (220, "US"),
    (223, "US"),
    (224, "US"),
    (225, "US"),
    (226, "CA"),
    (227, "US"),
    (228, "US"),
    (229, "US"),
    (231, "US"),
    (234, "US"),
    (235, "US"),
    (236, "CA"),
    (239, "US"),
    (240, "US"),
    (242, "BS"),
    (246, "BB"),
    (248, "US"),
    (249, "CA"),
    (250, "CA"),
    (251, "US"),
    (252, "US"),
    (253, "US"),
    (254, "US"),
    (256, "US"),
    (257, "CA"),
    (260, "US"),
    (262, "US"),
    (263, "CA"),
    (264, "AI"),
    (267, "US"),
    (268, "AG"),
    (269, "US"),
    (270, "US"),
    (272, "US"),
    (273, "CA"),
    (274, "US"),
    (276, "US"),
    (279, "US"),
    (281, "US"),
    (283, "US"),
    (284, "VG"),
    (289, "CA"),
    (301, "US"),
    (302, "US"),
    (303, "US"),
    (304, "US"),
    (305, "US"),
    (306, "CA"),
    (307, "US"),
    (308, "US"),
    (309, "US"),
    (310, "US"),
    (312, "US"),
    (313, "US"),
    (314, "US"),
    (315, "US"),
    (316, "US"),
    (317, "US"),
    (318, "US"),
    (319, "US"),
    (320, "US"),
    (321, "US"),
    (323, "US"),
    (324, "US"),
    (325, "US"),
    (326, "US"),
    (327, "US"),
    (329, "US"),
    (330, "US"),
    (331, "US"),
    (332, "US"),
    (334, "US"),
    (336, "US"),
    (337, "US"),
    (339, "US"),
    (340, "VI"),
    (341, "US"),
    (343, "CA"),
    (345, "KY"),
    (346, "US"),
    (347, "US"),
    (350, "US"),
    (351, "US"),
    (352, "US"),
    (353, "US"),
    (354, "CA"),
    (357, "US"),
    (360, "US"),
    (361, "US"),
    (363, "US"),
    (364, "US"),
    (365, "CA"),
    (367, "CA"),
    (368, "CA"),
    (369, "US"),
    (380, "US"),
    (382, "CA"),
    (385, "US"),
    (386, "US"),
    (401, "US"),
    (402, "US"),
    (403, "CA"),
    (404, "US"),
    (405, "US"),
    (406, "US"),
    (407, "US"),
    (408, "US"),
    (409, "US"),
    (410, "US"),
    (412, "US"),
    (413, "US"),
    (414, "US"),
    (415, "US"),
    (416, "CA"),
    (417, "US"),
    (418, "CA"),
    (419, "US"),
    (423, "US"),
    (424, "US"),
    (425, "US"),
    (428, "CA"),
    (430, "US"),
    (431, "CA"),
    (432, "US"),
    (434, "US"),
    (435, "US"),
    (436, "US"),
    (437, "CA"),
    (438, "CA"),
    (440, "US"),
    (441, "BM"),
    (442, "US"),
    (443, "US"),
    (445, "US"),
    (447, "US"),
    (448, "US"),
    (450, "CA"),
    (457, "US"),
    (458, "US"),
    (463, "US"),
    (464, "US"),
    (465, "US"),
    (468, "CA"),
    (469, "US"),
    (470, "US"),
    (471, "US"),
    (472, "US"),
    (473, "GD"),
    (474, "CA"),
    (475, "US"),
    (478, "US"),
    (479, "US"),
    (480, "US"),
    (483, "US"),
    (484, "US"),
    (500, "XN"),
    (501, "US"),
    (502, "US"),
    (503, "US"),
    (504, "US"),
    (505, "US"),
    (506, "CA"),
    (507, "US"),
    (508, "US"),
    (509, "US"),
    (510, "US"),
    (512, "US"),
    (513, "US"),
    (514, "CA"),
    (515, "US"),
    (516, "US"),
    (517, "US"),
    (518, "US"),
    (519, "CA"),
    (520, "US"),
    (521, "XN"),
    (522, "XN"),
    (523, "XN"),
    (524, "XN"),
    (525, "XN"),
    (526, "XN"),
    (527, "XN"),
    (528, "XN"),
    (529, "XN"),
    (530, "US"),
    (531, "US"),
    (532, "XN"),
    (533, "XN"),
    (534, "US"),
    (538, "XN"),
    (539, "US"),
    (540, "US"),
    (541, "US"),
    (542, "XN"),
    (543, "XN"),
    (544, "XN"),
    (548, "CA"),
    (551, "US"),
    (557, "US"),
    (559, "US"),
    (561, "US"),
    (562, "US"),
    (563, "US"),
    (564, "US"),
    (565, "US"),
    (566, "XN"),
    (567, "US"),
    (570, "US"),
    (571, "US"),
    (572, "US"),
    (573, "US"),
    (574, "US"),
    (575, "US"),
    (577, "XN"),
    (579, "CA"),
    (580, "US"),
    (581, "CA"),
    (582, "US"),
    (584, "CA"),
    (585, "US"),
    (586, "US"),
    (587, "CA"),
    (588, "XN"),
    (600, "CA"),
    (601, "US"),
    (602, "US"),
    (603, "US"),
    (604, "CA"),
    (605, "US"),
    (606, "US"),
    (607, "US"),
    (608, "US"),
    (609, "US"),
    (610, "US"),
    (612, "US"),
    (613, "CA"),
    (614, "US"),
    (615, "US"),
    (616, "US"),
    (617, "US"),
    (618, "US"),
    (619, "US"),
    (620, "US"),
    (621, "US"),
    (622, "CA"),
    (623, "US"),
    (624, "US"),
    (626, "US"),
    (628, "US"),
    (629, "US"),
    (630, "US"),
    (631, "US"),
    (633, "CA"),
    (636, "US"),
    (639, "CA"),
    (640, "US"),
    (641, "US"),
    (645, "US"),
    (646, "US"),
    (647, "CA"),
    (649, "TC"),
    (650, "US"),
    (651, "US"),
    (656, "US"),
    (657, "US"),
    (658, "JM"),
    (659, "US"),
    (660, "US"),
    (661, "US"),
    (662, "US"),
    (664, "MS"),
    (667, "US"),
    (669, "US"),
    (670, "MP"),
    (671, "GU"),
    (672, "CA"),
    (678, "US"),
    (679, "US"),
    (680, "US"),
    (681, "US"),
    (682, "US"),
    (683, "CA"),
    (684, "AS"),
    (686, "US"),
    (689, "US"),
    (700, "XN"),
    (701, "US"),
    (702, "US"),
    (703, "US"),
    (704, "US"),
    (705, "CA"),
    (706, "US"),
    (707, "US"),
    (708, "US"),
    (709, "CA"),
    (710, "XN"),
    (712, "US"),
    (713, "US"),
    (714, "US"),
    (715, "US"),
    (716, "US"),
    (717, "US"),
    (718, "US"),
    (719, "US"),
    (720, "US"),
    (721, "SX"),
    (724, "US"),
    (725, "US"),
    (726, "US"),
    (727, "US"),
    (728, "US"),
    (729, "US"),
    (730, "US"),
    (731, "US"),
    (732, "US"),
    (734, "US"),
    (737, "US"),
    (738, "US"),
    (740, "US"),
    (742, "CA"),
    (743, "US"),
    (747, "US"),
    (748, "US"),
    (753, "CA"),
    (754, "US"),
    (757, "US"),
    (758, "LC"),
    (760, "US"),
    (761, "US"),
    (762, "US"),
    (763, "US"),
    (765, "US"),
    (767, "DM"),
    (769, "US"),
    (770, "US"),
    (771, "US"),
    (772, "US"),
    (773, "US"),
    (774, "US"),
    (775, "US"),
    (778, "CA"),
    (779, "US"),
    (780, "CA"),
    (781, "US"),
    (782, "CA"),
    (784, "VC"),
    (785, "US"),
    (786, "US"),
    (787, "PR"),
    (800, "XF"),
    (801, "US"),
    (802, "US"),
    (803, "US"),
    (804, "US"),
    (805, "US"),
    (806, "US"),
    (807, "CA"),
    (808, "US"),
    (809, "DO"),
    (810, "US"),
    (812, "US"),
    (813, "US"),
    (814, "US"),
    (815, "US"),
    (816, "US"),
    (817, "US"),
    (818, "US"),
    (819, "CA"),
    (820, "US"),
    (821, "US"),
    (822, "XF"),
    (825, "CA"),
    (826, "US"),
    (828, "US"),
    (829, "DO"),
    (830, "US"),
    (831, "US"),
    (832, "US"),
    (833, "XF"),
    (835, "US"),
    (837, "US"),
    (838, "US"),
    (839, "US"),
    (840, "US"),
    (843, "US"),
    (844, "XF"),
    (845, "US"),
    (847, "US"),
    (848, "US"),
    (849, "DO"),
    (850, "US"),
    (854, "US"),
    (855, "XF"),
    (856, "US"),
    (857, "US"),
    (858, "US"),
    (859, "US"),
    (860, "US"),
    (861, "US"),
    (862, "US"),
    (863, "US"),
    (864, "US"),
    (865, "US"),
    (866, "XF"),
    (867, "CA"),
    (868, "TT"),
    (869, "KN"),
    (870, "US"),
    (872, "US"),
    (873, "CA"),
    (876, "JM"),
    (877, "XF"),
    (878, "US"),
    (879, "CA"),
    (888, "XF"),
    (900, "XP"),
    (901, "US"),
    (902, "CA"),
    (903, "US"),
    (904, "US"),
    (905, "CA"),
    (906, "US"),
    (907, "US"),
    (908, "US"),
    (909, "US"),
    (910, "US"),
    (912, "US"),
    (913, "US"),
    (914, "US"),
    (915, "US"),
    (916, "US"),
    (917, "US"),
    (918, "US"),
    (919, "US"),
    (920, "US"),
    (924, "US"),
    (925, "US"),
    (928, "US"),
    (929, "US"),
    (930, "US"),
    (931, "US"),
    (934, "US"),
    (936, "US"),
    (937, "US"),
    (938, "US"),
    (939, "PR"),
    (940, "US"),
    (941, "US"),
    (942, "CA"),
    (943, "US"),
    (945, "US"),
    (947, "US"),
    (948, "US"),
    (949, "US"),
    (951, "US"),
    (952, "US"),
    (954, "US"),
    (956, "US"),
    (959, "US"),
    (970, "US"),
    (971, "US"),
    (972, "US"),
    (973, "US"),
    (975, "US"),
    (978, "US"),
    (979, "US"),
    (980, "US"),
    (983, "US"),
    (984, "US"),
    (985, "US"),
    (986, "US"),
    (989, "US"),
];

/// Compile-time proof that [`NANP_AREA_CODES`] is strictly ascending by area
/// code. Strictness also rules out duplicate entries.
const _: () = {
    let mut i = 1;
    while i < NANP_AREA_CODES.len() {
        assert!(
            NANP_AREA_CODES[i - 1].0 < NANP_AREA_CODES[i].0,
            "NANP_AREA_CODES must be sorted ascending with no duplicates",
        );
        i += 1;
    }
};

/// The country or territory operating `area_code`, or `None` if the code is
/// unassigned, reserved, or structurally invalid.
///
/// The returned value is an ISO 3166-1 alpha-2 code for geographic codes and
/// one of the non-geographic sentinels otherwise; see the module docs.
pub(crate) fn country_for_area_code(area_code: u16) -> Option<&'static str> {
    if !is_valid_area_code_format(area_code) {
        return None;
    }

    NANP_AREA_CODES
        .binary_search_by_key(&area_code, |&(code, _)| code)
        .ok()
        .map(|index| NANP_AREA_CODES[index].1)
}

/// True if `area_code` is structurally valid per NANP rules: three digits,
/// first digit 2-9, and not an N11 service code (211, 311, ... 911).
///
/// Structural validity says nothing about assignment; use
/// [`country_for_area_code`] for that.
pub(crate) fn is_valid_area_code_format(area_code: u16) -> bool {
    (200..=999).contains(&area_code) && area_code % 100 != 11
}

/// True if `exchange` (the three-digit central office code) is structurally
/// valid: first digit 2-9, and not the N11 form.
pub(crate) fn is_valid_exchange_format(exchange: u16) -> bool {
    (200..=999).contains(&exchange) && exchange % 100 != 11
}

#[cfg(test)]
mod tests {
    use super::*;

    const N11_CODES: [u16; 8] = [211, 311, 411, 511, 611, 711, 811, 911];

    /// Codes NANPA lists as unassigned or merely reserved as of the source
    /// file date.
    const UNASSIGNED: [u16; 12] = [221, 230, 232, 387, 390, 456, 460, 487, 537, 568, 599, 999];

    #[test]
    fn table_is_sorted_ascending() {
        assert!(NANP_AREA_CODES.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn table_has_no_duplicates() {
        let mut codes: Vec<u16> = NANP_AREA_CODES.iter().map(|&(code, _)| code).collect();
        let total = codes.len();
        codes.dedup();
        assert_eq!(codes.len(), total);
    }

    #[test]
    fn every_entry_is_structurally_valid() {
        for &(code, country) in NANP_AREA_CODES {
            assert!(is_valid_area_code_format(code), "bad area code {code}");
            assert_eq!(country.len(), 2, "bad tag {country} for {code}");
        }
    }

    #[test]
    fn looks_up_us_area_codes() {
        for code in [212, 213, 312, 415, 646, 702, 917, 989] {
            assert_eq!(country_for_area_code(code), Some("US"), "code {code}");
        }
    }

    #[test]
    fn looks_up_canadian_area_codes() {
        for code in [
            204, 236, 289, 306, 416, 438, 514, 604, 709, 782, 867, 902, 905, 942,
        ] {
            assert_eq!(country_for_area_code(code), Some("CA"), "code {code}");
        }
    }

    #[test]
    fn looks_up_territory_area_codes() {
        let expected = [
            (242, "BS"),
            (246, "BB"),
            (264, "AI"),
            (268, "AG"),
            (284, "VG"),
            (340, "VI"),
            (345, "KY"),
            (441, "BM"),
            (473, "GD"),
            (649, "TC"),
            (658, "JM"),
            (664, "MS"),
            (670, "MP"),
            (671, "GU"),
            (684, "AS"),
            (721, "SX"),
            (758, "LC"),
            (767, "DM"),
            (784, "VC"),
            (787, "PR"),
            (809, "DO"),
            (829, "DO"),
            (849, "DO"),
            (868, "TT"),
            (869, "KN"),
            (876, "JM"),
            (939, "PR"),
        ];

        for (code, country) in expected {
            assert_eq!(country_for_area_code(code), Some(country), "code {code}");
        }
    }

    #[test]
    fn tags_non_geographic_codes() {
        for code in [800, 833, 844, 855, 866, 877, 888] {
            assert_eq!(country_for_area_code(code), Some(TOLL_FREE), "code {code}");
        }
        assert_eq!(country_for_area_code(900), Some(PREMIUM_RATE));
        assert_eq!(country_for_area_code(700), Some(NON_GEOGRAPHIC));
        assert_eq!(country_for_area_code(710), Some(NON_GEOGRAPHIC));

        assert!(is_non_geographic(TOLL_FREE));
        assert!(is_non_geographic(PREMIUM_RATE));
        assert!(is_non_geographic(NON_GEOGRAPHIC));
        assert!(!is_non_geographic("US"));
        assert!(!is_non_geographic("CA"));
    }

    #[test]
    fn canadian_non_geographic_codes_map_to_canada() {
        for code in [600, 622, 633] {
            assert_eq!(country_for_area_code(code), Some("CA"), "code {code}");
        }
    }

    #[test]
    fn rejects_n11_area_codes() {
        for code in N11_CODES {
            assert!(!is_valid_area_code_format(code), "code {code}");
            assert_eq!(country_for_area_code(code), None, "code {code}");
        }
    }

    #[test]
    fn rejects_n11_exchanges() {
        for code in N11_CODES {
            assert!(!is_valid_exchange_format(code), "exchange {code}");
        }
    }

    #[test]
    fn rejects_leading_zero_or_one() {
        for code in [0, 1, 11, 12, 100, 111, 123, 155, 199] {
            assert!(!is_valid_area_code_format(code), "code {code}");
            assert!(!is_valid_exchange_format(code), "exchange {code}");
            assert_eq!(country_for_area_code(code), None, "code {code}");
        }
    }

    #[test]
    fn rejects_out_of_range_values() {
        for code in [1000, 9999, u16::MAX] {
            assert!(!is_valid_area_code_format(code), "code {code}");
            assert_eq!(country_for_area_code(code), None, "code {code}");
        }
    }

    #[test]
    fn unassigned_codes_return_none() {
        for code in UNASSIGNED {
            assert_eq!(country_for_area_code(code), None, "code {code}");
        }
    }

    #[test]
    fn accepts_valid_exchanges() {
        for code in [200, 234, 555, 700, 999] {
            assert!(is_valid_exchange_format(code), "exchange {code}");
        }
    }
}
