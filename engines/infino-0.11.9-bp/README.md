# infino

[infino](https://github.com/infino-ai/infino) is a search-optimized
lakehouse format: one file is a valid Apache Parquet file with an embedded
BM25 full-text index baked in. This engine benchmarks infino's **supertable**
query path (manifest + per-segment fan-out) — the production query surface —
built with bounded memory into several segments, then compacted to a single
superfile and read fully in memory.

Compaction reorders documents by recursive graph bisection, the same technique
as the Lucene `-bp` column, so this engine is named `infino-0.11.9-bp`
accordingly. infino has no setting to turn the reordering off, so there is no
plain infino column; unlike Lucene's offline BP pass, the reordering is
maintained under real-time updates.

It builds against the **published crate**, pinned to the exact `infino =
"=0.11.9"` release — the version the engine is named for — so a run is fully
reproducible.

## Scope: benchmarked commands

| Command / query | Status | Reason |
|---|---|---|
| `TOP_10` / `TOP_100` / `TOP_1000` | ✅ benchmarked | ranked top-k with BlockMaxWAND / Block-Max-MaxScore pruning |
| union (`a b`) | ✅ | bare terms are shoulds under the `Or` default operator |
| intersection (`+a +b`) | ✅ | `+` must clauses, parsed natively by infino |
| mixed must/should (`+a b`) | ✅ | lucene `BooleanQuery` semantics: match on musts, shoulds raise scores |
| `COUNT` | ✅ benchmarked | native count path — posting-list traversal, no scoring |
| `TOP_*_COUNT` | ✅ benchmarked | one exhaustive scored pass (pruning off): top-k and exact count together |
| negation (`-term`) | ✅ | native must-not exclusion |
| phrase (`"a b"`) | ✅ | exact adjacency verified against positional postings |
| `TOP_*_FF` | ❌ UNSUPPORTED | results are score-ordered only |

## Tokenization & scoring

infino's `standard` analyzer: UAX #29 word segmentation plus Unicode
lowercasing, no stemming — the same split as Lucene's `StandardTokenizer` +
`LowerCaseFilter`. On the pre-transformed corpus (lowercase `[a-z]` and spaces
only) it reduces to whitespace splitting, identical to what infino's
`ascii_lower` analyzer produces. BM25 with Lucene defaults (`k1 = 1.2`, `b = 0.75`) and Lucene-style
IDF.

## Build & read

`build_index` streams JSON from stdin into a supertable with a multi-thread
writer pool and a 4 GiB auto-flush threshold, producing several segments with
bounded build memory (tuned for c7i.2xlarge / 16 GiB). After ingest, it calls
`optimize()` to compact all segments into one — matching the single-segment
shape that tantivy and Lucene produce, so query-path fan-out overhead is
equivalent. `do_query` opens the persisted supertable and preloads every
segment into an in-memory reader tier, so the query path is fully synchronous
— no per-query async/tokio overhead.
