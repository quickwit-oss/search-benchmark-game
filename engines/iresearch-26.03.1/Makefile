ROOT_DIR := $(abspath $(CURDIR)/../..)
DOCKER_IMAGE ?= iresearch-26.03.1-builder
DOCKER_TARGETARCH ?= $(shell uname -m | sed -e 's/^x86_64$$/amd64/' -e 's/^aarch64$$/arm64/')

submodules:
	@git -C serenedb submodule update --init --recursive --jobs $$(nproc)

clean:
	@echo "--- Cleaning iresearch ---"
	@rm -fr idx
	@rm -rf build

compile: submodules
	@echo "--- Building iresearch in docker ---"
	@docker build --build-arg TARGETARCH=$(DOCKER_TARGETARCH) -t $(DOCKER_IMAGE) $(ROOT_DIR)
	@rm -rf build
	@mkdir -p build/bin
	@docker run --rm \
		-v "$(CURDIR):/src:ro" \
		-v "$(CURDIR)/build/bin:/out" \
		$(DOCKER_IMAGE) \
		bash -lc 'rm -rf /container-build && mkdir -p /container-build && cp -a /src/serenedb /container-build/serenedb && cmake -S /container-build/serenedb -B /container-build/build --preset bench && cmake --build /container-build/build -t search-benchmark-game && cp /container-build/build/bin/search-benchmark-game-* /out/'

compile_debug: submodules
	@echo "--- Building iresearch ---"
	@cmake -S serenedb -B build_debug --preset lldb
	@cmake --build build_debug -t search-benchmark-game

compile_gen_pgo: submodules
	@echo "--- Building iresearch (base PGO gen) ---"
	@cmake -S serenedb -B build --preset bench -DSDB_GEN_PGO=ON
	@env LLVM_PROFILE_FILE="$(CURDIR)/pgo/code-%p.profraw" cmake --build build -t search-benchmark-game

compile_gen_cs_pgo:
	@echo "--- Building iresearch (CSIR PGO gen) ---"
	@cmake -S serenedb -B build --preset bench -DSDB_GEN_CS_PGO="$(CURDIR)/pgo/code.profdata"
	@env LLVM_PROFILE_FILE="$(CURDIR)/cs_pgo/code-%p.profraw" cmake --build build -t search-benchmark-game

compile_use_pgo:
	@echo "--- Building iresearch (use PGO) ---"
	@cmake -S serenedb -B build --preset bench -DSDB_USE_PGO="$(CURDIR)/pgo/code.profdata"
	@cmake --build build -t search-benchmark-game

index:
	@echo "--- Indexing iresearch ---"
	@mkdir -p idx
	@./build/bin/search-benchmark-game-build < ${CORPUS}

serve:
	@./build/bin/search-benchmark-game-query idx

serve_perf:
	@perf record -g -F 997 -o perf.data -- ./build/bin/search-benchmark-game-query idx

serve_gen_pgo:
	@env LLVM_PROFILE_FILE="$(CURDIR)/pgo/code-%p.profraw" ./build/bin/search-benchmark-game-query idx

serve_gen_cs_pgo:
	@env LLVM_PROFILE_FILE="$(CURDIR)/cs_pgo/code-%p.profraw" ./build/bin/search-benchmark-game-query idx
