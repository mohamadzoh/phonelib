//! Free-form text extraction: finding phone numbers embedded in prose.
//!
//! The scanner walks the input exactly once, in bytes/chars, and never converts
//! between character and byte indices by counting — every offset it reports is
//! the byte offset it was reading at, so extraction is linear in the length of
//! the input regardless of how many multi-byte characters it contains.
//!
//! ## Algorithm
//!
//! 1. **Start detection.** A candidate may begin at a digit, a `+` or a `(`,
//!    and only when the preceding character does not make it part of something
//!    else (see [`start_is_blocked`]). That keeps numbers out of identifiers,
//!    version strings and e-mail local parts, without hiding the numbers in a
//!    comma-separated list.
//! 2. **Greedy consumption.** From the start the scanner consumes
//!    separator-delimited digit *groups*. It refuses to grow past
//!    [`MAX_DIGITS`] digits or [`max_groups`] groups, and it records every
//!    position where the candidate could legally end (after a digit group, or
//!    after the closing parenthesis that balances it).
//! 3. **Backtracking.** The recorded stops are tried from longest to shortest:
//!    the first one that normalizes wins. This is what makes
//!    `"2025550173 415555"` collapse back to `"2025550173"` while keeping
//!    `"+1 202 555 0173"` whole. If nothing normalizes, the longest stop that
//!    still meets the minimum digit count is reported with `is_valid: false`.
//! 4. **Resume.** Scanning continues at the end of the accepted span, so a
//!    second number immediately after the first is still found.
//!
//! Because a candidate can never exceed [`MAX_DIGITS`] digits, [`MAX_GROUPS`]
//! groups or [`MAX_SEPARATOR_RUN`] consecutive separators, the work done per
//! start position is bounded by a constant — the whole scan is O(n).

use crate::constants::COUNTRIES;
use crate::definitions::Country;

/// Result of extracting a phone number from text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedPhoneNumber {
    /// The phone number as it appeared in the text
    pub raw: String,
    /// The normalized E.164 format if valid
    pub normalized: Option<String>,
    /// Start position in the original text (byte index)
    pub start: usize,
    /// End position in the original text (byte index)
    pub end: usize,
    /// Whether the extracted number is valid
    pub is_valid: bool,
}

/// Maximum number of digits an E.164 number can carry.
///
/// An *unbroken* run longer than this is skipped rather than truncated into a
/// bogus candidate. Crossing the ceiling by taking one more *separated* group
/// instead stops at the previous boundary — that case is two numbers side by
/// side, which the backtracking below recovers.
const MAX_DIGITS: usize = 15;

/// Storage capacity for one candidate's groups. See [`max_groups`] for the
/// limit actually applied while scanning.
const MAX_GROUPS: usize = 8;

/// How many separator-delimited digit groups one candidate may span.
///
/// With an explicit `+` the marker itself bounds the number, so a generous cap
/// is safe (`+49 (0) 30 12 34 56 78` is seven groups). Without one, a long run
/// of short groups is more likely two numbers than one — gluing
/// `06 45 34 25 45 06` fabricates a third that was never in the text.
fn max_groups(has_plus: bool) -> usize {
    if has_plus {
        MAX_GROUPS
    } else {
        5
    }
}

/// Maximum run of consecutive separator characters inside a candidate.
/// Three covers `" - "`, `" — "` and a double space.
const MAX_SEPARATOR_RUN: usize = 3;

/// Fewest digits an unhinted candidate written with a `+` must carry to be
/// worth reporting.
///
/// Derived from the table, never hardcoded: with no hint a candidate must spell
/// out its own calling code, so the floor is the smallest `calling code +
/// national number` it allows. A bare run of digits must also clear
/// [`crate::MIN_UNMARKED_DIGITS`], or [`redact`] would start eating order
/// numbers out of prose. With a hint the floor drops to that country's
/// shortest length.
const MIN_DIGITS: usize = shortest_dialable_length();

/// The digit floors a candidate must clear to be reported.
#[derive(Clone, Copy)]
struct Floors {
    /// For a candidate written with a leading `+`.
    marked: usize,
    /// For a bare run of digits.
    bare: usize,
}

// The country table must stay inside the bounds this module assumes.
const _: () = assert!(
    MIN_DIGITS >= crate::MIN_NATIONAL_LEN,
    "the extraction floor cannot be shorter than the shortest national number"
);
const _: () = assert!(
    crate::MAX_NATIONAL_LEN <= MAX_DIGITS,
    "a national number longer than E.164 allows cannot be extracted"
);

/// Number of decimal digits in a country calling code.
const fn prefix_len(prefix: u32) -> usize {
    if prefix >= 1000 {
        4
    } else if prefix >= 100 {
        3
    } else if prefix >= 10 {
        2
    } else {
        1
    }
}

/// The smallest `calling code + national number` digit count in the table.
const fn shortest_dialable_length() -> usize {
    let mut shortest = MAX_DIGITS;
    let mut i = 0;
    while i < COUNTRIES.len() {
        let plen = prefix_len(COUNTRIES[i].prefix);
        let lengths = COUNTRIES[i].phone_lengths;
        let mut j = 0;
        while j < lengths.len() {
            let total = plen + lengths[j] as usize;
            if total < shortest {
                shortest = total;
            }
            j += 1;
        }
        i += 1;
    }
    shortest
}

// ============================================================================
// Character classification
// ============================================================================

/// Decimal value of a digit character. ASCII digits and the fullwidth digits
/// used in CJK typography (U+FF10..U+FF19) are both recognised.
#[inline]
fn digit_value(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        '\u{FF10}'..='\u{FF19}' => Some((c as u32 - 0xFF10) as u8),
        _ => None,
    }
}

/// ASCII or fullwidth plus sign.
#[inline]
fn is_plus(c: char) -> bool {
    c == '+' || c == '\u{FF0B}'
}

/// ASCII or fullwidth opening parenthesis.
#[inline]
fn is_open_paren(c: char) -> bool {
    c == '(' || c == '\u{FF08}'
}

/// ASCII or fullwidth closing parenthesis.
#[inline]
fn is_close_paren(c: char) -> bool {
    c == ')' || c == '\u{FF09}'
}

/// Characters that may sit *between* two digit groups of one phone number.
///
/// Beyond the obvious space/hyphen/dot this covers the separators that real
/// documents are full of: non-breaking and narrow no-break spaces, the several
/// Unicode dashes, the ideographic space, and the fullwidth hyphen/full stop
/// used alongside fullwidth digits. Line breaks and commas are deliberately
/// *not* separators — they terminate a candidate.
#[inline]
fn is_separator(c: char) -> bool {
    matches!(
        c,
        ' '                 // space
            | '\t'          // tab
            | '-'           // hyphen-minus
            | '.'           // full stop
            | '\u{00A0}'    // no-break space
            | '\u{2007}'    // figure space
            | '\u{2009}'    // thin space
            | '\u{202F}'    // narrow no-break space
            | '\u{2010}'    // hyphen
            | '\u{2011}'    // non-breaking hyphen
            | '\u{2012}'    // figure dash
            | '\u{2013}'    // en dash
            | '\u{2014}'    // em dash
            | '\u{2015}'    // horizontal bar
            | '\u{2212}'    // minus sign
            | '\u{3000}'    // ideographic space
            | '\u{FF0D}'    // fullwidth hyphen-minus
            | '\u{FF0E}' // fullwidth full stop
    )
}

/// Characters a phone number may not directly follow.
///
/// A candidate starting right after one of these is a slice out of something
/// else: an identifier, a version, or an e-mail local part
/// (`user+12025550173@example.com` — hence `+` is in the set). Only ASCII
/// letters count, because CJK text routinely runs a label into the number.
#[inline]
fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || digit_value(c).is_some() || is_plus(c) || c == '_'
}

/// The two characters immediately before `pos`, nearest first.
fn preceding(text: &str, pos: usize) -> (Option<char>, Option<char>) {
    let mut back = text[..pos].chars().rev();
    (back.next(), back.next())
}

/// Whether a candidate starting at a position preceded by `prev` (itself
/// preceded by `prev2`) is really a slice out of the middle of something else.
///
/// `.` and `,` are deliberately *not* blanket blockers. They only block when a
/// digit sits before them, i.e. when they are acting as a decimal point or a
/// thousands separator (`1.2025550173`, `1,2025550173`). Treating them as
/// word-like unconditionally made every number in a comma-separated list or a
/// CSV column invisible, which meant [`redact`] left them in the clear.
fn start_is_blocked(prev: Option<char>, prev2: Option<char>) -> bool {
    let Some(p) = prev else {
        return false;
    };
    if is_word_char(p) {
        return true;
    }
    if p == '.' || p == ',' {
        return prev2.is_some_and(|q| digit_value(q).is_some());
    }
    false
}

/// The character starting at byte offset `pos`, if any.
///
/// `pos` is always a character boundary: every offset in this module comes from
/// stepping over whole characters.
#[inline]
fn char_at(text: &str, pos: usize) -> Option<char> {
    if pos >= text.len() {
        None
    } else {
        text[pos..].chars().next()
    }
}

/// Length of the maximal digit run starting at `pos`, as `(end_offset, count)`.
fn measure_digits(text: &str, pos: usize) -> (usize, usize) {
    let mut end = pos;
    let mut count = 0;
    while let Some(c) = char_at(text, end) {
        if digit_value(c).is_none() {
            break;
        }
        count += 1;
        end += c.len_utf8();
    }
    (end, count)
}

// ============================================================================
// Candidate scanning
// ============================================================================

/// A position at which a candidate may legally end: just past a digit group, or
/// just past the parenthesis that balances the group.
#[derive(Clone, Copy, Debug)]
struct Stop {
    /// Byte offset one past the last character of the span.
    end: usize,
    /// Digits accumulated up to this point.
    digits: usize,
    /// Digit groups accumulated up to this point.
    groups: usize,
}

/// Everything the scanner learned about one candidate span.
struct Candidate {
    /// Digits normalized to ASCII, in source order.
    digits: [u8; MAX_DIGITS],
    digit_count: usize,
    /// Whether the span opened with a `+`.
    has_plus: bool,
    /// Digit count of each group.
    group_len: [u8; MAX_GROUPS],
    /// Whether group `i` was opened by a parenthesis.
    group_paren: [bool; MAX_GROUPS],
    /// The separator preceding group `i`, when the gap was exactly one
    /// separator character and contained no parenthesis; `'\0'` otherwise.
    group_sep: [char; MAX_GROUPS],
    group_count: usize,
    stops: [Stop; MAX_GROUPS],
    stop_count: usize,
}

impl Candidate {
    fn new() -> Self {
        Candidate {
            digits: [0; MAX_DIGITS],
            digit_count: 0,
            has_plus: false,
            group_len: [0; MAX_GROUPS],
            group_paren: [false; MAX_GROUPS],
            group_sep: ['\0'; MAX_GROUPS],
            group_count: 0,
            stops: [Stop {
                end: 0,
                digits: 0,
                groups: 0,
            }; MAX_GROUPS],
            stop_count: 0,
        }
    }

    /// The digits to hand to the parser for a given span: everything collected
    /// up to that stop, minus a parenthesized trunk marker such as the `(0)` of
    /// `+44 (0) 20 7946 0958`, which is a dialing instruction rather than part
    /// of the number.
    fn dialable(&self, stop: Stop) -> String {
        let mut dialable = String::with_capacity(stop.digits);
        let mut offset = 0;
        for group in 0..stop.groups {
            let len = self.group_len[group] as usize;
            let trunk_marker =
                group > 0 && self.group_paren[group] && len == 1 && self.digits[offset] == b'0';
            if !trunk_marker {
                dialable.push_str(
                    std::str::from_utf8(&self.digits[offset..offset + len]).unwrap_or(""),
                );
            }
            offset += len;
        }
        dialable
    }

    /// The digits of group `idx`.
    fn group_digits(&self, idx: usize) -> &str {
        let mut offset = 0;
        for i in 0..idx {
            offset += self.group_len[i] as usize;
        }
        let len = self.group_len[idx] as usize;
        std::str::from_utf8(&self.digits[offset..offset + len]).unwrap_or("")
    }

    /// Numeric value of group `idx`; `u32::MAX` if it cannot be represented
    /// (only reachable for groups far longer than any heuristic inspects).
    fn group_value(&self, idx: usize) -> u32 {
        self.group_digits(idx).parse().unwrap_or(u32::MAX)
    }

    /// True when every gap up to `groups` was exactly the single character `sep`.
    fn separated_only_by(&self, groups: usize, sep: char) -> bool {
        (1..groups).all(|i| self.group_sep[i] == sep)
    }
}

/// What the scanner decided about one start position.
enum Outcome {
    /// A number was accepted; the outer scan resumes at its `end`.
    Found(ExtractedPhoneNumber),
    /// Nothing was accepted; resume at this byte offset (never before `start`).
    Skip(usize),
}

/// Scan one candidate beginning at `start` and decide what to do with it.
fn scan(text: &str, start: usize, hint: Option<&'static Country>, floors: Floors) -> Outcome {
    let mut cand = Candidate::new();
    let mut pos = start;
    let mut depth = 0usize;

    if let Some(c) = char_at(text, pos) {
        if is_plus(c) {
            cand.has_plus = true;
            pos += c.len_utf8();
        }
    }

    loop {
        if cand.group_count == max_groups(cand.has_plus) {
            break;
        }

        // --- the gap before the next group: separators and at most one '(' ---
        let mut cursor = pos;
        let mut sep_count = 0usize;
        let mut sep_char = '\0';
        let mut opened = false;
        while let Some(c) = char_at(text, cursor) {
            // Separators only make sense once something has been consumed;
            // a bare candidate always starts on '(' or a digit.
            if is_separator(c)
                && sep_count < MAX_SEPARATOR_RUN
                && (cand.group_count > 0 || cand.has_plus)
            {
                sep_count += 1;
                sep_char = c;
                cursor += c.len_utf8();
            } else if is_open_paren(c) && !opened {
                opened = true;
                cursor += c.len_utf8();
            } else if is_plus(c) && cand.group_count == 0 && !cand.has_plus {
                // A parenthesised calling code, `(+44) 20 7946 0958`.
                cand.has_plus = true;
                cursor += c.len_utf8();
            } else {
                break;
            }
        }

        // --- the group itself ---
        let (group_end, group_len) = measure_digits(text, cursor);
        if group_len == 0 {
            break;
        }
        if group_len > MAX_DIGITS {
            // Longer than E.164 permits: not a phone number. If this is the
            // first group the whole run is skipped, otherwise the groups
            // already accepted stand on their own.
            if cand.group_count == 0 {
                return Outcome::Skip(group_end);
            }
            break;
        }
        if cand.digit_count + group_len > MAX_DIGITS {
            break;
        }

        if cand.group_count > 0 {
            cand.group_sep[cand.group_count] = if sep_count == 1 && !opened {
                sep_char
            } else {
                '\0'
            };
        }
        let mut digit_pos = cursor;
        while digit_pos < group_end {
            let c = char_at(text, digit_pos).unwrap_or('\0');
            if let Some(v) = digit_value(c) {
                cand.digits[cand.digit_count] = b'0' + v;
                cand.digit_count += 1;
            }
            digit_pos += c.len_utf8();
        }
        cand.group_len[cand.group_count] = group_len as u8;
        cand.group_paren[cand.group_count] = opened;
        cand.group_count += 1;
        if opened {
            depth += 1;
        }
        pos = group_end;

        // --- closing parentheses that balance an opener inside the span ---
        while depth > 0 {
            match char_at(text, pos) {
                Some(c) if is_close_paren(c) => {
                    depth -= 1;
                    pos += c.len_utf8();
                }
                _ => break,
            }
        }

        // A span may only end with its parentheses balanced, so an unmatched
        // '(' can never leak into `raw`. A parenthesised group may only end the
        // span when it is the first: real formatting brackets a leading area
        // code, never a trailing group, so `"2025550173 (555)"` falls back.
        let parenthetical_tail = cand.group_paren[cand.group_count - 1] && cand.group_count > 1;
        if depth == 0 && !parenthetical_tail && cand.stop_count < MAX_GROUPS {
            cand.stops[cand.stop_count] = Stop {
                end: pos,
                digits: cand.digit_count,
                groups: cand.group_count,
            };
            cand.stop_count += 1;
        }
    }

    // --- backtrack: try the longest span first, dropping one group at a time ---
    let min_digits = if cand.has_plus {
        floors.marked
    } else {
        floors.bare
    };
    let mut fallback: Option<Stop> = None;
    for idx in (0..cand.stop_count).rev() {
        let stop = cand.stops[idx];
        if stop.digits < min_digits {
            break;
        }
        // Dates and ISBNs match on a precise shape, so they are rejected up
        // front: with a hint an ISO date often does normalize to a real number.
        if is_structured_data(&cand, stop) {
            return Outcome::Skip(stop.end);
        }
        if let Some(normalized) = validate(&cand.dialable(stop), cand.has_plus, hint) {
            return Outcome::Found(build(text, start, stop.end, Some(normalized)));
        }
        // The dotted-quad shape is far less discriminating — plenty of real
        // European numbers are written as four dot-separated groups — so it
        // only applies to a candidate that failed to normalize.
        if looks_like_dotted_quad(&cand, stop) {
            return Outcome::Skip(stop.end);
        }
        if fallback.is_none() {
            fallback = Some(stop);
        }
    }

    match fallback {
        Some(stop) => Outcome::Found(build(text, start, stop.end, None)),
        None => Outcome::Skip(start),
    }
}

/// Assemble a result. `raw` is sliced straight out of the source, so
/// `&text[start..end] == raw` holds by construction.
fn build(text: &str, start: usize, end: usize, normalized: Option<String>) -> ExtractedPhoneNumber {
    ExtractedPhoneNumber {
        raw: text[start..end].to_string(),
        is_valid: normalized.is_some(),
        normalized,
        start,
        end,
    }
}

// ============================================================================
// Validation
// ============================================================================

/// Normalize a candidate, optionally interpreting it as a national number of
/// the hint country. Priority, highest first:
///
/// 1. A candidate carrying an **explicit international marker** — a leading `+`
///    (or fullwidth `＋`), or an IDD prefix (`00`, `011`, `0011`) whose
///    remainder parses — is already a global number. It is parsed as written
///    and *never* given a second calling code, so `"+12025550173"` stays
///    American under a German hint.
/// 2. Otherwise the hint wins: a single leading trunk zero is dropped, the
///    hint's calling code is prepended, and that reading is used if it
///    validates. The hint deliberately beats an incidental self-parse, since a
///    bare national number often happens to fit some unrelated country's plan.
/// 3. Failing that, the digits are parsed exactly as they appear.
fn validate(digits: &str, has_plus: bool, hint: Option<&'static Country>) -> Option<String> {
    if digits.is_empty() {
        return None;
    }

    if has_plus {
        let mut with_plus = String::with_capacity(digits.len() + 1);
        with_plus.push('+');
        with_plus.push_str(digits);
        return crate::normalize_phone_number(&with_plus);
    }

    if let Some(country) = hint {
        // Delegate rather than reimplement: the hinted parser knows each
        // country's trunk digit, which plans keep a significant leading zero,
        // and how NANP numbers are dialled.
        return crate::PhoneNumber::try_parse_with_country(digits, country.code)
            .ok()
            .map(|parsed| parsed.e164().to_string());
    }

    // An access code only counts when what follows it parses, so a national
    // number merely starting with those digits (Italian `011`) survives.
    if let Some(rest) = strip_idd_prefix(digits) {
        if let Some(normalized) = crate::normalize_phone_number(&format!("+{}", rest)) {
            return Some(normalized);
        }
    }

    crate::normalize_phone_number(digits)
}

/// The digits after an international direct-dialing prefix, if one is present.
fn strip_idd_prefix(digits: &str) -> Option<&str> {
    for prefix in ["0011", "011", "00"] {
        if let Some(rest) = digits.strip_prefix(prefix) {
            if !rest.is_empty() {
                return Some(rest);
            }
        }
    }
    None
}

// ============================================================================
// False-positive heuristics
// ============================================================================

/// Shapes precise enough to reject before even trying to normalize: ISO and
/// European dates, and ISBN-13. Dotted quads are handled separately, after
/// validation, because that shape also matches real numbers.
///
/// A candidate written with an explicit `+` is exempt. Everything else numeric
/// — account numbers, order ids, ISBN-10, year ranges — is still reported,
/// usually with `is_valid: false`; no metadata here separates them.
fn is_structured_data(cand: &Candidate, stop: Stop) -> bool {
    if cand.has_plus {
        return false;
    }
    looks_like_date(cand, stop) || looks_like_isbn13(cand, stop)
}

fn looks_like_date(cand: &Candidate, stop: Stop) -> bool {
    if stop.groups < 2 || stop.groups > 3 || !cand.separated_only_by(stop.groups, '-') {
        return false;
    }
    let len = |i: usize| cand.group_len[i] as usize;
    let value = |i: usize| cand.group_value(i);

    // YYYY-MM[-DD]
    if len(0) == 4
        && (1900..=2199).contains(&value(0))
        && len(1) <= 2
        && (1..=12).contains(&value(1))
    {
        if stop.groups == 2 {
            return true;
        }
        if len(2) <= 2 && (1..=31).contains(&value(2)) {
            return true;
        }
    }

    // DD-MM-YYYY
    stop.groups == 3
        && len(0) <= 2
        && len(1) <= 2
        && len(2) == 4
        && (1..=31).contains(&value(0))
        && (1..=12).contains(&value(1))
        && (1900..=2199).contains(&value(2))
}

fn looks_like_dotted_quad(cand: &Candidate, stop: Stop) -> bool {
    !cand.has_plus
        && stop.groups == 4
        && stop.digits <= 12
        && cand.separated_only_by(stop.groups, '.')
        && (0..4).all(|i| cand.group_len[i] <= 3 && cand.group_value(i) <= 255)
}

fn looks_like_isbn13(cand: &Candidate, stop: Stop) -> bool {
    stop.digits == 13
        && stop.groups >= 4
        && cand.separated_only_by(stop.groups, '-')
        && matches!(cand.group_digits(0), "978" | "979")
}

// ============================================================================
// Public entry points
// ============================================================================

/// Extract every phone number candidate from free-form text.
///
/// Numbers that could not be normalized are still reported, with
/// `is_valid: false` and `normalized: None`. The returned spans are strictly
/// ascending and never overlap.
pub(crate) fn extract(text: &str) -> Vec<ExtractedPhoneNumber> {
    extract_with_hint(text, None)
}

/// Extract phone numbers, optionally reading unprefixed candidates as national
/// numbers of `hint`.
///
/// The hint takes priority over parsing the digits as they stand (see
/// [`validate`]), but it is never applied to a candidate that already carries an
/// international marker, so an international number cannot be given a second
/// calling code. A hint also lowers the minimum digit count to the shortest
/// national number that country accepts, which is what makes six-digit plans
/// such as Andorra's reachable at all.
pub(crate) fn extract_with_hint(
    text: &str,
    hint: Option<&'static Country>,
) -> Vec<ExtractedPhoneNumber> {
    let floors = match hint {
        Some(country) => {
            let shortest = country
                .phone_lengths
                .iter()
                .map(|&len| len as usize)
                .min()
                .unwrap_or(MIN_DIGITS)
                .min(MIN_DIGITS);
            Floors {
                marked: shortest,
                bare: shortest,
            }
        }
        None => Floors {
            marked: MIN_DIGITS,
            bare: MIN_DIGITS.max(crate::MIN_UNMARKED_DIGITS),
        },
    };

    let mut results = Vec::new();
    let mut pos = 0usize;

    while let Some(c) = char_at(text, pos) {
        let can_start = digit_value(c).is_some() || is_plus(c) || is_open_paren(c);
        let (prev, prev2) = preceding(text, pos);
        if can_start && !start_is_blocked(prev, prev2) {
            match scan(text, pos, hint, floors) {
                Outcome::Found(number) => {
                    let end = number.end;
                    results.push(number);
                    pos = end;
                    continue;
                }
                Outcome::Skip(resume) if resume > pos => {
                    pos = resume;
                    continue;
                }
                Outcome::Skip(_) => {}
            }
        }
        pos += c.len_utf8();
    }

    results
}

/// Replace every extracted phone number with the string produced by
/// `replacement`, leaving the rest of the text untouched.
pub(crate) fn replace<F>(text: &str, replacement: F) -> String
where
    F: Fn(&ExtractedPhoneNumber) -> String,
{
    let numbers = extract(text);
    if numbers.is_empty() {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;
    for number in &numbers {
        result.push_str(&text[last_end..number.start]);
        result.push_str(&replacement(number));
        last_end = number.end;
    }
    result.push_str(&text[last_end..]);
    result
}

/// Mask phone numbers for privacy.
///
/// The contract is deliberately blunt, because a partial mask is a leak:
///
/// * the **entire** matched span is replaced — separators, parentheses and all,
///   so nothing of the original formatting survives to be reassembled;
/// * with `visible_digits == 0`, or when `visible_digits` is at least the
///   number of digits in the span, the span becomes `[PHONE]`;
/// * otherwise the span becomes one `*` per hidden digit followed by the last
///   `visible_digits` digits of the span, rendered as ASCII digits
///   (`+1 (202) 555-0173` has 11 digits, so with 4 visible it becomes
///   `*******0173` — one `*` per *hidden digit*, not per replaced character).
///
/// Because spans never overlap, no digit of a second number can survive inside
/// the mask of a first one.
pub(crate) fn redact(text: &str, visible_digits: usize) -> String {
    replace(text, |number| {
        let total = number.raw.chars().filter_map(digit_value).count();

        if visible_digits == 0 || visible_digits >= total {
            return "[PHONE]".to_string();
        }

        let hidden = total - visible_digits;
        let mut masked = String::with_capacity(total);
        for _ in 0..hidden {
            masked.push('*');
        }
        for value in number.raw.chars().filter_map(digit_value).skip(hidden) {
            masked.push((b'0' + value) as char);
        }
        masked
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn country(code: &str) -> Option<&'static Country> {
        COUNTRIES.iter().find(|c| c.code == code)
    }

    /// Every reported span must slice back to exactly the reported `raw`, and
    /// spans must be strictly ascending and non-overlapping.
    fn assert_invariants(text: &str, numbers: &[ExtractedPhoneNumber]) {
        let mut previous_end = 0;
        for number in numbers {
            assert!(
                number.start >= previous_end,
                "overlapping or backwards spans in {:?}: {:?}",
                text,
                numbers
            );
            assert!(number.end > number.start, "empty span in {:?}", text);
            assert_eq!(
                &text[number.start..number.end],
                number.raw,
                "span/raw mismatch in {:?}",
                text
            );
            assert_eq!(number.is_valid, number.normalized.is_some());
            previous_end = number.end;
        }
    }

    fn extract_checked(text: &str) -> Vec<ExtractedPhoneNumber> {
        let numbers = extract(text);
        assert_invariants(text, &numbers);
        numbers
    }

    fn raws(numbers: &[ExtractedPhoneNumber]) -> Vec<&str> {
        numbers.iter().map(|n| n.raw.as_str()).collect()
    }

    // --- baseline behaviour ------------------------------------------------

    #[test]
    fn extracts_international_numbers() {
        let text = "Call me at +12025550173 or +442079460958 for support.";
        let numbers = extract_checked(text);

        assert_eq!(raws(&numbers), ["+12025550173", "+442079460958"]);
        assert!(numbers.iter().all(|n| n.is_valid));
        assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
        assert_eq!(numbers[1].normalized.as_deref(), Some("+442079460958"));
    }

    #[test]
    fn extracts_grouped_international_number_whole() {
        let numbers = extract_checked("ring +1 202 555 0173 today");
        assert_eq!(raws(&numbers), ["+1 202 555 0173"]);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
    }

    #[test]
    fn shortest_length_is_derived_from_the_country_table() {
        // Not hardcoded to 7: it is whatever the table's shortest
        // prefix+national combination happens to be.
        let expected = COUNTRIES
            .iter()
            .flat_map(|c| {
                c.phone_lengths
                    .iter()
                    .map(move |&len| prefix_len(c.prefix) + len as usize)
            })
            .min()
            .unwrap();
        assert_eq!(MIN_DIGITS, expected);
    }

    // --- defect 2: space-separated numbers must not merge -------------------

    #[test]
    fn two_space_separated_numbers_stay_separate() {
        let text = "Call 2025550173 4155552671 now";
        let numbers = extract_checked(text);

        assert_eq!(raws(&numbers), ["2025550173", "4155552671"]);
    }

    #[test]
    fn trailing_short_group_is_backtracked_away() {
        let text = "Call +12025550173 415555 now";
        let numbers = extract_checked(text);

        assert_eq!(numbers[0].raw, "+12025550173");
        assert!(numbers[0].is_valid);
    }

    #[test]
    fn many_numbers_in_one_line_are_all_found() {
        let text = "+12025550173 +12025550174 +12025550175 +12025550176";
        let numbers = extract_checked(text);
        assert_eq!(numbers.len(), 4);
        assert!(numbers.iter().all(|n| n.is_valid));
    }

    // --- defect 3: redaction must not leak digits ---------------------------

    #[test]
    fn redaction_does_not_leak_a_second_number() {
        let redacted = redact("Call 2025550173 4155552671 now", 4);
        assert_eq!(redacted, "Call ******0173 ******2671 now");
        assert!(!redacted.contains("5552671"));
    }

    #[test]
    fn redaction_hides_everything_when_asked() {
        assert_eq!(redact("Call +12025550173 now", 0), "Call [PHONE] now");
        assert_eq!(redact("Call +12025550173 now", 99), "Call [PHONE] now");
        assert_eq!(
            redact("Call +12025550173 now", 11),
            "Call [PHONE] now",
            "visible == total collapses to the placeholder"
        );
    }

    #[test]
    fn redaction_replaces_the_whole_span_including_punctuation() {
        assert_eq!(redact("Call (202) 555-0173 now", 4), "Call ******0173 now");
        assert_eq!(redact("Call (2025550173) now", 0), "Call [PHONE] now");
    }

    #[test]
    fn replacement_rebuilds_surrounding_text() {
        let text = "a +12025550173 b +442079460958 c";
        assert_eq!(replace(text, |_| "[X]".to_string()), "a [X] b [X] c");
        assert_eq!(
            replace("nothing here", |_| "[X]".to_string()),
            "nothing here"
        );
    }

    // --- defect 4: byte offsets ---------------------------------------------

    #[test]
    fn offsets_are_exact_around_multibyte_characters() {
        let cases = [
            "📞 +12025550173 приходи",
            "電話：+81 90 1234 5678 です",
            "e\u{0301}mile: +442079460958",
            "🇺🇸🇬🇧 call +12025550173 or +442079460958 🎉",
        ];
        for text in cases {
            let numbers = extract_checked(text);
            assert!(!numbers.is_empty(), "nothing found in {:?}", text);
        }
    }

    #[test]
    fn randomized_multibyte_corpus_preserves_invariants() {
        let fillers = [
            "a", " ", "  ", "\n", "📞", "電話", "é", "\u{00A0}", "-", ".", "(", ")", "+", "ID:",
            "\u{2013}", "，", "\t", "／",
        ];
        let numbers_pool = [
            "+12025550173",
            "(202) 555-0173",
            "202-555-0173",
            "0645342545",
            "＋８１９０１２３４５６７８",
            "2024-01-15",
            "192.168.1.1",
            "1234567890123456789",
            "06 45 34 25 45",
        ];

        let mut seed = 0x1234_5678_9abc_def0u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };

        for _ in 0..400 {
            let mut text = String::new();
            for _ in 0..24 {
                if next() % 3 == 0 {
                    text.push_str(numbers_pool[(next() % numbers_pool.len() as u64) as usize]);
                } else {
                    text.push_str(fillers[(next() % fillers.len() as u64) as usize]);
                }
            }
            let numbers = extract(&text);
            assert_invariants(&text, &numbers);
            // Replacement slices between the spans; a bad span panics here.
            let _ = replace(&text, |n| format!("<{}>", n.raw.len()));
            let _ = redact(&text, 3);
        }
    }

    // --- defect 5 & 6: parentheses ------------------------------------------

    #[test]
    fn parenthesized_number_keeps_both_parentheses() {
        let numbers = extract_checked("(2025550173)");
        assert_eq!(raws(&numbers), ["(2025550173)"]);
        assert_eq!(redact("(2025550173)", 0), "[PHONE]");
    }

    #[test]
    fn unmatched_open_parenthesis_is_left_out_of_the_span() {
        let numbers = extract_checked("(2025550173 rest");
        assert_eq!(raws(&numbers), ["2025550173"]);
        assert_eq!(redact("(2025550173 rest", 0), "([PHONE] rest");
    }

    #[test]
    fn area_code_in_parentheses_is_kept_whole() {
        let numbers = extract_checked("Call (202) 555-0173 now");
        assert_eq!(raws(&numbers), ["(202) 555-0173"]);
    }

    #[test]
    fn trailing_parenthesized_digit_is_not_swallowed() {
        let text = "Call 2025550173 (555) is the code";
        let numbers = extract_checked(text);

        assert_eq!(raws(&numbers), ["2025550173"]);
        assert_eq!(
            redact(text, 0),
            "Call [PHONE] (555) is the code",
            "surrounding words and parentheses must survive"
        );
    }

    #[test]
    fn parenthesized_area_code_after_a_country_code_is_kept_whole() {
        let numbers = extract_checked("Call +1 (202) 555-0173 now");
        assert_eq!(raws(&numbers), ["+1 (202) 555-0173"]);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
    }

    #[test]
    fn interior_trunk_parenthesis_is_kept() {
        let numbers = extract_checked("+44 (0) 20 7946 0958");
        assert_eq!(raws(&numbers), ["+44 (0) 20 7946 0958"]);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+442079460958"));
    }

    // --- defect 7 & 11: word boundaries -------------------------------------

    #[test]
    fn plus_inside_a_word_does_not_start_a_candidate() {
        assert!(extract_checked("user+12025550173@example.com").is_empty());
        assert!(extract_checked("abc+12025550173").is_empty());
    }

    #[test]
    fn digits_inside_a_word_do_not_start_a_candidate() {
        assert!(extract_checked("abc2025550173").is_empty());
        assert!(extract_checked("build1202555017399").is_empty());
        assert!(extract_checked("v1.2025550173").is_empty());
    }

    /// `.` and `,` block a candidate only when a digit sits in front of them,
    /// i.e. when they are acting as a decimal point or a thousands separator.
    /// Treating them as word characters unconditionally hid every number in a
    /// comma-separated list or a CSV column — and [`redact`] therefore left
    /// those numbers in the clear.
    #[test]
    fn a_dot_or_comma_blocks_only_after_a_digit() {
        // Directly, so both halves of the rule are pinned rather than inferred.
        assert!(!start_is_blocked(Some(','), None));
        assert!(!start_is_blocked(Some(','), Some(' ')));
        assert!(!start_is_blocked(Some(','), Some('e')));
        assert!(start_is_blocked(Some(','), Some('1')));
        assert!(!start_is_blocked(Some('.'), Some('v')));
        assert!(start_is_blocked(Some('.'), Some('1')));
        // Letters, digits, '_' and '+' still block unconditionally.
        for blocker in ['a', 'Z', '7', '_', '+'] {
            assert!(start_is_blocked(Some(blocker), None), "{blocker:?}");
        }
        // Nothing before the candidate never blocks.
        assert!(!start_is_blocked(None, None));

        // And the same rule seen through the extractor.
        assert_eq!(
            raws(&extract_checked("name,+12025550173,x")),
            ["+12025550173"]
        );
        assert_eq!(redact("name,+12025550173,x", 0), "name,[PHONE],x");
        assert_eq!(raws(&extract_checked("a,b,2025550173,c")), ["2025550173"]);
        assert!(extract_checked("1,2025550173").is_empty());
        assert!(extract_checked("v1.2025550173").is_empty());
        assert!(extract_checked("user+12025550173@example.com").is_empty());
    }

    /// The group cap is `+`-aware. A flat cap of [`MAX_GROUPS`] glued the six
    /// groups of `06 45 34 25 45 06` — two French-style numbers side by side —
    /// into a twelve-digit run that normalized to a number never present in the
    /// text.
    #[test]
    fn the_group_cap_is_lower_without_an_explicit_plus() {
        assert_eq!(max_groups(true), MAX_GROUPS);
        assert_eq!(max_groups(false), 5);
        assert!(max_groups(false) < max_groups(true));

        // Six bare groups: the sixth is left out rather than folded in.
        let pair = extract_checked("06 45 34 25 45 06");
        assert_eq!(raws(&pair), ["06 45 34 25 45"]);
        assert_eq!(pair[0].normalized, None);
        assert_eq!(
            extract_with_hint("06 45 34 25 45 06", country("FR"))[0]
                .normalized
                .as_deref(),
            Some("+33645342545")
        );

        // Five bare groups still form one candidate…
        let french = extract_with_hint("06 12 34 56 78", country("FR"));
        assert_eq!(raws(&french), ["06 12 34 56 78"]);
        assert_eq!(french[0].normalized.as_deref(), Some("+33612345678"));

        // …and with a '+' the author has marked where the number begins, so
        // eight groups are allowed.
        let german = extract_checked("+49 (0) 30 12 34 56 78");
        assert_eq!(raws(&german), ["+49 (0) 30 12 34 56 78"]);
        assert_eq!(german[0].normalized.as_deref(), Some("+493012345678"));
    }

    /// `validate` treats an IDD prefix as an international marker only when the
    /// digits behind it actually parse; otherwise the whole run is read as
    /// written. Stripping unconditionally would mangle every national number
    /// that merely begins with those digits.
    #[test]
    fn an_idd_prefix_only_counts_when_the_remainder_parses() {
        assert_eq!(strip_idd_prefix("00442079460958"), Some("442079460958"));
        assert_eq!(strip_idd_prefix("011442079460958"), Some("442079460958"));
        assert_eq!(strip_idd_prefix("0011442079460958"), Some("442079460958"));
        assert_eq!(strip_idd_prefix("00"), None, "nothing behind the prefix");
        assert_eq!(strip_idd_prefix("0111234567"), Some("1234567"));
        assert_eq!(strip_idd_prefix("2025550173"), None);

        // The remainder parses: the number behind the prefix is the answer.
        assert_eq!(
            validate("00442079460958", false, None).as_deref(),
            Some("+442079460958")
        );
        assert_eq!(
            validate("011442079460958", false, None).as_deref(),
            Some("+442079460958")
        );
        // It does not parse — `011` here is the Turin area code — so nothing is
        // claimed rather than a different number being invented.
        assert_eq!(validate("0111234567", false, None), None);
        // With the Italian hint the same digits are Turin, zero and all.
        assert_eq!(
            validate("0111234567", false, country("IT")).as_deref(),
            Some("+390111234567")
        );
    }

    #[test]
    fn word_boundaries_still_allow_normal_punctuation() {
        assert_eq!(raws(&extract_checked("tel:+12025550173")), ["+12025550173"]);
        assert_eq!(raws(&extract_checked("[+12025550173]")), ["+12025550173"]);
        assert_eq!(
            raws(&extract_checked("お電話は+12025550173です")),
            ["+12025550173"],
            "CJK labels run straight into the number"
        );
    }

    // --- defect 8: digit cap -------------------------------------------------

    #[test]
    fn fifteen_digits_is_the_maximum_accepted() {
        let numbers = extract_checked("id +123456789012345 end");
        assert_eq!(raws(&numbers), ["+123456789012345"]);
    }

    #[test]
    fn over_long_runs_are_rejected_not_truncated() {
        for text in [
            "id 1234567890123456 end",  // 16 digits
            "id 12345678901234567 end", // 17 digits
            "id 1234567890123456789012345 end",
        ] {
            let numbers = extract_checked(text);
            assert!(
                numbers.is_empty(),
                "over-long run must not produce a truncated candidate: {:?} -> {:?}",
                text,
                raws(&numbers)
            );
        }
    }

    #[test]
    fn a_number_after_an_over_long_run_is_still_found() {
        let numbers = extract_checked("ref 1234567890123456789 tel +12025550173");
        assert_eq!(raws(&numbers), ["+12025550173"]);
    }

    // --- defect 9: separators -------------------------------------------------

    #[test]
    fn unusual_separators_are_understood() {
        let cases = [
            "+1\u{00A0}202\u{00A0}555\u{00A0}0173", // no-break space
            "+1\u{202F}202\u{202F}555\u{202F}0173", // narrow no-break space
            "+1\u{2013}202\u{2013}555\u{2013}0173", // en dash
            "+1\u{2014}202\u{2014}555\u{2014}0173", // em dash
            "+1\u{2012}202\u{2012}555\u{2012}0173", // figure dash
            "+1\t202\t555\t0173",                   // tab
            "+1  202  555  0173",                   // double spaces
            "+1 - 202 - 555 - 0173",                // spaced hyphens
            "+1\u{3000}202\u{3000}555\u{3000}0173", // ideographic space
        ];
        for text in cases {
            let numbers = extract_checked(text);
            assert_eq!(numbers.len(), 1, "no number found in {:?}", text);
            assert_eq!(
                numbers[0].normalized.as_deref(),
                Some("+12025550173"),
                "wrong parse for {:?}",
                text
            );
            assert_eq!(numbers[0].raw, text);
        }
    }

    #[test]
    fn newlines_and_commas_terminate_a_candidate() {
        let numbers = extract_checked("+12025550173\n+442079460958");
        assert_eq!(raws(&numbers), ["+12025550173", "+442079460958"]);

        let listed = extract_checked("2025550173, 4155552671");
        assert_eq!(raws(&listed), ["2025550173", "4155552671"]);
    }

    // --- defect 10: fullwidth characters ---------------------------------------

    #[test]
    fn fullwidth_digits_and_plus_are_extracted() {
        let text = "お電話は＋８１９０１２３４５６７８までどうぞ";
        let numbers = extract_checked(text);

        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].raw, "＋８１９０１２３４５６７８");
        assert_eq!(numbers[0].normalized.as_deref(), Some("+819012345678"));
    }

    #[test]
    fn fullwidth_separators_and_parentheses_are_understood() {
        let numbers = extract_checked("＋８１（０）９０－１２３４－５６７８");
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+819012345678"));
    }

    #[test]
    fn fullwidth_digits_redact_to_ascii_mask() {
        assert_eq!(redact("＋８１９０１２３４５６７８", 4), "********5678");
    }

    // --- defect 12: minimum length ---------------------------------------------

    #[test]
    fn short_national_numbers_are_reachable_with_a_hint() {
        let numbers = extract_with_hint("849338", country("AD"));
        assert_eq!(numbers.len(), 1);
        assert!(numbers[0].is_valid);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+376849338"));
    }

    #[test]
    fn hint_shortens_the_minimum_only_for_that_country() {
        // Six digits is below the global floor, so without the hint there is
        // nothing to report.
        assert!(extract("849338").is_empty());
    }

    // --- defect 13: the hint must not re-prefix international numbers -----------

    #[test]
    fn hinted_extraction_never_reprefixes_an_international_number() {
        for (text, code) in [
            ("Call +12025550173", "DE"),
            ("Call +12025550173", "FR"),
            ("Call ＋１２０２５５５０１７３", "DE"),
        ] {
            let numbers = extract_with_hint(text, country(code));
            assert_eq!(numbers.len(), 1, "nothing found in {:?}", text);
            assert_eq!(
                numbers[0].normalized.as_deref(),
                Some("+12025550173"),
                "hint {} was prepended to an international number in {:?}",
                code,
                text
            );
        }
    }

    #[test]
    fn idd_prefixes_count_as_international_markers() {
        let numbers = extract_with_hint("Call 0012025550173", country("DE"));
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));

        let nanp_idd = extract_with_hint("Call 011442079460958", country("US"));
        assert_eq!(nanp_idd.len(), 1);
        assert_eq!(nanp_idd[0].normalized.as_deref(), Some("+442079460958"));
    }

    #[test]
    fn hint_beats_an_incidental_self_parse() {
        // The hint is the whole point of the hinted API: it must win over
        // whatever country a bare national number happens to collide with.
        let numbers = extract_with_hint("Call (202) 555-0173", country("US"));
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].normalized.as_deref(), Some("+12025550173"));
    }

    #[test]
    fn hinted_extraction_still_reads_national_numbers() {
        let cases = [
            ("0645342545", "FR", "+33645342545"),
            ("645342545", "FR", "+33645342545"),
            ("06 45 34 25 45", "FR", "+33645342545"),
            ("07911123456", "GB", "+447911123456"),
            ("030 12345678", "DE", "+493012345678"),
            ("0412345678", "AU", "+61412345678"),
            ("(202) 555-0173", "US", "+12025550173"),
            ("202.555.0173", "US", "+12025550173"),
            ("2025550173", "US", "+12025550173"),
        ];
        for (text, code, expected) in cases {
            let numbers = extract_with_hint(text, country(code));
            assert_eq!(numbers.len(), 1, "nothing found for {:?}", text);
            assert_eq!(
                numbers[0].normalized.as_deref(),
                Some(expected),
                "wrong normalization for {:?} with hint {}",
                text,
                code
            );
        }
    }

    #[test]
    fn hinted_extraction_keeps_foreign_international_numbers() {
        let numbers = extract_with_hint("+33645342545 and +12025550173", country("FR"));
        assert_eq!(
            numbers
                .iter()
                .map(|n| n.normalized.as_deref().unwrap())
                .collect::<Vec<_>>(),
            ["+33645342545", "+12025550173"]
        );
    }

    #[test]
    fn missing_hint_behaves_like_plain_extraction() {
        assert_eq!(
            extract_with_hint("+12025550173", None),
            extract("+12025550173")
        );
    }

    // --- defect 14: false positives ---------------------------------------------

    #[test]
    fn iso_dates_are_not_phone_numbers() {
        for text in [
            "2024-01-15",
            "on 2024-01-15 we met",
            "1999-12-31",
            "15-01-2024",
        ] {
            let numbers = extract_checked(text);
            assert!(numbers.is_empty(), "date extracted from {:?}", text);
        }
        // Even a hint whose national lengths would accept the digits.
        assert!(extract_with_hint("2024-01-15", country("DE")).is_empty());
    }

    #[test]
    fn dotted_quads_are_not_phone_numbers() {
        for text in [
            "192.168.1.1",
            "10.0.0.138",
            "255.255.255.0",
            "ping 172.16.254.1 now",
        ] {
            let numbers = extract_checked(text);
            assert!(numbers.is_empty(), "ip extracted from {:?}", text);
        }
        // A dotted US number keeps working: the last group is four digits.
        assert_eq!(raws(&extract_checked("202.555.0173")), ["202.555.0173"]);
    }

    #[test]
    fn isbn13_is_not_a_phone_number() {
        assert!(extract_checked("ISBN 978-3-16-148410-0").is_empty());
        assert!(extract_checked("979-8-6024-9013-0").is_empty());
    }

    #[test]
    fn a_number_after_a_date_is_still_found() {
        let numbers = extract_checked("On 2024-01-15 call +12025550173");
        assert_eq!(raws(&numbers), ["+12025550173"]);
    }

    #[test]
    fn an_explicit_plus_exempts_a_candidate_from_the_shape_filters() {
        let numbers = extract_checked("+2024-01-15");
        assert_eq!(raws(&numbers), ["+2024-01-15"]);
    }

    // --- structural guarantees ---------------------------------------------------

    #[test]
    fn spans_never_include_trailing_separators() {
        for text in ["+12025550173 - ", "+12025550173.", "(202) 555-0173 -- x"] {
            let numbers = extract_checked(text);
            assert!(!numbers.is_empty());
            let raw = &numbers[0].raw;
            let last = raw.chars().next_back().unwrap();
            assert!(
                digit_value(last).is_some() || is_close_paren(last),
                "span {:?} ends on a separator",
                raw
            );
        }
    }

    #[test]
    fn empty_and_degenerate_inputs_are_handled() {
        assert!(extract("").is_empty());
        assert!(extract("+").is_empty());
        assert!(extract("()").is_empty());
        assert!(extract("(((((").is_empty());
        assert!(extract("+++++").is_empty());
        assert!(extract("- . -").is_empty());
        assert_eq!(replace("", |_| "x".to_string()), "");
        assert_eq!(redact("", 4), "");
    }

    #[test]
    fn spaced_single_digits_do_not_form_a_candidate() {
        assert!(extract_checked("1 2 3 4 5 6 7 8 9").is_empty());
    }

    /// The scan is linear in the length of the input: spans are byte offsets
    /// sliced straight out of the source rather than recomputed by counting
    /// characters, and a candidate can never exceed [`MAX_DIGITS`] digits,
    /// [`MAX_GROUPS`] groups or [`MAX_SEPARATOR_RUN`] separators, so the work per
    /// starting position is bounded.
    ///
    /// Complexity is only observable from outside as wall-clock time, so this
    /// runs by default but is deliberately generous. Over an eight-fold increase
    /// in input size a linear scan takes about 8x longer and a quadratic one
    /// about 64x; the bound sits between them with a wide margin, and each size
    /// is measured three times with the fastest run kept, which is what makes
    /// the ratio survive a loaded machine. Run with `--nocapture` to see the
    /// per-doubling figures.
    ///
    /// The output-size assertion is the deterministic half of the test: a scan
    /// that quietly stopped early would keep the timings linear.
    #[test]
    fn extraction_is_linear() {
        use std::time::Instant;

        /// One repetition is ~165 bytes, so 200 of them is ~33 KB and the
        /// largest size below is ~264 KB.
        const UNIT: &str =
            "Contact +1 202 555 0173 or (415) 555-2671, ref 2024-01-15, ip 192.168.1.1. \
             Lorem ipsum dolor sit amet, 電話 ＋８１３１２３４５６７８ 📞 padding text. ";
        const BASE_REPEATS: usize = 200;
        const GROWTH: usize = 8;

        /// Fastest of three runs, in milliseconds, plus how many numbers were
        /// found.
        fn measure(text: &str) -> (f64, usize) {
            let mut best = f64::INFINITY;
            let mut found = 0;
            for _ in 0..3 {
                let started = Instant::now();
                let numbers = extract(text);
                let elapsed = started.elapsed().as_secs_f64() * 1000.0;
                best = best.min(elapsed);
                found = numbers.len();
            }
            (best, found)
        }

        // Warm the caches so the first measurement is not the slowest by
        // accident.
        let _ = extract(&UNIT.repeat(4));

        let mut measurements = Vec::new();
        let mut repeats = BASE_REPEATS;
        while repeats <= BASE_REPEATS * GROWTH {
            let text = UNIT.repeat(repeats);
            let (elapsed, found) = measure(&text);
            println!(
                "{:>7} bytes -> {:>6} numbers in {:>8.3} ms",
                text.len(),
                found,
                elapsed
            );
            measurements.push((repeats, text.len(), elapsed, found));
            repeats *= 2;
        }

        let (base_repeats, _, base_ms, base_found) = measurements[0];
        for &(repeats, bytes, elapsed, found) in &measurements[1..] {
            let factor = repeats / base_repeats;
            // Every repetition contributes the same numbers, so the output grows
            // exactly in step with the input.
            assert_eq!(
                found,
                base_found * factor,
                "{bytes} bytes produced {found} numbers, not {} — the scan did not \
                 cover the whole input",
                base_found * factor
            );
            let ratio = elapsed / base_ms;
            assert!(
                ratio < factor as f64 * 2.5,
                "{factor}x the input took {ratio:.2}x the time; linear is ~{factor}x and \
                 quadratic ~{}x",
                factor * factor
            );
        }
    }
}
