#!/usr/bin/env bash
# pg-forward.sh — Bridge TLS-decrypted stream to PostgreSQL STARTTLS
set -euo pipefail

DB_HOST="${DB_HOST:-db}"
DB_PORT="${DB_PORT:-5432}"

# Connect to PostgreSQL
exec 3<>/dev/tcp/${DB_HOST}/${DB_PORT}

# Send SSLRequest packet (length=8, magic=80877103)
printf '\x00\x00\x00\x08\x04\xd2\x16\x2f' >&3

# Read 1-byte response
response=$(dd bs=1 count=1 <&3 2>/dev/null)

if [ "$response" != "S" ]; then
    echo "ERROR: PostgreSQL did not accept SSL request (response='$response')" >&2
    exit 1
fi

# Bidirectional pipe
cat <&3 &
cat >&3
