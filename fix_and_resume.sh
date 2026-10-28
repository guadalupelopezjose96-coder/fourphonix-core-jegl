#!/usr/bin/env bash
set -euo pipefail

STACK_DIR="$HOME/auraintel-vpp-stack"
mkdir -p "$STACK_DIR"/{sql,certs,output,logs}
mkdir -p "$PREFIX/tmp"
cd "$STACK_DIR"

echo "[1/5] Verificando clúster PostgreSQL 18.6 en Termux..."
PGDATA="$PREFIX/var/lib/postgresql"
if ! pg_ctl -D "$PGDATA" status >/dev/null 2>&1; then
    pg_ctl -D "$PGDATA" -l "$STACK_DIR/logs/postgres.log" start
fi

# Esperar hasta que PostgreSQL acepte conexiones
for i in {1..10}; do
    if pg_isready -q; then
        break
    fi
    sleep 1
done

echo "[2/5] Garantizando existencia de 01_schema.sql y 02_seed_audit_node.sql..."
cat << 'SQL1' > "$STACK_DIR/sql/01_schema.sql"
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS der_registered_nodes (
    node_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_legal_name VARCHAR(255) NOT NULL,
    tax_entity_id VARCHAR(64) NOT NULL,
    tax_regime_code VARCHAR(16) NOT NULL,
    jurisdiction VARCHAR(32) NOT NULL CHECK (jurisdiction IN ('UsFercIso', 'MxCenaceMem', 'EuEnstoE')),
    iso_substation_node VARCHAR(64) NOT NULL,
    ieee1547_cert_hash CHAR(64) NOT NULL,
    ul1741sb_cert_hash CHAR(64) NOT NULL,
    mtls_client_cert_fingerprint CHAR(64) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS vpp_dispatch_settlement_ledger (
    record_id UUID PRIMARY KEY,
    node_id UUID NOT NULL REFERENCES der_registered_nodes(node_id),
    tax_entity_id VARCHAR(64) NOT NULL,
    jurisdiction VARCHAR(32) NOT NULL,
    dispatched_wh BIGINT NOT NULL CHECK (dispatched_wh > 0),
    gross_settlement_micro_usd BIGINT NOT NULL CHECK (gross_settlement_micro_usd >= 0),
    tax_withholding_micro_usd BIGINT NOT NULL CHECK (tax_withholding_micro_usd >= 0),
    net_payout_micro_usd BIGINT NOT NULL CHECK (net_payout_micro_usd >= 0),
    cfdi_uuid_or_1099_ref VARCHAR(64),
    timestamp_utc TIMESTAMPTZ NOT NULL,
    previous_record_hash CHAR(64) NOT NULL,
    record_hash CHAR(64) NOT NULL UNIQUE,
    hmac_signature CHAR(64) NOT NULL
);

CREATE OR REPLACE FUNCTION prevent_ledger_tampering()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'OPERACION_PROHIBIDA: Libro mayor inmutable (NIST AU-9 / Materialidad Fiscal).';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_immutable_vpp_ledger ON vpp_dispatch_settlement_ledger;
CREATE TRIGGER trg_immutable_vpp_ledger
BEFORE UPDATE OR DELETE OR TRUNCATE ON vpp_dispatch_settlement_ledger
FOR EACH STATEMENT
EXECUTE FUNCTION prevent_ledger_tampering();
SQL1

cat << 'SQL2' > "$STACK_DIR/sql/02_seed_audit_node.sql"
INSERT INTO der_registered_nodes (
    node_id, owner_legal_name, tax_entity_id, tax_regime_code,
    jurisdiction, iso_substation_node, ieee1547_cert_hash,
    ul1741sb_cert_hash, mtls_client_cert_fingerprint, is_active
) VALUES (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'SUNSHINE SOLAR STORAGE HOLDINGS LLC',
    '99-8765432',
    'W9-LLC',
    'UsFercIso',
    'FL-ORLANDO-SUB-01',
    'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    '8f434346648f6b96df89dda901c5176b10a6d83961dd3c1ac88b59b2dc327aa4',
    '2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae',
    TRUE
) ON CONFLICT (mtls_client_cert_fingerprint) DO NOTHING;
SQL2

# Crear DB si no existe y cargar tablas
if ! psql -lqt | cut -d \| -f 1 | grep -qw auraintel_vpp; then
    createdb auraintel_vpp
fi
psql -d auraintel_vpp -f "$STACK_DIR/sql/01_schema.sql" >/dev/null
psql -d auraintel_vpp -f "$STACK_DIR/sql/02_seed_audit_node.sql" >/dev/null
echo "[OK] Esquema PostgreSQL con Trigger de Inmutabilidad (NIST AU-9) activo."

# 3. Asegurar certificados mTLS ECDSA P-256 en $STACK_DIR/certs
if [ ! -f "$STACK_DIR/certs/ca_root.pem" ]; then
    echo "Generando cadena PKI mTLS en $STACK_DIR/certs..."
    openssl ecparam -name prime256v1 -genkey -noout -out "$STACK_DIR/certs/ca_root.key"
    openssl req -x509 -new -nodes -key "$STACK_DIR/certs/ca_root.key" -sha256 -days 3650 \
        -subj "/C=US/ST=Florida/L=Orlando/O=AuraIntel VPP LLC/CN=AuraIntel_Root_CA" \
        -out "$STACK_DIR/certs/ca_root.pem"

    openssl ecparam -name prime256v1 -genkey -noout -out "$STACK_DIR/certs/server.key"
    openssl req -new -key "$STACK_DIR/certs/server.key" \
        -subj "/C=US/ST=Florida/O=AuraIntel VPP LLC/CN=localhost" \
        -out "$STACK_DIR/certs/server.csr"
    printf "basicConstraints=CA:FALSE\nkeyUsage=digitalSignature,keyAgreement\nextendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\n" > "$STACK_DIR/certs/server_ext.cnf"
    openssl x509 -req -in "$STACK_DIR/certs/server.csr" -CA "$STACK_DIR/certs/ca_root.pem" -CAkey "$STACK_DIR/certs/ca_root.key" -CAcreateserial \
        -out "$STACK_DIR/certs/server.pem" -days 365 -sha256 -extfile "$STACK_DIR/certs/server_ext.cnf"

    openssl ecparam -name prime256v1 -genkey -noout -out "$STACK_DIR/certs/der_client.key"
    openssl req -new -key "$STACK_DIR/certs/der_client.key" \
        -subj "/C=US/ST=Florida/O=AuraIntel VPP LLC/CN=DER-NODE-001" \
        -out "$STACK_DIR/certs/der_client.csr"
    printf "basicConstraints=CA:FALSE\nkeyUsage=digitalSignature,keyAgreement\nextendedKeyUsage=clientAuth\n" > "$STACK_DIR/certs/client_ext.cnf"
    openssl x509 -req -in "$STACK_DIR/certs/der_client.csr" -CA "$STACK_DIR/certs/ca_root.pem" -CAkey "$STACK_DIR/certs/ca_root.key" -CAcreateserial \
        -out "$STACK_DIR/certs/der_client.pem" -days 365 -sha256 -extfile "$STACK_DIR/certs/client_ext.cnf"
fi

# 4. Asegurar que el parche de auraintel-tax-settlement esté aplicado en la copia del stack
if [ -f "$STACK_DIR/auraintel-tax-settlement/src/main.rs" ]; then
    python3 -c "
p = '$STACK_DIR/auraintel-tax-settlement/src/main.rs'
with open(p, 'r') as f: c = f.read()
old = '        iva_str,\n        subtotal_str,\n        iva_str,\n        iva_str\n    );'
new = '        iva_str,\n        subtotal_str,\n        iva_str\n    );'
if old in c:
    with open(p, 'w') as f: f.write(c.replace(old, new, 1))
"
fi

# 5. Ejecutar Servidor Agregador mTLS, Agente de Borde y Motor Fiscal
pkill -f auraintel-mtls-aggregator 2>/dev/null || true
pkill -f auraintel-edge-agent 2>/dev/null || true

echo "[3/5] Iniciando Servidor Agregador mTLS 1.3 (Puerto 8443)..."
(
    cd "$STACK_DIR/auraintel-mtls-aggregator"
    AURAINTEL_CA_CERT="$STACK_DIR/certs/ca_root.pem" \
    AURAINTEL_SERVER_CERT="$STACK_DIR/certs/server.pem" \
    AURAINTEL_SERVER_KEY="$STACK_DIR/certs/server.key" \
    cargo run --release > "$STACK_DIR/logs/aggregator.log" 2>&1 &
)
sleep 4

echo "[4/5] Ejecutando Agente de Borde IEEE 2030.5 / SunSpec (12 segundos de telemetría mTLS)..."
(
    cd "$STACK_DIR/auraintel-edge-agent"
    AURAINTEL_CA_CERT="$STACK_DIR/certs/ca_root.pem" \
    AURAINTEL_CLIENT_CERT="$STACK_DIR/certs/der_client.pem" \
    AURAINTEL_CLIENT_KEY="$STACK_DIR/certs/der_client.key" \
    timeout 12s cargo run --release > "$STACK_DIR/logs/edge_agent.log" 2>&1 || true
)

echo "Eventos registrados en el Servidor Agregador mTLS:"
tail -n 5 "$STACK_DIR/logs/aggregator.log" || true

echo ""
echo "[5/5] Ejecutando Motor de Liquidación Fiscal (IRS IRIS 1099-NEC & SAT CFDI 4.0)..."
(
    cd "$STACK_DIR/auraintel-tax-settlement"
    DATABASE_URL="host=localhost user=$(whoami) dbname=auraintel_vpp" \
    cargo run --release
    cp -r ./output/* "$STACK_DIR/output/" 2>/dev/null || true
)

echo "========================================================================="
echo " STACK AURAINTEL VPP 100% OPERATIVO EN TERMUX"
echo " - Reporte IRS 1099-NEC : $STACK_DIR/output/irs_iris_1099_nec_2026.json"
echo " - XML SAT CFDI 4.0     : $STACK_DIR/output/cfdi40_EDB210514AB9.xml"
echo "========================================================================="
