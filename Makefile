CORPUS = $(shell pwd)/corpus.json
export

# COMMANDS = COUNT TOP_10 TOP_10_COUNT
# COMMANDS = COUNT
# COMMANDS = TOP_10
COMMANDS = TOP_10_COUNT

ENGINES = bleve-0.8.0-boltdb bleve-0.8.0-scorch lucene-7.2.1 lucene-8.0.0 tantivy-0.9
# ENGINES = lucene-7.2.1 lucene-8.0.0

all: index

corpus:
	@echo "--- Downloading wiki-articles.json.bz2 ---"
	@curl -# -L "https://www.dropbox.com/s/wwnfnu441w1ec9p/wiki-articles.json.bz2" > $(shell pwd)/wiki-articles.json.bz2
	@echo "--- Extracting wiki-articles.json.bz2 ---"
	@bunzip2 -f $(shell pwd)/wiki-articles.json.bz2
	@echo "--- Creating corpus.json ---"
	@jq -c '. | {id: .url, text: .body}' $(shell pwd)/wiki-articles.json > $(CORPUS)

clean:
	@echo "--- Cleaning directories ---"
	@rm -fr results
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make clean ; done

# Target to build the indexes of
# all of the search engine
index: $(INDEX_DIRECTORIES)
	@echo "--- Indexing corpus ---"
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make index ; done

# Target to run the query benchmark for
# all of the search engines
bench: #index compile
	@echo "--- Benchmarking ---"
	@rm -fr results
	@mkdir results
	@python3 src/client.py queries.txt $(ENGINES)

compile:
	@echo "--- Compiling binaries ---"
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make compile ; done

serve:
	@echo "--- Serving results ---"
	@cp results.json web/output/results.json
	#@cd web/output && python -m SimpleHTTPServer 80
	@cd web/output && python3 -m http.server 80