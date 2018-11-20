import com.eclipsesource.json.Json;
import com.eclipsesource.json.JsonObject;
import org.apache.lucene.analysis.CharArraySet;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.*;
import org.apache.lucene.index.IndexWriter;
import org.apache.lucene.index.IndexWriterConfig;
import org.apache.lucene.store.FSDirectory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Path;
import java.nio.file.Paths;

public class BuildIndex {
    public static void main(String[] args) throws IOException {
        final Path outputPath = Paths.get(args[0]);

        final StandardAnalyzer standardAnalyzer = new StandardAnalyzer(CharArraySet.EMPTY_SET);
        final IndexWriterConfig config = new IndexWriterConfig(standardAnalyzer);
        config.setRAMBufferSizeMB(1000);
        try (IndexWriter writer = new IndexWriter(FSDirectory.open(outputPath), config)) {
            try (BufferedReader bufferedReader = new BufferedReader(new InputStreamReader(System.in))) {
                final Document document = new Document();

                StoredField idField = new StoredField("url",     "");
                TextField titleField = new TextField("title", "", Field.Store.NO);
                TextField textField = new TextField("body", "", Field.Store.NO);

                document.add(idField);
                document.add(titleField);
                document.add(textField);

                String line;
                while ((line = bufferedReader.readLine()) != null) {
                    if (line.trim().isEmpty()) {
                        continue;
                    }
                    final JsonObject parsed_doc = Json.parse(line).asObject();
                    final String id = parsed_doc.get("url").asString();
                    final String text = parsed_doc.get("body").asString();
                    final String title = parsed_doc.get("title").asString();
                    idField.setStringValue(id);
                    textField.setStringValue(text);
                    titleField.setStringValue(title);
                    writer.addDocument(document);
                }
            }

            writer.commit();
            writer.forceMerge(1, true);
        }
    }
}
