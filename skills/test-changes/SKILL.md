---
description: Test tantivy changes by pointing the engine at a worktree
---

# Test Changes

Create a worktree in the tantivy repo and point the engine at it via a path
dependency:
```bash
cd <tantivy-repo>
git worktree add <worktree-name>
(set `<worktree-name>` to current upstream/main)
```

Copy the existing tantivy directory from `engines/tantivy-main` to `engines/<new-tantivy-engine-name>
(usually worktree), update its `Cargo.toml` to point at the new library version, and add it
to `ENGINES` in the `Makefile`.

In `engines/<new-tantivy-engine-name>/Cargo.toml`:
```toml
tantivy = { path = "<tantivy-repo>/<worktree-name>" }
```
Rebuild and bench:
```bash
cd engines/<new-tantivy-engine-name>/ && cargo build --release && cd -
WARMUP_TIME=3 make bench
python tools/analyze.py <engine> TOP_100_COUNT --filter union
```
