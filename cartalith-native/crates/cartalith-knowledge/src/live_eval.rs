//! The live-evaluation scorer and report (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8, milestone RK-8).
//!
//! RK-0's [`crate::eval::score`] measures *retrieval* with no model. This module measures what a
//! *model* did with the retrieval: it takes one [`RunRecord`] (the question, what the app retrieved
//! for the model, what the model answered, and the cost of getting there) and turns it into a
//! [`RunScore`]; [`aggregate`] pools the runs of one model and mode and reports each metric with its
//! spread over the repeats; [`render_report`] writes the dated report's tables.
//!
//! # Where the records come from
//!
//! `godot-project/_llmknow_eval_cli.gd` runs the app's own path (real gate, conversation, index and
//! wire) once per question and writes one JSON line per run. This module never talks to a model: it
//! is a pure function of those lines, so the report is reproducible from the files alone and the
//! scorer is `cargo test`-sized. The same driver can be pointed at the in-test mock server, which is
//! how the scorer is validated end to end without a model (the controls below).
//!
//! # What each metric is, honestly
//!
//! None of these is a human or model judgement; each is a deterministic proxy and says so.
//!
//! * **recall@5 / recall@10** - the share of the question's gold ids found in the first 5 / 10 ids the
//!   app retrieved for the model (the *ledger*, in order of first retrieval; a pre-retrieval pack
//!   comes first, then each tool call). It measures what the model was given, not what it used.
//! * **correct** - every `expect` literal of the question appears in the answer ([`contains_literal`]:
//!   in order, adjacent, case and punctuation ignored). `expect share` is the fraction present, so a
//!   half-listed set of trade partners is visible.
//! * **grounded** - the answer asserts no figure the model was not given. [`unsupported_numbers`]
//!   counts figures of [`FIGURE_MIN_DIGITS`] or more digits in the answer that occur in neither the
//!   text the model was handed nor the question; and every `expect` literal that does appear in the
//!   answer must also appear in that text. A model that guesses the right number without being given
//!   it is *not* grounded. This is a proxy: it cannot see an invented word, only an invented figure
//!   or a right answer with no source.
//! * **citation verified** - the share of `[W:..]` / `[V:..]` citations in the answer that the app
//!   checked against its ledger (the app's own `verified` flag). **citation on gold** - the share
//!   that name a gold source of the question ([`id_matches`]). A run with no citation is counted in
//!   `uncited`, never scored 100% by default.
//! * **refusal** - on an unanswerable question, the answer carries one of [`REFUSAL_MARKERS`] and
//!   asserts no unsupported figure. The marker list is a labelled judgement (a model can refuse in
//!   words the list does not know); the controls below pin that a plain refusal passes and an
//!   invented answer fails.
//! * **leak** - a `forbid` canary of the question (a private note's secret) appears in the answer or in
//!   anything the model was handed. Must be 0; a leak is counted per run, never averaged away.
//!
//! # What it must never do
//!
//! Invent a pass rate (the module reports numbers; the owner sets the bar afterwards, s.8 RK-8), encode
//! "not measured" as a plausible value (a latency or token count the server did not report is `None`
//! and drops out of the median, and a metric with no applicable run prints `-`), read a clock, or
//! depend on hash-map order.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::eval::{contains_literal, Question, CATEGORIES};
use crate::model::id_matches;

/// The fewest digits for a number in an answer to count as a *figure* that needs a source. Three
/// digits and up covers years (`1204`), populations and census counts, and leaves "top 5", "two of 11"
/// and a list's own numbering alone. Labelled judgement; mutation: set to 1 and the numbered-list
/// test in this module goes red.
pub const FIGURE_MIN_DIGITS: usize = 3;

/// Phrases that mark an answer as a refusal ("I could not find ..."). A labelled judgement, lower
/// case, matched as substrings of the lower-cased answer. Deliberately specific phrases: a bare
/// "not" would pass any sentence containing it.
pub const REFUSAL_MARKERS: [&str; 14] = [
    "do not have",
    "don't have",
    "does not appear",
    "doesn't appear",
    "no information",
    "no record",
    "not mentioned",
    "not in the",
    "cannot find",
    "can't find",
    "could not find",
    "couldn't find",
    "no such",
    "not found",
];

/// One citation found in a model's answer, as the app's panel recorded it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationRec {
    /// The id inside the brackets.
    pub id: String,
    /// The app's verdict: the id is in the ledger of what it retrieved for this request.
    pub verified: bool,
}

/// Everything one run left behind. One JSON line per run in the driver's output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRecord {
    /// The model's display id (a report column; the driver does not interpret it).
    pub model: String,
    /// `tools`, `pre` or `both`: how the knowledge base reached the model.
    pub mode: String,
    /// The world seed the question was generated for.
    pub seed: i64,
    /// Which repeat of the same question this is (0-based).
    pub repeat: u32,
    /// The question with its ground truth, embedded so a record is self-describing.
    pub question: Question,
    /// `answer`, `failed` or `cancelled`.
    pub outcome: String,
    /// The model's final answer, empty unless `outcome` is `answer`.
    pub answer: String,
    /// The ids the app retrieved for the model, in order of first retrieval.
    pub ledger: Vec<String>,
    /// The citations in the answer, checked by the app.
    pub citations: Vec<CitationRec>,
    /// The retrieved text the model was handed: the pre-retrieval pack and each tool result.
    pub context: Vec<String>,
    /// The tool names the model called, in order.
    pub tool_calls: Vec<String>,
    /// Wall-clock milliseconds from sending to the end of the run, or `None` if not measured.
    pub latency_ms: Option<u64>,
    /// Prompt tokens summed over the run's requests, or `None` if the server reported none.
    pub prompt_tokens: Option<u64>,
    /// Completion tokens summed over the run's requests, or `None` if the server reported none.
    pub completion_tokens: Option<u64>,
    /// Why a run that did not answer did not (the app's refusal or the failure message), if given.
    #[serde(default)]
    pub reason: Option<String>,
}

/// The scored form of one [`RunRecord`]. Every `Option` is `None` where the metric does not apply to
/// the question (no gold, no expected literal, not an unanswerable question) and never `Some(0.0)`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunScore {
    /// The run ended in an answer.
    pub answered: bool,
    /// Share of gold ids in the first 5 ledger ids.
    pub recall5: Option<f64>,
    /// Share of gold ids in the first 10 ledger ids.
    pub recall10: Option<f64>,
    /// Share of `expect` literals present in the answer.
    pub expect_share: Option<f64>,
    /// Every `expect` literal is in the answer.
    pub correct: Option<bool>,
    /// Figures in the answer found nowhere in the context or the question.
    pub unsupported_numbers: usize,
    /// The answer is grounded (module doc). `None` when the run did not answer.
    pub grounded: Option<bool>,
    /// Citations in the answer.
    pub cited: usize,
    /// Of those, the ones the app verified.
    pub verified: usize,
    /// Of those, the ones naming a gold source.
    pub on_gold: usize,
    /// The answer refused (unanswerable questions only).
    pub refused: Option<bool>,
    /// A forbidden canary reached the answer or the context.
    pub leak: bool,
}

/// The digit runs of [`FIGURE_MIN_DIGITS`] or more in `text`. A run joined to letters (`ab1204`) is a
/// name fragment, not a figure, and a comma or a space inside a number (`6,115`, `6 115`) is read as
/// a break, so a figure written with a separator is two shorter runs: the strict reading, documented
/// because it can only ever *under*-count figures.
pub fn figures(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut run = String::new();
    // The character before the current digit run began, if it was a letter.
    let mut glued = false;
    let mut before: Option<char> = None;
    for c in text.chars().chain(std::iter::once(' ')) {
        if c.is_ascii_digit() {
            if run.is_empty() {
                glued = before.is_some_and(char::is_alphabetic);
            }
            run.push(c);
        } else {
            if run.len() >= FIGURE_MIN_DIGITS && !glued {
                out.push(std::mem::take(&mut run));
            }
            run.clear();
        }
        before = Some(c);
    }
    out
}

/// Figures in `answer` that appear in none of `context` and not in `question`.
pub fn unsupported_numbers(answer: &str, context: &[String], question: &str) -> usize {
    let mut known: Vec<String> = figures(question);
    for c in context {
        known.extend(figures(c));
    }
    figures(answer).into_iter().filter(|f| !known.contains(f)).count()
}

/// Whether `answer` reads as a refusal ([`REFUSAL_MARKERS`]).
pub fn is_refusal(answer: &str) -> bool {
    let a = answer.to_lowercase();
    REFUSAL_MARKERS.iter().any(|m| a.contains(m))
}

fn recall_at(gold: &[String], ids: &[String], k: usize) -> Option<f64> {
    if gold.is_empty() {
        return None;
    }
    let top = &ids[..ids.len().min(k)];
    Some(gold.iter().filter(|g| top.iter().any(|i| id_matches(g, i))).count() as f64 / gold.len() as f64)
}

/// Scores one run (module doc).
pub fn score_run(r: &RunRecord) -> RunScore {
    let q = &r.question;
    let answered = r.outcome == "answer";
    let context_text = r.context.join("\n");
    let unsupported = if answered { unsupported_numbers(&r.answer, &r.context, &q.text) } else { 0 };
    let (expect_share, correct) = if q.expect.is_empty() {
        (None, None)
    } else {
        let present = q.expect.iter().filter(|e| answered && contains_literal(&r.answer, e)).count();
        (Some(present as f64 / q.expect.len() as f64), Some(present == q.expect.len()))
    };
    let grounded = answered.then(|| {
        let right_without_source = q.expect.iter().any(|e| contains_literal(&r.answer, e) && !contains_literal(&context_text, e) && !contains_literal(&q.text, e));
        unsupported == 0 && !right_without_source
    });
    let on_gold = r.citations.iter().filter(|c| q.gold.iter().any(|g| id_matches(g, &c.id))).count();
    let leak = q.forbid.iter().any(|f| {
        let lf = f.to_lowercase();
        r.answer.to_lowercase().contains(&lf) || context_text.to_lowercase().contains(&lf) || r.ledger.iter().any(|i| i.to_lowercase().contains(&lf))
    });
    RunScore {
        answered,
        recall5: recall_at(&q.gold, &r.ledger, 5),
        recall10: recall_at(&q.gold, &r.ledger, 10),
        expect_share,
        correct,
        unsupported_numbers: unsupported,
        grounded,
        cited: r.citations.len(),
        verified: r.citations.iter().filter(|c| c.verified).count(),
        on_gold,
        refused: (q.category == "unanswerable").then(|| answered && is_refusal(&r.answer) && unsupported == 0),
        leak,
    }
}

/// A metric over the repeats: the median of the per-repeat values with their range. `repeats` is how
/// many repeats had the metric at all, so a single repeat shows `min == max` and says `n = 1` in the
/// report instead of looking like a tight spread.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Spread {
    /// Median of the per-repeat values (the mean of the middle two for an even count).
    pub median: f64,
    /// Smallest per-repeat value.
    pub min: f64,
    /// Largest per-repeat value.
    pub max: f64,
    /// Number of repeats that contributed.
    pub repeats: usize,
}

impl Spread {
    /// The spread of `values`, or `None` for an empty list (never a spread of zeros).
    pub fn of(values: &[f64]) -> Option<Spread> {
        if values.is_empty() {
            return None;
        }
        let mut v = values.to_vec();
        v.sort_by(f64::total_cmp);
        let n = v.len();
        let median = if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 };
        Some(Spread { median, min: v[0], max: v[n - 1], repeats: n })
    }

    /// `median (min..max)` with `digits` decimals, or `-` for `None`.
    pub fn show(s: Option<Spread>, digits: usize) -> String {
        match s {
            Some(s) => format!("{:.d$} ({:.d$}..{:.d$})", s.median, s.min, s.max, d = digits),
            None => "-".to_string(),
        }
    }
}

/// The nearest-rank percentile of `values` (`p` in 0..=100): the smallest value with at least `p` per
/// cent of the sample at or below it. `None` for an empty sample. Nearest rank is used because it
/// returns a value that was actually observed, which suits a latency report.
pub fn percentile(values: &[u64], p: f64) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_unstable();
    let rank = ((p / 100.0) * v.len() as f64).ceil().max(1.0) as usize;
    Some(v[rank.min(v.len()) - 1])
}

/// One category's metrics for a cell, each as a spread over the repeats.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CategoryLive {
    /// Runs in the category (all repeats).
    pub runs: usize,
    /// Mean recall@10 over questions with gold.
    pub recall10: Option<Spread>,
    /// Share of runs with every expected literal in the answer.
    pub correct: Option<Spread>,
    /// Share of answered runs that are grounded.
    pub grounded: Option<Spread>,
    /// Share of unanswerable runs that refused.
    pub refusal: Option<Spread>,
}

/// All runs of one model in one mode, aggregated.
#[derive(Debug, Clone, Serialize)]
pub struct CellReport {
    /// The model display id.
    pub model: String,
    /// `tools`, `pre` or `both`.
    pub mode: String,
    /// Runs pooled (all worlds, questions and repeats).
    pub runs: usize,
    /// Distinct repeats seen.
    pub repeats: usize,
    /// Distinct world seeds seen.
    pub worlds: Vec<i64>,
    /// Share of runs that ended in an answer.
    pub answered: Option<Spread>,
    /// Mean recall@5 over questions with gold.
    pub recall5: Option<Spread>,
    /// Mean recall@10 over questions with gold.
    pub recall10: Option<Spread>,
    /// Share of runs with every expected literal in the answer.
    pub correct: Option<Spread>,
    /// Mean share of expected literals in the answer.
    pub expect_share: Option<Spread>,
    /// Share of answered runs that are grounded.
    pub grounded: Option<Spread>,
    /// Share of citations the app verified.
    pub citation_verified: Option<Spread>,
    /// Share of citations naming a gold source.
    pub citation_on_gold: Option<Spread>,
    /// Share of answered runs with no citation at all.
    pub uncited: Option<Spread>,
    /// Share of unanswerable runs that refused.
    pub refusal: Option<Spread>,
    /// Leaks over every run (a count, summed over repeats, never averaged).
    pub leaks: usize,
    /// Latency p50 in ms, per repeat.
    pub latency_p50: Option<Spread>,
    /// Latency p95 in ms, per repeat.
    pub latency_p95: Option<Spread>,
    /// Mean prompt tokens per run, per repeat (runs where the server reported usage).
    pub prompt_tokens: Option<Spread>,
    /// Mean completion tokens per run, per repeat.
    pub completion_tokens: Option<Spread>,
    /// Runs with no latency, and with no usage, so a gap in the data is stated.
    pub missing_latency: usize,
    /// See above.
    pub missing_usage: usize,
    /// The per-category metrics.
    pub by_category: BTreeMap<String, CategoryLive>,
}

fn mean(v: &[f64]) -> Option<f64> {
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

fn share(flags: &[bool]) -> Option<f64> {
    (!flags.is_empty()).then(|| flags.iter().filter(|b| **b).count() as f64 / flags.len() as f64)
}

/// The values of `f` per repeat, dropping repeats where `f` is `None`, as a [`Spread`].
fn per_repeat(rows: &[(u32, &RunRecord, RunScore)], f: impl Fn(&[&(u32, &RunRecord, RunScore)]) -> Option<f64>) -> Option<Spread> {
    let mut by: BTreeMap<u32, Vec<&(u32, &RunRecord, RunScore)>> = BTreeMap::new();
    for r in rows {
        by.entry(r.0).or_default().push(r);
    }
    let vals: Vec<f64> = by.values().filter_map(|g| f(g)).collect();
    Spread::of(&vals)
}

/// Aggregates `records`, grouped by (model, mode), in sorted order so the report is deterministic.
pub fn aggregate(records: &[RunRecord]) -> Vec<CellReport> {
    let mut groups: BTreeMap<(String, String), Vec<&RunRecord>> = BTreeMap::new();
    for r in records {
        groups.entry((r.model.clone(), r.mode.clone())).or_default().push(r);
    }
    let mut out = Vec::new();
    for ((model, mode), recs) in groups {
        let rows: Vec<(u32, &RunRecord, RunScore)> = recs.iter().map(|r| (r.repeat, *r, score_run(r))).collect();
        let mut worlds: Vec<i64> = recs.iter().map(|r| r.seed).collect();
        worlds.sort_unstable();
        worlds.dedup();
        let repeats = rows.iter().map(|r| r.0).collect::<std::collections::BTreeSet<_>>().len();
        let cit_ratio = |pick: fn(&RunScore) -> usize| {
            move |g: &[&(u32, &RunRecord, RunScore)]| {
                let cited: usize = g.iter().map(|r| r.2.cited).sum();
                (cited > 0).then(|| g.iter().map(|r| pick(&r.2)).sum::<usize>() as f64 / cited as f64)
            }
        };
        let mut by_category = BTreeMap::new();
        for cat in CATEGORIES {
            let sub: Vec<(u32, &RunRecord, RunScore)> = rows.iter().filter(|r| r.1.question.category == cat).cloned().collect();
            by_category.insert(
                cat.to_string(),
                CategoryLive {
                    runs: sub.len(),
                    recall10: per_repeat(&sub, |g| mean(&g.iter().filter_map(|r| r.2.recall10).collect::<Vec<_>>())),
                    correct: per_repeat(&sub, |g| share(&g.iter().filter_map(|r| r.2.correct).collect::<Vec<_>>())),
                    grounded: per_repeat(&sub, |g| share(&g.iter().filter_map(|r| r.2.grounded).collect::<Vec<_>>())),
                    refusal: per_repeat(&sub, |g| share(&g.iter().filter_map(|r| r.2.refused).collect::<Vec<_>>())),
                },
            );
        }
        let tok = |pick: fn(&RunRecord) -> Option<u64>| move |g: &[&(u32, &RunRecord, RunScore)]| mean(&g.iter().filter_map(|r| pick(r.1)).map(|t| t as f64).collect::<Vec<_>>());
        let lat = |p: f64| move |g: &[&(u32, &RunRecord, RunScore)]| percentile(&g.iter().filter_map(|r| r.1.latency_ms).collect::<Vec<_>>(), p).map(|v| v as f64);
        out.push(CellReport {
            model,
            mode,
            runs: rows.len(),
            repeats,
            worlds,
            answered: per_repeat(&rows, |g| share(&g.iter().map(|r| r.2.answered).collect::<Vec<_>>())),
            recall5: per_repeat(&rows, |g| mean(&g.iter().filter_map(|r| r.2.recall5).collect::<Vec<_>>())),
            recall10: per_repeat(&rows, |g| mean(&g.iter().filter_map(|r| r.2.recall10).collect::<Vec<_>>())),
            correct: per_repeat(&rows, |g| share(&g.iter().filter_map(|r| r.2.correct).collect::<Vec<_>>())),
            expect_share: per_repeat(&rows, |g| mean(&g.iter().filter_map(|r| r.2.expect_share).collect::<Vec<_>>())),
            grounded: per_repeat(&rows, |g| share(&g.iter().filter_map(|r| r.2.grounded).collect::<Vec<_>>())),
            citation_verified: per_repeat(&rows, cit_ratio(|s| s.verified)),
            citation_on_gold: per_repeat(&rows, cit_ratio(|s| s.on_gold)),
            uncited: per_repeat(&rows, |g| share(&g.iter().filter(|r| r.2.answered).map(|r| r.2.cited == 0).collect::<Vec<_>>())),
            refusal: per_repeat(&rows, |g| share(&g.iter().filter_map(|r| r.2.refused).collect::<Vec<_>>())),
            leaks: rows.iter().filter(|r| r.2.leak).count(),
            latency_p50: per_repeat(&rows, lat(50.0)),
            latency_p95: per_repeat(&rows, lat(95.0)),
            prompt_tokens: per_repeat(&rows, tok(|r| r.prompt_tokens)),
            completion_tokens: per_repeat(&rows, tok(|r| r.completion_tokens)),
            missing_latency: rows.iter().filter(|r| r.1.latency_ms.is_none()).count(),
            missing_usage: rows.iter().filter(|r| r.1.prompt_tokens.is_none()).count(),
            by_category,
        });
    }
    out
}

/// Parses a JSON-lines file's text into records. A blank line is skipped; any other unreadable line
/// is an error naming its line number, so a truncated file is never silently a smaller sample.
pub fn parse_jsonl(text: &str) -> Result<Vec<RunRecord>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(line).map_err(|e| format!("line {}: {e}", n + 1))?);
    }
    Ok(out)
}

/// Writes the report's tables (markdown). `title` heads the section. The text states the repeats and
/// worlds behind every row, and prints `-` for a metric no run could produce.
pub fn render_report(cells: &[CellReport], title: &str) -> String {
    let mut s = format!("### {title}\n\n");
    if cells.is_empty() {
        s.push_str("No runs.\n");
        return s;
    }
    s.push_str("Each cell is `median (min..max)` over the repeats; `-` means no run could produce the metric.\n\n");
    s.push_str("| Model | Mode | Runs | Repeats | Worlds | Answered | Recall@5 | Recall@10 | Correct | Expect share | Grounded | Cite verified | Cite on gold | Uncited | Refusal | Leaks |\n");
    s.push_str("|---|---|---:|---:|---:|---|---|---|---|---|---|---|---|---|---|---:|\n");
    for c in cells {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            c.model,
            c.mode,
            c.runs,
            c.repeats,
            c.worlds.len(),
            Spread::show(c.answered, 3),
            Spread::show(c.recall5, 3),
            Spread::show(c.recall10, 3),
            Spread::show(c.correct, 3),
            Spread::show(c.expect_share, 3),
            Spread::show(c.grounded, 3),
            Spread::show(c.citation_verified, 3),
            Spread::show(c.citation_on_gold, 3),
            Spread::show(c.uncited, 3),
            Spread::show(c.refusal, 3),
            c.leaks,
        ));
    }
    s.push_str("\n| Model | Mode | Latency p50 ms | Latency p95 ms | Prompt tokens / run | Completion tokens / run | Runs without latency | Runs without usage |\n|---|---|---|---|---|---|---:|---:|\n");
    for c in cells {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            c.model,
            c.mode,
            Spread::show(c.latency_p50, 0),
            Spread::show(c.latency_p95, 0),
            Spread::show(c.prompt_tokens, 0),
            Spread::show(c.completion_tokens, 0),
            c.missing_latency,
            c.missing_usage,
        ));
    }
    for c in cells {
        s.push_str(&format!("\n#### {} / {} by category\n\n| Category | Runs | Recall@10 | Correct | Grounded | Refusal |\n|---|---:|---|---|---|---|\n", c.model, c.mode));
        for (cat, v) in &c.by_category {
            if v.runs == 0 {
                continue;
            }
            s.push_str(&format!("| {} | {} | {} | {} | {} | {} |\n", cat, v.runs, Spread::show(v.recall10, 3), Spread::show(v.correct, 3), Spread::show(v.grounded, 3), Spread::show(v.refusal, 3)));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn question(category: &str, gold: &[&str], expect: &[&str]) -> Question {
        Question {
            id: format!("{category}-01"),
            category: category.to_string(),
            text: "How many people live in Veldmark?".to_string(),
            substring_query: "Veldmark".to_string(),
            gold: gold.iter().map(|s| s.to_string()).collect(),
            expect: expect.iter().map(|s| s.to_string()).collect(),
            forbid: Vec::new(),
            absent_name: None,
        }
    }

    fn record(q: Question, answer: &str, ledger: &[&str], context: &[&str]) -> RunRecord {
        RunRecord {
            model: "m".into(),
            mode: "tools".into(),
            seed: 1,
            repeat: 0,
            question: q,
            outcome: "answer".into(),
            answer: answer.into(),
            ledger: ledger.iter().map(|s| s.to_string()).collect(),
            citations: Vec::new(),
            context: context.iter().map(|s| s.to_string()).collect(),
            tool_calls: Vec::new(),
            latency_ms: Some(1000),
            prompt_tokens: Some(100),
            completion_tokens: Some(10),
            reason: None,
        }
    }

    /// Protects the figure reader: years and counts of three digits or more are figures, a numbered
    /// list and "top 5" are not, and a digit run glued to letters is a name fragment. MUTATION: set
    /// `FIGURE_MIN_DIGITS` to 1 and the second assertion goes red.
    #[test]
    fn figures_are_three_digits_or_more_and_not_glued_to_letters() {
        assert_eq!(figures("Walled in 1204; population 6115."), vec!["1204", "6115"]);
        assert!(figures("1. first, 2. second, the top 5, 11 partners").is_empty());
        assert!(figures("Fort ab1204 and k99").is_empty());
        assert_eq!(figures("year 1100."), vec!["1100"]);
    }

    /// Protects groundedness's two halves with literal inputs: a figure the model was handed is
    /// supported; one it was not handed is counted; the question's own figure is allowed.
    #[test]
    fn a_figure_is_supported_only_by_the_context_or_the_question() {
        let ctx = vec!["population: 6115".to_string()];
        assert_eq!(unsupported_numbers("About 6115 live there.", &ctx, "How many?"), 0);
        assert_eq!(unsupported_numbers("About 7000 live there.", &ctx, "How many?"), 1);
        assert_eq!(unsupported_numbers("In 1204 it fell.", &[], "What happened in 1204?"), 0);
    }

    /// Protects `correct` and `grounded` together: the right number given in the context is correct and
    /// grounded; the SAME right number with an empty context is correct but not grounded (a lucky or
    /// memorised guess); a wrong number is neither.
    #[test]
    fn a_right_answer_with_no_source_is_correct_but_not_grounded() {
        let q = question("figure", &["W:settlement:1"], &["6115"]);
        let good = score_run(&record(q.clone(), "Veldmark holds 6115 people.", &["W:settlement:1"], &["population: 6115"]));
        assert_eq!((good.correct, good.grounded), (Some(true), Some(true)));
        let lucky = score_run(&record(q.clone(), "Veldmark holds 6115 people.", &[], &[]));
        assert_eq!((lucky.correct, lucky.grounded), (Some(true), Some(false)));
        let wrong = score_run(&record(q, "Veldmark holds 7000 people.", &["W:settlement:1"], &["population: 6115"]));
        assert_eq!((wrong.correct, wrong.grounded), (Some(false), Some(false)));
    }

    /// Protects the second half of `grounded`: an answer that holds an expected NAME (no figure, so the
    /// figure check cannot see it) is grounded only if the model was handed that name. MUTATION: drop
    /// `!right_without_source` and the empty-context assertion goes red.
    #[test]
    fn a_right_name_the_model_was_not_given_is_not_grounded() {
        let q = question("relational", &["W:faction:1"], &["Korrath"]);
        let lucky = score_run(&record(q.clone(), "Korrath owns it.", &[], &[]));
        assert_eq!((lucky.correct, lucky.grounded, lucky.unsupported_numbers), (Some(true), Some(false), 0));
        let given = score_run(&record(q, "Korrath owns it.", &["W:faction:1"], &["owner faction: Korrath"]));
        assert_eq!((given.correct, given.grounded), (Some(true), Some(true)));
    }

    /// Protects recall's depth and the none-applicable rule: a gold id at rank 6 is missed at 5 and
    /// found at 10; a question with no gold has `None`, not 0.
    #[test]
    fn recall_respects_depth_and_a_goldless_question_has_none() {
        let q = question("figure", &["W:settlement:1"], &[]);
        let ledger = ["a", "b", "c", "d", "e", "W:settlement:1"];
        let s = score_run(&record(q, "x", &ledger, &[]));
        assert_eq!((s.recall5, s.recall10), (Some(0.0), Some(1.0)));
        let none = score_run(&record(question("unanswerable", &[], &[]), "x", &ledger, &[]));
        assert_eq!((none.recall5, none.recall10), (None, None));
    }

    /// Protects the refusal check on the unanswerable category only: a plain refusal passes, an invented
    /// answer with a figure fails, a refusal that also invents a figure fails, and a non-unanswerable
    /// question has no refusal verdict. MUTATION: empty `REFUSAL_MARKERS` and the first assertion fails.
    #[test]
    fn refusal_needs_a_marker_and_no_invented_figure() {
        let q = question("unanswerable", &[], &[]);
        assert_eq!(score_run(&record(q.clone(), "I could not find Zarnok in the sources.", &[], &[])).refused, Some(true));
        assert_eq!(score_run(&record(q.clone(), "Zarnok has 54321 people.", &[], &[])).refused, Some(false));
        assert_eq!(score_run(&record(q.clone(), "I do not have it, but perhaps 54321.", &[], &[])).refused, Some(false));
        assert_eq!(score_run(&record(question("figure", &["W:x:1"], &["1"]), "I do not have that.", &[], &[])).refused, None);
        let mut failed = record(q, "", &[], &[]);
        failed.outcome = "failed".into();
        assert_eq!(score_run(&failed).refused, Some(false), "a failed run did not refuse");
    }

    /// Protects every refusal marker with its own literal sentence (so a marker removed from the list
    /// turns exactly one line red, and the test never reads the list it protects) and the converse: plain
    /// assertions are not refusals.
    #[test]
    fn each_refusal_phrasing_is_recognised_and_assertions_are_not() {
        for s in [
            "I do not have that.",
            "Sorry, I don't have anything on it.",
            "It does not appear in the sources.",
            "It doesn't appear anywhere.",
            "There is no information about Zarnok.",
            "There is no record of that place.",
            "Zarnok is not mentioned.",
            "That is not in the notes I was given.",
            "I cannot find Zarnok.",
            "I can't find Zarnok.",
            "I could not find Zarnok.",
            "I couldn't find Zarnok.",
            "There is no such settlement.",
            "Result: not found.",
        ] {
            assert!(is_refusal(s), "{s}");
        }
        for s in ["Zarnok is a harbour town.", "The population is large.", "It was walled by the Order of Dawn."] {
            assert!(!is_refusal(s), "{s}");
        }
        assert_eq!(REFUSAL_MARKERS.len(), 14);
    }

    /// Protects the leak check over the answer, the context and the ledger ids, case-insensitively.
    #[test]
    fn a_canary_in_the_answer_or_the_context_is_a_leak() {
        let mut q = question("private", &[], &[]);
        q.forbid = vec!["DMCANARY7".into()];
        assert!(!score_run(&record(q.clone(), "nothing to say", &[], &["clean"])).leak);
        assert!(score_run(&record(q.clone(), "the dmcanary7 is here", &[], &[])).leak);
        assert!(score_run(&record(q, "ok", &[], &["note says DMCANARY7"])).leak);
    }

    /// Protects the citation counts: verified is the app's flag, on-gold is the id match (a Vault note
    /// id answers its chunk), and a run with no citation does not contribute a 100% rate.
    #[test]
    fn citations_count_verified_and_gold_separately() {
        let mut r = record(question("figure", &["W:settlement:1", "V:Places/A.md"], &["6115"]), "6115 [W:settlement:1] [V:Places/A.md#Old Quarter] [W:settlement:9]", &[], &[]);
        r.citations = vec![
            CitationRec { id: "W:settlement:1".into(), verified: true },
            CitationRec { id: "V:Places/A.md#Old Quarter".into(), verified: true },
            CitationRec { id: "W:settlement:9".into(), verified: false },
        ];
        let s = score_run(&r);
        // Three citations, two verified, two on gold (the chunk id answers its note's gold id): the counts
        // differ from each other and from their complements, so an inverted test cannot pass.
        assert_eq!((s.cited, s.verified, s.on_gold), (3, 2, 2));
        let cells = aggregate(&[record(question("figure", &["W:x:1"], &["1"]), "no cite", &[], &[])]);
        assert_eq!(cells[0].citation_verified, None, "no citation anywhere means no rate");
        assert_eq!(cells[0].uncited.map(|u| u.median), Some(1.0));
    }

    /// Protects the spread and the percentile on hand-made values, including the empty case that must
    /// be `None` and the even-count median.
    #[test]
    fn spread_and_percentile_on_known_values() {
        let s = Spread::of(&[0.4, 0.2, 0.9]).unwrap();
        assert_eq!((s.median, s.min, s.max, s.repeats), (0.4, 0.2, 0.9, 3));
        assert_eq!(Spread::of(&[1.0, 3.0]).unwrap().median, 2.0);
        assert_eq!(Spread::of(&[]), None);
        let v: Vec<u64> = (1..=100).collect();
        assert_eq!((percentile(&v, 50.0), percentile(&v, 95.0), percentile(&v, 100.0)), (Some(50), Some(95), Some(100)));
        assert_eq!(percentile(&[7], 95.0), Some(7));
        // A rank that is not a whole number rounds UP (nearest rank): 9.5 of 10 is the 10th value, 1.5 of 3 the 2nd.
        let ten: Vec<u64> = (1..=10).collect();
        assert_eq!(percentile(&ten, 95.0), Some(10));
        assert_eq!(percentile(&[1, 2, 3], 50.0), Some(2));
        assert_eq!(percentile(&[], 50.0), None);
    }

    /// Protects the aggregation: two repeats whose correct rates are 1.0 and 0.0 report median 0.5 with
    /// range 0..1; a missing latency is dropped, not read as 0 ms; tokens average over the runs that
    /// have them.
    #[test]
    fn aggregation_reports_the_spread_over_repeats_and_drops_missing_values() {
        let q = question("figure", &["W:x:1"], &["6115"]);
        let mut a = record(q.clone(), "6115", &["W:x:1"], &["6115"]);
        let mut b = record(q, "wrong", &["W:x:1"], &["6115"]);
        a.repeat = 0;
        b.repeat = 1;
        b.latency_ms = None;
        b.prompt_tokens = None;
        let cells = aggregate(&[a, b]);
        let c = &cells[0];
        let corr = c.correct.unwrap();
        assert_eq!((corr.median, corr.min, corr.max, corr.repeats), (0.5, 0.0, 1.0, 2));
        assert_eq!(c.latency_p50.unwrap().repeats, 1, "the repeat with no latency contributes nothing");
        assert_eq!(c.latency_p50.unwrap().median, 1000.0);
        assert_eq!((c.missing_latency, c.missing_usage), (1, 1));
        assert_eq!(c.prompt_tokens.unwrap().median, 100.0);
    }

    /// Protects the JSON-lines reader: a bad line is an error naming it, never a smaller sample.
    #[test]
    fn a_bad_line_is_an_error_naming_its_line() {
        let good = serde_json::to_string(&record(question("figure", &["W:x:1"], &["1"]), "a", &[], &[])).unwrap();
        assert_eq!(parse_jsonl(&format!("{good}\n\n{good}\n")).unwrap().len(), 2);
        let err = parse_jsonl(&format!("{good}\n{{not json}}\n")).unwrap_err();
        assert!(err.starts_with("line 2:"), "{err}");
    }

    /// Protects the rendered report: it names the repeats, prints `-` for an absent metric and carries
    /// every category with runs.
    #[test]
    fn the_report_prints_a_dash_for_a_metric_nothing_produced() {
        let cells = aggregate(&[record(question("unanswerable", &[], &[]), "I do not have it.", &[], &[])]);
        let text = render_report(&cells, "T");
        assert!(text.contains("| m | tools | 1 | 1 | 1 |"), "{text}");
        assert!(text.contains("| unanswerable | 1 | - | - | 1.000 (1.000..1.000) | 1.000 (1.000..1.000) |"), "{text}");
    }
}
