#!/usr/bin/env bash
# Generates test fixtures for FileSource integration tests.
# Requires: openssl
#
# Usage (from repo root):  bash tests/fixtures/generate.sh
# Usage (from this dir):   bash generate.sh

set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Generating test fixtures in $DIR ..."

# RSA private key (PEM)
openssl genrsa -out "$DIR/test.key" 2048 2>/dev/null

# Self-signed certificate (PEM)
openssl req -new -x509 \
  -key "$DIR/test.key" \
  -out "$DIR/test.crt" \
  -days 3650 \
  -subj "/CN=secrets-rs-test" 2>/dev/null

# Certificate in DER (binary) format — exercises Secret<Vec<u8>>
openssl x509 \
  -in "$DIR/test.crt" \
  -out "$DIR/test.der" \
  -outform DER 2>/dev/null

echo "Done: test.key  test.crt  test.der"
