#!/usr/bin/env bash
# Ejecuta el motor de liquidacion en Rust y verifica/sella los artefactos IRS 1099 y SAT CFDI 4.0
set -euo pipefail

echo "[1/3] Compilando y ejecutando el motor de liquidacion fiscal en modo Release..."
cargo run --release

echo ""
echo "[2/3] Verificando reporte IRS IRIS 1099-NEC (JSON y CSV)..."
cat ./output/irs_iris_1099_nec_2026.csv
echo ""
cat ./output/irs_iris_1099_nec_2026.json

echo ""
echo "[3/3] Verificando XML base CFDI 4.0 y Cadena Original SAT..."
for cadena_file in ./output/cadena_original_*.txt; do
    echo "Cadena Original generada (${cadena_file}):"
    cat "${cadena_file}"
    echo ""
done

head -n 35 ./output/cfdi40_EDB210514AB9.xml
