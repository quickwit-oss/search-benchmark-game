import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Path;
import java.nio.file.Paths;

import org.apache.lucene.analysis.CharArraySet;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.index.DirectoryReader;
import org.apache.lucene.index.IndexReader;
import org.apache.lucene.queryparser.classic.ParseException;
import org.apache.lucene.queryparser.classic.QueryParser;
import org.apache.lucene.search.IndexSearcher;
import org.apache.lucene.search.Query;
import org.apache.lucene.search.TopScoreDocCollectorManager;
import org.apache.lucene.search.similarities.BM25Similarity;
import org.apache.lucene.store.FSDirectory;

public class DoQuery {
    public static void main(String[] args) throws IOException, ParseException {
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
                Query query = queryParser.parse(query_str);
                final long count;
                switch (command) {
                case "COUNT":
                case "UNOPTIMIZED_COUNT":
                    count = searcher.count(query);
                    break;
                case "TOP_10":
                {
                    searcher.search(query, new TopScoreDocCollectorManager(10, null, 10, false));
                    count = 1;
                }
                break;
                case "TOP_100":
                {
                    searcher.search(query, new TopScoreDocCollectorManager(100, null, 100, false));
                    count = 1;
                }
                break;
                case "TOP_1000":
                {
                    searcher.search(query, new TopScoreDocCollectorManager(1000, null, 1000, false));
                    count = 1;
                }
                break;
                case "TOP_10_COUNT":
                {
                    count = searcher.search(query, new TopScoreDocCollectorManager(10, null, Integer.MAX_VALUE, false)).totalHits.value();
                }
                break;
                case "TOP_100_COUNT":
                {
                   count = searcher.search(query, new TopScoreDocCollectorManager(100, null, Integer.MAX_VALUE, false)).totalHits.value();
                }
                break;
                case "TOP_1000_COUNT":
                {
                   count = searcher.search(query, new TopScoreDocCollectorManager(1000, null, Integer.MAX_VALUE, false)).totalHits.value();
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
}
