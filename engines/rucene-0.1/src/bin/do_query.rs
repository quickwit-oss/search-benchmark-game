use std::{env, mem};
use std::io::BufRead;
use std::path::Path;
use std::sync::Arc;
use rucene::core::search::collector::{SearchCollector, TopDocsCollector, Collector, ParallelLeafCollector, ChainedCollector};
use rucene::core::search::{DefaultIndexSearcher, IndexSearcher};
use rucene::core::store::directory::FSDirectory;
use rucene::core::index::reader::{StandardDirectoryReader, LeafReaderContext};
use rucene::core::codec::{CodecEnum, Codec};
use rucene::core::index::merge::{SerialMergeScheduler, TieredMergePolicy};
use rucene::core::search::query::{TermQuery, BooleanQuery, Query};
use rucene::core::doc::Term;
use rucene::core::search::scorer::Scorer;
use rucene::core::util::DocId;

#[derive(Default)]
struct Count {
    count: usize
}

impl ParallelLeafCollector for Count {
    fn finish_leaf(&mut self) -> rucene::error::Result<()> {
        Ok(())
    }
}

impl SearchCollector for Count {
    type LC = Count;

    fn set_next_reader<C: Codec>(&mut self, _reader: &LeafReaderContext<'_, C>) -> rucene::error::Result<()> {
        Ok(())
    }

    fn support_parallel(&self) -> bool {
        false
    }

    fn leaf_collector<C: Codec>(&self, _reader: &LeafReaderContext<'_, C>) -> rucene::error::Result<Self::LC> {
        Ok(Count::default())
    }

    fn finish_parallel(&mut self) -> rucene::error::Result<()> {
        unimplemented!()
    }
}

impl Collector for Count {
    fn needs_scores(&self) -> bool {
        false
    }

    fn collect<S: Scorer + ?Sized>(&mut self, _doc: DocId, _scorer: &mut S) -> rucene::error::Result<()> {
        self.count += 1;
        Ok(())
    }

}

fn main() {
    let args: Vec<String> = env::args().collect();
    main_inner(&Path::new(&args[1])).unwrap()
}

enum QueryType {
    Phrase,
    And,
    Or
}

fn parse_query(query_str: &str) -> Option<Box<dyn Query<CodecEnum>>> {
    let query_type;
    let terms: Vec<String>;
    if query_str.starts_with("\"") || query_str.starts_with("+\"") {
        query_type = QueryType::Phrase;
        let l = query_str.len() -1;
        terms = query_str[1..l-1].split_whitespace()
            .map(|term| term.to_string())
            .collect();
    } else if query_str.starts_with("+") {
        query_type = QueryType::And;
        terms = query_str.split_whitespace()
            .map(|term| term[1..].to_string())
            .collect();
    } else {
        query_type = QueryType::Or;
        terms = query_str.split_whitespace()
            .map(|term| term.to_string())
            .collect();
    }
    let terms: Vec<Term> = terms.into_iter()
        .map(|term| Term::new("text".into(), term.as_bytes().to_vec()))
        .collect();
    if terms.len() == 1 {
        return Some(Box::new(TermQuery::new(
            terms[0].clone(),
            1.0,
            None,
        )));
    }
    match query_type {
        QueryType::And => {
            return BooleanQuery::build(
                terms.iter()
                    .map(|term| {
                        let term_query: Box<dyn Query<CodecEnum>> = Box::new(TermQuery::new(term.clone(), 1.0, None));
                        term_query
                    })
                    .collect(),
                vec![],
                vec![]
            ).ok();
        }
        QueryType::Or => {
            return BooleanQuery::build(
                vec![],
                terms.iter()
                    .map(|term| {
                        let term_query: Box<dyn Query<CodecEnum>> = Box::new(TermQuery::new(term.clone(), 1.0, None));
                        term_query
                    })
                    .collect(),
                vec![]
            ).ok();
        }
        QueryType::Phrase => {
            // Disabling because it is not working.
            // let positions = (0i32..terms.len() as i32).collect();
            // return Ok(Box::new(PhraseQuery::new(terms, positions, 0, None, None)?))
            return None;
        }
    }
}

fn main_inner(index_dir: &Path) -> rucene::error::Result<()> {
    // create index writer
    let directory = Arc::new(FSDirectory::with_path(&index_dir)?);
    let reader: StandardDirectoryReader<FSDirectory, CodecEnum,  SerialMergeScheduler, TieredMergePolicy> = StandardDirectoryReader::open(directory)?;
    let searcher = DefaultIndexSearcher::new(Arc::new(reader), None, None);

    let stdin = std::io::stdin();
    for line_res in stdin.lock().lines() {
        let line = line_res?;
        let fields: Vec<&str> = line.split("\t").collect();
        assert_eq!(fields.len(), 2, "Expected a line in the format <COMMAND> query.");
        let command = fields[0];
        let query_str = fields[1].to_ascii_lowercase();
        let query_opt: Option<Box<dyn Query<_>>> = parse_query(&query_str);
        if query_opt.is_none() {
            println!("UNSUPPORTED");
            continue;
        }
        let query = query_opt.unwrap(); 
        let count;
        match command {
            "COUNT" => {
                let mut count_collector = Count::default();
                searcher.search(query.as_ref(), &mut count_collector)?;
                count = count_collector.count;
            }
            "TOP_10" => {
                let mut top_collector = TopDocsCollector::new(10);
                searcher.search(query.as_ref(), &mut top_collector)?;
                let _ = top_collector.top_docs();
                count = 1;
            }
            "TOP_10_COUNT" => {
                let mut count_collector = Count::default();
                let top_collector = TopDocsCollector::new(10);
                let mut chain = ChainedCollector::new(&mut count_collector, top_collector);
                searcher.search(query.as_ref(), &mut chain)?;
                mem::drop(chain);
                count = count_collector.count;
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
