import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.ArrayList;

import org.apache.lucene.analysis.CharArraySet;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.index.DirectoryReader;
import org.apache.lucene.index.IndexReader;
import org.apache.lucene.index.Term;
import org.apache.lucene.queryparser.classic.ParseException;
import org.apache.lucene.queryparser.classic.QueryParser;
import org.apache.lucene.search.IndexSearcher;
import org.apache.lucene.search.Query;
import org.apache.lucene.search.TopScoreDocCollector;
import org.apache.lucene.search.similarities.BM25Similarity;
import org.apache.lucene.search.WildcardQuery;
import org.apache.lucene.search.BooleanQuery;
import org.apache.lucene.store.FSDirectory;

import org.apache.lucene.queries.spans.SpanNearQuery;
import org.apache.lucene.queries.spans.SpanQuery;
import org.apache.lucene.queries.spans.SpanTermQuery;
import org.apache.lucene.queries.spans.SpanMultiTermQueryWrapper;


public class DoQuery {
    public static void main(String[] args) throws IOException, ParseException {
        BooleanQuery.setMaxClauseCount(1000000);
        final Path indexDir = Paths.get(args[0]);
        try (IndexReader reader = DirectoryReader.open(FSDirectory.open(indexDir));
                BufferedReader bufferedReader = new BufferedReader(new InputStreamReader(System.in))) {
            final IndexSearcher searcher = new IndexSearcher(reader);
            searcher.setQueryCache(null);
            searcher.setSimilarity(new BM25Similarity(0.9f, 0.4f));
            final QueryParser queryParser = new QueryParser("text", new StandardAnalyzer(CharArraySet.EMPTY_SET));
            String line;
            while ((line = bufferedReader.readLine()) != null) {
                final String[] fields = line.trim().split("\t");
                assert fields.length == 2;
                final String command = fields[0];
                final String query_str = fields[1];
                Query query;

                if (isPhraseWithWildcards(query_str)) {
                    query = buildWildcardPhraseQuery("text", query_str);
                } else {
                    query = queryParser.parse(query_str);
                }

                final int count;
                switch (command) {
                case "COUNT":
                case "UNOPTIMIZED_COUNT":
                    count = searcher.count(query);
                    break;
                case "TOP_10":
                {
                    final TopScoreDocCollector topScoreDocCollector = TopScoreDocCollector.create(10, 10);
                    searcher.search(query, topScoreDocCollector);
                    count = 1;
                }
                break;
                case "TOP_100":
                {
                    final TopScoreDocCollector topScoreDocCollector = TopScoreDocCollector.create(100, 100);
                    searcher.search(query, topScoreDocCollector);
                    count = 1;
                }
                break;
                case "TOP_1000":
                {
                    final TopScoreDocCollector topScoreDocCollector = TopScoreDocCollector.create(1000, 1000);
                    searcher.search(query, topScoreDocCollector);
                    count = 1;
                }
                break;
                case "TOP_10_COUNT":
                {
                    final TopScoreDocCollector topScoreDocCollector = TopScoreDocCollector.create(10, Integer.MAX_VALUE);
                    searcher.search(query, topScoreDocCollector);
                    count = topScoreDocCollector.getTotalHits();
                }
                break;
                case "TOP_100_COUNT":
                {
                    final TopScoreDocCollector topScoreDocCollector = TopScoreDocCollector.create(100, Integer.MAX_VALUE);
                    searcher.search(query, topScoreDocCollector);
                    count = topScoreDocCollector.getTotalHits();
                }
                break;
                case "TOP_1000_COUNT":
                {
                    final TopScoreDocCollector topScoreDocCollector = TopScoreDocCollector.create(1000, Integer.MAX_VALUE);
                    searcher.search(query, topScoreDocCollector);
                    count = topScoreDocCollector.getTotalHits();
                }
                break;
                default:
                    System.out.println("UNSUPPORTED");
                    count = 0;
                    break;
                }
                System.out.println(count);
            }
        }
    }

    private static boolean isPhraseWithWildcards(String queryStr) {
        return queryStr.startsWith("\"") && queryStr.endsWith("\"") && (queryStr.contains("*") || queryStr.contains("?"));
    }

    private static Query buildWildcardPhraseQuery(String field, String phrase) {
        // Default slop value
        int slop = 0;  

        // Find the tilde (~) and parse the slop if present
        int tildeIndex = phrase.lastIndexOf("~");
        if (tildeIndex != -1) {
            try {
                slop = Integer.parseInt(phrase.substring(tildeIndex + 1).trim());
                phrase = phrase.substring(0, tildeIndex).trim();  // Remove the slop part from the phrase
            } catch (NumberFormatException e) {
                // Handle the case where the slop value isn't a valid integer, if needed
                throw new IllegalArgumentException("Invalid slop value in query: " + phrase);
            }
        }

        // Remove quotes if present
        if (phrase.startsWith("\"") && phrase.endsWith("\"")) {
            phrase = phrase.substring(1, phrase.length() - 1);
        }

        // Split the phrase into terms
        String[] terms = phrase.split("\\s+");
        List<SpanQuery> clauses = new ArrayList<>();

        for (String term : terms) {
            SpanQuery spanQuery;
            if (term.contains("*") || term.contains("?")) {
                WildcardQuery wildcardQuery = new WildcardQuery(new Term(field, term));
                spanQuery = new SpanMultiTermQueryWrapper<>(wildcardQuery);
            } else {
                spanQuery = new SpanTermQuery(new Term(field, term));
            }
            clauses.add(spanQuery);
        }

        // Create a SpanNearQuery with the specified slop value
        return new SpanNearQuery(clauses.toArray(new SpanQuery[0]), 3, true);
    }


}
