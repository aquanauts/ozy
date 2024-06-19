#!/usr/bin/env bash
# Should be suitable to be `curl | bash`'d

set -euo pipefail

echo Installing ozy pointing at the ozy sample team configuration on GitHub...

OZY_CONFIG_URL=https://raw.githubusercontent.com/aquanauts/ozy/main/conf/sample-team-conf.yaml
OZY_OS=$(uname)
OZY_ARCH=$(uname -p)

if [ -n "${OZY_VERSION:-}" ]; then
    echo "Using ${OZY_VERSION} as it was set in the environment"
else
    echo "Reading config information from ${OZY_CONFIG_URL}"
    OZY_VERSION=$(curl --fail -sSL ${OZY_CONFIG_URL} | grep 'ozy_version:' | awk '{print $2}')
    echo "Found ozy version ${OZY_VERSION}"
fi

echo Installing ozy v${OZY_VERSION} for ${OZY_OS}-${OZY_ARCH}

TMP_INSTALL_DIR=${HOME}/.tmp-ozy-install-$$  # on same path as HOME because of issue #73
mkdir -p "${TMP_INSTALL_DIR}"
TMP_INSTALL_FILE=${TMP_INSTALL_DIR}/ozy # ozy needs to be called "ozy" to work
curl -sL -o "${TMP_INSTALL_FILE}" https://github.com/aquanauts/ozy/releases/download/v${OZY_VERSION}/ozy-${OZY_OS}-${OZY_ARCH}
chmod +x "${TMP_INSTALL_FILE}"
${TMP_INSTALL_FILE} init ${OZY_CONFIG_URL}
rm -rf "${TMP_INSTALL_DIR}"

echo
echo '********************************'
echo 'Installed ok. Make sure you follow any instructions above to ensure ozy is on your path.'
echo '********************************'
