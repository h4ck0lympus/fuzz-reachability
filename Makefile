# Top-level convenience targets. The analyzer build lives in analyzer/Makefile.
#
#   make build                 # build the analyzer (core / type-based); default LLVM
#                              #   auto-selected by scripts/select_llvm.sh (>= 21)
#   make build LLVM_MAJOR=23    # ... against LLVM 23
#   make build-svf             # build the SVF dependency + SVF-enabled analyzer (LLVM 21)
#   make test                  # run the full test suite
#   make matrix                # LLVM 21/22/23(+) compatibility matrix
#   make clean

# Default LLVM major: 21 if an llvm-config-21 is installed, otherwise the newest
# installed major above it (see scripts/select_llvm.sh). Override with e.g.
# `make build LLVM_MAJOR=23`.
LLVM_MAJOR  ?= $(shell bash $(CURDIR)/scripts/select_llvm.sh)
LLVM_CONFIG ?= llvm-config-$(LLVM_MAJOR)

# SVF only builds against LLVM 21 (upstream targets 21.1.x; it fails on 22/23 --
# see docs/llvm-support.md). The SVF targets therefore pin LLVM 21 by default,
# independent of the auto-selected core default, *unless* the user explicitly set
# LLVM_MAJOR (command line or environment), in which case that wins.
SVF_LLVM_MAJOR  := $(if $(filter command line environment,$(origin LLVM_MAJOR)),$(LLVM_MAJOR),21)
SVF_LLVM_CONFIG := llvm-config-$(SVF_LLVM_MAJOR)
GOBIN       := $(shell go env GOPATH 2>/dev/null)/bin
PY          := $(CURDIR)/.venv/bin/python
ANALYZER     := $(CURDIR)/analyzer/build/reachability-analyzer
ANALYZER_SVF := $(CURDIR)/analyzer/build-svf/reachability-analyzer

.PHONY: help venv build svf-deps build-svf test matrix clean

help:
	@grep -E '^[a-z-]+:.*##' $(MAKEFILE_LIST) | sed 's/:.*##/\t/'

venv: ## create the Python venv (.venv) with the driver + test deps
	bash scripts/setup_venv.sh

# Order-only prereq: create the venv if it doesn't exist yet.
$(PY):
	bash scripts/setup_venv.sh

build: ## build the analyzer (core, type-based backend)
	$(MAKE) -C analyzer LLVM_CONFIG=$(LLVM_CONFIG)

svf-deps: ## build the SVF dependency (vendored); LLVM 21 unless LLVM_MAJOR is set
	bash scripts/build_svf.sh $(SVF_LLVM_MAJOR)

build-svf: svf-deps ## build the analyzer with the SVF backend enabled (LLVM 21 by default)
	$(MAKE) -C analyzer SVF=1 BUILD=build-svf LLVM_CONFIG=$(SVF_LLVM_CONFIG)

test: build | $(PY) ## run the full test suite (SVF tests skip if the SVF binary is absent)
	cd driver && PATH="$(GOBIN):$$PATH" \
	  REACHABILITY_ANALYZER="$(ANALYZER)" \
	  REACHABILITY_ANALYZER_SVF="$(ANALYZER_SVF)" \
	  "$(PY)" -m pytest tests/ -q

matrix: ## build + test against every installed llvm-config-NN (NN >= 21)
	bash scripts/test_matrix.sh

clean: ## remove analyzer build outputs
	$(MAKE) -C analyzer clean BUILD=build
	$(MAKE) -C analyzer clean BUILD=build-svf
	rm -rf analyzer/build/2[0-9] analyzer/build/2[0-9]-svf
