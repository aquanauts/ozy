SHELL := $(shell which bash) # Use bash instead of bin/sh as shell
SYS_PYTHON := $(shell which python3 || echo ".python_is_missing")
CONDA := $(shell which conda || echo ".conda_is_missing")
VENV := .venv
DEPS := $(VENV)/.deps
PYTHON := $(VENV)/bin/python3
PYINSTALLER := $(VENV)/bin/pyinstaller
OUTPUT_NAME?=ozy
PYTHON_CMD := PYTHONPATH=$(shell pwd) $(PYTHON)
TEST_TYPES=$(filter-out __pycache__,$(notdir $(wildcard test/*)))

.PHONY: help
help: ## me
	@grep -E '^[0-9a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

$(DEPS): environment.yml
	rm -rf $(VENV)
	$(CONDA) env create -p $(VENV) -f environment.yml

.PHONY: deps
deps: $(DEPS) ## install the dependencies

.PHONY: test
test: $(TEST_TYPES) ## run all tests

.PHONY: watch
watch: $(DEPS) ## run unit tests in a continuous loop
	$(PYTHON_CMD) -m pytest_watch -n test/unit

.PHONY: $(TEST_TYPES)
$(TEST_TYPES): % : $(DEPS)
	$(PYTHON_CMD) -m pytest test/$@

.PHONY: dist
dist: deps  ## create distribution
	$(PYINSTALLER) --onefile --name $(OUTPUT_NAME) bin/ozy

.PHONY: clean
clean: ## remove venv and flush out pycache
	rm -rf $(VENV) dist *.egg-info .mypy_cache $(dir $(CONDA_YAML))
	find . -depth -name __pycache__ -type d -exec rm -fr {} \;
