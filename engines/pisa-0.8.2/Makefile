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

TBB_LIB := $(shell pwd)/build/pisa/external/tbb_cmake_build/tbb_cmake_build_subdir_release

serve: build/bin/do_query
	@LD_LIBRARY_PATH=$(TBB_LIB) ./build/bin/do_query idx

build/bin/%:
	@echo "--- Building PISA 0.8.2 ---"
	@mkdir -p build
	@cmake -B build/ .
	@make -C build/

idx: build/bin/build_index
	@echo "--- Indexing PISA 0.8.2 ---"
	@mkdir -p idx
	@LD_LIBRARY_PATH=$(TBB_LIB) ./build/bin/build_index < ${CORPUS}
