import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Path;
import java.nio.file.Paths;

import org.apache.lucene.analysis.CharArraySet;
import org.apache.lucene.analysis.core.KeywordAnalyzer;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.index.DirectoryReader;
import org.apache.lucene.index.IndexReader;
import org.apache.lucene.queryparser.classic.ParseException;
import org.apache.lucene.queryparser.classic.QueryParser;
import org.apache.lucene.search.BooleanClause.Occur;
import org.apache.lucene.search.BooleanQuery;
import org.apache.lucene.search.IndexSearcher;
import org.apache.lucene.search.Query;
import org.apache.lucene.search.TopDocs;
import org.apache.lucene.search.TopScoreDocCollectorManager;
import org.apache.lucene.search.similarities.BM25Similarity;
import org.apache.lucene.store.FSDirectory;

public class DoQuery {

	private static final String FILTER_SEPARATOR = " WHERE ";

	public static void main(String[] args) throws IOException, ParseException {
		final Path indexDir = Paths.get(args[0]);
		try (IndexReader reader = DirectoryReader.open(FSDirectory.open(indexDir));
				BufferedReader bufferedReader = new BufferedReader(new InputStreamReader(System.in))) {
			final IndexSearcher searcher = new IndexSearcher(reader);
			searcher.setQueryCache(null);
			searcher.setSimilarity(new BM25Similarity(0.9f, 0.4f));
			final QueryParser queryParser = new QueryParser("text", new StandardAnalyzer(CharArraySet.EMPTY_SET));
			final QueryParser filterParser = new QueryParser("text", new KeywordAnalyzer());
			String line;
			while ((line = bufferedReader.readLine()) != null) {
				final String[] fields = line.trim().split("\t");
				assert fields.length == 2;
				final String command = fields[0];
				String query_str = fields[1];

				int filterIdx = query_str.indexOf(FILTER_SEPARATOR);
				Query filter = null;
				if (filterIdx >= 0) {
					String filter_str = query_str.substring(filterIdx + FILTER_SEPARATOR.length());
					query_str = query_str.substring(0, filterIdx);
					filter = filterParser.parse(filter_str);
				}

				Query query = queryParser.parse(query_str);
				if (filter != null) {
					query = new BooleanQuery.Builder().add(query, Occur.MUST).add(filter, Occur.FILTER).build();
				}

				final long count;
				switch (command) {
				case "COUNT":
				case "UNOPTIMIZED_COUNT":
					count = searcher.count(query);
					break;
				case "TOP_10":
				{
					TopDocs topDocs = searcher.search(query, new TopScoreDocCollectorManager(10, null, 10, false));
					count = Math.min(topDocs.totalHits.value(), 10);
				}
				break;
				case "TOP_100":
				{
					TopDocs topDocs = searcher.search(query, new TopScoreDocCollectorManager(100, null, 100, false));
					count = Math.min(topDocs.totalHits.value(), 100);
				}
				break;
				case "TOP_1000":
				{
					TopDocs topDocs = searcher.search(query, new TopScoreDocCollectorManager(1000, null, 1000, false));
					count = Math.min(topDocs.totalHits.value(), 1000);
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
