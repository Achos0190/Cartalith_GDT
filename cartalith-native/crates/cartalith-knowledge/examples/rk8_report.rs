//! RK-8's report generator: reads the `*.jsonl` run files the live driver wrote
//! (`godot-project/_llmknow_eval_cli.gd`) and prints the report's tables as markdown.
//!
//!   cargo run -p cartalith-knowledge --example rk8_report -- <runs dir> ["Section title"]
//!
//! A pure function of the files: it opens no socket, reads no clock and invents no number
//! (`live_eval` module doc). An unreadable line stops it, naming the file and the line, so a
//! truncated run is never reported as a smaller sample.

use cartalith_knowledge::live_eval::{aggregate, parse_jsonl, render_report};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(dir) = args.next() else {
        eprintln!("usage: rk8_report <runs dir> [title]");
        std::process::exit(2);
    };
    let title = args.next().unwrap_or_else(|| "Live evaluation".to_string());
    let mut files: Vec<_> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "jsonl")).collect(),
        Err(e) => {
            eprintln!("cannot read {dir}: {e}");
            std::process::exit(2);
        }
    };
    files.sort();
    if files.is_empty() {
        eprintln!("no .jsonl run files in {dir}");
        std::process::exit(2);
    }
    let mut records = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap_or_else(|e| {
            eprintln!("cannot read {}: {e}", f.display());
            std::process::exit(2);
        });
        match parse_jsonl(&text) {
            Ok(mut r) => records.append(&mut r),
            Err(e) => {
                eprintln!("{}: {e}", f.display());
                std::process::exit(1);
            }
        }
    }
    println!("{}", render_report(&aggregate(&records), &title));
}
