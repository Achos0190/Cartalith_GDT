//! The grounding check: what in a reply is backed by the facts, and what is not (LLM-11).
//!
//! WHY: telling a model to use only the facts is a request, not a guarantee. After the reply is
//! complete this module compares it with the text the model was given (the WORLD DATA block, the
//! Vault note excerpts sent, and the place's existing note) and returns two lists the card shows:
//! **facts used** and **unverified claims**. It never blocks the user and never edits the reply;
//! it only shows (owner, 2026-10-26).
//!
//! ## What it checks, and what it cannot
//!
//! * **Numbers** written as digits (`4213`, `4,213`, `4 213`, `12.5`) and numbers written as words
//!   when they are large (`four thousand`, a hundred or more). A number is *matched* when the same
//!   value appears in the corpus, *approximate* when it is the corpus value rounded (`12` for
//!   `12.3`; `4,200` for `4213` within 5 percent and ending in a zero), and *unverified* otherwise.
//!   Small number words (`two gates`) are NOT checked: `one` and `a` are too common in prose.
//! * **Names**: capitalised words. A sentence-initial word is skipped unless it is also in the
//!   corpus (then it is counted as used) or starts a run of capitalised words. A short list of
//!   ordinary capitalised words (months, compass words, generic place nouns such as `River`) is
//!   skipped. A name is matched when the same word (case-insensitive, possessive stripped) occurs
//!   anywhere in the corpus.
//! * **Flavour**: text the model put in single-asterisk italics. The prompt tells it to mark
//!   anything it invents that way, so names and numbers inside such a span are reported as
//!   *marked as flavour*, not as unverified.
//!
//! It cannot judge a *relationship* ("Oakhurst rules Veldmark") or a contradiction in prose; it
//! sees only tokens. The report says so, and the live comparison reads contradictions by hand.
//!
//! MUST NEVER: block, edit or reorder the reply; treat a made-up number as matched because it is
//! small; compare against anything the user was not shown.

/// Words that are capitalised in ordinary prose and say nothing about the world. Labelled
/// judgement; lower-case.
const STOP_NAMES: &[&str] = &[
    "i", "january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november",
    "december", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday", "spring", "summer",
    "autumn", "fall", "winter", "god", "gods", "lord", "lady", "king", "queen", "north", "south", "east", "west",
    "northern", "southern", "eastern", "western", "northeast", "northwest", "southeast", "southwest", "river", "sea",
    "lake", "mountain", "mountains", "hills", "valley", "forest", "road", "gate", "coast", "island", "bay", "port",
    "fort", "castle", "temple", "hall", "market", "square", "harbour", "harbor", "bridge", "street", "tower", "wall",
    "walls", "keep", "guild", "city", "town", "village", "hamlet", "capital", "the", "a", "an", "and", "but", "in",
    "on", "at", "of", "for", "to", "it", "its", "this", "that", "these", "those", "here", "there", "when", "while",
    "as", "by", "with", "from", "each", "every", "some", "many", "most", "all", "no", "not", "if", "then", "so",
    // Section words an entry for a Vault note naturally uses (the owner's own request names them), and the
    // Vault itself; seen as false "names" in the first live run (2026-10-26).
    "vault", "daily", "life", "history", "trade", "faith", "neighbours", "neighbors", "economy", "religion", "geography",
    "defence", "defense", "overview", "summary", "culture", "government", "people", "landscape", "climate",
];

/// Whether `w` (any case) is in [`STOP_NAMES`].
fn is_stop(w: &str) -> bool {
    let l = w.to_lowercase();
    STOP_NAMES.contains(&l.as_str())
}

/// The result of one check.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Grounding {
    /// Numbers (as written in the reply) found in the corpus.
    pub numbers_matched: Vec<String>,
    /// `(as written, corpus value)` for numbers that are a rounding of a corpus value.
    pub numbers_approx: Vec<(String, String)>,
    /// Numbers not found in the corpus.
    pub numbers_unverified: Vec<String>,
    /// Names found in the corpus.
    pub names_matched: Vec<String>,
    /// Names not found in the corpus.
    pub names_unverified: Vec<String>,
    /// Italic spans the model marked as invented flavour.
    pub flavour: Vec<String>,
    /// How many numbers and names were examined.
    pub numbers_checked: usize,
    /// Names examined.
    pub names_checked: usize,
}

impl Grounding {
    /// Count of claims the check could not verify.
    pub fn unverified(&self) -> usize {
        self.numbers_unverified.len() + self.names_unverified.len()
    }
}

/// A number found in text: the characters as written and the value.
#[derive(Debug, Clone, PartialEq)]
struct Num {
    written: String,
    value: f64,
}

fn is_sep(c: char) -> bool {
    matches!(c, ',' | '\u{202f}' | '\u{a0}' | ' ')
}

/// Digit numbers in `text`, skipping list markers (`1.` at a line start), ordinals (`3rd`) and
/// digits glued to letters (`H2O`, `A1`).
fn scan_numbers(text: &str) -> Vec<Num> {
    let c: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if !c[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        let glued_before = i > 0 && c[i - 1].is_alphabetic();
        let mut digits = String::new();
        while i < c.len() && c[i].is_ascii_digit() {
            digits.push(c[i]);
            i += 1;
        }
        // Thousands groups: only after a first group of 1-3 digits, each a separator then exactly
        // three digits then a non-digit (`4,213`, `4 213`, `1,234,567`).
        if digits.len() <= 3 {
            while i + 4 <= c.len()
                && is_sep(c[i])
                && c[i + 1..i + 4].iter().all(char::is_ascii_digit)
                && !c.get(i + 4).is_some_and(char::is_ascii_digit)
            {
                digits.extend(&c[i + 1..i + 4]);
                i += 4;
            }
        }
        let mut written: String = c[start..i].iter().collect();
        let mut value_text = digits.clone();
        if i + 1 < c.len() && c[i] == '.' && c[i + 1].is_ascii_digit() {
            let mut frac = String::from(".");
            i += 1;
            while i < c.len() && c[i].is_ascii_digit() {
                frac.push(c[i]);
                i += 1;
            }
            written = c[start..i].iter().collect();
            value_text.push_str(&frac);
        }
        // Ordinal suffix or a glued word: not a quantity.
        let suffix: String = c[i..].iter().take(2).collect::<String>().to_lowercase();
        if matches!(suffix.as_str(), "st" | "nd" | "rd" | "th") && !c.get(i + 2).is_some_and(|x| x.is_alphabetic()) {
            continue;
        }
        if glued_before {
            continue;
        }
        // List marker: only bullets/hashes/whitespace before it on its line, then `.` or `)` and a space.
        let line_start = c[..start].iter().rposition(|&x| x == '\n').map_or(0, |p| p + 1);
        let only_prefix = c[line_start..start].iter().all(|x| x.is_whitespace() || matches!(x, '#' | '>' | '-' | '*'));
        if only_prefix && c.get(i).is_some_and(|&x| x == '.' || x == ')') && c.get(i + 1).is_some_and(|x| x.is_whitespace()) {
            continue;
        }
        if let Ok(value) = value_text.parse::<f64>() {
            out.push(Num { written: written.trim().to_owned(), value });
        }
    }
    out
}

/// Large numbers spelled as words (`four thousand`, `a hundred and twenty`), value >= 100 only.
fn scan_number_words(text: &str) -> Vec<Num> {
    let words: Vec<String> = text
        .split(|ch: char| !(ch.is_alphabetic() || ch == '-'))
        .filter(|w| !w.is_empty())
        .flat_map(|w| w.split('-').map(str::to_lowercase).collect::<Vec<_>>())
        .collect();
    let small = |w: &str| -> Option<f64> {
        Some(match w {
            "zero" => 0.0, "one" => 1.0, "two" => 2.0, "three" => 3.0, "four" => 4.0, "five" => 5.0, "six" => 6.0,
            "seven" => 7.0, "eight" => 8.0, "nine" => 9.0, "ten" => 10.0, "eleven" => 11.0, "twelve" => 12.0,
            "thirteen" => 13.0, "fourteen" => 14.0, "fifteen" => 15.0, "sixteen" => 16.0, "seventeen" => 17.0,
            "eighteen" => 18.0, "nineteen" => 19.0, "twenty" => 20.0, "thirty" => 30.0, "forty" => 40.0,
            "fifty" => 50.0, "sixty" => 60.0, "seventy" => 70.0, "eighty" => 80.0, "ninety" => 90.0,
            _ => return None,
        })
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let w = words[i].as_str();
        let starts = small(w).is_some() || matches!(w, "hundred" | "thousand" | "million") || (w == "a" && words.get(i + 1).is_some_and(|n| matches!(n.as_str(), "hundred" | "thousand" | "million")));
        if !starts {
            i += 1;
            continue;
        }
        let begin = i;
        let (mut total, mut cur, mut big) = (0.0f64, 0.0f64, false);
        let mut j = i;
        while j < words.len() {
            let w = words[j].as_str();
            if let Some(v) = small(w) {
                cur += v;
            } else if w == "a" && words.get(j + 1).is_some_and(|n| matches!(n.as_str(), "hundred" | "thousand" | "million")) {
                cur = 1.0;
            } else if w == "hundred" {
                cur = cur.max(1.0) * 100.0;
                big = true;
            } else if w == "thousand" || w == "million" {
                total += cur.max(1.0) * if w == "thousand" { 1_000.0 } else { 1_000_000.0 };
                cur = 0.0;
                big = true;
            } else if w == "and" && words.get(j + 1).is_some_and(|n| small(n).is_some() || matches!(n.as_str(), "hundred" | "thousand")) && big {
                // "a hundred and twenty": keep going.
            } else {
                break;
            }
            j += 1;
        }
        if big && total + cur >= 100.0 {
            out.push(Num { written: words[begin..j].join(" "), value: total + cur });
        }
        i = j.max(i + 1);
    }
    out
}

/// Replace `text`'s single-asterisk italic spans with spaces and return them. `**bold**` is not
/// a span. A span is on one line and at most 240 characters.
fn split_flavour(text: &str) -> (String, Vec<String>) {
    let c: Vec<char> = text.chars().collect();
    let mut kept = String::with_capacity(text.len());
    let mut spans = Vec::new();
    let mut i = 0;
    while i < c.len() {
        let single = c[i] == '*' && c.get(i + 1) != Some(&'*') && (i == 0 || c[i - 1] != '*');
        if single {
            if let Some(rel) = c[i + 1..].iter().take(240).position(|&x| x == '*' || x == '\n') {
                if c[i + 1 + rel] == '*' && c.get(i + 2 + rel) != Some(&'*') && rel > 0 {
                    spans.push(c[i + 1..i + 1 + rel].iter().collect::<String>().trim().to_owned());
                    kept.push(' ');
                    i += rel + 2;
                    continue;
                }
            }
        }
        kept.push(c[i]);
        i += 1;
    }
    (kept, spans)
}

/// Lower-case alphabetic words of the corpus, with possessives stripped and hyphenated parts
/// added, as a sorted unique list.
fn corpus_words(corpus: &str) -> std::collections::BTreeSet<String> {
    let mut set = std::collections::BTreeSet::new();
    for raw in corpus.split(|ch: char| !(ch.is_alphabetic() || ch == '\'' || ch == '\u{2019}' || ch == '-')) {
        let w = raw.trim_matches(|ch| ch == '\'' || ch == '\u{2019}' || ch == '-').to_lowercase();
        if w.is_empty() {
            continue;
        }
        let w = w.trim_end_matches("'s").trim_end_matches("\u{2019}s").to_owned();
        for part in w.split('-') {
            if !part.is_empty() {
                set.insert(part.to_owned());
            }
        }
        set.insert(w);
    }
    set
}

/// Capitalised-word candidates in `text`: `(word, sentence_initial, followed_by_capital)`.
fn scan_names(text: &str) -> Vec<(String, bool, bool)> {
    let c: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if !c[i].is_alphabetic() {
            i += 1;
            continue;
        }
        let start = i;
        while i < c.len() && (c[i].is_alphabetic() || ((c[i] == '\'' || c[i] == '\u{2019}' || c[i] == '-') && c.get(i + 1).is_some_and(|x| x.is_alphabetic()))) {
            i += 1;
        }
        let word: String = c[start..i].iter().collect();
        if !word.chars().next().is_some_and(char::is_uppercase) || word.chars().count() < 2 {
            continue;
        }
        // A heading or bold lead-in line ("## History", "**Daily Life**") names a topic, not a place.
        let line_start = c[..start].iter().rposition(|&x| x == '\n').map_or(0, |p| p + 1);
        let line_end = c[start..].iter().position(|&x| x == '\n').map_or(c.len(), |p| start + p);
        let line: String = c[line_start..line_end].iter().collect();
        let line = line.trim();
        if line.starts_with('#') || (line.starts_with("**") && line.split_whitespace().count() <= 6 && !line.trim_matches('*').ends_with(['.', '!', '?'])) {
            continue;
        }
        // Sentence-initial: walk back over spaces, quotes, markdown marks, bullets.
        let mut k = start;
        let initial = loop {
            if k == 0 {
                break true;
            }
            let p = c[k - 1];
            if matches!(p, ' ' | '\t' | '"' | '\u{201c}' | '\u{2018}' | '(' | '*' | '_' | '>' | '-' | '#') {
                k -= 1;
                continue;
            }
            break matches!(p, '.' | '!' | '?' | '\n' | '\u{2026}');
        };
        // Followed by another capitalised word after exactly one space?
        let mut n = i;
        let mut spaces = 0;
        while n < c.len() && c[n] == ' ' {
            n += 1;
            spaces += 1;
        }
        let next_cap = spaces == 1 && c.get(n).is_some_and(|x| x.is_uppercase());
        // Strip a possessive for the lookup.
        let clean = word.trim_end_matches("'s").trim_end_matches("\u{2019}s").to_owned();
        out.push((clean, initial, next_cap));
    }
    out
}

/// Compare `reply` against `corpus` (see the module docs). `corpus` is every text the model was
/// shown plus the place's existing note.
pub fn check(reply: &str, corpus: &str) -> Grounding {
    let (plain, spans) = split_flavour(reply);
    let mut g = Grounding { flavour: spans.iter().filter(|s| !s.is_empty()).cloned().collect(), ..Default::default() };
    // ---- numbers ----
    let corpus_nums: Vec<f64> = scan_numbers(corpus).into_iter().map(|n| n.value).collect();
    let mut seen: Vec<String> = Vec::new();
    let mut reply_nums = scan_numbers(&plain);
    reply_nums.extend(scan_number_words(&plain));
    for n in reply_nums {
        let key = format!("{}", n.value);
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        g.numbers_checked += 1;
        if corpus_nums.iter().any(|v| (v - n.value).abs() < 1e-6) {
            g.numbers_matched.push(n.written);
            continue;
        }
        // Rounding of a corpus figure.
        // A rounding is `v` rounded to a whole number, or to one or two significant figures
        // (4213 -> 4200 or 4000; 612 -> 610 or 600). A tolerance band (within 5 percent) was tried
        // first and matched an INVENTED 5300 to an unrelated 5380.4 in the corpus, so the rule is the
        // rounding itself: 5380.4 rounds to 5400 and 5000, never 5300.
        let approx = corpus_nums.iter().copied().find(|&v| {
            let int_round = n.value.fract() == 0.0 && v.fract() != 0.0 && n.value == v.round();
            let sig = |digits: i32| -> f64 {
                if v <= 0.0 {
                    return v;
                }
                let mag = 10f64.powi(v.log10().floor() as i32 + 1 - digits);
                (v / mag).round() * mag
            };
            let round_big = n.value >= 100.0 && n.value % 10.0 == 0.0 && v > 0.0 && (n.value == sig(1) || n.value == sig(2));
            int_round || round_big
        });
        match approx {
            Some(v) => g.numbers_approx.push((n.written, trim_num(v))),
            None => g.numbers_unverified.push(n.written),
        }
    }
    // ---- names ----
    let words = corpus_words(corpus);
    let mut seen_names: Vec<String> = Vec::new();
    for (w, initial, next_cap) in scan_names(&plain) {
        let low = w.to_lowercase();
        let in_corpus = words.contains(&low) || w.split('-').all(|p| words.contains(&p.to_lowercase()));
        if is_stop(&w) {
            continue;
        }
        // An ordinary sentence opener is neither used nor unverified, unless the corpus has it
        // (then it is the place's own name at the start of a sentence) or it opens a name run.
        if initial && !next_cap && !in_corpus {
            continue;
        }
        if seen_names.contains(&low) {
            continue;
        }
        seen_names.push(low);
        g.names_checked += 1;
        if in_corpus {
            g.names_matched.push(w);
        } else {
            g.names_unverified.push(w);
        }
    }
    // Flavour spans are reported as they are; their names and numbers are deliberately not checked.
    g
}

/// A number printed without a trailing `.0`.
fn trim_num(v: f64) -> String {
    if v.fract() == 0.0 { format!("{}", v as i64) } else { format!("{v}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FACTS: &str = "[The place]\nname: Veldmark\npopulation: 4213\nsize class: city\n- Oakhurst (village; pop 612; 12.3 km E; faith Sunfaith)\nfaction: Kingdom of Vael\nwalls: stone\nGarrison 180";

    /// Protects the owner's bar: an INVENTED number is flagged, the engine's number is matched.
    /// MUTATION: make `check` treat every number as matched and the first assert goes red.
    #[test]
    fn invented_number_is_flagged_and_real_is_matched() {
        let g = check("Veldmark holds 5300 souls and 180 soldiers.", FACTS);
        assert_eq!(g.numbers_unverified, ["5300"]);
        assert_eq!(g.numbers_matched, ["180"]);
        let g = check("About 4,213 people live in Veldmark.", FACTS);
        assert_eq!(g.numbers_matched, ["4,213"], "{g:?}");
        assert!(g.numbers_unverified.is_empty());
    }

    /// Protects: a rounding of a real figure is "approximate", not unverified and not exact: the
    /// figure rounded to a whole number or to 1 or 2 significant figures. 3,000 is not a rounding of
    /// 4213 (that is 4,000 or 4,200) and 13 is not a rounding of 12.3.
    /// MUTATION: add a third significant figure, or loosen the whole-number rule, and a literal flips.
    #[test]
    fn roundings_are_approximate() {
        let g = check("Some 4,200 people, 12 km east, and 3,000 elsewhere.", FACTS);
        assert_eq!(g.numbers_approx, [("4,200".into(), "4213".into()), ("12".into(), "12.3".into())], "{g:?}");
        assert_eq!(g.numbers_unverified, ["3,000"]);
        // 13 is not a rounding of 12.3.
        assert_eq!(check("13 km", FACTS).numbers_unverified, ["13"]);
        // The rule is deliberately narrow: a three-figure rounding of 4213 (4,210) is not accepted.
        assert_eq!(check("about 4,210", FACTS).numbers_unverified, ["4,210"]);
        // The live-probe finding: an INVENTED 5300 must not hide behind an unrelated 5380.4.
        let g = check("a town of 5300 people", "elevation: 5380.4 m");
        assert_eq!(g.numbers_unverified, ["5300"], "{g:?}");
        // ...while the true rounding of that figure is accepted.
        assert_eq!(check("about 5400 m", "elevation: 5380.4 m").numbers_approx, [("5400".into(), "5380.4".into())]);
    }

    /// Protects the owner's bar: a MADE-UP town name is flagged; the real neighbour is matched;
    /// ordinary sentence openers and generic nouns are not flagged.
    /// MUTATION: drop the corpus lookup, or flag sentence openers, and a literal changes.
    #[test]
    fn made_up_name_is_flagged() {
        let g = check("The town of Korvash lies east. Oakhurst is nearer. Trade follows the River road.", FACTS);
        assert_eq!(g.names_unverified, ["Korvash"], "{g:?}");
        assert!(g.names_matched.contains(&"Oakhurst".to_owned()), "{g:?}");
        assert!(!g.names_unverified.contains(&"The".to_owned()) && !g.names_unverified.contains(&"Trade".to_owned()));
        assert!(!g.names_unverified.contains(&"River".to_owned()));
        // A sentence opener that is no stop word and not in the corpus is neither used nor unverified.
        let g = check("Rainfall is heavy. Oakhurst floods.", FACTS);
        assert!(g.names_unverified.is_empty() && g.names_matched == ["Oakhurst"], "{g:?}");
    }

    /// Protects: the possessive is stripped; a run of capitalised words at a sentence start is
    /// checked word by word; the place's own name opening a sentence counts as used.
    #[test]
    fn possessives_and_runs() {
        let g = check("Veldmark's walls are old. Port Veldmark and Highmoor Keep trade.", FACTS);
        assert!(g.names_matched.contains(&"Veldmark".to_owned()));
        assert_eq!(g.names_unverified, ["Highmoor"], "{g:?}");
    }

    /// Protects: italic spans are the model's declared flavour and are not counted as unverified;
    /// `**bold**` is not a span.
    /// MUTATION: skip `split_flavour` and Zorrun / 77 become unverified.
    #[test]
    fn italic_flavour_is_reported_not_flagged() {
        let g = check("Veldmark is a city. *Locals tell of the hermit Zorrun, who lived 77 winters.* Fish go to **Mara** daily.", FACTS);
        assert_eq!(g.flavour, ["Locals tell of the hermit Zorrun, who lived 77 winters."]);
        assert!(!g.names_unverified.contains(&"Zorrun".to_owned()) && g.numbers_unverified.is_empty(), "{g:?}");
        assert!(g.names_unverified.contains(&"Mara".to_owned()), "bold is not flavour: {g:?}");
    }

    /// Protects (first live run): section headings and bold lead-ins are topics, not names, but a
    /// capitalised name inside a normal sentence is still checked, and a made-up one in a bold lead-in
    /// sentence that ends with a full stop is still flagged.
    /// MUTATION: drop the heading rule and "Daily" and "Zorrunhold" appear as claims.
    #[test]
    fn headings_are_not_names() {
        let g = check("## Zorrunhold\n**Daily Life**\nVeldmark is a city. Trade goes to Korvash.\n**Life in Zorrunhold is hard.**", FACTS);
        assert_eq!(g.names_unverified, ["Korvash", "Zorrunhold"], "{g:?}");
    }

    /// Protects: list markers, ordinals and glued digits are not quantities.
    #[test]
    fn markers_are_not_numbers() {
        let g = check("1. First point\n2) Second\nThe 3rd gate and H2O and 42 cats", FACTS);
        assert_eq!(g.numbers_unverified, ["42"], "{g:?}");
    }

    /// Protects: large numbers in words are read; small ones are not.
    /// MUTATION: lower the 100 threshold and "two gates" becomes a claim.
    #[test]
    fn number_words() {
        let g = check("About four thousand two hundred souls and two gates.", FACTS);
        assert_eq!(g.numbers_approx, [("four thousand two hundred".into(), "4213".into())], "{g:?}");
        assert!(g.numbers_unverified.is_empty());
        let g = check("A hundred and twenty guards.", FACTS);
        assert_eq!(g.numbers_unverified, ["a hundred and twenty"], "{g:?}");
    }

    /// Protects: decimals and thousands groups parse to the same value.
    #[test]
    fn number_shapes() {
        let v: Vec<f64> = scan_numbers("4,213 and 4 213 and 4213 and 12.5 and 1,234,567").into_iter().map(|n| n.value).collect();
        assert_eq!(v, [4213.0, 4213.0, 4213.0, 12.5, 1_234_567.0]);
    }
}
