//! Build an infino supertable from newline-delimited JSON, then compact it.
//!
//! Each input line is `{"id": "...", "text": "...", "sort_field": <u64>}`.
//! Only `text` is indexed. Docs are streamed in 50 k-doc batches; a 4 GiB
//! auto-flush threshold causes the writer to commit several segments
//! incrementally (bounded build memory). After ingest, `optimize()` compacts
//! all segments into one, matching the single-segment shape that tantivy and
//! Lucene produce — so query-path fan-out overhead is equivalent.

use std::env;
use std::io::{self, BufRead};
use std::sync::Arc;
use std::time::Duration;

use arrow_array::{LargeStringArray, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use infino::{CompactionSettings, GcSettings, OptimizeOptions};
use infino::storage::{LocalFsStorageProvider, StorageProvider};
use infino::superfile::builder::FtsConfig;
use infino::superfile::fts::tokenize::AsciiLowerTokenizer;
use infino::supertable::{Supertable, SupertableOptions};
use serde::Deserialize;

const COLUMN: &str = "text";
const COMPACT_TARGET_MB: u64 = 32 * 1024;
const BATCH: usize = 50_000;

#[derive(Deserialize)]
struct Doc {
    text: String,
}

fn schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![Field::new(
        COLUMN,
        DataType::LargeUtf8,
        false,
    )]))
}

fn writer_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(4)
}

fn options(storage: Arc<dyn StorageProvider>) -> SupertableOptions {
    let pool = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(writer_threads())
            .build()
            .expect("build writer pool"),
    );
    SupertableOptions::new(
        schema(),
        vec![FtsConfig {
            column: COLUMN.to_string(),
            positions: true,
        }],
        vec![],
        Some(Arc::new(AsciiLowerTokenizer)),
    )
    .expect("valid supertable options")
    .with_writer_pool(pool)
    .with_commit_threshold_size_mb(4096)
    .with_storage(storage)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let storage: Arc<dyn StorageProvider> =
        Arc::new(LocalFsStorageProvider::new(&args[1]).expect("open local storage"));
    let st = Supertable::create(options(storage)).expect("create supertable");
    let mut writer = st.writer().expect("acquire writer");
    let schema = schema();

    let mut buf: Vec<String> = Vec::with_capacity(BATCH);
    let mut total: u64 = 0;
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("read line");
        if line.trim().is_empty() {
            continue;
        }
        let doc: Doc = serde_json::from_str(&line).expect("parse json");
        buf.push(doc.text);
        if buf.len() == BATCH {
            total += buf.len() as u64;
            append(&mut writer, &schema, &mut buf);
            if total % 1_000_000 == 0 {
                eprintln!("{total}");
            }
        }
    }
    if !buf.is_empty() {
        total += buf.len() as u64;
        append(&mut writer, &schema, &mut buf);
    }

    writer.commit().expect("commit");
    drop(writer);
    eprintln!("indexed {total} docs into the supertable");

    eprintln!("compacting…");
    st.optimize(&OptimizeOptions::compact(CompactionSettings {
        target_superfile_size_mb: COMPACT_TARGET_MB,
        ..Default::default()
    }).with_gc(GcSettings {
        safety_gap: Duration::ZERO,
    }))
    .expect("optimize");
    eprintln!("compact done");
}

fn append(writer: &mut infino::supertable::SupertableWriter, schema: &Arc<arrow_schema::Schema>, buf: &mut Vec<String>) {
    let arr = LargeStringArray::from(buf.iter().map(String::as_str).collect::<Vec<_>>());
    let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(arr)]).expect("record batch");
    writer.append(&batch).expect("append batch");
    buf.clear();
}
