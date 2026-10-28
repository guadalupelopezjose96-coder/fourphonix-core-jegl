#!/usr/bin/env bash
# Generador de Cadena PKI ECDSA P-256 (IEEE 2030.5 / TLS 1.3) para pruebas de integracion mTLS
set -euo pipefail

CERT_DIR="./certs"
mkdir -p "${CERT_DIR}"
chmod 700 "${CERT_DIR}"

echo "[1/3] Generando Root CA Interna de AuraIntel VPP (ECDSA P-256)..."
openssl ecparam -name prime256v1 -genkey -noout -out "${CERT_DIR}/ca_root.key"
openssl req -x509 -new -nodes -key "${CERT_DIR}/ca_root.key" -sha256 -days 3650 \
    -subj "/C=US/ST=Florida/L=Orlando/O=AuraIntel VPP LLC/OU=Root_PKI_Authority/CN=AuraIntel_IEEE2030_5_Root_CA" \
    -out "${CERT_DIR}/ca_root.pem"

echo "[2/3] Generando y firmando Certificado del Servidor Agregador..."
openssl ecparam -name prime256v1 -genkey -noout -out "${CERT_DIR}/server.key"
openssl req -new -key "${CERT_DIR}/server.key" \
    -subj "/C=US/ST=Florida/O=AuraIntel VPP LLC/OU=Aggregator_Server/CN=localhost" \
    -out "${CERT_DIR}/server.csr"

cat << 'EXT' > "${CERT_DIR}/server_ext.cnf"
basicConstraints = CA:FALSE
keyUsage = digitalSignature, keyAgreement
extendedKeyUsage = serverAuth
subjectAltName = DNS:localhost, DNS:telemetry.auraintel-vpp.com, IP:127.0.0.1
EXT

openssl x509 -req -in "${CERT_DIR}/server.csr" \
    -CA "${CERT_DIR}/ca_root.pem" -CAkey "${CERT_DIR}/ca_root.key" -CAcreateserial \
    -out "${CERT_DIR}/server.pem" -days 365 -sha256 -extfile "${CERT_DIR}/server_ext.cnf"

echo "[3/3] Generando y firmando Certificado de Cliente (Inversor / Nodo DER)..."
openssl ecparam -name prime256v1 -genkey -noout -out "${CERT_DIR}/der_client.key"
openssl req -new -key "${CERT_DIR}/der_client.key" \
    -subj "/C=US/ST=Florida/O=AuraIntel VPP LLC/OU=DER_Edge_Node/CN=DER-NODE-001" \
    -out "${CERT_DIR}/der_client.csr"

cat << 'EXT' > "${CERT_DIR}/client_ext.cnf"
basicConstraints = CA:FALSE
keyUsage = digitalSignature, keyAgreement
extendedKeyUsage = clientAuth
EXT

openssl x509 -req -in "${CERT_DIR}/der_client.csr" \
    -CA "${CERT_DIR}/ca_root.pem" -CAkey "${CERT_DIR}/ca_root.key" -CAcreateserial \
    -out "${CERT_DIR}/der_client.pem" -days 365 -sha256 -extfile "${CERT_DIR}/client_ext.cnf"

# Calcular el LFDI (primeros 20 bytes / 40 hex chars del SHA-256 del certificado DER conforme a IEEE 2030.5)
CLIENT_CERT_SHA256=$(openssl x509 -in "${CERT_DIR}/der_client.pem" -outform DER | sha256sum | awk '{print $1}')
CLIENT_LFDI=$(echo "${CLIENT_CERT_SHA256}" | cut -c1-40 | tr '[:lower:]' '[:upper:]')

echo "========================================================================="
echo "Cadena PKI mTLS generada en ${CERT_DIR}/"
echo "Certificado Cliente SHA-256 : ${CLIENT_CERT_SHA256}"
echo "IEEE 2030.5 LFDI (40 Hex)   : ${CLIENT_LFDI}"
echo "${CLIENT_LFDI}" > "${CERT_DIR}/der_client.lfdi"
echo "========================================================================="
