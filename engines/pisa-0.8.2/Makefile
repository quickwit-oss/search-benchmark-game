ROOT_DIR := $(abspath $(CURDIR)/../..)
DOCKER_IMAGE ?= pisa-0.8.2-builder
HOST_OS := $(shell uname -s)
RUN_IN_DOCKER ?= $(if $(filter Darwin,$(HOST_OS)),1,0)
TBB_LIB := $(CURDIR)/build/pisa/external/tbb_cmake_build/tbb_cmake_build_subdir_release
CONTAINER_TBB_LIB := /home/pisa/src/build/pisa/external/tbb_cmake_build/tbb_cmake_build_subdir_release

help:
	@grep '^[^#[:space:]].*:' Makefile

submodules:
	@git -C $(ROOT_DIR) submodule update --init --recursive engines/pisa-0.8.2/pisa

clean:
	@echo "--- Cleaning PISA 0.8.2 ---"
	@rm -fr idx
	@rm -fr build
	@rm -fr CMakeCache.txt
	@rm -fr CMakeFiles


compile-within-container: build/bin/build_index build/bin/do_query
	echo "compiling"

compile: submodules
	docker build -t $(DOCKER_IMAGE) .
	@rm -rf build
	# Run a throwaway builder container with this directory mounted as /home/pisa/src.
	# --rm removes the stopped container; --user avoids root-owned build outputs.
	docker run --rm --user "$$(id -u):$$(id -g)" -v "$(CURDIR)":/home/pisa/src/ $(DOCKER_IMAGE)
	@rm -f pisa/external/tbb/cmake/TBBConfig.cmake pisa/external/tbb/cmake/TBBConfigVersion.cmake

index: idx

serve: build/bin/do_query
ifeq ($(RUN_IN_DOCKER),1)
	@docker run --rm --user "$$(id -u):$$(id -g)" -v "$(CURDIR)":/home/pisa/src/ -i --entrypoint bash $(DOCKER_IMAGE) -lc 'LD_LIBRARY_PATH=$(CONTAINER_TBB_LIB) ./build/bin/do_query idx'
else
	@LD_LIBRARY_PATH=$(TBB_LIB) ./build/bin/do_query idx
endif

build/bin/%: submodules
	@$(MAKE) compile >&2

idx: build/bin/build_index
	@echo "--- Indexing PISA 0.8.2 ---"
	@rm -rf idx
	@mkdir -p idx
ifeq ($(RUN_IN_DOCKER),1)
	@docker run --rm --user "$$(id -u):$$(id -g)" -v "$(CURDIR)":/home/pisa/src/ -i --entrypoint bash $(DOCKER_IMAGE) -lc 'LD_LIBRARY_PATH=$(CONTAINER_TBB_LIB) ./build/bin/build_index' < ${CORPUS}
else
	@LD_LIBRARY_PATH=$(TBB_LIB) ./build/bin/build_index < ${CORPUS}
endif
