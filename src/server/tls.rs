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
        let exe_path = std::env::current_exe().map_err(AppError::Io)?;
        let base_dir = exe_path.parent().ok_or_else(|| {
            AppError::Validation("Cannot determine executable directory".to_string())
        })?;

        let cert_path = base_dir.join(cert_dir);
        fs::create_dir_all(&cert_path).map_err(AppError::Io)?;

        Ok(Self {
            cert_dir: cert_path,
            validity_days,
        })
    }

    pub fn get_rustls_config(&self, server_name: &str, port: u16) -> Result<Arc<ServerConfig>> {
        let cert_file = self.get_cert_path(server_name, port);
        let key_file = self.get_key_path(server_name, port);

        // Zertifikat erstellen falls nicht vorhanden
        if !cert_file.exists() || !key_file.exists() {
            self.generate_certificate(server_name, port)?;
        }

        // Zertifikat und Key laden
        let cert_chain = self.load_certificates(&cert_file)?;
        let private_key = self.load_private_key(&key_file)?;

        // Rustls Konfiguration erstellen
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .map_err(|e| AppError::Validation(format!("TLS config error: {}", e)))?;

        Ok(Arc::new(config))
    }

    fn generate_certificate(&self, server_name: &str, port: u16) -> Result<()> {
        log::info!("Generating TLS certificate for {}:{}", server_name, port);

        // Subject Alternative Names - Wildcard für Proxy, spezifisch für Server
        let subject_alt_names = if server_name == "proxy" {
            vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "*.localhost".to_string(), // Wildcard für alle Subdomains
                "proxy.localhost".to_string(),
            ]
        } else {
            vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                format!("{}.localhost", server_name), // spezifische Subdomain
                format!("{}:{}", server_name, port),
            ]
        };

        let mut params = CertificateParams::new(subject_alt_names);

        // Distinguished Name - Korrekte Common Names
        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::OrganizationName, "Rush Sync Server");

        let common_name = if server_name == "proxy" {
            "*.localhost" // Wildcard CN für Proxy
        } else {
            &format!("{}.localhost", server_name)
        };

        dn.push(rcgen::DnType::CommonName, common_name);
        params.distinguished_name = dn;

        // Rest der Funktion bleibt gleich...
        params.not_before = time::OffsetDateTime::now_utc() - time::Duration::days(1);
        params.not_after =
            time::OffsetDateTime::now_utc() + time::Duration::days(self.validity_days as i64);

        params.key_usages = vec![
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::KeyEncipherment,
        ];

        params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];

        // Zertifikat generieren
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
                    // Parse server-port aus Dateiname
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
        // Prüfen ob bereits Let's Encrypt Zertifikat existiert
        let cert_file = self.cert_dir.join(format!("{}.fullchain.pem", domain));
        let key_file = self.cert_dir.join(format!("{}.privkey.pem", domain));

        if cert_file.exists() && key_file.exists() {
            log::info!("Loading existing Let's Encrypt certificate for {}", domain);
            let cert_chain = self.load_certificates(&cert_file)?;
            let private_key = self.load_private_key(&key_file)?;

            let config = ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(cert_chain, private_key)
                .map_err(|e| AppError::Validation(format!("TLS config error: {}", e)))?;

            return Ok(Arc::new(config));
        }

        log::warn!("No Let's Encrypt certificate found for {}", domain);
        log::info!("Using self-signed certificate for development");

        // Fallback zu self-signed
        self.get_rustls_config("proxy", 443)
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
