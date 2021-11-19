import org.apache.lucene.analysis.CharArraySet;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.index.DirectoryReader;
import org.apache.lucene.index.IndexReader;
import org.apache.lucene.queryparser.classic.ParseException;
import org.apache.lucene.queryparser.classic.QueryParser;
import org.apache.lucene.search.*;
import org.apache.lucene.store.FSDirectory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Path;
import java.nio.file.Paths;

public class DoQuery {
    public static void main(String[] args) throws IOException, ParseException {
        final Path indexDir = Paths.get(args[0]);
        try (IndexReader reader = DirectoryReader.open(FSDirectory.open(indexDir))) {
            final IndexSearcher searcher = new IndexSearcher(reader);
            searcher.setQueryCache(null);
            try (BufferedReader bufferedReader = new BufferedReader(new InputStreamReader(System.in))) {
                final QueryParser queryParser = new QueryParser("text", new StandardAnalyzer(CharArraySet.EMPTY_SET));
                String line;
                while ((line = bufferedReader.readLine()) != null) {
                    final String[] fields = line.trim().split("\t");
                    assert fields.length == 2;
                    final String command = fields[0];
                    final String query_str = fields[1];
                    Query query = queryParser
                            .parse(query_str)
                            .rewrite(reader);
                    final int count;
                    final TotalHitCountCollector countCollector = new TotalHitCountCollector();
                    switch (command) {
                        case "COUNT":
                            count = searcher.count(query);
                            break;
                        case "TOP_10":
                            {

                                final TopDocs topDocs = searcher.search(query, 10);
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
}
