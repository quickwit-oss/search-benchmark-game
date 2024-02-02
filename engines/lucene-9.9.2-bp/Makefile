clean:
	@rm -fr idx

serve:
	@java -XX:+UseParallelGC -cp "${shell pwd}/../lucene-9.9.2/build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT.jar:${shell pwd}/../lucene-9.9.2/build/dependencies/*" DoQuery idx

index: idx

compile:
	cd ${shell pwd}/../lucene-9.9.2 && make compile

idx: compile
	cd ${shell pwd}/../lucene-9.9.2 && make index
	@echo "--- Reordering index with Lucene 9.9.2 ---"
	@java -server -cp "${shell pwd}/../lucene-9.9.2/build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT.jar:${shell pwd}/../lucene-9.9.2/build/dependencies/*" ReorderIndex ${shell pwd}/../lucene-9.9.2/idx idx
