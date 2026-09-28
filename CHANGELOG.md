# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.0] - 2026-09-28

A correctness and performance release, measured against Google's
libphonenumber metadata (release v9.0.40) used as an oracle. Validation stays
structural — a plan's lengths, never its range allocations — but the lengths
are now complete, so a real number is no longer rejected for being a freephone,
premium, UAN or other service number, and a number in a newly opened range is
still accepted.

Measured over 35,306 valid numbers generated from libphonenumber's per-type
patterns across all 254 of its regions, 2.0.0 rejected or mis-normalized 4,285
(12%) when written internationally and 4,547 when written nationally with a
country hint. This release rejects 15, all Dominican toll-free numbers whose
exchange starts with `1`, which the NANP's structure forbids. It attributes
every number to the same country libphonenumber does, and still rejects all
51,550 numbers of a length their plan does not use.

### Changed

Results change in ways that are fixes, but will change your output:

- **Numbers in a territory's own block resolve to that territory.** `+7 7…`
  is Kazakhstan, `+44 1481` / `7781` Guernsey, `+44 1534` Jersey, `+44 1624`
  the Isle of Man, `+39 06 698` the Vatican, `+47 79` Svalbard, `+358 18`
  Aland, `+262 269` / `639` Mayotte, `+290 8` Tristan da Cunha, `+599 3` /
  `4` / `7` Bonaire, and so on, as libphonenumber attributes them. Other
  numbers still resolve to the main region, and `extract_countries` lists the
  resolved country first. The prefixes involved only choose which territory is
  reported; they never make a number invalid.
- **`phone_lengths` widened on 95 entries** to every length libphonenumber
  gives any number type of that plan, capped at the E.164 maximum. One entry
  narrowed: Zimbabwe, to the 7, 9 and 10 digits dialable from abroad (its 5-
  and 6-digit forms are local-only dialling, and it has no 8-digit numbers). The per-entry
  comments in `src/constants.rs` explain each change.
- **Type detection uses derived range tables for every region**, replacing the
  hand-written tables for 27 countries, which disagreed with libphonenumber on
  about a third of sampled numbers (Norwegian `800` freephone as fixed line,
  Swiss `68x`/`69x` mobiles as fixed line, and more). Classifications that
  change include UK `084`/`087` (premium rate since Ofcom's 2015 reform,
  formerly shared cost), Korean `050` (personal number, formerly VoIP) and
  Irish `0818` (UAN, formerly shared cost). A number in a block no table
  claims is `Unknown`.
- **National format writes the prefix the country actually dials.** Chinese
  mobiles, Australian `1800` numbers, Lebanese mobiles and others take no trunk
  `0`; Belarus writes `8 0` before geographic numbers only; Gabon writes its
  `0`. 99.6% of national formats now carry the same digits as libphonenumber's,
  up from 89.6%. A number under a non-geographic calling code has no national
  form and is written internationally.
- **`parse_with_country` no longer reads a national number as a foreign one.**
  It used to fall back to reading the digits as whatever calling code they
  spelled, so `("93123456789", "CZ")` became an Afghan number. Only the hint's
  own calling code is recognised without a `+` now.
- `generate_random_phone_number` returns `None` only for `EH` and `GS`; every
  other territory on a shared calling code gets a number from its own block.
- **`countries()` now includes nine entries whose `code` is `"001"`**, not an
  ISO 3166-1 alpha-2 code: the non-geographic calling codes listed under
  **Added**. A number that parsed under 2.0.0 never resolves to one — only
  `+800`, `+870` and similar numbers, which 2.0.0 rejected — but code that
  lists every entry (a country picker, say) or assumes a two-letter code
  should filter them out with `c.code != "001"`.

### Added

- **The ITU's non-geographic calling codes**: `+800`, `+808`, `+870`, `+878`,
  `+881`, `+882`, `+883`, `+888` and `+979`, under the code `"001"` as in
  libphonenumber. Inmarsat numbers (`+870`) validate again.
- **Tristan da Cunha** (`TA`, `+290 8xxx`).
- **Numbers that begin with `0`** are kept whole where the plan has them —
  Congolese and Burkinabe mobiles, Panamanian `00800` and Japanese `0037`
  freephone, old-plan Gabonese numbers — replacing the fixed list of five
  countries. A trunk `0` written after the calling code is still dropped.
- **Trunk prefixes for every plan**, including `8` (Russia, Kazakhstan,
  Belarus, Turkmenistan), `06` (Hungary) and `1` (Marshall Islands), both when
  parsing a national number and when one is written after the calling code
  (`+7 8 916 …`, `+36 06 1 …`).
- **Each country's international access codes** when parsing with a hint:
  `8 10` from Russia and its neighbours, `010` from Japan, the `00xx` carrier
  codes of Brazil and Korea, and more, each ordered so that it is never
  confused with a national number of that country.
- The README's examples are compiled and run as doctests.

### Fixed

- **NANP: N11 exchanges are valid.** Only area codes are reserved in the N11
  form; Montserrat's fixed lines are `664-411-xxxx`, and `1-800-411-7246` is a
  long-running toll-free number. The exchange must still start with 2-9.
- Kazakh numbers were classified with Russia's ranges, so mobiles came back as
  fixed lines.

### Performance

Every table the parser consults is folded into flat arrays at compile time: a
calling code or NANP area code is found by direct index instead of binary
search, and a length check is a bit test. `is_valid_phone_number`,
`extract_country` and `detect_phone_number_type` no longer allocate, and a
`PhoneNumber` holds its E.164 form and its original input in one allocation.
Against 2.0.0 on the same machine: validation 48-60% faster, country
extraction 48-56%, type detection 32%, `parse_with_country` 50%, rejected
parses 30%, `PhoneNumber::parse` 7-16%, and normalization, E.164 and
international formatting and text extraction 4-15%; national formatting, which
now does more, takes the same time. The compiled tables add about 80 KB to a
release binary.

## [2.0.0] - 2026-07-26

A correctness release. An independent audit of 1.2.0 produced 134 findings — 3
critical, 32 high, 49 medium, 38 low, 12 nits — across country data, parsing,
formatting, classification, text extraction, the public API, the documentation
and the build. This release addresses them. The catalogue below is grouped by
what it means for a caller; it is a summary of the audit, not a complete
transcript of it.

Behaviour changed in ways that will change your results. Read **Breaking**
before upgrading.

### Breaking

**Country resolution results changed.** Resolution is now deterministic:
calling codes shared by several countries have a designated primary rather than
whichever entry happened to be declared first.

| Calling code | 1.2.0 | 2.0.0 |
| --- | --- | --- |
| `+44` | `GB-CYM` ("Cymru") | `GB` |
| `+1` | `US` for everything except the 26 four-digit territory prefixes (23 territories); Canadian numbers were among those collapsed into `US` | `US` / `CA` / the correct territory, from the area code |
| `+7` | `KZ` | `RU` |
| `+212` | `EH` | `MA` |
| `+358` | `AX` | `FI` |
| `+262` | `YT` was shadowed by `RE` | `YT` still shadowed by `RE`; use `extract_countries` |
| `+47` | a 10-digit national reached `BV` | `NO` only; `BV` removed |
| `+590` | `BL` | `GP` |
| `+379` | `VA` | rejected with `UnknownCountryCode`; `VA` moved to `+39` |

If you have persisted resolved countries, re-resolve them.

- **Removed the `GB-CYM` ("Cymru") entry.** It is not an ISO 3166-1 code, and
  because it shared calling code `44` and a 10-digit length with the real `GB`
  entry it shadowed it: *every* UK number resolved to "Cymru", which in turn
  made the `GB` arm of type classification and the `GB` national-format table
  unreachable. It was introduced in 0.3.0 and advertised as a feature in 0.3.0
  and 1.0.0. It was a bug.
- **`PhoneNumber`'s fields are private.** Use `original()`, `e164()`,
  `national_number()`, `country()`, `country_code()`, `phone_type()` and
  `extension()`. Public fields allowed a `PhoneNumber` to be built in a state
  its own methods could not survive — `format()` and `national_number()` could
  panic with an out-of-bounds or non-char-boundary slice.
- **`PhoneNumber::country()` and `phone_type()` no longer return `Option`.** A
  parsed number always has both; the `Option` was never `None`, and
  `national_number()` silently returned the whole E.164 string when it was.
- **`PhoneNumber::national_number()` returns `&str`** instead of `String`, and
  **`country_code()` returns `u32`** instead of `Option<u32>`. Both methods
  already existed in 1.2.0, so this breaks callers that never touched the
  fields.
- **`FromStr::Err` is now `ParseError`** instead of `&'static str`.
- **`PhoneNumberSet::find_duplicates` is renamed to `get`.** The old name was a
  misnomer: it returns at most one element and finds no duplicates.
- **`Country`, `PhoneFormat`, `PhoneNumberType`, `ParseError` and
  `PhoneNumberAnalysis` are `#[non_exhaustive]`.** A `match` over the enums now
  needs a wildcard arm, and neither `Country` nor `PhoneNumberAnalysis` can be
  constructed or exhaustively destructured outside the crate.
  `PhoneNumberAnalysis` had all-public fields in 1.2.0, so hand-built test
  fixtures must now come from `analyze_phone_numbers_batch`.
- **`PhoneNumberType` gained `FixedLineOrMobile` and lost `Emergency` and
  `Voicemail`.** North American geographic ranges carry both fixed lines and
  mobiles, and number portability has decoupled the prefix from the technology,
  so `+1 202 555 0173` now reports `FixedLineOrMobile` instead of a guess.
  Countries with no range table now report `Unknown` rather than a first-digit
  guess that was wrong more often than right for CN, IT, BR and others.
  `Emergency` and `Voicemail` were never produced by the classifier in any
  release, but they were public variants: code that matched on or constructed
  them no longer compiles.
- **Normalization changed where a leading zero is part of the number.** It used
  to be stripped everywhere as a trunk prefix. The five plans where it is
  significant — `IT`, `VA` (which shares the Italian plan), `SM`, `BJ` and `CI`
  — now keep it: `normalize_phone_number("+390669821234")` returns
  `"+390669821234"`, not `"+39669821234"`, and San Marino's
  `"+3780549123456"` no longer collapses to `"+378549123456"`. Every other
  country still has its trunk zero stripped; a zero written in the national
  format is not on its own enough to qualify.
- **`PhoneFormat::National` now emits the country's trunk prefix** where it has
  one (`020 7946 0958`, `030 1234 5678`), and short/long national numbers are
  grouped rather than emitted as an undelimited blob.
- **`PhoneFormat::RFC3966` output changed** from a hyphenated
  `tel:+1-202-555-0173` to the conformant global-number form `tel:+12025550173`,
  with `;ext=` when the number carries an extension. The crate can now re-parse
  its own RFC 3966 output; previously it rejected all 252 strings it produced.
- **`PhoneNumber::parse_with_country` rejects an unrecognised hint** with
  `ParseError::UnknownCountry`. Previously a typo'd hint was silently ignored,
  making it *more* permissive than a correct one.
- **`PhoneNumberSet::add` returns `false` for both "already present" and "could
  not be parsed".** Use the new `try_add` to tell them apart.
- **`normalize_phone_number_in_place` leaves the normalized value in the
  buffer** and returns the same string; on failure the buffer is left unchanged.
  Previously it left the bare national number in the buffer on success and
  destroyed the caller's string on failure.
- **A number must now say which country it belongs to.** Input with no `+` and
  no `00` international access code is rejected with the new
  `ParseError::MissingCallingCode` instead of having its leading digits read as
  a calling code. That reading is a guess, and it was frequently wrong:
  `"(202) 555-0173"` is a Washington DC number, but its digits are also a
  well-formed Egyptian `+20 2555 0173`, and only 1.2.0's (incorrect) narrower
  Egyptian length list was hiding it. Supply the `+`, or use
  `PhoneNumber::parse_with_country`. Calling code 1 is exempt, because its area
  code is checked against the numbering plan rather than merely fitting a
  length, so `"12025550173"` still resolves on its own.

  A single leading zero is a national trunk prefix, not an access code, so
  `"0442079460958"` is ambiguous too; `"00442079460958"` is not.

- **Country table entries changed**, from 252 to 249: 7 entries removed and 4
  added.
  - **Removed (7):** `GB-CYM` (`+44`); the uninhabited territories `AQ` (`+672`),
    `BV` (`+47`), `HM` (`+672`) and `TF` (`+262`), whose fabricated
    `phone_lengths` were pure false accepts; `PN` (`+870`, an Inmarsat service
    code rather than a country calling code); and `VA` on `+379`.
  - **Added (4):** Kosovo (`XK`, `+383`); Bonaire, Sint Eustatius and Saba
    (`BQ`, `+599`), previously misattributed to Curaçao; Jamaica's `658` overlay
    (`+1658`); and `VA` on `+39`.
- **Vatican City moved off calling code `+379`.** `+379` is assigned to the Holy
  See by the ITU but has never been brought into service — Vatican numbers are
  dialled through Italy — so the entry now sits on `+39`. Every `+379` number
  that validated in 1.2.0 is now rejected with `ParseError::UnknownCountryCode`.
- **`phone_lengths` changed on 74 of the 245 surviving entries** — 35 widened
  (they accept lengths 1.2.0 rejected, so nothing regresses), 32 narrowed, and 7
  both gained and lost a length. See **Country data** under **Fixed** for what
  drove them, and the bullet below for what the narrowings cost you.
- **Some numbers 1.2.0 accepted are now rejected.** Sweeping every
  (entry, length) pair the 1.2.0 table accepted — and attributing each to the
  country 1.2.0 actually resolved it to, so entries that were already shadowed
  do not count — 388 pairs were accepted by 1.2.0 and 70 of them are rejected by
  2.0.0. Thirteen of those 70 belong to the removed entries and to the `+1`
  range check; the other 57 are ordinary length narrowings spread over 35
  surviving countries:

  `AC`, `AM`, `AR`, `AT`, `BD`, `BE`, `BJ`, `CI`, `CK`, `CN`, `CX`, `CZ`, `FO`,
  `GA`, `GG`, `KP`, `KR`, `LK`, `LY`, `MG`, `MW`, `MY`, `NA`, `NG`, `PM`, `RO`,
  `SB`, `SD`, `SM`, `SS`, `SY`, `TD`, `TM`, `TO`, `VU`

  Most were plainly wrong before — Côte d'Ivoire's national numbers are 10
  digits, not 8 or 9; Czech numbers are 9, not 6, 7 or 8; Romania's are 9, not
  10 — but the effect on a caller is the same. **Re-validate stored numbers for
  these countries before upgrading.** Separately, `+1` numbers whose area code
  was never assigned are now rejected by the NANP range check — see **North
  American Numbering Plan** under **Fixed**.
- **MSRV is declared as 1.74**, replacing the previously declared 1.70, which
  could not be tested at all: the `criterion`
  dev-dependency requires rustc 1.86, so `cargo test` never ran at the declared
  MSRV. The MSRV applies to the library (`cargo check --lib`), not to the
  development tooling.

### Fixed

**Country data**

- The national leading zero was stripped unconditionally, silently rewriting a
  canonical Italian E.164 number into a different, valid, unreachable one, and
  making `are_phone_numbers_equal` declare two genuinely different numbers
  equal. 147 (country, length) combinations across the 1.2.0 table rewrote a
  zero-leading national this way. `IT`, `VA`, `SM`, `BJ` and `CI` now keep it.
- 13 countries — including `CA`, `GB`, `RU` and `MA` — were reachable by no
  input at all, shadowed by another entry sharing their calling code and length.
- Guernsey and Jersey carried `phone_lengths` of `[6]`; real GG/JE national
  numbers are 10 digits.
- Japan rejected every 9-digit landline and freephone number; Romania rejected
  every real Romanian number (the NSN is 9, not 10); Egypt, the UAE, Thailand
  and the Philippines all omitted their landline NSN length.
- The Åland Islands entry swallowed every Finnish number of length 5-10: it
  shared calling code `358` with Finland and was declared first, so it won
  resolution. `FI` is now the designated primary on `+358`, and `AX` no longer
  claims 5-digit numbers.
- Austria's maximum length of 14 allowed a 16-digit number, exceeding the E.164
  15-digit ceiling.
- Jamaica's `658` overlay was missing, so `+1 658` numbers were attributed to
  the United States.
- Those are the headline cases; the pass was wider. `phone_lengths` changed on
  74 entries in total — 35 widened, 32 narrowed, 7 both gained and lost a
  length. The widenings admit real numbers 1.2.0 rejected; the narrowings are
  called out under **Breaking**, because they reject numbers 1.2.0 accepted.

**Parsing and validation**

- `is_valid_phone_number` and `normalize_phone_number` disagreed on a large
  class of inputs; `PhoneNumber::parse` and `PhoneNumberSet::add` accepted
  numbers `is_valid_phone_number` rejected; `analyze_phone_numbers_batch` and
  `validate_phone_numbers_batch` reported contradictory validity for the same
  slice. All of them now run one parser, so they cannot disagree.
- Letters were accepted anywhere in the input, so arbitrary prose validated as a
  phone number. Vanity letters are now only converted when at least three digits
  precede them.
- Non-ASCII digits, bidirectional marks, emoji and variation selectors were
  silently deleted, changing which number was parsed. They are now rejected,
  and fullwidth digits and the fullwidth plus are recognised properly.
- Unhandled extension markers were vanity-converted into digits and appended to
  the subscriber number. `ext`, `ext.`, `ext:`, `extn`, `extension`, a bare `x`,
  `#`, `,`, `;`, `p` and `w`, and RFC 3966's `;ext=` are all recognised, only at
  a word boundary and only when digits follow — so "next" and "text" are safe.
- National-format numbers with a trunk `0` were misrouted to an unrelated
  country, and country resolution matched on the pre-strip length so trunk-zero
  input only worked where a country's length list happened to contain both `L`
  and `L+1`.
- NANP territory numbers were only accepted in the `1 + AAA + 7` shape.
- `PhoneNumber::parse_with_country` force-assigned the hint country onto a
  number whose `normalized`, `phone_type` and `national_number` had been
  computed for a different country, silently corrupting all three.

**North American Numbering Plan**

- `+1` validation was length-only: any 10-digit string was accepted regardless
  of whether its area code was ever assigned. Area codes are now checked against
  a 490-entry table derived from the NANP Administrator's public NPA report, and
  central office codes are checked for structural validity, so `+1 111 111 1111`
  and `+1 202 111 1111` are rejected with `InvalidAreaCode` / `InvalidExchange`.
- NANP-wide service ranges (toll-free, premium rate, the 500-series personal
  numbers, 700, 710) are tagged internally with ISO 3166-1 user-assigned
  elements so that they resolve by *type* rather than by geography, and the
  toll-free arm now covers all eight assigned 8XX SACs (800, 822, 833, 844,
  855, 866, 877, 888) instead of only seven of them. The tags are an internal
  detail: the country table has no non-geographic entry, so `extract_country`
  still reports `US` for these numbers. Treat the type, not the country, as the
  meaningful answer for them.

**Text extraction, redaction and replacement**

- Extraction was quadratic in (input size x match count). On one machine, in a
  release build, over a phone-number-dense corpus, 1.2.0 took 27.9 ms for 32 KB,
  119 ms for 64 KB, 440 ms for 128 KB and 1.73 s for 256 KB — close to 4x per
  doubling. It is now a single linear pass: the same four inputs take 0.85 ms,
  1.5 ms, 2.6 ms and 5.5 ms. Absolute times are machine-dependent; the change in
  shape is not.
- `extract_phone_numbers_with_country_hint` re-prefixed numbers that already
  carried a `+` country code, returning a different real number with
  `is_valid = true`.
- `redact_phone_numbers` leaked digits of a second phone number when two numbers
  were separated by a single space (`"Call [PHONE]2671 now"`).
- Two whitespace-separated numbers merged into one bogus candidate, silently
  dropping the second; a number followed by `" (<digit>"` swallowed the paren and
  digit, so replacement and redaction deleted surrounding words and left a
  dangling `)`; an unmatched `(` was included in `raw`/`start` while `)` was not.
- A `+` embedded in a word, and a `(` start, skipped the left-boundary check and
  produced `is_valid = true` false positives.
- The digit cap admitted 16 digits and truncated 17+ digit runs into a bogus
  16-digit candidate.
- No-break spaces, en dashes, tabs, double spaces and spaced hyphens inside a
  number caused total extraction failure.
- The whole input was materialised as a `Vec<char>`, giving a 6x transient
  allocation peak even for text with no matches.
- `&text[n.start..n.end] == n.raw` now holds for every extracted number, and
  spans never overlap.

**Formatting**

- `PhoneFormat::National` output carried no country calling code and no trunk
  prefix, so it could not be round-tripped. Probing one number per table entry:
  of the 239 entries that were reachable at all, the national form of 202
  re-parsed to a *different* valid E.164 number attributed to a different
  country, and the rest did not re-parse. 2.0.0 emits the trunk prefix and
  documents that `National` must be round-tripped through
  `PhoneNumber::parse_with_country`.
- `PhoneFormat::RFC3966` dropped the extension and could not be re-parsed by the
  crate at all.

**Classification**

- The `GB` arm was dead code (every `+44` number resolved to `GB-CYM`), so 0800
  toll-free numbers classified as `Mobile`.
- Germany's `0800` arm was unreachable because leading zeros were stripped before
  classification, so `+49 800` numbers classified as `FixedLine`.
- France reported 089x premium-rate as `TollFree` and 09xx VoIP as `FixedLine`;
  Australia reported 1300 local-rate as `Unknown` and mapped a nonexistent 188
  range to `TollFree`.

**Generation**

- `generate_random_phone_number("CA")` never produced a number that resolved
  back to `CA` — 100% of 20 000 samples resolved to `US` or a territory. NANP
  area codes are now drawn from the codes assigned to the requested country.
- `generate_random_phone_numbers("US")` emitted ~2.3% Caribbean/Pacific
  territory numbers.
- A zero leading digit was clamped to `'1'` rather than resampled, doubling
  `P(lead = '1')` and producing structurally impossible NANP area codes in 20%
  of US output.
- Generation always used the shortest permitted length, so generated corpora had
  no length diversity.
- The PRNG took its digit from bit 16 up, making digit parity perfectly periodic
  with period 131 072.
- Every returned number is now re-validated and round-tripped before it is
  returned, and `None` is returned rather than an unresolvable number for the 14
  entries whose range cannot be told apart by length from another country
  sharing the calling code (`AX`, `BL`, `CC`, `CX`, `EH`, `GG`, `GS`, `IM`,
  `JE`, `KZ`, `MF`, `SJ`, `VA`, `YT`).

**Suggestions and heuristics**

- `suggest_phone_number_corrections` ranked candidates by byte order before
  truncating to five, so the zero-edit suggestion was discarded whenever the
  input's first digit sorted high; its length-repair branches were gated at
  `>15` / `<10` digits and were hint-only, so ordinary one-digit typos yielded an
  empty list; it omitted the trunk/IDD stripping `parse_with_country` already
  implemented; and with no hint it returned every country whose prefix happened
  to fit. Candidates are now generated one edit-cost tier at a time and the
  search stops at the first tier that produces anything.
- `guess_country_from_number` contradicted `extract_country` for NANP
  territories, never returned `None` for a 10-digit input, and had unreachable
  fallback arms.
- `is_potentially_valid_phone_number` used a hardcoded 7..=15 digit window that
  contradicted the country table in both directions, and returned `false` for
  vanity numbers `is_valid_phone_number` accepted. Its bounds now come from the
  table.

**Panics and resource use**

- `PhoneNumber::format()` and `national_number()` could panic with an
  out-of-bounds or non-char-boundary index.
- `generate_random_phone_numbers` passed an unvalidated `count` straight to
  `Vec::with_capacity`, panicking on `usize::MAX` and aborting the process on
  `1 << 40`.
- `suggest_phone_number_corrections` was quadratic in the digit count with a
  hint, and allocated per iteration without one. Its input is now bounded.

**Performance**

- The country prefix table was a 1940-slot `Vec<Vec<..>>` (one slot per integer
  up to the largest prefix, `1939`) of which only 233 slots were ever populated,
  so 88% of it was empty. Built in a `OnceLock`, it leaked for the process
  lifetime: a 46 560-byte outer allocation plus one heap allocation per
  populated slot. Building it on first use made the first parse hundreds of
  times slower than steady state — 650-860x, measured over three runs on one
  machine. It is now a compile-time sorted array that is binary-searched, with
  no runtime initialization and no first-call cliff.
- `group_equivalent_phone_numbers` was quadratic with two full re-normalizations
  per comparison, and was not reflexive on inputs that failed to normalize. It
  is now a single hash pass, and every input appears exactly once in the output.

**Build and packaging**

- Removed the two `from_utf8_unchecked` sites; the crate is now
  `#![forbid(unsafe_code)]` and `#![warn(missing_docs)]`.
- `Cargo.toml`'s `include` omitted `CHANGELOG.md`, breaking the README's
  changelog link on crates.io. Added.
- Removed `[badges] maintenance`, which crates.io ignores.
- `mod tests;` was ungated at the crate root, safe only via a redundant inner
  `#[cfg(test)]`.

**Documentation**

- 17 of 26 doctests contained no assertion, and 8 of those documented output the
  code contradicted. Every doctest now asserts.
- The README's canonical example number, `+1234567890` / `1234567890`, is not a
  valid phone number at all: 8 snippets printed the opposite of their comments,
  all four formatting examples returned `None`, and the redaction sample had the
  wrong star count. The README was rewritten against the current crate and every
  Rust block in it now compiles and passes as a doctest.
- The README claimed 248 countries against a table of 252, and advertised the
  `GB-CYM` entry as a feature.
- Specific doc claims corrected: `validate_phone_numbers_batch` claimed
  `[true, false]` where the actual result was `[false, false]`;
  `normalize_phone_numbers_batch` claimed two `Some` values where the actual
  result was `[None, None]`; `redact_phone_numbers` claimed separators it never
  emits; `suggest_phone_number_corrections` claimed a suggestion it could never
  return; `generate_random_phone_number` showed a shape the generator could not
  produce; `PhoneFormat`'s variant comments described output the crate never
  produced.
- The crate docs now carry an honest "Accuracy and scope" section: outside the
  NANP validation is length-based only, so a number of plausible length is
  accepted even if its prefix is not in service.

### Added

- **`ParseError` and a `try_*` API.** `try_normalize_phone_number`,
  `PhoneNumber::try_parse`, `PhoneNumber::try_parse_with_country` and
  `PhoneNumberSet::try_add` report *why* a number was rejected: `Empty`,
  `InvalidCharacter`, `TooLong`, `TooShort`, `UnknownCountryCode`,
  `MissingCallingCode`, `InvalidLength`, `InvalidAreaCode`, `InvalidExchange`,
  `UnknownCountry`.
  `ParseError` implements `Display` and `std::error::Error`, and is also
  surfaced on `PhoneNumberAnalysis::error`.
- **Country table access:** `countries()`, `country_by_code()`,
  `countries_by_code()` and `countries_by_calling_code()`. Previously there was
  no way to enumerate the table or look an entry up. ISO 3166-1 alpha-2 codes
  are matched case-insensitively everywhere a country code is accepted, so
  `"gb"` and `"GB"` resolve alike; a code that is genuinely not in the table
  still fails, and fails closed.
- **`extract_countries()`**, returning every country whose numbering plan could
  account for a number, most likely first — the escape hatch for calling codes
  that length alone cannot disambiguate.
- **NANP area-code data** (`490` entries) derived from the NANP Administrator's
  public NPA report, with the non-geographic service ranges tagged separately.
- **Extension support.** Extensions are parsed, preserved on the `PhoneNumber`,
  exposed through `extension()`, and emitted as `;ext=` in RFC 3966 output,
  which re-parses.
- **`PhoneNumberType::FixedLineOrMobile`.**
- **The crate still has no dependencies and no feature flags.** Serialization is
  covered without one: `PhoneFormat::RFC3966` renders a number and any extension
  as a single `tel:` URI that parses straight back, so storing and restoring a
  `PhoneNumber` is one line in each direction and the restore path is the same
  validated parser as everything else.
- **Trait impls:** `Display` for `Country`, `PhoneNumberType` and `ParseError`;
  `Ord`, `PartialOrd`, `AsRef<str>`, `TryFrom<&str>` and `TryFrom<String>` for
  `PhoneNumber`; `PartialEq`, `Eq`, `IntoIterator` and `Extend` for
  `PhoneNumberSet`. (`Default` and `FromIterator` were already there in 1.2.0.)
- **`PhoneNumberSet::try_add`.** (`get` is the `find_duplicates` rename listed
  under Breaking; `remove`, `iter` and `normalized_numbers` already existed in
  1.2.0.)
- **`clippy.toml`** pinning Clippy's MSRV, and **`rustfmt.toml`** pinning the
  formatting configuration.
- **Benchmark coverage** for paths that had none:
  `extract_phone_numbers_with_country_hint`, `redact_phone_numbers`,
  `group_equivalent_phone_numbers`, the parse rejection path, and formatting
  through the free function. The parsing and formatting benchmarks now use a
  fixed, explicit corpus whose every entry is asserted to parse and to resolve
  to the expected country at setup, instead of deriving their inputs by
  filtering out whatever currently fails (which hid 12 of 89 failures in 1.2.0
  and made results incomparable across versions). Text extraction is now
  benchmarked at 2, 32, 64, 128 and 256 KiB; the old suite stopped at 2.2 KB,
  where the quadratic curve was still flat.

## [1.2.0] - 2026-06-07

### Added

- Added the missing NANP overlay area codes so multi-code regions resolve from any of their area codes: Dominican Republic `829` and `849` (in addition to `809`), and Puerto Rico (`787` and `939`), which was previously absent from the dataset

### Fixed

- Fixed `normalize_phone_number()` (and `normalize_phone_number_in_place()`) emitting strings the library itself rejected: stripping a national trunk-prefix zero could shorten a number below a valid length. Normalization now re-validates the national length after stripping and returns `None` instead of a non-idempotent/invalid result
- Fixed `generate_random_phone_number()`/`generate_random_phone_numbers()` producing invalid numbers: removed the bogus `GB` leading-zero special case (which made ~10% of `GB` numbers invalid) so national numbers always start with a non-zero digit, and the generator now re-validates each candidate and retries, guarding against rare collisions (e.g. a `US` national beginning with a Caribbean area code plus a trunk zero)
- Fixed `RFC3966` formatting leaving an orphaned trailing digit (e.g. `tel:+1-202-555-017-3`); it now groups the national number using the same group-size tables as the national/international formats, joined with `-` (e.g. `tel:+1-202-555-0173`)
- Removed dead code in `guess_country_from_number()` (a no-op `continue` branch)
- Fixed country detection for the 22 NANP territories that use a 4-digit dialing prefix (country code 1 plus a 3-digit area code, e.g. `1268` Antigua, `1876` Jamaica). `extract_country()` now resolves these to their own country instead of collapsing them into `US`, and `prefix_digit_count()`/`push_prefix_digits()` handle 4-digit prefixes. Corrected the affected `phone_lengths` entries to the 7 significant digits that follow the prefix
- Fixed `strip_extension()` to only treat `ext`/`ext.` as an extension marker at a word boundary, so words such as "next" or "text" are no longer mistaken for an extension
- Fixed a potential panic in random seed generation when the system clock is set before the Unix epoch

### Performance

- Kept country lookup on the original fast path: codes outside the NANP are prefix-free and resolve shortest-prefix-first in one or two probes, so they match 1.1.3 speed. `+1` numbers add only a few-nanosecond compile-time bitmap test to rule out the common US/Canada case before considering the new 4-digit territory prefixes
- `generate_random_phone_number()` only validates/retries for the bare NANP code (US/CA); every other country is built directly

### Changed

- Documented that the phone-number generator uses a non-cryptographic PRNG suitable only for sample/test data
- Resolved all Clippy lints across the crate, tests, and benches; `cargo clippy --all-targets -- -D warnings` now passes clean

> **Correction (2026-07-25, added in 2.0.0):** the `RFC3966` change above shipped
> as described, but the resulting `tel:+1-202-555-0173` form is not a conformant
> RFC 3966 global number and the crate could not re-parse it. 2.0.0 replaced it
> with `tel:+12025550173`. The 4-digit NANP territory prefixes shipped as
> described, but `+1` resolution as a whole remained length-only until 2.0.0
> added the area-code table.

## [1.1.3] - 2026-03-15

### Fixed

- Fixed `generate_random_phone_numbers()` reseeding logic so batch generation no longer produces repeated values when numbers are created in rapid succession
- Fixed `contains_invalid_character()` so top-level validation accepts leading `+`, reports invalid characters correctly, and stays consistent with normalization
- Fixed `suggest_phone_number_corrections()` repeated country hint lookups inside inner loops
- ~~Fixed `classify_phone_number_type()` for `GB` numbers by matching single-digit prefixes with `d0` and service ranges with `n2`~~ — **withdrawn.** The code changed as described, but it was unreachable: every `+44` number resolved to the `GB-CYM` entry, so the `GB` arm of `classify_phone_number_type` never ran and this fix had no observable effect. GB classification actually started working in 2.0.0, which removed `GB-CYM`.
- Fixed `analyze_phone_numbers_batch()` to derive validation, normalization, country, and type from a single parse pass per input

### Changed

- Extended national formatting with table-driven grouping for `GB`, `DE`, `FR`, `IN`, and `AU`, and replaced the generic split-in-half fallback with length-based grouping
- Wired extension stripping and vanity-letter conversion into the shared normalization path and aligned `extract_country()` with that preprocessing
- Aligned `normalize_phone_number_in_place()` with shared preprocessing for vanity numbers and extension suffixes

> **Correction (2026-07-25, added in 2.0.0):** the `GB` national-formatting table
> added under **Changed** was unreachable for the same reason as the withdrawn
> fix above.

## [1.1.0] - 2026-03-09

### Performance

Major performance optimizations to phone number formatting.

- Eliminated redundant re-normalization in `PhoneNumber::format()` by using cached normalized data
- Replaced all `format!()` macro usage in hot paths with `String::with_capacity()` + `push_str()`
- Optimized country code prefix handling with `#[inline(always)] const fn` digit counting
- Replaced `u32::to_string()` allocations with zero-alloc `push_prefix_digits()` helper
- Optimized `extract_country_data()` with byte-level prefix comparison instead of `str::parse::<u32>()`

### Added

- Added `strip_extension()` for removing extension markers (e.g., "ext. 1234", "ext 987")
- Added `convert_vanity_letters()` for phone keypad letter-to-digit conversion (e.g., "1-800-FLOWERS")
- Added IDD prefix stripping in `parse_with_country()` (handles "0011", "011", "00" prefixes)
- Added trunk prefix handling in `parse_with_country()` (strips leading 0 for national format numbers)
- Added country hint validation - `parse_with_country()` now verifies parsed country matches the hint
- Added Canada (CA) to the country database (prefix 1, 10-digit national numbers)
- Added comparative benchmarks against `rlibphonenumber` and `rust-phonenumber` for parsing and formatting — **shipped as described, then silently removed.** `benches/competitors/parsing_bench.rs` and `benches/competitors/format_bench.rs` really did exist, and `Cargo.toml` really did declare `rlibphonenumber = "1.1.1"` and `phonenumber = "0.3.9"` alongside `criterion` (commits `e550835` and `220ad94`). Commit `356e61a` deleted both files and both dev-dependencies without a changelog entry, while leaving the comparative speed table they justified in the README — where it stayed through 1.1.3 and 1.2.0, quoting measurements nothing in the tree could reproduce. 2.0.0 removes that table from the README. No benchmark comparing this crate to another one exists in the repository today.

### Fixed

- Fixed `parse_with_country()` incorrectly matching "(650) 253-0000" as Singapore instead of US
- Fixed Argentina phone_lengths to include 11-digit mobile numbers

### Changed

- **Country data corrections (18 countries fixed):**
  - Brazil (BR): Added 10-digit landline support (was mobile-only 11 digits)
  - Italy (IT): Expanded phone lengths to [6-11] (was only [10])
  - New Zealand (NZ): Expanded phone lengths to [8, 9, 10] (was only [8])
  - Belgium (BE): Added 8-digit geographic number support
  - China (CN): Expanded phone lengths to [7, 8, 10, 11] (was only [11])
  - Vietnam (VN): Added 10-digit numbers (2018 number expansion)
  - Iran (IR): Corrected phone lengths to [10] (removed incorrect 11)
  - Turkey (TR): Corrected phone lengths to [10] (removed incorrect 11)
  - Reunion (RE): Corrected phone lengths to [9] (was incorrect [10])
  - Libya (LY): Expanded phone lengths to [8, 9, 10] (was only [10])
  - Finland (FI): Expanded phone lengths to [5-12] (was only [9, 11])
  - Taiwan (TW): Added 8-digit landline support (was only 9)
  - Ireland (IE): Expanded phone lengths to [7, 8, 9] (was only [9])
- **Country name updates:**
  - "Swaziland" renamed to "Eswatini"
  - "Macedonia, the Former Yugoslav Republic of" renamed to "North Macedonia"

> **Correction (2026-07-25, added in 2.0.0):** the Reunion (`RE`) correction to
> `[9]` shipped and took effect — `+262` with a 9-digit national resolved to
> `RE` in 1.2.0 and still does in 2.0.0. The entry it left unreachable is
> Mayotte (`YT`), which shares calling code `262` and the same 9-digit length;
> `YT` is still shadowed in 2.0.0, and `extract_countries` is the way to see it.
> The Finland (`FI`) range of 5-12 digits is what let the Åland Islands entry
> swallow every Finnish number, but `FI` was **not** changed in 2.0.0: it is
> still `[5, 6, 7, 8, 9, 10, 11, 12]`, byte for byte, so a 5-digit Finnish
> "number" still validates. What 2.0.0 changed on `+358` is the `AX` entry
> (`[5..10]` to `[6..10]`) and, decisively, the resolution order — `FI` is now
> the designated primary, so it wins every shared length.

## [1.0.1] - 2026-01-12

- Fixed an edge case in phone number normalization for certain country codes

## [1.0.0] - 2026-01-10

### First Stable Release

This release marks the first stable version of phonelib, with a production-ready API.

### Changed (Breaking from 0.3.x)

- **API Improvement**: All functions now accept `&str` instead of `String` for better ergonomics and performance:
  - `is_valid_phone_number(&str)` - no longer requires `.to_string()`
  - `extract_country(&str)` - works directly with string slices
  - `normalize_phone_number(&str)` - accepts borrowed data
  - `format_phone_number(&str, PhoneFormat)` - more flexible input
  - `detect_phone_number_type(&str)` - no allocation needed
  - `is_mobile_number(&str)` - accepts references
  - `is_landline_number(&str)` - accepts references
  - `is_toll_free_number(&str)` - accepts references
  - `are_phone_numbers_equal(&str, &str)` - compare without cloning
  - `suggest_phone_number_corrections(&str, Option<&str>)` - accepts references
  - `is_potentially_valid_phone_number(&str)` - accepts references
  - `guess_country_from_number(&str)` - accepts references

- **API Improvement**: Batch functions now accept `&[T]` where `T: AsRef<str>` instead of `Vec<String>`:
  - `validate_phone_numbers_batch(&[T])` - works with slices of any string-like type
  - `normalize_phone_numbers_batch(&[T])` - no need to create Vec<String>
  - `extract_countries_batch(&[T])` - accepts arrays, slices, or Vecs
  - `detect_phone_number_types_batch(&[T])` - flexible input types
  - `analyze_phone_numbers_batch(&[T])` - works with borrowed data
  - `group_equivalent_phone_numbers(&[T])` - accepts slices

### Added

- **Comprehensive Benchmark Suite**: Added Criterion-based benchmarks covering:
  - Single and batch validation performance
  - Normalization (clean and dirty numbers)
  - Country extraction
  - Phone formatting (E.164, International, National)
  - Type detection (mobile, landline, toll-free)
  - Text extraction from paragraphs
  - Phone number comparison
  - Scaling tests (10, 100, 1000 numbers)

> **Correction (2026-07-25, added in 2.0.0):** this entry originally also listed
> "Added support for Cymru (Wales) with country code `GB-CYM`". That was a
> duplicate of the 0.3.0 entry — nothing was added in 1.0.0 — and the entry it
> duplicated described a bug rather than a feature. See the 0.3.0 note below.

### Migration Guide from 0.3.x

Both snippets below describe historical APIs and are marked as plain text rather
than Rust, because neither compiles against the current crate — the 0.3.x form
never will, and the 1.0.0 form predates the 2.0.0 breaking changes. For a
migration guide that is compiled and tested, see the one in the README.

Before (0.3.x):
```text
is_valid_phone_number("+12025550173".to_string());
validate_phone_numbers_batch(vec!["+12025550173".to_string()]);
```

After (1.0.0):
```text
is_valid_phone_number("+12025550173");
validate_phone_numbers_batch(&["+12025550173"]);
```

## [0.3.0] - 2026-01-10

### Added

- Added support for Cymru (Wales) with country code `GB-CYM`, prefix `44`, and phone length of 10 digits

> **Correction (2026-07-25, added in 2.0.0):** this shipped exactly as described,
> and is the reason it is recorded here rather than deleted — but it was a bug,
> not a feature. `GB-CYM` is not an ISO 3166-1 code, and because the entry shared
> calling code `44` and a 10-digit length with the real `GB` entry it shadowed
> it: from 0.3.0 through 1.2.0 every UK number resolved to "Cymru", and the `GB`
> type-classification and national-formatting tables were unreachable. The entry
> was removed in 2.0.0.

## [0.2.1] - Initial tracked release

- Initial release with phone number validation, formatting, parsing, and manipulation
- Support for international phone numbers with country-specific validation
