clean:
	@echo "--- Cleaning Lucene 7.2.1 ---"
	@rm -fr idx
	@rm -fr build

compile: build

serve:
	@java -cp build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT-all.jar DoQuery idx

index: idx

build:
	@echo "--- Building Lucene 7.2.1 ---"
	@gradle clean shadowJar

idx: build
	@echo "--- Indexing Lucene 7.2.1 ---"
	java -server -cp build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT-all.jar BuildIndex idx < $(CORPUS)
