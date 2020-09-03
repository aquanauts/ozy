#!/usr/bin/env bash
# Should be suitable to be `curl | bash`'d

set -euo pipefail

echo Installing ozy pointing at the ozy sample team configuration on GitHub...

OZY_CONFIG_URL=https://raw.githubusercontent.com/aquanauts/ozy/master/conf/sample-team-conf.yaml
OZY_OS=linux
if [[ "$(uname)" == "Darwin" ]]; then OZY_OS=osx; fi
OZY_VERSION=$(curl -sL ${OZY_CONFIG_URL} | grep 'ozy_version:' | awk '{print $2}')

TMP_INSTALL_FILE="$(mktemp)"
curl -sL -o "${TMP_INSTALL_FILE}" https://github.com/aquanauts/ozy/releases/download/v${OZY_VERSION}/ozy-${OZY_OS}
chmod +x "${TMP_INSTALL_FILE}"
${TMP_INSTALL_FILE} init ${OZY_CONFIG_URL}
rm -f "${TMP_INSTALL_FILE}"

echo
echo '********************************'
echo 'Installed ok. Make sure you follow any instructions above to ensure ozy is on your path.'
echo '********************************'
