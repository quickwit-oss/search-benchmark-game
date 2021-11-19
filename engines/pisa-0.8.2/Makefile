help:
	@grep '^[^#[:space:]].*:' Makefile

clean:
	@echo "--- Cleaning PISA 0.8.2 ---"
	@rm -fr idx
	@rm -fr build
	@rm -fr CMakeCache.txt
	@rm -fr CMakeFiles


compile-within-container: build/bin/build_index build/bin/do_query
	echo "compiling"

compile:
	docker build -t pisa-builder .
	docker run --rm -it -v "$(shell pwd)":/home/pisa/src/ pisa-builder

index: idx

serve: build/bin/do_query
	@./build/bin/do_query idx

build/bin/%:
	@echo "--- Building PISA 0.8.2 ---"
	@mkdir -p build
	@cmake -B build/ .
	@make -C build/

idx: build/bin/build_index
	@echo "--- Indexing PISA 0.8.2 ---"
	@mkdir -p idx
	@./build/bin/build_index < ${CORPUS}
