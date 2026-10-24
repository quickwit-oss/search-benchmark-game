# Current-main same-host SBG measurements

Engine `12c80decb094d57ee13ba187a01bf1de8788ca2f`; official harness `3347d5c70b671969db71a8e8f6a8d8d8ff9e8363`, adapter fork base `7dd66348acc21cf2ea5836fc9c301a5ef6720d0a`. Completed 2026-10-24 09:12:28 UTC. All three engines were rerun; no earlier timings were reused.

## Protocol and limits

Intel Core Ultra 9 185H, Ubuntu 24.04.4/WSL2, 27 GiB RAM, Rust 1.99.1, OpenJDK 21.0.13.0.2. Each query command used the unmodified official driver, `taskset -c 0-1`, `WARMUP_TIME=60`, `NUM_ITER=10`, `RAYON_NUM_THREADS=4`, math threads 1. All heavy host work held `~/hms-timing.lock`; builds/indexes and timed queries were serialized. Initial peer-process review found the unrelated research process and left it untouched. Fixed engine order HMS→Tantivy 0.26→Lucene 10.4.0 per command; fixed upstream query shuffle.

All 962 queries and seven primary query types completed, all five commands, 10 timed passes, zero unsupported. Mean latency below uses all samples as the current official web UI/tools do; README best-of-ten prose differs. QPS is reciprocal mean serial pipe-request latency, not bulk-server throughput. Count and rank correctness are separate: COUNT and TOP_100_COUNT return counts agreeing 962/962 among all three engines; other TOP_K responses are acknowledgement 1 and do not establish score/rank identity.

These are same-host diagnostic measurements, not published standings. Kernel VmSwap reached 46,808 KiB for HMS during the final command; its phase/effect cannot be reconstructed. There is no no-swap claim or cross-machine scaling. PISA/IResearch initially required unavailable Docker; their native official-CMake fallback is still queued separately. They are not measured columns here.

Corpus: official Dropbox English Wikipedia, transformed with unchanged `corpus_transform.py`, 5,032,104 lines, 391,611 empty skipped, 0 HMS rejects. Transformed SHA256 `b11c3c90041e4b88296f19c10b6469590bac200b76a4094a3b455e22e9bc71eb`; raw acquisition SHA/provenance and current query/client hashes are in committed evidence. No benchmark query was used for code tuning in this run.

## Whole-query-set means

|Command|HMS µs / derivedQPS|Tantivy µs / derivedQPS|Lucene µs / derivedQPS|
|---|---:|---:|---:|
|TOP_10|1,988.61 / 502.86|952.94 / 1,049.39|913.70 / 1,094.45|
|TOP_100|3,066.54 / 326.10|1,181.28 / 846.54|1,251.68 / 798.93|
|TOP_1000|4,494.53 / 222.49|1,623.40 / 615.99|1,766.07 / 566.23|
|TOP_100_COUNT|4,001.20 / 249.92|1,896.47 / 527.29|2,990.26 / 334.42|
|COUNT|2,733.97 / 365.77|983.89 / 1,016.38|951.50 / 1,050.97|

HMS has higher whole-set mean latency than both comparators for every command. Per-type outcomes and losses are retained below.

## Mean latency by query type

|Command / type|Queries|HMS µs|Tantivy µs|Lucene µs|
|---|---:|---:|---:|---:|
|TOP_10/term|1|461.00|1,085.40|841.60|
|TOP_10/intersection|300|1,103.97|570.70|627.49|
|TOP_10/union|301|1,658.84|967.87|734.95|
|TOP_10/phrase|300|3,107.20|1,157.85|1,208.87|
|TOP_10/intersection_union|40|2,834.80|1,975.68|1,758.13|
|TOP_10/negated|19|903.89|302.56|419.19|
|TOP_10/two-phase-critic|1|19,358.20|20,971.70|27,718.90|
|TOP_100/term|1|2,620.60|1,362.80|2,478.60|
|TOP_100/intersection|300|1,202.38|614.75|779.63|
|TOP_100/union|301|2,691.19|1,563.08|1,206.96|
|TOP_100/phrase|300|5,205.85|1,222.26|1,637.43|
|TOP_100/intersection_union|40|3,773.64|2,108.93|1,840.93|
|TOP_100/negated|19|1,003.37|348.39|639.76|
|TOP_100/two-phase-critic|1|44,858.80|22,462.30|27,426.70|
|TOP_1000/term|1|11,440.80|2,487.10|6,618.40|
|TOP_1000/intersection|300|1,559.96|677.20|894.59|
|TOP_1000/union|301|5,425.53|2,837.10|2,459.93|
|TOP_1000/phrase|300|6,344.80|1,264.12|1,839.53|
|TOP_1000/intersection_union|40|5,258.22|2,279.55|2,215.67|
|TOP_1000/negated|19|1,881.86|450.49|780.82|
|TOP_1000/two-phase-critic|1|61,699.30|23,118.50|28,203.40|
|TOP_100_COUNT/term|1|2,683.90|8,435.20|17,041.00|
|TOP_100_COUNT/intersection|300|1,251.81|654.05|756.26|
|TOP_100_COUNT/union|301|3,874.78|3,699.75|6,421.94|
|TOP_100_COUNT/phrase|300|6,803.08|1,279.51|1,927.69|
|TOP_100_COUNT/intersection_union|40|4,500.81|2,294.91|2,054.45|
|TOP_100_COUNT/negated|19|1,167.18|393.10|684.05|
|TOP_100_COUNT/two-phase-critic|1|61,491.70|23,014.50|26,226.90|
|COUNT/term|1|49.30|44.70|47.50|
|COUNT/intersection|300|936.11|520.96|464.79|
|COUNT/union|301|1,040.16|1,328.12|598.49|
|COUNT/phrase|300|6,545.12|1,196.79|1,866.63|
|COUNT/intersection_union|40|77.50|126.28|62.30|
|COUNT/negated|19|555.85|255.63|383.31|
|COUNT/two-phase-critic|1|58,905.80|21,458.10|25,945.60|

## Memory, disk, traceability

Observed kernel serving VmHWM during the final command: HMS 12,876,028 KiB, Tantivy 576,936 KiB, Lucene 846,944 KiB. The observer began during Lucene indexing; no HMS/Tantivy index-build peak or precise build-time claim is made. The first query waits for HMS journal replay; that open work precedes the timed passes.

Allocated index disk bytes (`du -sB1`): HMS 4,254,650,368; Tantivy 3,031,109,632; Lucene 2,815,549,440. Apparent preallocated length is not substituted for allocated bytes.

[Raw per-command JSON, exact source/input/binary hashes, resource summary and versions](evidence/20261024-current-main/) are committed here. `summary.json` includes 105 per-command/type/engine cells, p50/p95, supported sample counts and applicable count checks. Raw host logs and resource trace remain in the private coordination archive with this run. Host incremental billed cost $0; monetary valuation of owned hardware is not estimated. No new submission, ready transition, merge or published standing.

The configured 60-second warmup includes the initial server-open wait and completes only after an entire query pass; HMS therefore does not receive an additional fixed 60 seconds after journal replay. This is the unchanged upstream driver behavior.
