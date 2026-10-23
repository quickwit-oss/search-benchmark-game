INDEX ?= idx

.PHONY: compile clean index serve
clean:
	rm -fr $(INDEX)
	rm -fr target

compile:
	RUSTFLAGS='-C target-cpu=native' cargo build --release --locked --bins

index: $(INDEX)

serve: target/release/do_query
	@target/release/do_query $(INDEX)

target/release/build_index target/release/do_query:
	$(MAKE) compile

$(INDEX): target/release/build_index
	target/release/build_index "$@" < ${CORPUS}
