use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ring::digest::{digest, SHA256};
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

const LE_PRODUCTION: &str = "https://acme-v02.api.letsencrypt.org/directory";
const LE_STAGING: &str = "https://acme-staging-v02.api.letsencrypt.org/directory";

// ACME challenge token storage (shared with web server route handlers)
static ACME_CHALLENGES: OnceLock<Arc<RwLock<HashMap<String, String>>>> = OnceLock::new();

pub fn get_challenge_response(token: &str) -> Option<String> {
    ACME_CHALLENGES
        .get()
        .and_then(|map| map.read().ok())
        .and_then(|map| map.get(token).cloned())
}

fn set_challenge(token: String, key_auth: String) {
    let challenges = ACME_CHALLENGES.get_or_init(|| Arc::new(RwLock::new(HashMap::new())));
    if let Ok(mut map) = challenges.write() {
        map.insert(token, key_auth);
    }
}

fn remove_challenge(token: &str) {
    if let Some(challenges) = ACME_CHALLENGES.get() {
        if let Ok(mut map) = challenges.write() {
            map.remove(token);
        }
    }
}

// =============================================================================
// ACME Status Tracking
// =============================================================================

static ACME_STATUS: OnceLock<Arc<RwLock<AcmeStatusInfo>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum AcmeState {
    Idle,
    Provisioning,
    Success,
    Failed,
}

impl AcmeState {
    fn as_str(&self) -> &'static str {
        match self {
            AcmeState::Idle => "idle",
            AcmeState::Provisioning => "provisioning",
            AcmeState::Success => "success",
            AcmeState::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AcmeStatusInfo {
    pub status: AcmeState,
    pub domain: String,
    pub subdomains: Vec<String>,
    pub cert_dir: PathBuf,
    pub last_attempt: Option<u64>,
    pub last_success: Option<u64>,
    pub last_error: Option<String>,
    pub attempt_count: u32,
    pub next_check: Option<u64>,
}

fn get_or_init_status() -> &'static Arc<RwLock<AcmeStatusInfo>> {
    ACME_STATUS.get_or_init(|| {
        Arc::new(RwLock::new(AcmeStatusInfo {
            status: AcmeState::Idle,
            domain: String::new(),
            subdomains: Vec::new(),
            cert_dir: PathBuf::new(),
            last_attempt: None,
            last_success: None,
            last_error: None,
            attempt_count: 0,
            next_check: None,
        }))
    })
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn init_status(domain: &str, subdomains: &[String], cert_dir: &Path) {
    let status = get_or_init_status();
    if let Ok(mut info) = status.write() {
        info.domain = domain.to_string();
        info.subdomains = subdomains.to_vec();
        info.cert_dir = cert_dir.to_path_buf();
    }
}

fn update_status(state: AcmeState, error: Option<&str>) {
    let status = get_or_init_status();
    if let Ok(mut info) = status.write() {
        info.last_attempt = Some(now_unix());
        info.attempt_count += 1;
        if matches!(state, AcmeState::Success) {
            info.last_success = Some(now_unix());
            info.last_error = None;
        }
        if let Some(err) = error {
            info.last_error = Some(err.to_string());
        }
        info.status = state;
    }
}

fn set_next_check(timestamp: u64) {
    let status = get_or_init_status();
    if let Ok(mut info) = status.write() {
        info.next_check = Some(timestamp);
    }
}

/// Get ACME/TLS status as JSON for the API endpoint.
pub fn get_acme_status() -> serde_json::Value {
    let status = get_or_init_status();
    let info = match status.read() {
        Ok(i) => i.clone(),
        Err(_) => return serde_json::json!({"error": "lock poisoned"}),
    };

    if info.domain.is_empty() {
        return serde_json::json!({
            "status": "not_configured",
            "message": "ACME/Let's Encrypt is not configured"
        });
    }

    // Read cert metadata
    let cert_path = info.cert_dir.join(format!("{}.fullchain.pem", info.domain));
    let (cert_exists, age_days, days_until_renewal) = if cert_path.exists() {
        if let Ok(metadata) = std::fs::metadata(&cert_path) {
            if let Ok(modified) = metadata.modified() {
                let age = modified.elapsed().unwrap_or_default();
                let age_d = age.as_secs() / (24 * 60 * 60);
                // 90-day cert, renew 30 days before expiry = 60-day max age
                let days_until = 60u64.saturating_sub(age_d) as i64;
                (true, Some(age_d), Some(days_until))
            } else {
                (true, None, None)
            }
        } else {
            (true, None, None)
        }
    } else {
        (false, None, None)
    };

    // Read SANs from .sans.json
    let sans_path = info.cert_dir.join(format!("{}.sans.json", info.domain));
    let cert_sans: Vec<String> = std::fs::read_to_string(&sans_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default();

    let format_ts = |ts: Option<u64>| -> serde_json::Value {
        match ts {
            Some(t) => chrono::DateTime::from_timestamp(t as i64, 0)
                .map(|dt| serde_json::Value::String(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()))
                .unwrap_or(serde_json::Value::Null),
            None => serde_json::Value::Null,
        }
    };

    serde_json::json!({
        "status": info.status.as_str(),
        "domain": info.domain,
        "subdomains": info.subdomains,
        "certificate": {
            "exists": cert_exists,
            "age_days": age_days,
            "days_until_renewal": days_until_renewal,
            "sans": cert_sans,
        },
        "last_attempt": format_ts(info.last_attempt),
        "last_success": format_ts(info.last_success),
        "last_error": info.last_error,
        "attempt_count": info.attempt_count,
        "next_renewal_check": format_ts(info.next_check),
    })
}

#[derive(serde::Deserialize)]
struct AcmeDirectory {
    #[serde(rename = "newNonce")]
    new_nonce: String,
    #[serde(rename = "newAccount")]
    new_account: String,
    #[serde(rename = "newOrder")]
    new_order: String,
}

#[derive(serde::Deserialize)]
struct AcmeOrder {
    status: String,
    authorizations: Vec<String>,
    finalize: String,
    certificate: Option<String>,
}

#[derive(serde::Deserialize)]
struct AcmeAuthorization {
    status: String,
    challenges: Vec<AcmeChallenge>,
}

#[derive(serde::Deserialize)]
struct AcmeChallenge {
    #[serde(rename = "type")]
    challenge_type: String,
    url: String,
    token: String,
    #[allow(dead_code)]
    status: String,
}

struct AcmeClient {
    http: reqwest::Client,
    key_pair: EcdsaKeyPair,
    rng: SystemRandom,
    directory: AcmeDirectory,
    account_url: Option<String>,
    cert_dir: PathBuf,
}

impl AcmeClient {
    async fn new(cert_dir: &Path, staging: bool) -> Result<Self, String> {
        let rng = SystemRandom::new();

        std::fs::create_dir_all(cert_dir)
            .map_err(|e| format!("Failed to create cert dir: {}", e))?;

        let account_key_path = cert_dir.join("acme-account.key");
        let pkcs8_bytes = if account_key_path.exists() {
            std::fs::read(&account_key_path)
                .map_err(|e| format!("Failed to read account key: {}", e))?
        } else {
            let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
                .map_err(|e| format!("Key generation failed: {}", e))?;
            let bytes = pkcs8.as_ref().to_vec();
            std::fs::write(&account_key_path, &bytes)
                .map_err(|e| format!("Failed to save account key: {}", e))?;
            bytes
        };

        let key_pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8_bytes, &rng)
                .map_err(|e| format!("Failed to load key pair: {}", e))?;

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("HTTP client failed: {}", e))?;

        let dir_url = if staging { LE_STAGING } else { LE_PRODUCTION };
        let directory: AcmeDirectory = http
            .get(dir_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch ACME directory: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Invalid ACME directory: {}", e))?;

        Ok(Self {
            http,
            key_pair,
            rng,
            directory,
            account_url: None,
            cert_dir: cert_dir.to_path_buf(),
        })
    }

    async fn get_nonce(&self) -> Result<String, String> {
        let resp = self
            .http
            .head(&self.directory.new_nonce)
            .send()
            .await
            .map_err(|e| format!("Nonce request failed: {}", e))?;
        resp.headers()
            .get("replay-nonce")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| "No nonce in response".to_string())
    }

    fn jwk_thumbprint(&self) -> String {
        let public_key = self.key_pair.public_key().as_ref();
        let x = URL_SAFE_NO_PAD.encode(&public_key[1..33]);
        let y = URL_SAFE_NO_PAD.encode(&public_key[33..65]);
        // Canonical JWK JSON (alphabetical key order per RFC 7638)
        let jwk_json = format!(r#"{{"crv":"P-256","kty":"EC","x":"{}","y":"{}"}}"#, x, y);
        let hash = digest(&SHA256, jwk_json.as_bytes());
        URL_SAFE_NO_PAD.encode(hash.as_ref())
    }

    fn jws_with_jwk(&self, url: &str, payload: &str, nonce: &str) -> Result<String, String> {
        let public_key = self.key_pair.public_key().as_ref();
        let x = URL_SAFE_NO_PAD.encode(&public_key[1..33]);
        let y = URL_SAFE_NO_PAD.encode(&public_key[33..65]);

        let header = serde_json::json!({
            "alg": "ES256",
            "jwk": { "crv": "P-256", "kty": "EC", "x": x, "y": y },
            "nonce": nonce,
            "url": url
        });

        self.sign_jws(&header, payload)
    }

    fn jws_with_kid(&self, url: &str, payload: &str, nonce: &str) -> Result<String, String> {
        let kid = self.account_url.as_deref().ok_or("No account URL")?;

        let header = serde_json::json!({
            "alg": "ES256",
            "kid": kid,
            "nonce": nonce,
            "url": url
        });

        self.sign_jws(&header, payload)
    }

    fn sign_jws(&self, header: &serde_json::Value, payload: &str) -> Result<String, String> {
        let protected = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
        let payload_b64 = if payload.is_empty() {
            String::new()
        } else {
            URL_SAFE_NO_PAD.encode(payload.as_bytes())
        };

        let signing_input = format!("{}.{}", protected, payload_b64);
        let signature = self
            .key_pair
            .sign(&self.rng, signing_input.as_bytes())
            .map_err(|e| format!("Signing failed: {}", e))?;
        let sig_b64 = URL_SAFE_NO_PAD.encode(signature.as_ref());

        let jws = serde_json::json!({
            "protected": protected,
            "payload": payload_b64,
            "signature": sig_b64
        });

        Ok(jws.to_string())
    }

    async fn register_account(&mut self, email: &str) -> Result<(), String> {
        let nonce = self.get_nonce().await?;

        let payload = if email.is_empty() {
            serde_json::json!({ "termsOfServiceAgreed": true }).to_string()
        } else {
            serde_json::json!({
                "termsOfServiceAgreed": true,
                "contact": [format!("mailto:{}", email)]
            })
            .to_string()
        };

        let url = self.directory.new_account.clone();
        let body = self.jws_with_jwk(&url, &payload, &nonce)?;

        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/jose+json")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Account registration failed: {}", e))?;

        self.account_url = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if self.account_url.is_none() {
            return Err("No account URL in response".to_string());
        }

        log::info!("ACME account registered");
        Ok(())
    }

    async fn request_certificate(&mut self, domain: &str, subdomains: &[String]) -> Result<(), String> {
        // Build list of domains: bare domain + www + additional subdomains.
        // Every SAN must have a valid DNS A record pointing to this server,
        // otherwise Let's Encrypt HTTP-01 validation fails for the ENTIRE certificate.
        let mut domains: Vec<String> = vec![
            domain.to_string(),
            format!("www.{}", domain),
        ];
        for sub in subdomains {
            let fqdn = if sub.contains('.') {
                sub.clone() // already fully qualified (e.g. "www.example.com")
            } else {
                format!("{}.{}", sub, domain)
            };
            if !domains.contains(&fqdn) {
                domains.push(fqdn);
            }
        }
        log::info!("ACME: Requesting certificate for {} SANs: {:?}", domains.len(), domains);
        let identifiers: Vec<serde_json::Value> = domains
            .iter()
            .map(|d| serde_json::json!({"type": "dns", "value": d}))
            .collect();

        // 1. Create order
        let nonce = self.get_nonce().await?;
        let payload = serde_json::json!({
            "identifiers": identifiers
        })
        .to_string();

        let new_order_url = self.directory.new_order.clone();
        let body = self.jws_with_kid(&new_order_url, &payload, &nonce)?;

        let resp = self
            .http
            .post(&new_order_url)
            .header("Content-Type", "application/jose+json")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Order creation failed: {}", e))?;

        let order_status = resp.status();
        let order_url = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let order_body = resp
            .text()
            .await
            .unwrap_or_else(|_| "no body".to_string());

        if !order_status.is_success() || order_url.is_none() {
            log::error!(
                "ACME order creation: status={}, location={:?}, body={}",
                order_status,
                order_url,
                &order_body[..order_body.len().min(500)]
            );
            return Err(format!(
                "Order creation failed ({}): {}",
                order_status,
                &order_body[..order_body.len().min(300)]
            ));
        }

        let order_url = order_url.unwrap();
        let order: AcmeOrder = serde_json::from_str(&order_body)
            .map_err(|e| format!("Invalid order response: {}", e))?;

        if order.authorizations.is_empty() {
            return Err("No authorizations in order".to_string());
        }

        // 2. Process ALL authorizations (one per domain in the order)
        let thumbprint = self.jwk_thumbprint();
        for auth_url in &order.authorizations {
            let nonce = self.get_nonce().await?;
            let body = self.jws_with_kid(auth_url, "", &nonce)?;

            let resp = self
                .http
                .post(auth_url)
                .header("Content-Type", "application/jose+json")
                .body(body)
                .send()
                .await
                .map_err(|e| format!("Authorization fetch failed: {}", e))?;

            let auth: AcmeAuthorization = resp
                .json()
                .await
                .map_err(|e| format!("Invalid authorization: {}", e))?;

            // Skip already-valid authorizations
            if auth.status == "valid" {
                log::info!("ACME authorization already valid");
                continue;
            }

            // 3. Find HTTP-01 challenge
            let challenge = auth
                .challenges
                .iter()
                .find(|c| c.challenge_type == "http-01")
                .ok_or("No HTTP-01 challenge found")?;

            // 4. Set up challenge response
            let key_auth = format!("{}.{}", challenge.token, thumbprint);
            log::info!("ACME challenge: token={}", challenge.token);
            set_challenge(challenge.token.clone(), key_auth);

            // 5. Tell ACME to verify
            let nonce = self.get_nonce().await?;
            let challenge_url = challenge.url.clone();
            let body = self.jws_with_kid(&challenge_url, "{}", &nonce)?;

            self.http
                .post(&challenge_url)
                .header("Content-Type", "application/jose+json")
                .body(body)
                .send()
                .await
                .map_err(|e| format!("Challenge response failed: {}", e))?;

            // 6. Poll authorization until valid
            let mut auth_ok = false;
            for attempt in 0..30 {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                let nonce = self.get_nonce().await?;
                let body = self.jws_with_kid(auth_url, "", &nonce)?;

                let resp = self
                    .http
                    .post(auth_url)
                    .header("Content-Type", "application/jose+json")
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| format!("Auth poll failed: {}", e))?;

                let poll_auth: AcmeAuthorization = resp
                    .json()
                    .await
                    .map_err(|e| format!("Invalid auth response: {}", e))?;

                match poll_auth.status.as_str() {
                    "valid" => {
                        log::info!("ACME authorization valid");
                        auth_ok = true;
                        break;
                    }
                    "invalid" => {
                        remove_challenge(&challenge.token);
                        return Err("Authorization failed".to_string());
                    }
                    _ => {
                        log::debug!("ACME auth poll attempt {}: status={}", attempt + 1, poll_auth.status);
                    }
                }
            }

            remove_challenge(&challenge.token);

            if !auth_ok {
                return Err("Authorization timeout".to_string());
            }
        }

        // 7. Generate CSR and key pair for all domains
        // IMPORTANT: key_pem is held in memory — NOT written to disk yet!
        // Writing the key before the cert is saved causes a cert/key mismatch
        // if ACME fails partway through (the old cert would pair with the new key).
        let (csr_der, key_pem) = self.generate_csr_and_key_multi(&domains)?;

        // 8. Finalize order with CSR
        let csr_b64 = URL_SAFE_NO_PAD.encode(&csr_der);
        let nonce = self.get_nonce().await?;
        let payload = serde_json::json!({"csr": csr_b64}).to_string();
        let body = self.jws_with_kid(&order.finalize, &payload, &nonce)?;

        let finalize_resp = self
            .http
            .post(&order.finalize)
            .header("Content-Type", "application/jose+json")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Finalize failed: {}", e))?;

        let finalize_status = finalize_resp.status();
        let finalize_body = finalize_resp
            .text()
            .await
            .unwrap_or_else(|_| "no body".to_string());

        log::info!(
            "ACME finalize response: status={}, body={}",
            finalize_status,
            &finalize_body[..finalize_body.len().min(500)]
        );

        if !finalize_status.is_success() {
            return Err(format!("Finalize rejected ({}): {}", finalize_status, finalize_body));
        }

        // 9. Poll order for certificate URL
        let cert_url = {
            let mut result = None;
            for attempt in 0..30 {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                let nonce = self.get_nonce().await?;
                let body = self.jws_with_kid(&order_url, "", &nonce)?;

                let resp = self
                    .http
                    .post(&order_url)
                    .header("Content-Type", "application/jose+json")
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| format!("Order poll failed: {}", e))?;

                let order: AcmeOrder = resp
                    .json()
                    .await
                    .map_err(|e| format!("Invalid order: {}", e))?;

                match order.status.as_str() {
                    "valid" => {
                        result = order.certificate;
                        break;
                    }
                    "invalid" => {
                        return Err("Order became invalid".to_string());
                    }
                    _ => {
                        log::debug!(
                            "Order poll attempt {}: status={}",
                            attempt + 1,
                            order.status
                        );
                    }
                }
            }
            result.ok_or("Certificate URL not received")?
        };

        // 10. Download certificate
        let nonce = self.get_nonce().await?;
        let body = self.jws_with_kid(&cert_url, "", &nonce)?;

        let resp = self
            .http
            .post(&cert_url)
            .header("Content-Type", "application/jose+json")
            .header("Accept", "application/pem-certificate-chain")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Cert download failed: {}", e))?;

        let cert_pem = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read certificate: {}", e))?;

        // 11. Save certificate AND private key together (atomic pair).
        // The key is saved ONLY after the cert is fully downloaded — this prevents
        // cert/key mismatch if ACME fails partway (e.g. network error during download).
        let cert_path = self.cert_dir.join(format!("{}.fullchain.pem", domain));
        let key_path = self.cert_dir.join(format!("{}.privkey.pem", domain));

        std::fs::write(&cert_path, &cert_pem)
            .map_err(|e| format!("Failed to save certificate: {}", e))?;
        std::fs::write(&key_path, &key_pem)
            .map_err(|e| format!("Failed to save private key: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&key_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&key_path, perms);
            }
        }

        log::info!(
            "Let's Encrypt certificate + key saved for {}: {:?}",
            domain,
            cert_path
        );

        // Save SAN list for change detection on future restarts.
        // When subdomains change (e.g. new server added), check_and_renew()
        // compares the stored list with the requested list and re-provisions.
        let sans_path = self.cert_dir.join(format!("{}.sans.json", domain));
        let _ = std::fs::write(
            &sans_path,
            serde_json::to_string(&domains).unwrap_or_default(),
        );

        Ok(())
    }

    fn generate_csr_and_key_multi(&self, domains: &[String]) -> Result<(Vec<u8>, String), String> {
        let mut params = rcgen::CertificateParams::new(domains.to_vec());
        let mut dn = rcgen::DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, &domains[0]);
        params.distinguished_name = dn;

        let cert = rcgen::Certificate::from_params(params)
            .map_err(|e| format!("CSR generation failed: {}", e))?;

        let csr_der = cert
            .serialize_request_der()
            .map_err(|e| format!("CSR serialization failed: {}", e))?;

        let key_pem = cert.serialize_private_key_pem();

        Ok((csr_der, key_pem))
    }
}

/// ACME Status Dashboard HTML template. The placeholder `__ACME_DATA__` is replaced
/// with the current ACME status JSON at render time.
pub const ACME_DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>ACME/TLS Status - Rush Sync Server</title>
<link rel="icon" href="/.rss/favicon.svg" type="image/svg+xml">
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;background:#0a0a0f;color:#e4e4ef;min-height:100vh}
.container{max-width:900px;margin:0 auto;padding:24px}
.header{display:flex;justify-content:space-between;align-items:center;margin-bottom:24px}
.header h1{font-size:24px;font-weight:700;letter-spacing:-0.5px}
.header h1 span{color:#6c63ff}
.nav-links{display:flex;gap:12px}
.nav-links a{color:#6c63ff;text-decoration:none;font-size:14px}
.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:16px;margin-bottom:24px}
.card{background:#14141f;border:1px solid #2a2a3a;border-radius:12px;padding:20px}
.card .lbl{font-size:12px;color:#8888a0;text-transform:uppercase;letter-spacing:0.5px;margin-bottom:8px}
.card .val{font-size:28px;font-weight:700}
.card .val.green{color:#00d4aa}
.card .val.red{color:#ff4466}
.card .val.yellow{color:#ffaa00}
.card .val.blue{color:#00a8ff}
.card .val.purple{color:#6c63ff}
.card .sub{font-size:11px;color:#8888a0;margin-top:4px}
.section{background:#14141f;border:1px solid #2a2a3a;border-radius:12px;padding:20px;margin-bottom:16px}
.section h2{font-size:15px;margin-bottom:16px;font-weight:600;color:#c0c0d0}
.info-row{display:flex;justify-content:space-between;padding:10px 0;border-bottom:1px solid #1a1a2a;font-size:13px}
.info-row:last-child{border-bottom:none}
.info-row .label{color:#8888a0}
.info-row .value{font-weight:500;color:#e4e4ef}
.info-row .value.success{color:#00d4aa}
.info-row .value.error{color:#ff4466}
.info-row .value.warn{color:#ffaa00}
.info-row .value code{background:#1a1a2a;padding:2px 6px;border-radius:4px;font-size:12px}
.status-banner{padding:16px 20px;border-radius:12px;margin-bottom:24px;display:flex;align-items:center;gap:12px;font-weight:600}
.status-banner .dot{width:12px;height:12px;border-radius:50%;flex-shrink:0}
.status-banner.success{background:#00d4aa15;border:1px solid #00d4aa40;color:#00d4aa}
.status-banner.success .dot{background:#00d4aa;box-shadow:0 0 8px #00d4aa80}
.status-banner.failed{background:#ff446615;border:1px solid #ff446640;color:#ff4466}
.status-banner.failed .dot{background:#ff4466;box-shadow:0 0 8px #ff446680}
.status-banner.provisioning{background:#ffaa0015;border:1px solid #ffaa0040;color:#ffaa00}
.status-banner.provisioning .dot{background:#ffaa00;box-shadow:0 0 8px #ffaa0080;animation:pulse 1.5s infinite}
.status-banner.idle{background:#6c63ff15;border:1px solid #6c63ff40;color:#6c63ff}
.status-banner.idle .dot{background:#6c63ff}
.status-banner.not_configured{background:#55555515;border:1px solid #55555540;color:#888}
.status-banner.not_configured .dot{background:#555}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:0.4}}
.san-list{display:flex;flex-wrap:wrap;gap:6px}
.san-tag{background:#1a1a2a;border:1px solid #2a2a3a;border-radius:6px;padding:4px 10px;font-size:12px;font-family:monospace}
.error-box{background:#ff446610;border:1px solid #ff446630;border-radius:8px;padding:12px 16px;font-size:13px;color:#ff8899;font-family:monospace;word-break:break-all}
.footer{text-align:center;font-size:11px;color:#555;padding:16px}
</style>
</head>
<body>
<div class="container">
<div class="header"><h1>ACME/TLS <span>Status</span></h1><div class="nav-links"><a href="/.rss/">&larr; Dashboard</a><a href="/api/analytics/dashboard">Analytics</a><a href="/api/acme/status">JSON</a></div></div>
<div id="banner"></div>
<div class="cards" id="cards"></div>
<div class="section" id="cert-section"><h2>Certificate</h2><div id="cert-info"></div></div>
<div class="section" id="sans-section"><h2>Subject Alternative Names</h2><div id="sans-list"></div></div>
<div class="section" id="detail-section"><h2>Details</h2><div id="details"></div></div>
<div id="error-section" style="display:none" class="section"><h2>Last Error</h2><div id="error-box"></div></div>
<div class="footer" id="foot">Loading...</div>
</div>
<script>
var D=__ACME_DATA__;
function render(){
var s=D.status||'not_configured';
var labels={'success':'Certificate Active','failed':'Provisioning Failed','provisioning':'Provisioning in Progress...','idle':'Idle','not_configured':'Not Configured'};
document.getElementById('banner').innerHTML='<div class="status-banner '+s+'"><span class="dot"></span>'+esc(labels[s]||s)+'</div>';
if(s==='not_configured'){document.getElementById('cards').innerHTML='<div class="card"><div class="lbl">Status</div><div class="val purple">N/A</div><div class="sub">ACME not configured</div></div>';document.getElementById('foot').textContent='ACME/Let\'s Encrypt is not enabled';return}
var cert=D.certificate||{};
document.getElementById('cards').innerHTML='<div class="card"><div class="lbl">Status</div><div class="val '+(s==='success'?'green':s==='failed'?'red':s==='provisioning'?'yellow':'blue')+'">'+esc(s.charAt(0).toUpperCase()+s.slice(1))+'</div></div>'+'<div class="card"><div class="lbl">Certificate</div><div class="val '+(cert.exists?'green':'red')+'">'+(cert.exists?'Valid':'Missing')+'</div>'+(cert.age_days!=null?'<div class="sub">'+cert.age_days+' days old</div>':'')+'</div>'+'<div class="card"><div class="lbl">Renewal</div><div class="val '+((cert.days_until_renewal!=null&&cert.days_until_renewal>14)?'green':(cert.days_until_renewal!=null&&cert.days_until_renewal>0)?'yellow':'red')+'">'+(cert.days_until_renewal!=null?cert.days_until_renewal+' days':'N/A')+'</div><div class="sub">until renewal</div></div>'+'<div class="card"><div class="lbl">Attempts</div><div class="val purple">'+(D.attempt_count||0)+'</div><div class="sub">provisioning attempts</div></div>';
var ci='';
ci+='<div class="info-row"><span class="label">Domain</span><span class="value"><code>'+esc(D.domain||'')+'</code></span></div>';
ci+='<div class="info-row"><span class="label">Subdomains</span><span class="value">'+(D.subdomains&&D.subdomains.length?D.subdomains.map(function(s){return '<code>'+esc(s)+'</code>'}).join(' '):'<em>none</em>')+'</span></div>';
ci+='<div class="info-row"><span class="label">Certificate Exists</span><span class="value '+(cert.exists?'success':'error')+'">'+(cert.exists?'Yes':'No')+'</span></div>';
if(cert.age_days!=null)ci+='<div class="info-row"><span class="label">Certificate Age</span><span class="value">'+cert.age_days+' days</span></div>';
if(cert.days_until_renewal!=null)ci+='<div class="info-row"><span class="label">Days Until Renewal</span><span class="value '+(cert.days_until_renewal>14?'success':'warn')+'">'+cert.days_until_renewal+' days</span></div>';
document.getElementById('cert-info').innerHTML=ci;
var sans=cert.sans||[];
if(sans.length>0){document.getElementById('sans-section').style.display='';document.getElementById('sans-list').innerHTML='<div class="san-list">'+sans.map(function(s){return '<span class="san-tag">'+esc(s)+'</span>'}).join('')+'</div>'}else{document.getElementById('sans-section').style.display='none'}
var di='';
di+='<div class="info-row"><span class="label">Last Attempt</span><span class="value">'+(D.last_attempt?fmtTime(D.last_attempt):'Never')+'</span></div>';
di+='<div class="info-row"><span class="label">Last Success</span><span class="value '+(D.last_success?'success':'')+'">'+(D.last_success?fmtTime(D.last_success):'Never')+'</span></div>';
di+='<div class="info-row"><span class="label">Next Renewal Check</span><span class="value">'+(D.next_renewal_check?fmtTime(D.next_renewal_check):'Not scheduled')+'</span></div>';
di+='<div class="info-row"><span class="label">Attempt Count</span><span class="value">'+(D.attempt_count||0)+'</span></div>';
document.getElementById('details').innerHTML=di;
if(D.last_error){document.getElementById('error-section').style.display='';document.getElementById('error-box').innerHTML='<div class="error-box">'+esc(D.last_error)+'</div>'}else{document.getElementById('error-section').style.display='none'}
document.getElementById('foot').textContent='Last updated: '+new Date().toLocaleTimeString()+' \u00b7 Auto-refresh in 15s'}
function fmtTime(s){try{var d=new Date(s);return d.toLocaleString()}catch(e){return s}}
function esc(s){var d=document.createElement('div');d.textContent=s;return d.innerHTML}
render();setTimeout(function(){location.reload()},15000);
</script>
</body></html>"#;

// Public API

/// Provision a Let's Encrypt certificate for a domain.
/// The proxy must be running on port 80 to serve HTTP-01 challenges.
pub async fn provision_certificate(
    domain: &str,
    cert_dir: &Path,
    email: &str,
    staging: bool,
    subdomains: &[String],
) -> Result<(), String> {
    log::info!(
        "Starting Let's Encrypt provisioning for {} (staging={})",
        domain,
        staging
    );

    let mut client = AcmeClient::new(cert_dir, staging).await?;
    client.register_account(email).await?;
    client.request_certificate(domain, subdomains).await?;

    log::info!("Let's Encrypt certificate provisioned for {}", domain);
    Ok(())
}

/// Check if a certificate exists and is valid. Returns true if renewal was performed.
pub async fn check_and_renew(
    domain: &str,
    cert_dir: &Path,
    email: &str,
    staging: bool,
    renew_before_days: u32,
    subdomains: &[String],
) -> Result<bool, String> {
    let cert_path = cert_dir.join(format!("{}.fullchain.pem", domain));
    let key_path = cert_dir.join(format!("{}.privkey.pem", domain));

    if !cert_path.exists() || !key_path.exists() {
        log::info!("No certificate found for {}, provisioning...", domain);
        provision_certificate(domain, cert_dir, email, staging, subdomains).await?;
        return Ok(true);
    }

    // Check if requested subdomains differ from what the cert was provisioned with.
    // This detects when a new server/subdomain is added and triggers re-provisioning.
    if !subdomains.is_empty() {
        let sans_path = cert_dir.join(format!("{}.sans.json", domain));
        let mut expected: Vec<String> = vec![domain.to_string(), format!("www.{}", domain)];
        for sub in subdomains {
            let fqdn = if sub.contains('.') {
                sub.clone()
            } else {
                format!("{}.{}", sub, domain)
            };
            if !expected.contains(&fqdn) {
                expected.push(fqdn);
            }
        }
        expected.sort();

        let sans_mismatch = if let Ok(content) = std::fs::read_to_string(&sans_path) {
            if let Ok(mut stored) = serde_json::from_str::<Vec<String>>(&content) {
                stored.sort();
                stored != expected
            } else {
                true // corrupt file
            }
        } else {
            // No .sans.json yet — cert was provisioned before SAN tracking.
            // Re-provision to ensure all subdomains are included.
            true
        };

        if sans_mismatch {
            log::info!(
                "ACME: Certificate SANs changed for {}, re-provisioning with {} subdomains...",
                domain,
                subdomains.len()
            );
            provision_certificate(domain, cert_dir, email, staging, subdomains).await?;
            return Ok(true);
        }
    }

    // Check certificate age (simple file modification time check)
    let metadata = std::fs::metadata(&cert_path)
        .map_err(|e| format!("Failed to read cert metadata: {}", e))?;

    let modified = metadata
        .modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?;

    let age = modified.elapsed().unwrap_or_default();
    let max_age = std::time::Duration::from_secs((90 - renew_before_days) as u64 * 24 * 60 * 60);

    if age > max_age {
        log::info!(
            "Certificate for {} is due for renewal ({} days old)",
            domain,
            age.as_secs() / (24 * 60 * 60)
        );
        provision_certificate(domain, cert_dir, email, staging, subdomains).await?;
        return Ok(true);
    }

    let remaining_days = (max_age.as_secs().saturating_sub(age.as_secs())) / (24 * 60 * 60);
    log::debug!(
        "Certificate for {} is valid ({} days until renewal)",
        domain,
        remaining_days
    );
    Ok(false)
}

/// Start background ACME provisioning and renewal.
/// Runs initial check after a short delay (to let proxy start), then every 24h.
/// After provisioning/renewal, hot-reloads the proxy's TLS config automatically.
/// If provisioning with subdomains fails, retries with bare domain only.
pub fn start_acme_background(domain: String, cert_dir: PathBuf, email: String, staging: bool, subdomains: Vec<String>) {
    init_status(&domain, &subdomains, &cert_dir);

    tokio::spawn(async move {
        // Wait for proxy + HTTP redirect server to be fully ready
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Initial provisioning/renewal
        update_status(AcmeState::Provisioning, None);
        let provisioned = match check_and_renew(&domain, &cert_dir, &email, staging, 30, &subdomains).await {
            Ok(renewed) => {
                update_status(AcmeState::Success, None);
                if renewed {
                    log::info!("ACME: Certificate provisioned/renewed for {} (with {} subdomains)", domain, subdomains.len());
                } else {
                    log::info!("ACME: Certificate for {} is still valid", domain);
                }
                true
            }
            Err(e) => {
                log::error!("ACME: Failed to provision with subdomains {:?}: {}", subdomains, e);
                update_status(AcmeState::Failed, Some(&e));

                // CRITICAL: Do NOT fall back to bare domain if a cert already exists!
                // The bare domain fallback would OVERWRITE a good multi-SAN certificate
                // with one that only has the bare domain, breaking all subdomain HTTPS.
                let cert_path = cert_dir.join(format!("{}.fullchain.pem", domain));
                if cert_path.exists() {
                    log::warn!(
                        "ACME: Keeping existing certificate for {} (will retry with all subdomains next cycle)",
                        domain
                    );
                    true // reload existing cert into proxy
                } else {
                    // No certificate at all — try bare domain as last resort to get HTTPS working
                    log::info!("ACME: No certificate exists. Trying bare domain only: {}", domain);
                    update_status(AcmeState::Provisioning, None);
                    match check_and_renew(&domain, &cert_dir, &email, staging, 30, &[]).await {
                        Ok(renewed) => {
                            update_status(AcmeState::Success, None);
                            if renewed {
                                log::info!("ACME: Certificate provisioned for {} (bare domain fallback)", domain);
                            }
                            true
                        }
                        Err(e2) => {
                            log::error!("ACME: Bare domain fallback also failed for {}: {}", domain, e2);
                            update_status(AcmeState::Failed, Some(&e2));
                            false
                        }
                    }
                }
            }
        };

        // ALWAYS reload TLS on startup — even if the cert wasn't renewed, the proxy
        // may have started with a self-signed cert and needs to load the LE cert.
        if provisioned {
            crate::proxy::handler::reload_proxy_tls(&domain);
        } else {
            // Retry sooner (60s) instead of waiting 24h
            log::info!("ACME: Will retry in 60 seconds...");
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            update_status(AcmeState::Provisioning, None);
            match check_and_renew(&domain, &cert_dir, &email, staging, 30, &subdomains).await {
                Ok(true) => {
                    update_status(AcmeState::Success, None);
                    log::info!("ACME: Certificate provisioned on retry for {}", domain);
                    crate::proxy::handler::reload_proxy_tls(&domain);
                }
                Ok(false) => {
                    update_status(AcmeState::Success, None);
                    log::info!("ACME: Certificate for {} is valid on retry", domain);
                }
                Err(e) => {
                    update_status(AcmeState::Failed, Some(&e));
                    log::error!("ACME: Retry also failed for {}: {}", domain, e);
                }
            }
        }

        // Periodic renewal check (every 24 hours)
        set_next_check(now_unix() + 24 * 60 * 60);
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(24 * 60 * 60));
        loop {
            interval.tick().await;
            update_status(AcmeState::Provisioning, None);
            match check_and_renew(&domain, &cert_dir, &email, staging, 30, &subdomains).await {
                Ok(true) => {
                    update_status(AcmeState::Success, None);
                    log::info!("ACME: Certificate renewed for {}", domain);
                    crate::proxy::handler::reload_proxy_tls(&domain);
                }
                Ok(false) => {
                    update_status(AcmeState::Success, None);
                }
                Err(e) => {
                    update_status(AcmeState::Failed, Some(&e));
                    log::error!("ACME: Renewal check failed for {}: {}", domain, e);
                }
            }
            set_next_check(now_unix() + 24 * 60 * 60);
        }
    });
}
