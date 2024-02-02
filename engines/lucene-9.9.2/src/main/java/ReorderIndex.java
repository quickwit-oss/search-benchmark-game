import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.concurrent.ForkJoinPool;

import org.apache.lucene.analysis.CharArraySet;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.index.CodecReader;
import org.apache.lucene.index.DirectoryReader;
import org.apache.lucene.index.IndexReader;
import org.apache.lucene.index.IndexWriter;
import org.apache.lucene.index.IndexWriterConfig;
import org.apache.lucene.index.IndexWriterConfig.OpenMode;
import org.apache.lucene.index.SlowCodecReaderWrapper;
import org.apache.lucene.misc.index.BPIndexReorderer;
import org.apache.lucene.store.Directory;
import org.apache.lucene.store.FSDirectory;

public class ReorderIndex {

    public static void main(String[] args) throws Exception {
        final Path indexPath = Paths.get(args[0]);
        final Path reorderedIndexPath = Paths.get(args[1]);

        final StandardAnalyzer standardAnalyzer = new StandardAnalyzer(CharArraySet.EMPTY_SET);
        final IndexWriterConfig config = new IndexWriterConfig(standardAnalyzer)
                .setOpenMode(OpenMode.CREATE);

        try (Directory dir = FSDirectory.open(indexPath);
                Directory reorderedDir = FSDirectory.open(reorderedIndexPath);
                IndexReader reader = DirectoryReader.open(dir);
                IndexWriter writer = new IndexWriter(reorderedDir, config)) {

            System.out.println("Reorder");
            if (reader.leaves().size() != 1) {
                throw new Error("Expected force-merged input index");
            }
            BPIndexReorderer reorderer = new BPIndexReorderer();
            reorderer.setForkJoinPool(ForkJoinPool.commonPool());
            reorderer.setMinDocFreq(1024);
            reorderer.setRAMBudgetMB(256);
            CodecReader reordered = reorderer.reorder(SlowCodecReaderWrapper.wrap(reader.leaves().get(0).reader()), reorderedDir);
            System.out.println("Write reordered index");
            writer.addIndexes(reordered);
        }
    }
}
