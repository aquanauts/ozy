SHELL := $(shell which bash) # Use bash instead of bin/sh as shell
SYS_PYTHON := $(shell which python3 || echo ".python_is_missing")
ifndef VIRTUAL_ENV
# We're not already in a virtual environment; so create one
VENV := .venv
else
# We're already in a virtual environment (e.g. travis). Use that instead
VENV := $(VIRTUAL_ENV)
endif
PIP := $(VENV)/bin/pip
DEPS := $(VENV)/.deps
PYTHON := $(VENV)/bin/python3
PYINSTALLER := $(VENV)/bin/pyinstaller
PYTHON_CMD := PYTHONPATH=$(shell pwd) $(PYTHON)
TEST_TYPES=$(filter-out __pycache__,$(notdir $(wildcard test/*)))

.PHONY: help
help: ## me
	@grep -E '^[0-9a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

$(VENV):
	$(SYS_PYTHON) -m venv $(VENV)

$(DEPS): $(VENV) requirements.txt
	$(PIP) install --upgrade pip
	$(PIP) install -r requirements.txt
	touch $(DEPS)

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
	$(PYINSTALLER) --onefile bin/ozy

.PHONY: clean
clean: ## remove venv and flush out pycache TODO (will fail on mac because --no-run-if-empty isn't bsd xargs)
	rm -rf $(VENV) dist *.egg-info .mypy_cache $(dir $(CONDA_YAML))
	find . -depth -name __pycache__ -type d -exec rm -fr {} \;
