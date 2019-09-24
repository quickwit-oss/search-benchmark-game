clean:
	@echo "--- Cleaning tantivy 0.8 ---"
	@rm -fr idx
	@rm -fr target

compile: target/release/build_index target/release/do_query

index: idx

serve: target/release/do_query
	@target/release/do_query idx

target/release/%:
	@echo "--- Building tantivy 0.8 ---"
	@cargo build --release --bin $(notdir $@)

idx: target/release/build_index
	@echo "--- Indexing tantivy 0.8 ---"
	@mkdir -p idx
	export RUST_LOG=info && target/release/build_index "$@" < ${CORPUS}
