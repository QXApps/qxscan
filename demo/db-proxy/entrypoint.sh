#!/usr/bin/env bash
# entrypoint.sh — Wrapper around socat for db-proxy
# Ensures socat is called with correct address arguments

exec socat \
  "openssl-listen:5433,reuseaddr,fork,cert=/certs/db-proxy.crt,key=/certs/db-proxy.key,verify=0" \
  "EXEC:/usr/bin/env bash /pg-forward.sh"
