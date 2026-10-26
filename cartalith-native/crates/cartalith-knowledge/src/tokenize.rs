//! The tokenizer every index and every query goes through (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.6.4).
//!
//! # What it does
//!
//! Text becomes a list of lower-case ASCII-folded alphanumeric tokens:
//!
//! 1. Unicode lower-casing, then an ASCII fold of the Latin diacritics (`Kélvhold` and `Kelvhold`
//!    give the same token, so an invented name typed without its accent still finds the note).
//! 2. A split on every non-alphanumeric character.
//! 3. A comma between two digits is dropped, so `8,420` is the one token `8420` and a figure
//!    written with a thousands separator in a note matches the same figure written without one in
//!    a world document. (A full stop is **not** dropped: `0.65` is `0` and `65`, which keeps a
//!    sentence-ending full stop from fusing two numbers.)
//!
//! No stemming (s.6.4: "no stemming in v1") and no synonym table: an invented name has no stem,
//! and a stemmer would conflate real names. Stop words are removed by [`content_tokens`] only,
//! because the query side needs them gone and the index side gains nothing from storing them.
//!
//! # What it must never do
//!
//! * Depend on the locale, the clock, a hash seed or an iteration order: it is a pure function,
//!   which is what makes two runs (and two processes) produce the same ranking.
//! * Panic on any input. It walks `char`s, never byte offsets.
//! * Return an empty token.

/// The query/index stop words: a short English list [judgement, the classic closed-class words;
/// no domain word is on it]. Sorted, so [`is_stopword`] can binary-search it (a test pins the
/// order). **No name, noun or number a player would search for may be added**: every word here is
/// invisible to retrieval.
pub const STOPWORDS: [&str; 61] = [
    "a", "about", "an", "and", "are", "as", "at", "be", "been", "but", "by", "can", "did", "do",
    "does", "for", "from", "had", "has", "have", "how", "i", "if", "in", "is", "it", "its", "me",
    "my", "of", "on", "or", "our", "s", "she", "so", "t", "that", "the", "their", "them", "there",
    "these", "they", "this", "to", "us", "was", "we", "were", "what", "when", "where", "which",
    "who", "whom", "why", "will", "with", "would", "you",
];

/// Whether `token` (already folded and lower-cased) is a stop word.
pub fn is_stopword(token: &str) -> bool {
    STOPWORDS.binary_search(&token).is_ok()
}

/// Appends the ASCII fold of `c` (already lower-cased) to `out`. Characters with no fold are
/// pushed unchanged, so non-Latin alphabets still tokenize (as themselves, lower-cased).
fn push_folded(c: char, out: &mut String) {
    let s: &str = match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c",
        'ď' | 'đ' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => "e",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g",
        'ĥ' | 'ħ' => "h",
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => "i",
        'ĵ' => "j",
        'ķ' => "k",
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l",
        'ñ' | 'ń' | 'ņ' | 'ň' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => "o",
        'ŕ' | 'ŗ' | 'ř' => "r",
        'ś' | 'ŝ' | 'ş' | 'š' => "s",
        'ţ' | 'ť' | 'ŧ' => "t",
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u",
        'ŵ' => "w",
        'ý' | 'ÿ' | 'ŷ' => "y",
        'ź' | 'ż' | 'ž' => "z",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        'þ' => "th",
        _ => {
            out.push(c);
            return;
        }
    };
    out.push_str(s);
}

/// Every token of `text`, in order, stop words included. See the module doc for the rules.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_alphanumeric() {
            for lc in c.to_lowercase() {
                push_folded(lc, &mut cur);
            }
            continue;
        }
        // A comma flanked by digits joins a figure ("8,420" -> "8420"): the token stays open.
        if c == ','
            && i > 0
            && chars[i - 1].is_ascii_digit()
            && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit())
        {
            continue;
        }
        if !cur.is_empty() {
            tokens.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

/// [`tokenize`] without the stop words: what the index stores and what a query is reduced to.
pub fn content_tokens(text: &str) -> Vec<String> {
    tokenize(text).into_iter().filter(|t| !is_stopword(t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects the stop-word table's sort order: `binary_search` on an unsorted table silently
    /// misses words, which would make a stop word a content word for some inputs only. Also pins
    /// the literal length so an edit that adds a word moves this number on purpose.
    #[test]
    fn the_stopword_table_is_sorted_and_has_the_pinned_length() {
        assert!(STOPWORDS.windows(2).all(|w| w[0] < w[1]), "must be strictly sorted");
        assert_eq!(STOPWORDS.len(), 61);
        for w in ["the", "of", "what", "you"] {
            assert!(is_stopword(w), "{w}");
        }
        for w in ["kelvhold", "ashfall", "population", "trade", "port"] {
            assert!(!is_stopword(w), "{w} is a searchable word");
        }
    }

    /// Protects the invented-name path: case and diacritics fold, so the accented and the plain
    /// spelling of a name produce the SAME token. MUTATION: remove the `é` arm of the fold and
    /// the first assert goes red.
    #[test]
    fn diacritics_and_case_fold_to_one_token() {
        assert_eq!(tokenize("Kélvhold"), tokenize("KELVHOLD"));
        assert_eq!(tokenize("Kélvhold"), vec!["kelvhold".to_string()]);
        assert_eq!(tokenize("Straße Œuvre"), vec!["strasse".to_string(), "oeuvre".to_string()]);
    }

    /// Protects the figure rule: a thousands comma joins, a full stop and a sentence comma split.
    /// MUTATION: drop the comma arm and `8,420` becomes two tokens.
    #[test]
    fn a_thousands_comma_joins_and_other_punctuation_splits() {
        assert_eq!(tokenize("8,420 people"), vec!["8420".to_string(), "people".to_string()]);
        assert_eq!(tokenize("0.65"), vec!["0".to_string(), "65".to_string()]);
        assert_eq!(tokenize("a,b 3, 4"), vec!["a".to_string(), "b".to_string(), "3".to_string(), "4".to_string()]);
    }

    /// Protects totality: empty and punctuation-only text give no token (never an empty string),
    /// and a lone surrogate-adjacent or emoji input does not panic.
    #[test]
    fn empty_and_odd_inputs_never_yield_empty_tokens() {
        assert!(tokenize("").is_empty());
        assert!(tokenize("   ...  ,,, ").is_empty());
        assert!(tokenize("🙂 ok 🙂").iter().all(|t| !t.is_empty()));
        assert_eq!(content_tokens("The Ashfall of the North"), vec!["ashfall".to_string(), "north".to_string()]);
    }
}
