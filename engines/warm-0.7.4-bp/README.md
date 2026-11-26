# WARM

[WARM](https://github.com/writerslogic/warm) is an Apache-2.0 Rust engine for local search and
agent memory. This engine builds against the published crate, pinned to the exact
`warm-rust = "=0.7.4"` release the engine is named for, with `Cargo.lock` committed.

Compaction reorders documents by recursive graph bisection, the technique behind the Lucene `-bp`
column, so this engine is named `warm-0.7.4-bp`. The adapter has no setting that skips the
reordering.

## Queries

All benchmark query types are supported: term, union, intersection, mixed required/optional,
negation and phrase. Required clauses must match; optional clauses score and are required only
when there are no required clauses; excluded clauses remove matches. Phrases are ordered and
adjacent. `TOP_*_FF` is not supported.

COUNT counts matching articles without scoring. `TOP_N_COUNT` returns ranked hits and an exact
count. Union top-k uses block-max MaxScore; intersections and phrases use posting cursors. The
pruning is exact: `WARM_LEXICAL_EXECUTION=exhaustive` selects the full evaluation, and
`WARM_DIFFERENTIAL=1` checks every result against it.

## Tokenization and scoring

Unicode alphanumeric tokens, lowercased, no stemming. On the pre-transformed corpus (lowercase
ASCII and spaces) this is whitespace splitting. BM25 with k1 = 1.2, b = 0.75,
`idf = ln(1 + (N - df + 0.5) / (df + 0.5))`, exact article lengths and f64 arithmetic. Empty
articles are skipped.

## Build and read

`build_index` streams the corpus from stdin, indexes each whole article as one lexical chunk with
token positions (text is not stored), then compacts the store, which reorders it, and flushes.
`do_query` opens the store from disk; per-list rank tables are built at open, not from queries.
The binaries use the mimalloc global allocator.
