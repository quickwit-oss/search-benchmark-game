import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.UncheckedIOException;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

import org.apache.lucene.analysis.CharArraySet;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.Document;
import org.apache.lucene.document.Field;
import org.apache.lucene.document.StoredField;
import org.apache.lucene.document.TextField;
import org.apache.lucene.index.IndexWriter;
import org.apache.lucene.index.IndexWriterConfig;
import org.apache.lucene.index.IndexWriterConfig.OpenMode;
import org.apache.lucene.store.Directory;
import org.apache.lucene.store.FSDirectory;
import org.apache.lucene.util.ThreadInterruptedException;

import com.eclipsesource.json.Json;
import com.eclipsesource.json.JsonObject;

public class BuildIndex {

	public static void main(String[] args) throws Exception {
		final Path outputPath = Paths.get(args[0]);

		final StandardAnalyzer standardAnalyzer = new StandardAnalyzer(CharArraySet.EMPTY_SET);
		final IndexWriterConfig config = new IndexWriterConfig(standardAnalyzer)
				.setRAMBufferSizeMB(1024)
				.setOpenMode(OpenMode.CREATE);

		try (Directory dir = FSDirectory.open(outputPath);
				IndexWriter writer = new IndexWriter(dir, config);
				BufferedReader bufferedReader = new BufferedReader(new InputStreamReader(System.in))) {
			final BlockingQueue<String> workQueue = new ArrayBlockingQueue<>(1000);
			final AtomicBoolean done = new AtomicBoolean();

			final Thread[] threads = new Thread[Runtime.getRuntime().availableProcessors()];
			final AtomicInteger indexed = new AtomicInteger();
			for (int i = 0; i < threads.length; ++i) {

				final Document document = new Document();
				StoredField idField = new StoredField("id", "");
				TextField textField = new TextField("text", "", Field.Store.NO);

				document.add(idField);
				document.add(textField);

				threads[i] = new Thread(() -> {
					while (true) {
						String line;
						try {
							line = workQueue.poll(100, TimeUnit.MILLISECONDS);
						} catch (InterruptedException e) {
							throw new ThreadInterruptedException(e);
						}
						if (line == null) {
							if (done.get()) {
								break;
							} else {
								continue;
							}
						}

						line = line.trim();
						if (line.isEmpty()) {
							continue;
						}
						final JsonObject parsed_doc = Json.parse(line).asObject();
						final String id = parsed_doc.get("id").asString();
						final String text = parsed_doc.get("text").asString();
						idField.setStringValue(id);
						textField.setStringValue(text);
						try {
							writer.addDocument(document);
							final int numIndexed = indexed.getAndIncrement();
							if (numIndexed % 100_000 == 0) {
							    System.out.println("Indexed: " + numIndexed);
							}
						} catch (IOException e) {
							throw new UncheckedIOException(e);
						}
					}
				});
			}

			System.out.println("Index");
			for (Thread thread : threads) {
				thread.start();
			}
			String line;
			while ((line = bufferedReader.readLine()) != null) {
				workQueue.put(line);
			}
			done.set(true);
			for (Thread thread : threads) {
				thread.join();
			}
			System.out.println("Merge");
			writer.forceMerge(1, true);
		}
	}
}
