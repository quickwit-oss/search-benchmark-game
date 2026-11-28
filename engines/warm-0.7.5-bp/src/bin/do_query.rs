// Copyright 2024-2026 WritersLogic Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Context, Result};
use writerslogic_warm::core::documents::{LexicalExecution, LexicalOptions, LexicalQuery};
use writerslogic_warm::WarmCore;
use std::io::{BufRead, Write};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .context("usage: do_query <index dir>")?;
    let core = WarmCore::new(1024, Some(path), None)?;
    if std::env::var_os("WARM_RESOURCE_RECEIPT").is_some() {
        let status = std::fs::read_to_string("/proc/self/status")
            .context("read Linux memory receipt after reopening")?;
        eprintln!(
            "WARM_RESOURCE_AFTER_OPEN={}",
            serde_json::to_string(&status)?
        );
    }
    let execution = match std::env::var("WARM_LEXICAL_EXECUTION")
        .as_deref()
        .unwrap_or("auto")
    {
        "auto" => LexicalExecution::Auto,
        "daat" => LexicalExecution::DocumentAtATime,
        "wand" => LexicalExecution::BlockMaxWand,
        "exhaustive" => LexicalExecution::Exhaustive,
        other => bail!("unknown WARM_LEXICAL_EXECUTION {other:?}"),
    };
    let dump = std::env::var_os("WARM_DUMP").is_some();
    let differential = std::env::var_os("WARM_DIFFERENTIAL").is_some();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let (command, text) = line
            .split_once('\t')
            .context("expected <COMMAND>\t<query>")?;
        let (k, count) = match command {
            "TOP_10" => (10, false),
            "TOP_100" => (100, false),
            "TOP_1000" => (1000, false),
            "COUNT" => (0, true),
            "TOP_10_COUNT" => (10, true),
            "TOP_100_COUNT" => (100, true),
            "TOP_1000_COUNT" => (1000, true),
            // The only commands outside the supported list are the TOP_*_FF ones.
            other if other.starts_with("TOP_") && other.ends_with("_FF") => {
                writeln!(out, "UNSUPPORTED")?;
                out.flush()?;
                continue;
            }
            other => bail!("unknown command {other:?}"),
        };
        // A query that does not parse or a search that fails is a defect, not an unsupported
        // command: it stops the server with the error.
        let query = LexicalQuery::parse(text)
            .with_context(|| format!("parse query {text:?}"))?;
        let options = LexicalOptions {
            k,
            count,
            execution,
            ..Default::default()
        };
        let result = core
            .search_lexical(&query, &options)
            .with_context(|| format!("search {command} {text:?}"))?;
        if differential {
            let reference = core.search_lexical(
                &query,
                &LexicalOptions {
                    execution: LexicalExecution::Exhaustive,
                    ..options
                },
            )?;
            anyhow::ensure!(result == reference, "differential mismatch: {text}");
        }
        if dump {
            let hits: Vec<String> = result
                .hits
                .iter()
                .map(|h| format!("{}:{:.4}", h.document_id, h.score))
                .collect();
            eprintln!("{command}\t{text}\t{:?}\t{}", result.count, hits.join(" "));
        }
        writeln!(out, "{}", result.count.unwrap_or(1))?;
        out.flush()?;
        std::hint::black_box(result);
    }
    std::process::exit(0);
}
