use anyhow::{Context, Result};
use holographic_memory::core::documents::{
    BatchTooLarge, EncodeMode, IngestOptions, MAX_BATCH_ITEMS,
};
use holographic_memory::{DocumentInput, HmsCore};
use serde::Deserialize;
use std::io::BufRead;

#[derive(Deserialize)]
struct Doc {
    text: String,
}

/// One article per lexical chunk; positions remain available without retaining text.
fn input(ordinal: u64, text: String, chunk_words: u32) -> DocumentInput {
    DocumentInput {
        id: ordinal.to_string(),
        text,
        source_uri: None,
        version: None,
        metadata: None,
        chunk_words: Some(chunk_words),
        overlap_words: Some(0),
        store_text: Some(false),
        embeddings: None,
        title_weight: None,
    }
}

struct Indexer {
    core: HmsCore,
    options: IngestOptions,
    rejected: u64,
}

impl Indexer {
    fn commit(&mut self, batch: Vec<DocumentInput>) -> Result<()> {
        match self
            .core
            .memorize_documents_with(batch.clone(), &self.options)
        {
            Ok(_) => Ok(()),
            Err(error) if error.is::<BatchTooLarge>() && batch.len() > 1 => {
                let mut batch = batch;
                let tail = batch.split_off(batch.len() / 2);
                self.commit(batch)?;
                self.commit(tail)
            }
            Err(_) if batch.len() > 1 => {
                for document in batch {
                    self.commit(vec![document])?;
                }
                Ok(())
            }
            Err(error) => {
                let document = batch.into_iter().next().context("empty batch")?;
                eprintln!("rejected document {}: {error:#}", document.id);
                self.rejected += 1;
                Ok(())
            }
        }
    }
}

fn main() -> Result<()> {
    let output = std::env::args()
        .nth(1)
        .context("usage: build_index <index dir>")?;
    let mut indexer = Indexer {
        core: HmsCore::new(1024, Some(output), None)?,
        options: IngestOptions {
            encode: EncodeMode::Lexical,
            // The benchmark indexes words as written: no stemming.
            unstemmed: true,
            positions: true,
            whole_document: true,
        },
        rejected: 0,
    };
    let (mut ordinal, mut empty) = (0u64, 0u64);
    let mut batch = Vec::with_capacity(MAX_BATCH_ITEMS);
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let doc: Doc = serde_json::from_str(&line)?;
        ordinal += 1;
        if ordinal % 100_000 == 0 {
            println!("{ordinal}");
        }
        if doc.text.trim().is_empty() {
            // HMS refuses a document without words; such an article matches no query.
            empty += 1;
            continue;
        }
        batch.push(input(ordinal, doc.text, 4096));
        if batch.len() == MAX_BATCH_ITEMS {
            indexer.commit(std::mem::take(&mut batch))?;
        }
    }
    if !batch.is_empty() {
        indexer.commit(batch)?;
    }
    indexer.core.flush()?;
    println!(
        "{ordinal} documents read, {empty} without words skipped, {} rejected",
        indexer.rejected
    );
    if std::env::var_os("HMS_RESOURCE_RECEIPT").is_some() {
        let status = std::fs::read_to_string("/proc/self/status")
            .context("read Linux memory receipt after indexing")?;
        eprintln!(
            "HMS_RESOURCE_AFTER_FLUSH={}",
            serde_json::to_string(&status)?
        );
    }
    // The store is durable after flush; skip tearing down the in-memory index.
    std::process::exit(0);
}
