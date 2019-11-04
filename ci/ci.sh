#!/bin/bash

set -euxo pipefail

SCRIPTPATH="$(
  cd "$(dirname "$0")"
  pwd -P
)"

cd "${SCRIPTPATH}"/..

make test
make dist

curl -u"${ARTIFACTORY_USERNAME}:${ARTIFACTORY_PASSWORD}" \
  -T dist/ozy "https://artifactory.aq.tc/artifactory/core-generic-base-local/ozy"
curl -u"${ARTIFACTORY_USERNAME}:${ARTIFACTORY_PASSWORD}" \
  -T conf/sample-team-conf.yaml "https://artifactory.aq.tc/artifactory/core-generic-base-local/ozy.yaml"
