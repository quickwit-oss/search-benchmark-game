clean:
	rm -fr idx
	rm -fr target

compile: target/release/build_index target/release/do_query

index: idx

serve: target/release/do_query
	@target/release/do_query idx

target/release/build_index target/release/do_query &:
	@echo "\n\n\n--- Building WARM's binary ---" >&2
	@RUSTFLAGS='-C target-cpu=native' cargo build --release --locked --bin build_index --bin do_query

idx: target/release/build_index
	@echo "\n\n\n---- Indexing WARM ----"
	@mkdir -p idx
	target/release/build_index idx < ${CORPUS}
