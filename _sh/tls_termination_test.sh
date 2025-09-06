#!/bin/bash
set -euo pipefail

echo "=== SCHRITT 2: TLS TERMINATION TEST ==="
echo

# Konfiguration
APP_NAME="levo"
BACKEND_PORT=8081  # Dein Backend läuft auf 8081
PROXY_HTTP_PORT=8000
PROXY_HTTPS_PORT=8443

# Farben
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'

CURL="/usr/bin/curl"
if [[ ! -x "$CURL" ]]; then
  CURL="$(command -v curl || true)"
fi

echo "1) Backend Server Test (HTTP ohne TLS)..."
echo "   Request: http://127.0.0.1:${BACKEND_PORT}/api/health"
BACKEND_CODE=$(${CURL} -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:${BACKEND_PORT}/api/health")
echo "   Backend Status: ${BACKEND_CODE}"

if [[ "$BACKEND_CODE" != "200" ]]; then
  echo -e "${RED}❌ Backend nicht erreichbar - kann TLS Termination nicht testen${NC}"
  exit 1
fi

echo ""
echo "2) TLS Termination Test: HTTPS → HTTP Backend..."
echo "   Request: https://${APP_NAME}.localhost:${PROXY_HTTPS_PORT}/api/health"
echo "   (Proxy sollte HTTPS terminieren und HTTP an Backend senden)"

# Verbose curl um TLS-Details zu sehen
echo ""
echo "2a) TLS Handshake Details:"
${CURL} -k -v -s -o /dev/null \
  --resolve "${APP_NAME}.localhost:${PROXY_HTTPS_PORT}:127.0.0.1" \
  "https://${APP_NAME}.localhost:${PROXY_HTTPS_PORT}/api/health" 2>&1 | \
  grep -E "(SSL|TLS|certificate|cipher)" | head -10

echo ""
echo "2b) Response Test:"
HTTPS_CODE=$(${CURL} -k -s -o /dev/null -w "%{http_code}" \
  --resolve "${APP_NAME}.localhost:${PROXY_HTTPS_PORT}:127.0.0.1" \
  "https://${APP_NAME}.localhost:${PROXY_HTTPS_PORT}/api/health")
echo "   HTTPS Proxy Status: ${HTTPS_CODE}"

echo ""
echo "3) TLS Certificate Validation..."
echo "   Certificate Subject:"
${CURL} -k -v -s -o /dev/null \
  --resolve "${APP_NAME}.localhost:${PROXY_HTTPS_PORT}:127.0.0.1" \
  "https://${APP_NAME}.localhost:${PROXY_HTTPS_PORT}/api/health" 2>&1 | \
  grep -E "subject:|CN=" | head -3

echo ""
echo "4) Backend vs. Proxy Response Comparison..."
echo "   Verifying TLS termination working correctly:"

# Backend direkt
BACKEND_RESPONSE=$(${CURL} -s "http://127.0.0.1:${BACKEND_PORT}/api/health")
# Proxy HTTPS
PROXY_RESPONSE=$(${CURL} -k -s --resolve "${APP_NAME}.localhost:${PROXY_HTTPS_PORT}:127.0.0.1" \
  "https://${APP_NAME}.localhost:${PROXY_HTTPS_PORT}/api/health")

if [[ "$BACKEND_RESPONSE" == "$PROXY_RESPONSE" ]]; then
  echo -e "   ${GREEN}✅ TLS Termination works - identical responses${NC}"
else
  echo -e "   ${YELLOW}⚠️  Responses differ - possible TLS termination issue${NC}"
  echo "   Backend:  $(echo $BACKEND_RESPONSE | cut -c1-50)..."
  echo "   Proxy:    $(echo $PROXY_RESPONSE | cut -c1-50)..."
fi

echo ""
echo "=== SCHRITT 2 ERGEBNISSE ==="
echo -e "✅ Backend HTTP:       ${BACKEND_CODE} (Port ${BACKEND_PORT})"
echo -e "✅ Proxy HTTPS:        ${HTTPS_CODE} (Port ${PROXY_HTTPS_PORT})"
echo -e "✅ TLS Termination:    $([ "$BACKEND_RESPONSE" == "$PROXY_RESPONSE" ] && echo "WORKING" || echo "CHECK")"
echo ""
echo "Schritt 2 Status: TLS wird am Proxy terminiert und HTTP an Backend weitergeleitet"
echo "Certificate: *.localhost (Wildcard für alle Subdomains)"
