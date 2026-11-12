# HMS (holographic-memory)

[HMS](https://github.com/writerslogic/holographic-memory) is an Apache-2.0 Rust library. This adapter uses whole-article lexical indexing with unstemmed tokens and positional postings. The tested revision of `main` is pinned in `Cargo.toml` and `Cargo.lock`. The Boolean, phrase and count execution it uses landed in [PR 200](https://github.com/writerslogic/holographic-memory/pull/200).

## Queries

All 962 benchmark queries are supported: term, union, intersection, intersection_union, negated, phrase and two-phase-critic. Ranked commands and COUNT use the same Boolean contract. Required clauses must match; optional clauses score and are required only when there are no required clauses; excluded clauses remove matches. Empty and solely negative queries match nothing. Duplicate clauses are idempotent. Phrases are ordered, adjacent, allow overlapping occurrences and preserve repeated terms. Nonzero slop and syntax outside the benchmark's flat grammar fail explicitly.

COUNT counts live articles without scoring. TOP_N_COUNT returns ranked hits and an exact article count. Set `HMS_LEXICAL_EXECUTION=auto|daat|wand|exhaustive` to select an execution path without rebuilding. WAND falls back to DAAT for unsupported shapes or unproven bounds. `HMS_DIFFERENTIAL=1` compares each result, including scores and tie order, with the full-scan/full-sort evaluator.

## Analyzer and scores

HMS splits Unicode alphanumeric tokens, lowercases and disables stemming here. The transformed corpus is lowercase ASCII with spaces. Lucene and Tantivy apply different maximum-token-length filters; queried tokens are at most 14 characters. Count parity is checked separately from ranking parity.

BM25 uses k1=1.2, b=0.75, `idf=ln(1+(N-df+.5)/(df+.5))`, exact article lengths and f64 arithmetic. Phrases use overlapping occurrence frequency and the sum of their terms' IDFs. Scores sort descending, then HMS document IDs ascending. Empty articles are skipped, so they do not contribute to HMS N or average length. External compressed norms, float width, empty-article statistics and phrase scoring can differ; external score/rank identity is not claimed.

Whole-article mode preserves conjunctions, exclusions and phrases across the former 4,096-word and 64-KiB chunk boundaries, up to HMS's 8-MiB document limit. Empty articles are skipped; an article HMS refuses is reported on stderr and counted in the final `build_index` line.

## Storage and verification

The journal persists term frequencies and versioned delta-coded positions. Replay reconstructs 128-posting bitpacked blocks and conservative BM25 impacts. Shared immutable dictionaries and compact frequencies/offsets reduce memory without changing persisted term bytes. Older non-positional stores remain readable, and phrase queries on them return an explicit capability error.

`uv run --no-project check_queries.py` checks the real binaries against an independent token-window count oracle and checks optimized/exhaustive equality for all queries, six commands and three execution choices. It includes repeated terms, negative-only queries, missing terms and the former chunk boundary.

## Resources and current measurements

The engine is pinned to `12c80decb094d57ee13ba187a01bf1de8788ca2f`. [Current results and raw evidence](RESULTS.md) cover all five commands, 962 queries and three engines rerun on the same host. HMS has higher whole-set mean latency than Tantivy and Lucene for each command; all per-type outcomes are retained. COUNT and TOP_100_COUNT counts agree on every query. TOP_K acknowledgement responses do not establish score/rank identity.

Serving kernel VmHWM observed during the final command was 12,876,028KiB; VmSwap reached 46,808KiB, so the timings carry a swap caveat. The allocated HMS index is 4,254,650,368 bytes. Its first request waits for journal replay, before timed passes. Earlier revision's resource estimates are superseded here; indexing peak/time were not measured separately in this run. PISA and IResearch native fallback measurements remain pending. This draft makes no published-standing claim.
