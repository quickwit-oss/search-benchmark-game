// Copyright 2024-2026 WritersLogic Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Context, Result};
use holographic_memory::core::documents::{LexicalExecution, LexicalOptions, LexicalQuery};
use holographic_memory::HmsCore;
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

/// The harness starts the next command's server as soon as it closes this one's input, so the
/// store lock may still be held for a moment by the previous process.
fn open(path: &str) -> Result<HmsCore> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        match HmsCore::new(1024, Some(path.to_owned()), None) {
            Ok(core) => return Ok(core),
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)),
            Err(error) => return Err(error),
        }
    }
}

fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .context("usage: do_query <index dir>")?;
    let core = open(&path)?;
    if std::env::var_os("HMS_RESOURCE_RECEIPT").is_some() {
        let status = std::fs::read_to_string("/proc/self/status")
            .context("read Linux memory receipt after reopening")?;
        eprintln!(
            "HMS_RESOURCE_AFTER_OPEN={}",
            serde_json::to_string(&status)?
        );
    }
    let execution = match std::env::var("HMS_LEXICAL_EXECUTION")
        .as_deref()
        .unwrap_or("auto")
    {
        "auto" => LexicalExecution::Auto,
        "daat" => LexicalExecution::DocumentAtATime,
        "wand" => LexicalExecution::BlockMaxWand,
        "exhaustive" => LexicalExecution::Exhaustive,
        other => bail!("unknown HMS_LEXICAL_EXECUTION {other:?}"),
    };
    let differential = std::env::var_os("HMS_DIFFERENTIAL").is_some();
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
            _ => {
                writeln!(out, "UNSUPPORTED")?;
                out.flush()?;
                continue;
            }
        };
        // A query outside the supported grammar is reported, not fatal to the server.
        let query = match LexicalQuery::parse(text) {
            Ok(query) => query,
            Err(error) => {
                eprintln!("unsupported query {text:?}: {error:#}");
                writeln!(out, "UNSUPPORTED")?;
                out.flush()?;
                continue;
            }
        };
        let options = LexicalOptions {
            k,
            count,
            execution,
            ..Default::default()
        };
        let result = core
            .search_lexical(&query, &options)
            .with_context(|| format!("query {text:?}"))?;
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
        writeln!(out, "{}", result.count.unwrap_or(1))?;
        out.flush()?;
        std::hint::black_box(result);
    }
    std::process::exit(0);
}
