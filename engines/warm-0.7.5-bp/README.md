# WARM

[WARM](https://github.com/writerslogic/warm) is an Apache-2.0 Rust engine for local search and
agent memory. This engine builds against the published `warm-rust` 0.7.5 (see Dependency), with `Cargo.lock`
committed.

Compaction reorders documents by recursive graph bisection, the technique behind the Lucene `-bp`
column, so this engine is named `warm-0.7.5-bp`. The adapter has no setting that skips the
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
ASCII and spaces) this is whitespace splitting. BM25 with WARM's defaults (k1 = 1.2, b = 0.75, as tantivy and infino; the (k1+1) numerator is a constant factor and does not change ranking),
`idf = ln(1 + (N - df + 0.5) / (df + 0.5))`, exact article lengths and f64 arithmetic. Empty
articles are skipped.

## Build and read

`build_index` streams the corpus from stdin and indexes each whole article as one lexical chunk
with token positions (text is not stored). Every document must be accepted: a rejected document
stops the build with the error. It then compacts the store, which reorders it, and flushes.
The checkpoint is version 4: block-encoded postings with impacts, skip tables and word
positions, and a section of tables derived from the index, written when the checkpoint is
written: per-slot word counts and classes, per-list rank tables, union window bounds, and
direct frequency tables for the longest lists (every list with at least 512 postings and a 1/32
share of the slots, longest first, within a 32 MiB budget or 128 tables). Which lists get a
table depends only on the index. `do_query` opens the store, reads those tables and keeps no
state between queries that depends on them. `WARM_DENSE_TABLES=0` opens without the direct
frequency tables so their effect can be checked; results are identical. The binaries use the
mimalloc global allocator.

`do_query` stops with the error when a query does not parse or a search fails. `UNSUPPORTED` is
printed only for the `TOP_*_FF` commands.

## Dependency

`warm-rust = "=0.7.5"` from crates.io; the exact version and `Cargo.lock` are committed.
