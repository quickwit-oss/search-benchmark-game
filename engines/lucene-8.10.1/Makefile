clean:
	@echo "--- Cleaning Lucene 8.10.1 ---"
	@rm -fr idx
	@rm -fr build

compile: build

serve:
	@java -XX:+UseParallelGC -cp build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT-all.jar DoQuery idx

index: idx

build:
	@echo "--- Building Lucene 8.4.0 ---"
	@gradle clean shadowJar

idx: build
	@echo "--- Indexing Lucene 8.4.0 ---"
	java -server -cp build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT-all.jar BuildIndex idx < $(CORPUS)
