clean:
	@echo "--- Cleaning PISA 0.8.0 ---"
	@rm -fr idx
	@rm -fr bin

compile: bin/build_index bin/do_query

index: idx

serve: bin/do_query
	@./bin/do_query idx

bin/%:
	@echo "--- Building PISA 0.8.0 ---"
# 	@cargo build --release --bin $(notdir $@)

idx: bin/build_index
	@echo "--- Indexing PISA 0.8.0 ---"
	@mkdir -p idx
	@./bin/build_index < ${CORPUS}