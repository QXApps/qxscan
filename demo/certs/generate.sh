#!/usr/bin/env bash
# generate.sh — Generate CA, PQC-ready (ECDSA) + legacy (RSA) certificates for QXScan demo.
# Run inside Alpine container with openssl installed.
set -euo pipefail

CERT_DIR="${CERT_DIR:-/certs}"
DAYS_CA=3650
DAYS_CERT=365
KEY_BITS=2048
EC_CURVE=prime256v1

mkdir -p "$CERT_DIR"
cd "$CERT_DIR"

echo "=== Generating Certificate Authority ==="
# CA key — ECDSA for PQC-ready tier
openssl ecparam -genkey -name "$EC_CURVE" -out ca.key
openssl req -new -x509 -days "$DAYS_CA" -key ca.key -out ca.crt \
  -subj "/C=US/ST=CA/O=QXScan Demo/CN=QXScan Demo CA" \
  -extensions v3_ca \
  -config <(cat /etc/ssl/openssl.cnf <(printf "\n[v3_ca]\nbasicConstraints=CA:TRUE\nkeyUsage=keyCertSign,cRLSign\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid:always\n"))

# PQC-ready services: ECDSA keys (frontend-pqc, backend, caddy, mail, alb)
# Legacy service: RSA key (frontend-legacy)
SERVICES_PQC="frontend-pqc backend db caddy mail alb db-proxy caddy-pqc"
SERVICES_LEGACY="frontend-legacy"

for svc in $SERVICES_PQC; do
  echo "=== Generating PQC-ready ECDSA cert for $svc ==="
  openssl ecparam -genkey -name "$EC_CURVE" -out "${svc}.key"

  cat > "${svc}.cnf" <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no
[req_distinguished_name]
CN = ${svc}
[v3_req]
keyUsage = keyEncipherment, digitalSignature
extendedKeyUsage = serverAuth
subjectAltName = @alt_names
[alt_names]
DNS.1 = ${svc}
DNS.2 = localhost
IP.1 = 127.0.0.1
EOF

  openssl req -new -key "${svc}.key" -out "${svc}.csr" -config "${svc}.cnf"
  openssl x509 -req -days "$DAYS_CERT" -in "${svc}.csr" -CA ca.crt -CAkey ca.key \
    -CAcreateserial -out "${svc}.crt" \
    -extfile "${svc}.cnf" -extensions v3_req

  rm -f "${svc}.cnf" "${svc}.csr"
done

for svc in $SERVICES_LEGACY; do
  echo "=== Generating legacy RSA cert for $svc ==="
  openssl genrsa -out "${svc}.key" "$KEY_BITS"

  cat > "${svc}.cnf" <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no
[req_distinguished_name]
CN = ${svc}
[v3_req]
keyUsage = keyEncipherment, digitalSignature
extendedKeyUsage = serverAuth
subjectAltName = @alt_names
[alt_names]
DNS.1 = ${svc}
DNS.2 = localhost
IP.1 = 127.0.0.1
EOF

  openssl req -new -key "${svc}.key" -out "${svc}.csr" -config "${svc}.cnf"
  openssl x509 -req -days "$DAYS_CERT" -in "${svc}.csr" -CA ca.crt -CAkey ca.key \
    -CAcreateserial -out "${svc}.crt" \
    -extfile "${svc}.cnf" -extensions v3_req

  rm -f "${svc}.cnf" "${svc}.csr"
done

# Set ownership for PostgreSQL
chown -R 70:70 db.key db.crt 2>/dev/null || true

# Secure permissions
chmod 644 *.crt
chmod 600 *.key

echo "=== Certificate generation complete ==="
ls -la "$CERT_DIR"
