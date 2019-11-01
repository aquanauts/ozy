#!/usr/bin/env bash
# Should be suitable to be `curl | bash`'d
# Has **AQUATIC SPECIFIC** configuration in it.

set -euo pipefail

echo Installing ozy customized for Aquatic, please wait...

AQUATIC_BASE_URL=https://artifactory.aq.tc/artifactory/core-generic-base-local/

TMP_INSTALL_DIR=/tmp/install-ozy-$$
mkdir ${TMP_INSTALL_DIR}
TMP_INSTALL_FILE=${TMP_INSTALL_DIR}/ozy
# ozy needs to be called "ozy" to work
curl -s -o ${TMP_INSTALL_FILE} ${AQUATIC_BASE_URL}/ozy
chmod +x ${TMP_INSTALL_FILE}
${TMP_INSTALL_FILE} init ${AQUATIC_BASE_URL}/ozy.yaml
rm -rf ${TMP_INSTALL_DIR}

~/.ozy/bin/ozy info

echo
echo '********************************'
echo 'Installed ok. Make sure you follow any instructions above to ensure ozy is on your path.'
echo '********************************'
