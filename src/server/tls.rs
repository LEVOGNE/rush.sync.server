use crate::core::prelude::*;
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use rustls::{Certificate as RustlsCertificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
pub struct TlsManager {
    cert_dir: PathBuf,
    validity_days: u32,
}

impl TlsManager {
    pub fn new(cert_dir: &str, validity_days: u32) -> Result<Self> {
        let base_dir = crate::core::helpers::get_base_dir()?;

        let cert_path = base_dir.join(cert_dir);
        fs::create_dir_all(&cert_path).map_err(AppError::Io)?;

        Ok(Self {
            cert_dir: cert_path,
            validity_days,
        })
    }

    pub fn get_rustls_config(&self, server_name: &str, port: u16) -> Result<Arc<ServerConfig>> {
        self.get_rustls_config_for_domain(server_name, port, "localhost")
    }

    pub fn get_rustls_config_for_domain(
        &self,
        server_name: &str,
        port: u16,
        production_domain: &str,
    ) -> Result<Arc<ServerConfig>> {
        let cert_file = self.get_cert_path(server_name, port);
        let key_file = self.get_key_path(server_name, port);

        // Generate certificate if it doesn't exist
        if !cert_file.exists() || !key_file.exists() {
            self.generate_certificate_with_domain(server_name, port, production_domain)?;
        }

        // Load certificate and key
        let cert_chain = self.load_certificates(&cert_file)?;
        let private_key = self.load_private_key(&key_file)?;

        // Build rustls configuration
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .map_err(|e| AppError::Validation(format!("TLS config error: {}", e)))?;

        Ok(Arc::new(config))
    }

    fn generate_certificate_with_domain(
        &self,
        server_name: &str,
        port: u16,
        production_domain: &str,
    ) -> Result<()> {
        log::info!("Generating TLS certificate for {}:{}", server_name, port);

        // SANs: wildcard for proxy, specific subdomain for individual servers
        let mut subject_alt_names = if server_name == "proxy" {
            vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "*.localhost".to_string(),
                "proxy.localhost".to_string(),
            ]
        } else {
            vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                format!("{}.localhost", server_name),
                format!("{}:{}", server_name, port),
            ]
        };

        // Add production domain SANs if configured
        if production_domain != "localhost" {
            subject_alt_names.push(production_domain.to_string());
            subject_alt_names.push(format!("*.{}", production_domain));
            if server_name != "proxy" {
                subject_alt_names.push(format!("{}.{}", server_name, production_domain));
            }
        }

        let mut params = CertificateParams::new(subject_alt_names);

        // Distinguished Name
        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::OrganizationName, "Rush Sync Server");

        let common_name = if production_domain != "localhost" {
            if server_name == "proxy" {
                format!("*.{}", production_domain)
            } else {
                format!("{}.{}", server_name, production_domain)
            }
        } else if server_name == "proxy" {
            "*.localhost".to_string()
        } else {
            format!("{}.localhost", server_name)
        };
        let common_name = &common_name;

        dn.push(rcgen::DnType::CommonName, common_name);
        params.distinguished_name = dn;

        // Validity period and key usage
        params.not_before = time::OffsetDateTime::now_utc() - time::Duration::days(1);
        params.not_after =
            time::OffsetDateTime::now_utc() + time::Duration::days(self.validity_days as i64);

        params.key_usages = vec![
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::KeyEncipherment,
        ];

        params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];

        // Generate and serialize certificate
        let cert = Certificate::from_params(params)
            .map_err(|e| AppError::Validation(format!("Certificate generation failed: {}", e)))?;

        let cert_pem = cert.serialize_pem().map_err(|e| {
            AppError::Validation(format!("Certificate serialization failed: {}", e))
        })?;
        let key_pem = cert.serialize_private_key_pem();

        let cert_file = self.get_cert_path(server_name, port);
        let key_file = self.get_key_path(server_name, port);

        fs::write(&cert_file, cert_pem).map_err(AppError::Io)?;
        fs::write(&key_file, key_pem).map_err(AppError::Io)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_file).map_err(AppError::Io)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_file, perms).map_err(AppError::Io)?;
        }

        log::info!("TLS certificate generated with CN: {}", common_name);
        log::info!("Certificate: {:?}", cert_file);
        log::info!("Private Key: {:?}", key_file);

        Ok(())
    }

    fn load_certificates(&self, path: &Path) -> Result<Vec<RustlsCertificate>> {
        let cert_file = fs::File::open(path).map_err(AppError::Io)?;
        let mut reader = BufReader::new(cert_file);

        let cert_chain = certs(&mut reader)
            .map_err(|e| AppError::Validation(format!("Certificate parsing error: {}", e)))?
            .into_iter()
            .map(RustlsCertificate)
            .collect();

        Ok(cert_chain)
    }

    fn load_private_key(&self, path: &Path) -> Result<PrivateKey> {
        let key_file = fs::File::open(path).map_err(AppError::Io)?;
        let mut reader = BufReader::new(key_file);

        let keys = pkcs8_private_keys(&mut reader)
            .map_err(|e| AppError::Validation(format!("Private key parsing error: {}", e)))?;

        if keys.is_empty() {
            return Err(AppError::Validation("No private key found".to_string()));
        }

        Ok(PrivateKey(keys[0].clone()))
    }

    fn get_cert_path(&self, server_name: &str, port: u16) -> PathBuf {
        self.cert_dir.join(format!("{}-{}.cert", server_name, port))
    }

    fn get_key_path(&self, server_name: &str, port: u16) -> PathBuf {
        self.cert_dir.join(format!("{}-{}.key", server_name, port))
    }

    pub fn certificate_exists(&self, server_name: &str, port: u16) -> bool {
        let cert_file = self.get_cert_path(server_name, port);
        let key_file = self.get_key_path(server_name, port);
        cert_file.exists() && key_file.exists()
    }

    pub fn remove_certificate(&self, server_name: &str, port: u16) -> Result<()> {
        let cert_file = self.get_cert_path(server_name, port);
        let key_file = self.get_key_path(server_name, port);

        if cert_file.exists() {
            fs::remove_file(&cert_file).map_err(AppError::Io)?;
            log::info!("Removed certificate: {:?}", cert_file);
        }

        if key_file.exists() {
            fs::remove_file(&key_file).map_err(AppError::Io)?;
            log::info!("Removed private key: {:?}", key_file);
        }

        Ok(())
    }

    pub fn get_certificate_info(&self, server_name: &str, port: u16) -> Option<CertificateInfo> {
        let cert_file = self.get_cert_path(server_name, port);

        if !cert_file.exists() {
            return None;
        }

        let metadata = fs::metadata(&cert_file).ok()?;
        let size = metadata.len();
        let modified = metadata.modified().ok()?;

        Some(CertificateInfo {
            cert_path: cert_file,
            key_path: self.get_key_path(server_name, port),
            file_size: size,
            created: modified,
            valid_days: self.validity_days,
        })
    }

    pub fn list_certificates(&self) -> Result<Vec<CertificateInfo>> {
        let mut certificates = Vec::new();

        let entries = fs::read_dir(&self.cert_dir).map_err(AppError::Io)?;

        for entry in entries {
            let entry = entry.map_err(AppError::Io)?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("cert") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Parse server-port from filename
                    if let Some((server, port_str)) = stem.rsplit_once('-') {
                        if let Ok(port) = port_str.parse::<u16>() {
                            if let Some(info) = self.get_certificate_info(server, port) {
                                certificates.push(info);
                            }
                        }
                    }
                }
            }
        }

        certificates.sort_by(|a, b| b.created.cmp(&a.created));
        Ok(certificates)
    }

    pub fn get_production_config(&self, domain: &str) -> Result<Arc<ServerConfig>> {
        // Check for existing Let's Encrypt certificate
        let cert_file = self.cert_dir.join(format!("{}.fullchain.pem", domain));
        let key_file = self.cert_dir.join(format!("{}.privkey.pem", domain));

        if cert_file.exists() && key_file.exists() {
            log::info!("Loading Let's Encrypt certificate for {}", domain);
            let cert_chain = match self.load_certificates(&cert_file) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("LE cert corrupt for {}: {} — deleting for re-provision", domain, e);
                    let _ = fs::remove_file(&cert_file);
                    let _ = fs::remove_file(&key_file);
                    return Err(e);
                }
            };
            let private_key = match self.load_private_key(&key_file) {
                Ok(k) => k,
                Err(e) => {
                    log::error!("LE key corrupt for {}: {} — deleting for re-provision", domain, e);
                    let _ = fs::remove_file(&cert_file);
                    let _ = fs::remove_file(&key_file);
                    return Err(e);
                }
            };

            match ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(cert_chain, private_key)
            {
                Ok(config) => return Ok(Arc::new(config)),
                Err(e) => {
                    // Cert/key mismatch (e.g. key was overwritten by a failed ACME attempt).
                    // Delete both files so ACME will re-provision on the next cycle.
                    log::error!(
                        "LE cert/key mismatch for {}: {} — deleting for re-provision",
                        domain, e
                    );
                    let _ = fs::remove_file(&cert_file);
                    let _ = fs::remove_file(&key_file);
                    return Err(AppError::Validation(format!(
                        "Cert/key mismatch for {}: {}",
                        domain, e
                    )));
                }
            }
        }

        log::warn!("No Let's Encrypt certificate found for {}", domain);

        // Return error so the caller can generate a proper self-signed cert
        // with the correct production domain SANs (not *.localhost)
        Err(AppError::Validation(format!(
            "No Let's Encrypt certificate found for {}",
            domain
        )))
    }
}

#[derive(Debug)]
pub struct CertificateInfo {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub file_size: u64,
    pub created: std::time::SystemTime,
    pub valid_days: u32,
}

impl CertificateInfo {
    pub fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.created.elapsed() {
            elapsed.as_secs() > (self.valid_days as u64 * 24 * 60 * 60)
        } else {
            true
        }
    }

    pub fn days_until_expiry(&self) -> i64 {
        if let Ok(elapsed) = self.created.elapsed() {
            let elapsed_days = elapsed.as_secs() / (24 * 60 * 60);
            (self.valid_days as i64) - (elapsed_days as i64)
        } else {
            0
        }
    }
}
