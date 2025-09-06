#!/bin/bash
# Quick Proxy Debug - Run this to test the correct server name

echo "=== PROXY DEBUG - Finding correct server name ==="

# 1. Test mit dem richtigen Server-Namen aus den Logs
echo "1) Testing with 'levo' (from logs)..."
curl -s -o /dev/null -w "Status: %{http_code}\n" \
  --resolve "levo.localhost:8000:127.0.0.1" \
  "http://levo.localhost:8000/api/health"

# 2. Test mit dem falschen Namen
echo "2) Testing with 'rss-001' (wrong name)..."
curl -s -o /dev/null -w "Status: %{http_code}\n" \
  --resolve "rss-001.localhost:8000:127.0.0.1" \
  "http://rss-001.localhost:8000/api/health"

# 3. Test HTTPS
echo "3) Testing HTTPS with 'levo'..."
curl -k -s -o /dev/null -w "Status: %{http_code}\n" \
  --resolve "levo.localhost:8443:127.0.0.1" \
  "https://levo.localhost:8443/api/health"

# 4. Direct server test (should work)
echo "4) Direct server test (bypass proxy)..."
curl -s -o /dev/null -w "Status: %{http_code}\n" \
  "http://127.0.0.1:8080/api/health"

echo ""
echo "Expected Results:"
echo "  1) levo.localhost:8000    → 200"
echo "  2) rss-001.localhost:8000 → 404 (wrong name)"
echo "  3) levo.localhost:8443    → 200"
echo "  4) Direct 127.0.0.1:8080  → 200"
