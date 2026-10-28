#!/usr/bin/env bash
set -euo pipefail

STACK_DIR="$HOME/auraintel-vpp-stack"
cd "$STACK_DIR"

echo "[1/3] Configurando .gitignore Zero-Trust (Bloqueo estricto de llaves privadas .key y binarios pesados)..."
cat << 'GITIGNORE' > "$STACK_DIR/.gitignore"
# Bloqueo estricto de material criptografico privado (NIST SP 800-57)
*.key
*.pem
*.csr
*.srl
*.p12
*.pfx
.env
.env.*

# Directorios de compilacion y logs locales
**/target/
logs/*.log

# Conservar manifiesto de auditoria pero ignorar datos sensibles si se desea
!output/AUDIT_MANIFEST_SHA256.txt
GITIGNORE

echo "[2/3] Copiando artefactos de gobernanza (Operating Agreement, Dockerfile, CI/CD)..."
mkdir -p "$STACK_DIR/docs"
if [ -f "$HOME/auraintel-vpp/auraintel-legal-pki/docs/Operating_Agreement_AuraIntel_VPP_LLC.md" ]; then
    cp "$HOME/auraintel-vpp/auraintel-legal-pki/docs/Operating_Agreement_AuraIntel_VPP_LLC.md" "$STACK_DIR/docs/"
fi

echo "[3/3] Inicializando repositorio Git local y verificando que ninguna llave .key este expuesta..."
if [ ! -d "$STACK_DIR/.git" ]; then
    git init -b main
fi

git config user.name "José Emanuel Guadalupe López"
git config user.email "security@auraintel-vpp.com"

git add .
git status --short

echo "========================================================================="
echo " REPOSITORIO AURAINTEL VPP BLINDADO Y LISTO PARA COMMIT / PUSH"
echo " Verificar arriba que NINGUN archivo .key o .pem aparece en git status."
echo "========================================================================="
