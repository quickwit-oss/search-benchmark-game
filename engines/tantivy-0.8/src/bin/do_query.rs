#![macro_use]
extern crate tantivy;

use tantivy::Index;
use tantivy::query::QueryParser;
use tantivy::tokenizer::TokenizerManager;
use tantivy::collector::{Count, TopDocs};

use std::env;
use std::io::BufRead;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    main_inner(&Path::new(&args[1])).unwrap()
}

fn main_inner(index_dir: &Path) -> tantivy::Result<()> {
    let index = Index::open_in_dir(index_dir).expect("failed to open index");
    let text_field = index.schema().get_field("text").expect("no all field?!");
    let query_parser = QueryParser::new(
        index.schema(),
        vec![text_field],
        TokenizerManager::default());

    let searcher = index.searcher();

    let stdin = std::io::stdin();
    for line_res in stdin.lock().lines() {
        let line = line_res?;
        let fields: Vec<&str> = line.split("\t").collect();
        assert_eq!(fields.len(), 2, "Expected a line in the format <COMMAND> query.");
        let command = fields[0];
        let query = query_parser.parse_query(fields[1])?;
        let count;
        match command {
            "COUNT" => {
                count = query.count(&searcher)?;
            }
            "TOP_10" => {
                let _top_k = searcher.search(&query, &TopDocs::with_limit(10))?;
                count = 1;
            }
            "TOP_10_COUNT" => {
                let (_top_k, count_) = searcher.search(&query, &(TopDocs::with_limit(10), Count))?;
                count = count_;
            }
            _ => {
                println!("UNSUPPORTED");
                continue;
            }
        }
        println!("{}", count);
    }

    Ok(())
}
