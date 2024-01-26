#!/bin/sh

set -ex

cd ../private-test-data/

for game in "$@"; do
  #printf "%s\n" "${game}"
  tar cJ "${game}" | gpg --symmetric --pinentry-mode loopback --passphrase-file="${HOME}/pw" > "../.builds/data/${game}.tar.xz.gpg"
done
