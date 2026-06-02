//! Serve count / top-k queries against an infino FTS blob.
//!
//! Reads `COMMAND\t<lucene-query>` lines from stdin and prints a single
//! result line per query (LineWriter flushes on each newline, so no
//! explicit flush is needed).
//!
//! Supported commands: COUNT, TOP_10/100/1000, TOP_{1,5,10,100,1000}_COUNT.
//! Everything else (phrase queries, fast-field ordering `*_FF`,
//! UNOPTIMIZED_COUNT) is answered "UNSUPPORTED" — see README.md.

use std::env;
use std::io::{self, BufRead};
use std::path::Path;

use bytes::Bytes;
use infino::superfile::fts::reader::{BoolMode, FtsReader};
use infino::superfile::fts::tokenize::AsciiLowerTokenizer;

const COLUMN: &str = "text";
const COLUMNS_JSON: &str = r#"[{"name":"text","tokenizer":"ascii_lower"}]"#;

fn main() {
    let args: Vec<String> = env::args().collect();
    let index_dir = Path::new(&args[1]);
    main_inner(index_dir).expect("serve queries");
}

fn main_inner(index_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let blob = std::fs::read(index_dir.join("fts.blob"))?;
    let reader = FtsReader::open(Bytes::from(blob), COLUMNS_JSON)?;

    let tokenizer = AsciiLowerTokenizer;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let mut parts = line.splitn(2, '\t');
        let command = parts.next().unwrap_or("");
        let query = parts.next().unwrap_or("");

        // Phrase queries need positional postings, which infino's FTS
        // index does not store. Bail before parsing.
        if query.contains('"') {
            println!("UNSUPPORTED");
            continue;
        }

        let (terms, mode) = parse_query(query, &tokenizer);
        let term_refs: Vec<&str> = terms.iter().map(String::as_str).collect();

        let result = match command {
            _ if term_refs.is_empty() => Ok(0usize),
            "COUNT" => reader
                .search(COLUMN, &term_refs, usize::MAX, mode)
                .map(|v| v.len()),
            "TOP_10" | "TOP_100" | "TOP_1000" => {
                let k = top_k(command);
                reader.search(COLUMN, &term_refs, k, mode).map(|_| 1)
            }
            "TOP_1_COUNT" | "TOP_5_COUNT" | "TOP_10_COUNT" | "TOP_100_COUNT"
            | "TOP_1000_COUNT" => reader
                .search(COLUMN, &term_refs, usize::MAX, mode)
                .map(|v| v.len()),
            _ => {
                println!("UNSUPPORTED");
                continue;
            }
        };
        match result {
            Ok(count) => println!("{count}"),
            Err(e) => {
                eprintln!("search error for {:?} {:?}: {e}", command, query);
                println!("0");
            }
        }
    }
    Ok(())
}

/// Map a `TOP_<k>` command to its k.
fn top_k(command: &str) -> usize {
    match command {
        "TOP_10" => 10,
        "TOP_100" => 100,
        "TOP_1000" => 1000,
        _ => 10,
    }
}

/// Parse a benchmark Lucene query into infino terms + a boolean mode.
///
/// `+a +b` (every term required) -> AND; `a b` -> OR. Mixed required/
/// optional queries do not appear in the corpus and collapse to OR.
/// Each raw term is run through the index tokenizer so query and corpus
/// tokenization match exactly.
fn parse_query(query: &str, tokenizer: &AsciiLowerTokenizer) -> (Vec<String>, BoolMode) {
    let raw: Vec<&str> = query.split_whitespace().collect();
    let all_required = !raw.is_empty() && raw.iter().all(|t| t.starts_with('+'));
    let mode = if all_required {
        BoolMode::And
    } else {
        BoolMode::Or
    };

    let mut terms = Vec::new();
    for t in raw {
        let t = t.trim_start_matches(['+', '-']);
        tokenizer.tokenize_each_inline(t, |tok| terms.push(tok.to_string()));
    }
    (terms, mode)
}
