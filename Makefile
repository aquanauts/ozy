SHELL := $(shell which bash) # Use bash instead of bin/sh as shell
SYS_PYTHON := $(shell which python3.7 || echo ".python_is_missing")
DOCKER := $(shell which docker || echo ".docker_is_missing")
NOMAD := $(shell which nomad || echo ".nomad_is_missing")
VENV = .venv
PIP := $(VENV)/bin/pip
DEPS := $(VENV)/.deps
PYTHON := $(VENV)/bin/python3
TWINE := $(VENV)/bin/twine
PYTHON_CMD := PYTHONPATH=$(shell pwd) $(PYTHON)
PACKAGE_NAME := ozy
TEST_TYPES=$(filter-out __pycache__,$(notdir $(wildcard test/*)))

.PHONY: help
help: ## me
	@grep -E '^[0-9a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

$(VENV):
	$(SYS_PYTHON) -m venv $(VENV)

$(DEPS): $(VENV) requirements.txt setup.py
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


.PHONY: package-pip
package-pip: test
	$(PYTHON) setup.py sdist



.PHONY: publish-pip
publish-pip: clean package ## build and publish the package to artifactory
	$(TWINE) upload --repository-url https://artifactory.aq.tc/artifactory/api/pypi/core-pypi dist/$(PACKAGE_NAME)-*


.PHONY: publish
publish: clean package publish-pip ## publish pip

.PHONY: clean
clean: ## remove venv and flush out pycache TODO (will fail on mac because --no-run-if-empty isn't bsd xargs)
	rm -rf $(VENV) dist *.egg-info .mypy_cache $(dir $(CONDA_YAML))
	find . -name __pycache__ | xargs --no-run-if-empty rm -rf
