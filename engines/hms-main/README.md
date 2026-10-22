# HMS (holographic-memory)

[HMS](https://github.com/writerslogic/holographic-memory) is an Apache-2.0 Rust library for local-first document memory: BM25 lexical search, vector search, and hybrid search over the same store. This engine uses only its lexical side. Documents are memorized with `EncodeMode::Lexical`, so no vectors are computed, and queries go through `HmsCore::search_documents` and `HmsCore::count_documents`. The dependency is pinned to a git revision of `main` in `Cargo.toml`.

## Supported queries

| Query type | TOP_10 / TOP_100 / TOP_1000 | COUNT / TOP_N_COUNT |
|---|---|---|
| term, union (`a b`) | yes | yes |
| intersection (`+a +b`) | UNSUPPORTED | UNSUPPORTED |
| intersection_union (`+a b`) | UNSUPPORTED | UNSUPPORTED |
| negated (`+a -b`) | UNSUPPORTED | UNSUPPORTED |
| phrase (`"a b"`) | UNSUPPORTED | UNSUPPORTED |

HMS search is a BM25 disjunction over the query's words. It has no required or excluded clause, and it stores no positions, so the engine answers `UNSUPPORTED` for any query containing `+`, `-` or `"`. `TOP_N_COUNT` runs the top-N search, then `count_documents`, as two calls.

## Differences from Lucene

- **Tokenization.** HMS splits on non-alphanumeric characters and lowercases. On this corpus, which `corpus_transform.py` has already reduced to lowercase ASCII letters and spaces, that matches whitespace tokenization. Words are indexed without stemming (`IngestOptions::unstemmed`).
- **Scoring.** BM25 with k1 = 1.2, b = 0.75 and idf = ln(1 + (N - df + 0.5) / (df + 0.5)), as in Lucene. HMS keeps exact document lengths rather than Lucene's one-byte norms, and it scales scores by k1 + 1, which does not change the ranking.
- **Long articles.** HMS indexes documents as chunks of at most 4,096 words, the largest size it accepts, with no overlap. 18,538 of the 5,032,104 articles (0.4%) are longer than that and are split into 40,643 chunks. For those articles, BM25 statistics (N, df, length) are per chunk and a ranked hit is a chunk. Counts are per article: a document is counted once however many of its chunks match.
- **Empty articles.** 391,611 corpus lines have no words left after the transform. HMS rejects a document without text, so `build_index` skips them. They match no query in any engine.
- **Index.** The store is an append-only journal that HMS replays into memory when it opens. It takes 1.5 GB on disk.

## Resources

On an Intel Core Ultra 9 185H, the full corpus needs about 14 GB of RAM, both to index and to serve. Each `serve` takes 3 to 5.5 minutes to open before it answers its first query, depending on host load, and the client starts one `serve` per command. The warmup progress bar stays at 0% during that time; the run has not hung. Indexing with 4 threads takes about 6 minutes.
