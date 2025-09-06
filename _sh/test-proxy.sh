#!/bin/bash
set -euo pipefail

echo "=== RUSH SYNC SERVER - Proxy Test (no-start) ==="
echo

# -------------------------
# Konfiguration
# -------------------------
APP_NAME="levo"                 # Dein laufender Server
APP_HOST="${APP_NAME}.localhost"

HTTP_PROXY_PORT=8000
HTTPS_PROXY_PORT=8443

# Redirect: zuerst 80, sonst 8080 (nur Test)
REDIRECT_PORT=80
REDIRECT_FALLBACK=8080

# Pfad, der 200 liefern soll (laut Logs vorhanden)
TEST_PATH="/api/health"

# Farben
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'

# curl absolut bestimmen (fix für sudo/PATH)
CURL="/usr/bin/curl"
if [[ ! -x "$CURL" ]]; then
  CURL="$(command -v curl || true)"
fi
if [[ -z "${CURL}" || ! -x "${CURL}" ]]; then
  echo -e "${RED}curl nicht gefunden. Bitte installieren (macOS: xcode-select --install).${NC}"
  exit 1
fi

need() { command -v "$1" >/dev/null 2>&1 || { echo -e "${RED}Fehlt: $1${NC}"; exit 1; }; }
need lsof

# Port-Check
check_port () {
  local PORT="$1"
  if lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Port ${PORT} lauscht${NC}"
    return 0
  else
    echo -e "${RED}✗ Port ${PORT} lauscht NICHT${NC}"
    return 1
  fi
}

# Nur Statuscode holen – ohne Array-Extras
http_code () {
  local SCHEME="$1" HOST="$2" PORT="$3" PATH="$4"
  if [[ "$SCHEME" == "https" ]]; then
    "${CURL}" -k -s -o /dev/null \
      --resolve "${HOST}:${PORT}:127.0.0.1" \
      -w "%{http_code}" "https://${HOST}:${PORT}${PATH}"
  else
    "${CURL}" -s -o /dev/null \
      --resolve "${HOST}:${PORT}:127.0.0.1" \
      -w "%{http_code}" "http://${HOST}:${PORT}${PATH}"
  fi
}

echo "1) Proxy-Ports prüfen…"
HTTP_OK=0 ; check_port "$HTTP_PROXY_PORT"  || HTTP_OK=1
HTTPS_OK=0; check_port "$HTTPS_PROXY_PORT" || HTTPS_OK=1
echo
[[ $HTTP_OK -ne 0 || $HTTPS_OK -ne 0 ]] && \
  echo -e "${YELLOW}! Hinweis: Mindestens ein Proxy-Port ist DOWN. Tests können fehlschlagen.${NC}\n"

echo "2) HTTP Proxy testen (Port ${HTTP_PROXY_PORT})…"
echo "   Request: http://${APP_HOST}:${HTTP_PROXY_PORT}${TEST_PATH}"
CODE=$(http_code http "${APP_HOST}" "${HTTP_PROXY_PORT}" "${TEST_PATH}")
echo "   HTTP Status: ${CODE}"
echo

echo "3) HTTPS Proxy testen (Port ${HTTPS_PROXY_PORT})…"
echo "   Request: https://${APP_HOST}:${HTTPS_PROXY_PORT}${TEST_PATH}"
CODE=$(http_code https "${APP_HOST}" "${HTTPS_PROXY_PORT}" "${TEST_PATH}")
echo "   HTTPS Status: ${CODE}"
echo

echo "4) HTTP → HTTPS Redirect testen…"
TEST_REDIR_PORT="$REDIRECT_PORT"
if ! lsof -nP -iTCP:"$TEST_REDIR_PORT" -sTCP:LISTEN >/dev/null 2>&1; then
  if lsof -nP -iTCP:"$REDIRECT_FALLBACK" -sTCP:LISTEN >/dev/null 2>&1; then
    TEST_REDIR_PORT="$REDIRECT_FALLBACK"
    echo "   Hinweis: Port 80 down → teste ersatzweise Port ${REDIRECT_FALLBACK}"
  else
    echo -e "${YELLOW}   (Übersprungen – weder 80 noch ${REDIRECT_FALLBACK} verfügbar)${NC}"
    TEST_REDIR_PORT=""
  fi
fi

if [[ -n "$TEST_REDIR_PORT" ]]; then
  echo "   Request: http://${APP_HOST}:${TEST_REDIR_PORT}/"
  "${CURL}" -I -s --resolve "${APP_HOST}:${TEST_REDIR_PORT}:127.0.0.1" \
    "http://${APP_HOST}:${TEST_REDIR_PORT}/" | grep -E "^(HTTP|Location)" || true
fi
echo

echo "=== Test Complete ==="
echo
echo "Erwartet:"
echo "  - http://${APP_HOST}:${HTTP_PROXY_PORT}${TEST_PATH}  → 200"
echo "  - https://${APP_HOST}:${HTTPS_PROXY_PORT}${TEST_PATH} → 200"
echo "  - Redirect: http://${APP_HOST}:80 (oder :${REDIRECT_FALLBACK}) → Location: https://${APP_HOST}:8443/"
