use anyhow::{Context, Result};
use holographic_memory::{HmsCore, SearchOptions};
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

/// The terms of a query HMS can run: one or more bare terms, scored as a BM25 disjunction.
/// Phrases, required (`+`) and excluded (`-`) clauses have no HMS operator.
fn disjunction(query: &str) -> Option<&str> {
    let bare = !query.trim().is_empty()
        && !query.contains('"')
        && query
            .split_whitespace()
            .all(|term| !term.starts_with('+') && !term.starts_with('-'));
    bare.then_some(query)
}

/// The harness starts the next command's server as soon as it closes this one's input, so the
/// store lock may still be held for a moment by the previous process.
fn open(path: &str) -> Result<HmsCore> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        match HmsCore::new(1024, Some(path.to_owned()), None) {
            Ok(core) => return Ok(core),
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)),
            Err(error) => return Err(error),
        }
    }
}

fn top_k(core: &HmsCore, terms: &str, k: u32) -> Result<()> {
    let options = SearchOptions {
        k: Some(k),
        ..Default::default()
    };
    std::hint::black_box(core.search_documents(terms, &options)?);
    Ok(())
}

/// The top `k` documents and the exact number of matching documents, as two calls: HMS ranks
/// and counts separately.
fn top_k_count(core: &HmsCore, terms: &str, k: u32) -> Result<String> {
    top_k(core, terms, k)?;
    Ok(core.count_documents(terms)?.to_string())
}

fn main() -> Result<()> {
    let path = std::env::args().nth(1).context("usage: do_query <index dir>")?;
    let core = open(&path)?;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let (command, query) = line
            .split_once('\t')
            .context("expected a line in the format <COMMAND>\\t<query>")?;
        let answer = match (command, disjunction(query)) {
            ("TOP_10", Some(terms)) => top_k(&core, terms, 10).map(|()| "1".to_owned())?,
            ("TOP_100", Some(terms)) => top_k(&core, terms, 100).map(|()| "1".to_owned())?,
            ("TOP_1000", Some(terms)) => top_k(&core, terms, 1000).map(|()| "1".to_owned())?,
            ("COUNT", Some(terms)) => core.count_documents(terms)?.to_string(),
            ("TOP_10_COUNT", Some(terms)) => top_k_count(&core, terms, 10)?,
            ("TOP_100_COUNT", Some(terms)) => top_k_count(&core, terms, 100)?,
            ("TOP_1000_COUNT", Some(terms)) => top_k_count(&core, terms, 1000)?,
            _ => "UNSUPPORTED".to_owned(),
        };
        writeln!(out, "{answer}")?;
        out.flush()?;
    }
    // Answers are all written; skip tearing down the in-memory index.
    std::process::exit(0);
}
