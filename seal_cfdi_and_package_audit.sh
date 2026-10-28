#!/usr/bin/env bash
# Sellado Criptografico RSA-SHA256 (Anexo 20 SAT) + Empaquetado Forense de Auditoria
set -euo pipefail

STACK_DIR="$HOME/auraintel-vpp-stack"
cd "$STACK_DIR"

CSD_DIR="$STACK_DIR/certs/sat_csd"
mkdir -p "$CSD_DIR"

echo "[1/3] Generando par CSD RSA-2048 de prueba (Estándar Anexo 20 SAT)..."
if [ ! -f "$CSD_DIR/csd_emisor.key" ]; then
    openssl genpkey -algorithm RSA -out "$CSD_DIR/csd_emisor.key" -pkeyopt rsa_keygen_bits:2048 2>/dev/null
    openssl req -x509 -new -nodes -key "$CSD_DIR/csd_emisor.key" -sha256 -days 1460 \
        -subj "/C=MX/ST=CDMX/L=Cuauhtemoc/O=AURAINTEL VPP MEXICO SAPI DE CV/OU=CSD_Facturacion/CN=AIV260101AAA" \
        -out "$CSD_DIR/csd_emisor.cer.pem" 2>/dev/null
fi

echo "[2/3] Firmando Cadena Original con RSA-SHA256 e inyectando Sello y Certificado Base64 en el CFDI 4.0..."
CADENA_FILE="$STACK_DIR/output/cadena_original_EDB210514AB9.txt"
XML_FILE="$STACK_DIR/output/cfdi40_EDB210514AB9.xml"

# Firma RSA-SHA256 codificada en Base64 en una sola linea
SELLO_BASE64=$(openssl dgst -sha256 -sign "$CSD_DIR/csd_emisor.key" "$CADENA_FILE" | base64 | tr -d '\n\r')

# Certificado X.509 codificado en Base64 sin encabezados PEM
CERT_BASE64=$(grep -v '^-----' "$CSD_DIR/csd_emisor.cer.pem" | tr -d '\n\r')

python3 - "$XML_FILE" "$SELLO_BASE64" "$CERT_BASE64" << 'PYEOF'
import sys, re

xml_path, sello, cert = sys.argv[1], sys.argv[2], sys.argv[3]
with open(xml_path, "r") as f:
    xml = f.read()

xml = re.sub(r'Sello="[^"]+"', f'Sello="{sello}"', xml)
xml = re.sub(r'Certificado="[^"]+"', f'Certificado="{cert}"', xml)

with open(xml_path, "w") as f:
    f.write(xml)
PYEOF

echo "[OK] XML CFDI 4.0 sellado criptográficamente:"
head -n 16 "$XML_FILE"

echo ""
echo "[3/3] Generando Manifiesto de Integridad SHA-256 de todos los Binarios y Reportes Fiscales..."
sha256sum \
    "$STACK_DIR/auraintel-mtls-aggregator/target/release/auraintel-mtls-aggregator" \
    "$STACK_DIR/auraintel-edge-agent/target/release/auraintel-edge-agent" \
    "$STACK_DIR/auraintel-tax-settlement/target/release/auraintel-tax-settlement" \
    "$STACK_DIR/output/irs_iris_1099_nec_2026.json" \
    "$STACK_DIR/output/irs_iris_1099_nec_2026.csv" \
    "$STACK_DIR/output/cfdi40_EDB210514AB9.xml" \
    | tee "$STACK_DIR/output/AUDIT_MANIFEST_SHA256.txt"
