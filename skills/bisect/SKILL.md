---
description: Bisect a performance regression in tantivy using the search-benchmark-game
---

# Bisect a Regression

Point the engine at a specific commit in `engines/<engine>/Cargo.toml`:
```toml
tantivy = { git = "https://github.com/quickwit-oss/tantivy.git", rev = "<commit>" }
```

Rebuild and bench:
```bash
cd engines/<engine>/ && cargo build --release && cd -
WARMUP_TIME=3 make bench
python tools/analyze.py <engine> TOP_100_COUNT --filter union
```
