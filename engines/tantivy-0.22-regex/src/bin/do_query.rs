#![macro_use]
extern crate tantivy;

use tantivy::collector::{Collector, Count, SegmentCollector, TopDocs};
use tantivy::query::{wildcard_query_to_regex_str, Query, QueryParser, RegexPhraseQuery, Weight};
use tantivy::schema::Field;
use tantivy::tokenizer::TokenizerManager;
use tantivy::{DocId, Index, Score, SegmentReader, Term, TERMINATED};

use std::collections::BinaryHeap;
use std::env;
use std::io::BufRead;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    main_inner(Path::new(&args[1])).unwrap()
}

struct Float(Score);

use std::cmp::Ordering;

impl Eq for Float {}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Float {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}

fn checkpoints_pruning(
    weight: &dyn Weight,
    reader: &SegmentReader,
    n: usize,
) -> tantivy::Result<Vec<(DocId, Score, Score)>> {
    let mut heap: BinaryHeap<Float> = BinaryHeap::with_capacity(n);
    let mut checkpoints: Vec<(DocId, Score, Score)> = Vec::new();
    let mut limit: Score = 0.0;
    weight.for_each_pruning(Score::MIN, reader, &mut |doc, score| {
        checkpoints.push((doc, score, score));
        // println!("pruning doc={} score={} limit={}", doc, score, limit);
        heap.push(Float(score));
        if heap.len() > n {
            heap.pop().unwrap();
        }
        limit = heap.peek().unwrap().0;
        limit
    })?;
    Ok(checkpoints)
}

fn checkpoints_no_pruning(
    weight: &dyn Weight,
    reader: &SegmentReader,
    n: usize,
) -> tantivy::Result<Vec<(DocId, Score, Score)>> {
    let mut heap: BinaryHeap<Float> = BinaryHeap::with_capacity(n);
    let mut checkpoints: Vec<(DocId, Score, Score)> = Vec::new();
    let mut scorer = weight.scorer(reader, 1.0)?;
    let mut limit = Score::MIN;
    loop {
        if scorer.doc() == TERMINATED {
            break;
        }
        let doc = scorer.doc();
        let score = scorer.score();
        if score > limit {
            // println!("nopruning doc={} score={} limit={}", doc, score, limit);
            checkpoints.push((doc, limit, score));
            heap.push(Float(score));
            if heap.len() > n {
                heap.pop().unwrap();
            }
            limit = heap.peek().unwrap().0;
        }
        scorer.advance();
    }
    Ok(checkpoints)
}

fn assert_nearly_equals(left: Score, right: Score) -> bool {
    (left - right).abs() * 2.0 / (left + right).abs() < 0.000001
}

struct UnoptimizedCount;

struct UnoptimizedCountSegmentCollector(u64);

impl SegmentCollector for UnoptimizedCountSegmentCollector {
    type Fruit = u64;

    #[inline]
    fn collect(&mut self, _doc: DocId, _score: Score) {
        self.0 += 1;
    }

    #[inline]
    fn harvest(self) -> Self::Fruit {
        self.0
    }
}

impl Collector for UnoptimizedCount {
    type Fruit = u64;

    type Child = UnoptimizedCountSegmentCollector;

    #[inline]
    fn for_segment(
        &self,
        _segment_local_id: tantivy::SegmentOrdinal,
        _segment: &SegmentReader,
    ) -> tantivy::Result<Self::Child> {
        Ok(UnoptimizedCountSegmentCollector(0u64))
    }

    #[inline]
    fn requires_scoring(&self) -> bool {
        false
    }

    #[inline]
    fn merge_fruits(&self, segment_fruits: Vec<u64>) -> tantivy::Result<u64> {
        Ok(segment_fruits.into_iter().sum())
    }
}

fn main_inner(index_dir: &Path) -> tantivy::Result<()> {
    env_logger::init();

    let index = Index::open_in_dir(index_dir).expect("failed to open index");
    let text_field = index.schema().get_field("text").expect("no all field?!");
    let query_parser = QueryParser::new(
        index.schema(),
        vec![text_field],
        TokenizerManager::default(),
    );
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let stdin = std::io::stdin();
    for line_res in stdin.lock().lines() {
        let line = line_res?;
        let fields: Vec<&str> = line.split("\t").collect();
        assert_eq!(
            fields.len(),
            2,
            "Expected a line in the format <COMMAND> query."
        );
        let command = fields[0];

        let query_str = &fields[1];
        let query = if query_str.contains("*") {
            regex_phrase_query(query_str, text_field)
        } else {
            query_parser.parse_query(fields[1])?
        };

        let count;
        match command {
            "COUNT" => {
                count = query.count(&searcher)?;
            }
            "UNOPTIMIZED_COUNT" => {
                count = searcher.search(&query, &UnoptimizedCount)? as usize;
            }
            "TOP_10" => {
                let _top_k = searcher.search(&query, &TopDocs::with_limit(10))?;
                count = 1;
            }
            "TOP_100" => {
                let _top_k = searcher.search(&query, &TopDocs::with_limit(100))?;
                count = 1;
            }
            "TOP_1000" => {
                let _top_k = searcher.search(&query, &TopDocs::with_limit(1000))?;
                count = 1;
            }
            "TOP_1_COUNT" => {
                let (_top_k, count_) = searcher.search(&query, &(TopDocs::with_limit(1), Count))?;
                count = count_;
            }
            "TOP_5_COUNT" => {
                let (_top_k, count_) = searcher.search(&query, &(TopDocs::with_limit(5), Count))?;
                count = count_;
            }
            "TOP_10_COUNT" => {
                let (_top_k, count_) =
                    searcher.search(&query, &(TopDocs::with_limit(10), Count))?;
                count = count_;
            }
            "TOP_100_COUNT" => {
                let (_top_k, count_) =
                    searcher.search(&query, &(TopDocs::with_limit(100), Count))?;
                count = count_;
            }
            "TOP_1000_COUNT" => {
                let (_top_k, count_) =
                    searcher.search(&query, &(TopDocs::with_limit(1000), Count))?;
                count = count_;
            }
            "DEBUG_TOP_10" => {
                let weight = query.weight(tantivy::query::EnableScoring::enabled_from_searcher(
                    &searcher,
                ))?;
                for reader in searcher.segment_readers() {
                    let checkpoints_left = checkpoints_no_pruning(&*weight, reader, 10)?;
                    let checkpoints_right = checkpoints_pruning(&*weight, reader, 10)?;
                }
                count = 0;
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

fn regex_phrase_query(query: &str, field: Field) -> Box<dyn Query> {
    let mut query = query.to_string();
    let mut slop = 0;

    // Check if the query contains slop in the form "phrase"~3
    if let Some(pos) = query.rfind('~') {
        // Extract the slop value
        if let Ok(parsed_slop) = query[pos + 1..].trim().parse::<u32>() {
            slop = parsed_slop;
            query = query[..pos].trim().to_string(); // Remove the slop part
        }
    }

    // Remove starting and ending quotes if present
    if query.starts_with("\"") {
        query = query[1..].to_string();
    }
    if query.ends_with("\"") {
        query.pop();
    }

    // Split terms and create a list of terms
    let terms: Vec<String> = query
        .split_whitespace()
        .map(wildcard_query_to_regex_str)
        .collect();

    // Create a RegexPhraseQuery and set properties
    let mut regex_phrase_query = RegexPhraseQuery::new(field, terms);
    regex_phrase_query.set_max_expansions(1_000_000);
    regex_phrase_query.set_slop(slop); // Set slop value

    Box::new(regex_phrase_query)
}
