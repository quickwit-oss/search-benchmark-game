clean:
	@echo "--- Cleaning Lucene 9.12.0 ---"
	@rm -fr idx
	@rm -fr build

serve:
	@java -XX:+UseParallelGC --add-modules jdk.incubator.vector -cp "build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT.jar:build/dependencies/*" DoQuery idx

index: idx

compile:
	@echo "--- Building Lucene 9.12.0"
	@./gradlew clean build copyDependencies

idx: compile
	@echo "--- Indexing Lucene 9.12.0 ---"
	@java -server -cp "build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT.jar:build/dependencies/*" BuildIndex idx < $(CORPUS)
