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

init:
	@echo "--- Initializing Bleve 0.8.0 boltdb ---"
	@$(GO) mod init github.com/quickwit-oss/search-benchmark-game

format:
	@echo "--- Formatting code ---"
	@$(GO) fmt $(PACKAGES)

clean:
	@echo "--- Cleaning Bleve 0.8.0 boltdb ---"
	@rm -fr ./idx
	@rm -fr ./bin

compile: build

index: idx

serve:
	@./bin/do_query ./idx

build:
	@echo "--- Building Bleve 0.8.0 boltdb ---"
	@$(GO) build -tags="$(BUILD_TAGS)" $(LDFLAGS) -o $(SRCPATH)/bin/build_index  $(SRCPATH)/build_index.go
	@$(GO) build -tags="$(BUILD_TAGS)" $(LDFLAGS) -o $(SRCPATH)/bin/do_query  $(SRCPATH)/do_query.go

idx: build
	@echo "--- Indexing Bleve 0.8.0 boltdb ---"
	./bin/build_index ./idx "$@" < ${CORPUS}
