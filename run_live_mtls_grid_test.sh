#!/usr/bin/env bash
set -euo pipefail

STACK_DIR="$HOME/auraintel-vpp-stack"
cd "$STACK_DIR"

pkill -f auraintel-mtls-aggregator 2>/dev/null || true
pkill -f auraintel-edge-agent 2>/dev/null || true

echo "[1/3] Levantando binario de producción auraintel-mtls-aggregator (TLS 1.3 mTLS en :8443)..."
AURAINTEL_CA_CERT="$STACK_DIR/certs/ca_root.pem" \
AURAINTEL_SERVER_CERT="$STACK_DIR/certs/server.pem" \
AURAINTEL_SERVER_KEY="$STACK_DIR/certs/server.key" \
"$STACK_DIR/auraintel-mtls-aggregator/target/release/auraintel-mtls-aggregator" > "$STACK_DIR/logs/aggregator_live.log" 2>&1 &
AGG_PID=$!
sleep 1

echo "[2/3] Ejecutando binario auraintel-edge-agent durante 6 segundos (2 ciclos de telemetría IEEE 2030.5)..."
AURAINTEL_CA_CERT="$STACK_DIR/certs/ca_root.pem" \
AURAINTEL_CLIENT_CERT="$STACK_DIR/certs/der_client.pem" \
AURAINTEL_CLIENT_KEY="$STACK_DIR/certs/der_client.key" \
timeout 6s "$STACK_DIR/auraintel-edge-agent/target/release/auraintel-edge-agent" > "$STACK_DIR/logs/edge_live.log" 2>&1 || true

kill "$AGG_PID" 2>/dev/null || true

echo ""
echo "[3/3] Telemetría mTLS 1.3 y Órdenes de Despacho Firmadas (En Vivo):"
echo "--- LOG SERVIDOR AGREGADOR ---"
cat "$STACK_DIR/logs/aggregator_live.log"
echo ""
echo "--- LOG AGENTE DE BORDE (INVERSOR/BESS) ---"
cat "$STACK_DIR/logs/edge_live.log"
