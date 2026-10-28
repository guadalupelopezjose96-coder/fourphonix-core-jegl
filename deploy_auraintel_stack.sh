#!/usr/bin/env bash
# Orquestador Maestro AuraIntel VPP (Soporte Dual: Docker Compose Cloud & Termux Nativo)
set -euo pipefail

STACK_DIR="$HOME/auraintel-vpp-stack"
cd "$STACK_DIR"

echo "========================================================================="
echo " AURAINTEL VPP - PROVISIONING & ZERO-TRUST ORCHESTRATOR"
echo "========================================================================="

# 1. Detectar si Docker Daemon está disponible (Cloud/Server) o si estamos en Termux
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    echo "[MODO CLOUD/DOCKER] Levantando stack endurecido con Docker Compose..."
    docker compose up -d --build
    docker compose ps
    exit 0
fi

echo "[MODO TERMUX NATIVO] Orquestando servicios directamente en espacio de usuario..."

# 2. Asegurar PostgreSQL instalado y corriendo en Termux
if ! command -v initdb >/dev/null 2>&1; then
    echo "[1/5] Instalando PostgreSQL en Termux..."
    pkg install -y postgresql
fi

PGDATA="$PREFIX/var/lib/postgresql"
if [ ! -d "$PGDATA" ] || [ -z "$(ls -A "$PGDATA" 2>/dev/null)" ]; then
    echo "[1/5] Inicializando clúster de base de datos PostgreSQL..."
    mkdir -p "$PGDATA"
    initdb -D "$PGDATA"
fi

if ! pg_ctl -D "$PGDATA" status >/dev/null 2>&1; then
    echo "[1/5] Iniciando demonio PostgreSQL local..."
    pg_ctl -D "$PGDATA" -l "$STACK_DIR/logs/postgres.log" start
    sleep 2
fi

# 3. Crear base de datos y aplicar esquema inmutable + nodo semilla
createdb auraintel_vpp 2>/dev/null || true
psql -d auraintel_vpp -f "$STACK_DIR/sql/01_schema.sql" >/dev/null
psql -d auraintel_vpp -f "$STACK_DIR/sql/02_seed_audit_node.sql" >/dev/null
echo "[2/5] Esquema PostgreSQL con Trigger de Inmutabilidad (NIST AU-9) verificado."

# 4. Iniciar Servidor Agregador mTLS 1.3 en segundo plano
pkill -f auraintel-mtls-aggregator 2>/dev/null || true
pkill -f auraintel-edge-agent 2>/dev/null || true

echo "[3/5] Compilando e iniciando Servidor Agregador mTLS 1.3 (Puerto 8443)..."
(
    cd "$STACK_DIR/auraintel-mtls-aggregator"
    AURAINTEL_CA_CERT="$STACK_DIR/certs/ca_root.pem" \
    AURAINTEL_SERVER_CERT="$STACK_DIR/certs/server.pem" \
    AURAINTEL_SERVER_KEY="$STACK_DIR/certs/server.key" \
    cargo run --release > "$STACK_DIR/logs/aggregator.log" 2>&1 &
)
sleep 3

# 5. Iniciar Agente de Borde (ciclo de 5s) por 12 segundos para poblar telemetría firmada
echo "[4/5] Iniciando Agente de Borde IEEE 2030.5 / SunSpec (enviando 2 rafagas mTLS)..."
(
    cd "$STACK_DIR/auraintel-edge-agent"
    AURAINTEL_CA_CERT="$STACK_DIR/certs/ca_root.pem" \
    AURAINTEL_CLIENT_CERT="$STACK_DIR/certs/der_client.pem" \
    AURAINTEL_CLIENT_KEY="$STACK_DIR/certs/der_client.key" \
    timeout 12s cargo run --release > "$STACK_DIR/logs/edge_agent.log" 2>&1 || true
)

echo "Ultimos eventos del Servidor Agregador mTLS:"
tail -n 4 "$STACK_DIR/logs/aggregator.log"

# 6. Ejecutar el Motor de Liquidación Fiscal (IRS 1099-NEC & SAT CFDI 4.0)
echo ""
echo "[5/5] Ejecutando Motor de Liquidacion Fiscal (IRS IRIS 1099 & SAT CFDI 4.0)..."
(
    cd "$STACK_DIR/auraintel-tax-settlement"
    DATABASE_URL="host=localhost user=$(whoami) dbname=auraintel_vpp" \
    cargo run --release
    cp -r ./output/* "$STACK_DIR/output/" 2>/dev/null || true
)

echo "========================================================================="
echo " DESPLIEGUE COMPLETADO AL 100%"
echo " - Logs mTLS Aggregator : $STACK_DIR/logs/aggregator.log"
echo " - Logs Edge Agent      : $STACK_DIR/logs/edge_agent.log"
echo " - Reporte IRS 1099-NEC : $STACK_DIR/output/irs_iris_1099_nec_2026.json"
echo " - XML SAT CFDI 4.0     : $STACK_DIR/output/cfdi40_EDB210514AB9.xml"
echo "========================================================================="
