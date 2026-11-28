use anyhow::{Context, Result};
use writerslogic_warm::core::documents::{
    BatchTooLarge, EncodeMode, IngestOptions, MAX_BATCH_ITEMS,
};
use writerslogic_warm::{DocumentInput, WarmCore};
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
    core: WarmCore,
    options: IngestOptions,
}

impl Indexer {
    /// Indexes every document of `batch`. A batch over the size limit is halved and both halves
    /// are indexed; any other error, including a rejected document, stops the build.
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
            Err(error) => {
                let first = batch.first().map_or("?", |document| document.id.as_str());
                Err::<(), _>(error).with_context(|| {
                    format!("indexing a batch of {} documents from {first}", batch.len())
                })
            }
        }
    }
}

fn main() -> Result<()> {
    let output = std::env::args()
        .nth(1)
        .context("usage: build_index <index dir>")?;
    let mut indexer = Indexer {
        core: WarmCore::new(1024, Some(output), None)?,
        options: IngestOptions {
            encode: EncodeMode::Lexical,
            // The benchmark indexes words as written: no stemming.
            unstemmed: true,
            // Each article is one chunk with word positions, so counts, intersections and
            // phrases are article-level.
            positions: true,
            whole_document: true,
        },
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
            // An article with no words has no postings in any engine and matches no query, and
            // WARM does not take a document without words. It is counted and reported, not a
            // rejection: every document WARM is given must be accepted.
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
    // The explicit compaction reorders the documents (document_checkpoint.reorder).
    indexer.core.compact()?;
    indexer.core.flush()?;
    println!("{ordinal} documents read, {empty} without words not indexed, every other one indexed");
    if std::env::var_os("WARM_RESOURCE_RECEIPT").is_some() {
        let status = std::fs::read_to_string("/proc/self/status")
            .context("read Linux memory receipt after indexing")?;
        eprintln!(
            "WARM_RESOURCE_AFTER_FLUSH={}",
            serde_json::to_string(&status)?
        );
    }
    // The store is durable after flush; skip tearing down the in-memory index.
    std::process::exit(0);
}
