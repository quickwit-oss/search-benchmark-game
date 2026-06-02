//! Build an infino FTS index from newline-delimited JSON on stdin.
//!
//! Each input line is `{"id": "...", "text": "...", "sort_field": <u64>}`.
//! Only `text` is indexed — the benchmark answers count/top-k queries, so
//! we never need to materialize stored fields. We build a single infino
//! FTS blob (the same posting/skip-table/FST structure that a superfile
//! embeds) and stream it to `idx/fts.blob` via `finish_to`, which keeps
//! peak builder memory bounded by the spill threshold rather than the
//! corpus size.

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufWriter};
use std::path::Path;
use std::sync::Arc;

use infino::superfile::fts::builder::FtsBuilder;
use infino::superfile::fts::tokenize::AsciiLowerTokenizer;
use serde::Deserialize;

#[derive(Deserialize)]
struct Doc {
    text: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let out_dir = Path::new(&args[1]);
    main_inner(out_dir).expect("build index");
}

fn main_inner(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = FtsBuilder::new(Arc::new(AsciiLowerTokenizer));
    let col_id = builder.register_column("text".to_string())?;

    let stdin = io::stdin();
    let mut doc_id: u32 = 0;
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let doc: Doc = serde_json::from_str(&line)?;
        builder.add_doc(col_id, doc_id, &doc.text)?;
        doc_id += 1;
        if doc_id % 100_000 == 0 {
            eprintln!("{doc_id}");
        }
    }

    let blob_path = out_dir.join("fts.blob");
    let file = File::create(&blob_path)?;
    let writer = BufWriter::new(file);
    builder.finish_to(writer)?;

    eprintln!("indexed {doc_id} docs -> {}", blob_path.display());
    Ok(())
}
