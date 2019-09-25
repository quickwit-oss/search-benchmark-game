CORPUS = $(shell pwd)/corpus.json
export

WIKI_SRC = "https://www.dropbox.com/s/wwnfnu441w1ec9p/wiki-articles.json.bz2"
WIKI_DEST = $(shell pwd)/wiki-articles.json.bz2
WIKI_JSON = $(shell pwd)/wiki-articles.json


COMMANDS = COUNT TOP_10 TOP_10_COUNT
# COMMANDS = COUNT
# COMMANDS = TOP_10
# COMMANDS = TOP_10_COUNT

ENGINES = bleve-0.8.0-scorch lucene-8.0.0 tantivy-0.9
# ENGINES = bleve-0.8.0-scorch
# ENGINES = lucene-8.0.0
# ENGINES = tantivy-0.9

PORT = 8080

all: index

corpus:
	@echo "--- Downloading $(WIKI_SRC) ---"
	@curl -# -L "$(WIKI_SRC)" > $(WIKI_DEST)
	@echo "--- Extracting $(WIKI_DEST) ---"
	@bunzip2 -f $(WIKI_DEST)
	@echo "--- $(CORPUS) ---"
	@jq -c '. | {id: .url, text: .body}' $(WIKI_JSON) > $(CORPUS)

clean:
	@echo "--- Cleaning directories ---"
	@rm -fr results
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make clean ; done

index:
	@echo "--- Indexing corpus ---"
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make index ; done

bench:
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
	@cd web/output && python3 -m http.server $(PORT)
