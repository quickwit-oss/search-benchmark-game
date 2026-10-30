//! Shared configuration for the build and query binaries.
//!
//! `build_index` and `do_query` MUST construct byte-identical
//! `SupertableOptions` — the supertable stamps a digest of the options
//! into the manifest at commit time and verifies it on `open`. Centralizing
//! the schema / FTS column / tokenizer / pool config here guarantees they
//! agree.

use std::collections::HashSet;
use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema};
use infino::storage::StorageProvider;
use infino::superfile::builder::FtsConfig;
use infino::supertable::manifest::list::PartitionStrategy;
use infino::supertable::reader_cache::{DiskCacheConfig, DiskCacheStore};
use infino::supertable::{Consistency, SupertableOptions};

/// The single indexed full-text column.
pub const COLUMN: &str = "text";

/// User schema (the `_id` column is auto-injected by the supertable).
pub fn schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![Field::new(
        COLUMN,
        DataType::LargeUtf8,
        false,
    )]))
}

/// Number of writer-pool threads (also the number of segments produced per
/// commit, since a commit shards across `min(pool_threads, rows)`).
///
/// Tuned for c7i.2xlarge (8 vCPU, 16 GiB). Capped at 4: fewer threads → fewer
/// segments per commit (~12 total vs ~24 at 8) → less query-time fan-out, and
/// lower build peak memory (4 parallel shard builds instead of 8). Build is a
/// little slower but stays well within 16 GiB. A post-ingest `optimize`
/// (see `build_index`) then compacts every segment into one superfile.
pub fn writer_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(4)
}

/// Options shared by builder and reader.
///
/// Bounded-memory build: a multi-thread writer pool builds segments in parallel
/// and a `with_commit_threshold_size_mb` auto-flush caps the in-memory write
/// buffer, so the corpus is committed in several rounds rather than held whole
/// in RAM. This emits multiple superfiles; `build_index`'s post-ingest
/// `optimize` compacts them into one.
pub fn options(storage: Arc<dyn StorageProvider>) -> SupertableOptions {
    let writer_pool = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(writer_threads())
            .build()
            .expect("build writer pool"),
    );
    let reader_pool = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .expect("build reader pool"),
    );

    let disk_cache = DiskCacheStore::new(
        Arc::clone(&storage),
        DiskCacheConfig {
            cache_root: std::env::temp_dir().join("infino-bench-disk-cache"),
            ..Default::default()
        },
        Arc::new(HashSet::new),
    )
    .expect("build disk cache");

    SupertableOptions::new(
        schema(),
        // Token positions on: phrase queries are first-class. Index-only
        // (stored(false)), matching how the other engines build the SBG
        // index (Lucene does not store the field either): queries here
        // only rank and count, never read the text back, so skipping the
        // stored copy keeps the on-disk size comparable. The `standard`
        // analyzer (UAX #29 word segmentation + Unicode lowercase) is the
        // Lucene-parity tokenizer, so the split matches Lucene's
        // `StandardTokenizer` on any corpus, not only the pre-transformed
        // one benched here.
        vec![FtsConfig::new(COLUMN)
            .positions(true)
            .stored(false)],
        vec![],
    )
    .expect("valid supertable options")
    .with_partition_strategy(PartitionStrategy::Hash {
        column: "_id".to_string(),
        n_buckets: 1,
    })
    .with_writer_pool(writer_pool)
    .with_reader_pool(reader_pool)
    // Snapshot read consistency: the bench index is built once and read many
    // times in-process, so pin the manifest at open and never pay the
    // per-query pointer re-check that the default BoundedStaleness policy does.
    .with_read_consistency(Consistency::Snapshot)
    // Auto-flush every 4 GiB of buffered rows so each commit's peak memory
    // stays bounded (buffer + that chunk's index) instead of the whole corpus
    // at once. Ingest emits several superfiles; `build_index`'s post-ingest
    // `optimize` compacts them into one.
    .with_commit_threshold_size_mb(4096)
    .with_storage(storage)
    .with_disk_cache(disk_cache)
}
