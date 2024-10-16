clean:
	@echo "--- Cleaning Lucene 10.0.0 ---"
	@rm -fr idx
	@rm -fr build

serve:
	@java -XX:+UseParallelGC --add-modules jdk.incubator.vector --enable-native-access=ALL-UNNAMED -cp "build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT.jar:build/dependencies/*" DoQuery idx

index: idx

compile:
	@echo "--- Building Lucene 10.0.0"
	@./gradlew clean build copyDependencies

idx: compile
	@echo "--- Indexing Lucene 10.0.0 ---"
	@java -server --add-modules jdk.incubator.vector --enable-native-access=ALL-UNNAMED -cp "build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT.jar:build/dependencies/*" BuildIndex idx < $(CORPUS)
