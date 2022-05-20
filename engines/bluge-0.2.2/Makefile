SRCPATH=$(shell pwd)

GOOS ?=
GOARCH ?=
CGO_ENABLED ?= 0
CGO_CFLAGS ?=
CGO_LDFLAGS ?=
BUILD_TAGS ?=
BIN_EXT ?=

GO := GOOS=$(GOOS) GOARCH=$(GOARCH) CGO_ENABLED=$(CGO_ENABLED) CGO_CFLAGS=$(CGO_CFLAGS) CGO_LDFLAGS=$(CGO_LDFLAGS) GO111MODULE=on go

PACKAGES = $(shell $(GO) list ./... | grep -v '/vendor/')

LDFLAGS = -ldflags "-s -w"

ifeq ($(GOOS),windows)
  BIN_EXT = .exe
endif

.DEFAULT_GOAL := compile

format:
	@echo "--- Formatting code ---"
	@$(GO) fmt $(PACKAGES)

clean:
	@echo "--- Cleaning Bluge 0.1.9 ---"
	@rm -fr ./idx
	@rm -fr ./bin

compile: build

index: idx

serve:
	@./bin/do_query ./idx

build:
	@echo "--- Building Bluge 0.1.9 ---"
	@$(GO) build -tags="$(BUILD_TAGS)" $(LDFLAGS) -o $(SRCPATH)/bin/build_index  $(SRCPATH)/build_index.go
	@$(GO) build -tags="$(BUILD_TAGS)" $(LDFLAGS) -o $(SRCPATH)/bin/do_query  $(SRCPATH)/do_query.go

idx: build
	@echo "--- Indexing Bluge 0.1.9 ---"
	./bin/build_index ./idx "$@" < ${CORPUS}
