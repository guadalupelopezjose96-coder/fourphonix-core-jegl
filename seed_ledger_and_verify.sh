#!/usr/bin/env bash
set -euo pipefail

STACK_DIR="$HOME/auraintel-vpp-stack"
cd "$STACK_DIR"

echo "[1/4] Generando registros de despacho energético con firma HMAC-SHA256 e insertando en PostgreSQL..."

python3 - << 'PYEOF' | psql -d auraintel_vpp >/dev/null
import hashlib, hmac

key = b"CLAVE_PRODUCCION_ZERO_TRUST_AURAINTEL_VPP_2026"

h_us = hashlib.sha256(b"AURAINTEL_US_DISPATCH_2026_001").hexdigest()
sig_us = hmac.new(key, h_us.encode(), hashlib.sha256).hexdigest()

h_mx = hashlib.sha256(b"AURAINTEL_MX_DISPATCH_2026_002").hexdigest()
sig_mx = hmac.new(key, h_mx.encode(), hashlib.sha256).hexdigest()

sql = f"""
-- 1. Asegurar Nodo Mexicano en der_registered_nodes
INSERT INTO der_registered_nodes (
    node_id, owner_legal_name, tax_entity_id, tax_regime_code,
    jurisdiction, iso_substation_node, ieee1547_cert_hash,
    ul1741sb_cert_hash, mtls_client_cert_fingerprint, is_active
) VALUES (
    'b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22',
    'ENERGIA DISTRIBUIDA DEL BAJIO SAPI DE CV',
    'EDB210514AB9',
    '601',
    'MxCenaceMem',
    'MX-CENACE-QRO-01',
    'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    '8f434346648f6b96df89dda901c5176b10a6d83961dd3c1ac88b59b2dc327aa4',
    '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
    TRUE
) ON CONFLICT (mtls_client_cert_fingerprint) DO NOTHING;

-- 2. Insertar liquidación EE. UU. (22.5 MWh = $2,700.00 USD) en el Libro Mayor Inmutable
INSERT INTO vpp_dispatch_settlement_ledger (
    record_id, node_id, tax_entity_id, jurisdiction,
    dispatched_wh, gross_settlement_micro_usd, tax_withholding_micro_usd,
    net_payout_micro_usd, timestamp_utc, previous_record_hash, record_hash, hmac_signature
) VALUES (
    'c1eebc99-9c0b-4ef8-bb6d-6bb9bd380a33',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    '99-8765432',
    'UsFercIso',
    22500000,
    2700000000,
    0,
    2700000000,
    NOW(),
    '0000000000000000000000000000000000000000000000000000000000000000',
    '{h_us}',
    '{sig_us}'
) ON CONFLICT (record_hash) DO NOTHING;

-- 3. Insertar liquidación México (15 MWh = $35,000.00 Subtotal + $5,600.00 IVA) en el Libro Mayor Inmutable
INSERT INTO vpp_dispatch_settlement_ledger (
    record_id, node_id, tax_entity_id, jurisdiction,
    dispatched_wh, gross_settlement_micro_usd, tax_withholding_micro_usd,
    net_payout_micro_usd, timestamp_utc, previous_record_hash, record_hash, hmac_signature
) VALUES (
    'd2eebc99-9c0b-4ef8-bb6d-6bb9bd380a44',
    'b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22',
    'EDB210514AB9',
    'MxCenaceMem',
    15000000,
    35000000000,
    5600000000,
    40600000000,
    NOW(),
    '{h_us}',
    '{h_mx}',
    '{sig_mx}'
) ON CONFLICT (record_hash) DO NOTHING;
"""
print(sql)
PYEOF

echo "[2/4] Ejecutando el binario compilado auraintel-tax-settlement contra PostgreSQL..."
(
    cd "$STACK_DIR/auraintel-tax-settlement"
    DATABASE_URL="host=localhost user=$(whoami) dbname=auraintel_vpp" \
    ./target/release/auraintel-tax-settlement
    cp -r ./output/* "$STACK_DIR/output/"
)

echo ""
echo "[3/4] Mostrando artefactos fiscales generados (IRS 1099-NEC y SAT CFDI 4.0):"
echo "-------------------------------------------------------------------------"
cat "$STACK_DIR/output/irs_iris_1099_nec_2026.json"
echo "-------------------------------------------------------------------------"
head -n 25 "$STACK_DIR/output/cfdi40_EDB210514AB9.xml"
echo "-------------------------------------------------------------------------"

echo ""
echo "[4/4] Probando blindaje forense (Intentando borrar una fila del libro mayor en PostgreSQL):"
psql -d auraintel_vpp -c "DELETE FROM vpp_dispatch_settlement_ledger;" || true
