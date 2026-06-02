# infino

[infino](https://github.com/infino-ai/infino) is a search-optimized
lakehouse format: one file is a valid Apache Parquet file with an
embedded BM25 full-text index (and vector index) baked in. This engine
benchmarks infino's embedded FTS index directly — the same posting /
skip-table / FST structures, BlockMaxWAND + MaxScore/Block-Max-MaxScore
walks, and PFOR-delta posting codec that `SuperfileReader::bm25_search`
uses.

## Tokenization

Indexing uses infino's `AsciiLowerTokenizer`: split on any byte outside
`[A-Za-z0-9]`, ASCII-lowercase, no stemming. The benchmark corpus is
pre-transformed to `[a-z ]+` (lowercased, non-alphabetic → space), so
this is equivalent to whitespace splitting and matches Lucene's
`StandardTokenizer` on this corpus. Tokens containing non-ASCII bytes
are dropped (irrelevant after the corpus transform).

## Scoring

BM25 with Lucene defaults (`k1 = 1.2`, `b = 0.75`) and Lucene-style IDF
`ln(1 + (N - df + 0.5) / (df + 0.5))`.

## Query support

| Query / command | Status |
|---|---|
| Single term | ✅ |
| Union (`a b`) | ✅ `BoolMode::Or` |
| Intersection (`+a +b`) | ✅ `BoolMode::And` |
| `COUNT` | ✅ full unpruned posting walk |
| `TOP_10` / `TOP_100` / `TOP_1000` | ✅ BlockMaxWAND / Block-Max-MaxScore pruning |
| `TOP_{1,5,10,100,1000}_COUNT` | ✅ (see note) |
| Phrase (`"a b"`) | ❌ **UNSUPPORTED** — no positional postings |
| `TOP_*_FF` (sort by fast field) | ❌ **UNSUPPORTED** — FTS results are score-ordered only |
| `UNOPTIMIZED_COUNT` | ❌ **UNSUPPORTED** |

**`*_COUNT` note:** infino has no fused top-k + total-count collector, so
the matching-document count comes from a full unpruned walk
(`search(..., k = usize::MAX)`). The top-k commands (`TOP_10` etc.) use
the pruning walks and are where infino's WAND-family algorithms show.

## Layout

`build_index` reads newline-delimited JSON from stdin and writes a single
FTS blob to `idx/fts.blob`, streamed via `finish_to` so peak build memory
is bounded by the spill threshold, not the corpus size. `do_query` mmaps
nothing fancy — it reads the blob into memory and serves queries
in-process.
