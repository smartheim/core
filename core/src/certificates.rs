use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, SanType, date_time_ymd, KeyPair, PKCS_ECDSA_P256_SHA256};
use std::fs;

use log::{warn, error};

use std::fs::File;
use std::io::{BufReader, Read, ErrorKind};
use chrono::{Datelike, DateTime, Utc};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;

pub use snafu::{ResultExt, Snafu};
use crate::http_server::HttpServiceControl;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to read configuration from {}: {}", path.display(), source))]
    IOError { source: std::io::Error, path: PathBuf },

    #[snafu(display("Unable to Parse X509"))]
    X509Parser,

    #[snafu(display("http certificates expired!"))]
    CertificateExpired,

    #[snafu(display("RcgenError: {}", source))]
    RCGen { source: rcgen::RcgenError },
}

const KEY_FILENAME: &'static str = "https_key.pem";
const KEY_FILENAME_DER: &'static str = "https_key.der";
const PUBLIC_FILENAME: &'static str = "https_cert.pem";
const PUBLIC_FILENAME_DER: &'static str = "https_cert.der";

pub enum FileFormat {
    DER,
    PEM,
}

pub fn key_filename(cert_dir: &Path, format: FileFormat) -> PathBuf {
    match format {
        FileFormat::DER => cert_dir.join(KEY_FILENAME_DER),
        FileFormat::PEM => cert_dir.join(KEY_FILENAME),
    }
}

pub fn cert_filename(cert_dir: &Path, format: FileFormat) -> PathBuf {
    match format {
        FileFormat::DER => cert_dir.join(PUBLIC_FILENAME_DER),
        FileFormat::PEM => cert_dir.join(PUBLIC_FILENAME),
    }
}

/// Checks the certificates at the given path.
/// If they are still valid a tuple is returned: (false, duration_until_expire).
///
/// If there are no certificates or they are about to expire (14 days left),
/// a new self signed certificate is generated and saved to the given path.
/// The function returns (true, duration_until_expire).
pub fn check_gen_certificates(cert_dir: &Path) -> Result<(bool, Duration), Error> {
    if let Ok(duration) = check_existing(&cert_dir) {
        return Ok((false, duration));
    }

    std::fs::create_dir_all(&cert_dir).context(IOError { path: cert_dir })?;
    let now = chrono::Utc::now();
    let cert = create_cert(&cert_dir, now)?;
    use x509_parser::parse_x509_der;
    let cert = cert.serialize_der().context(RCGen {})?;
    let x509 = parse_x509_der(cert.as_slice())
        .map_err(|_e| Error::X509Parser)?;
    let duration = x509.1.tbs_certificate.validity.time_to_expiration()
        .ok_or(Error::CertificateExpired)?;
    Ok((true, duration))
}

pub struct RefreshSelfSigned {
    http_control: HttpServiceControl,
    shutdown_rx: tokio::sync::mpsc::Receiver<()>,
    cert_dir: PathBuf,
}

impl RefreshSelfSigned {
    pub fn new(http_control: HttpServiceControl, cert_dir: PathBuf) -> (Self, tokio::sync::mpsc::Sender<()>) {
        let (mut cert_watch_shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);
        (RefreshSelfSigned { http_control, shutdown_rx, cert_dir }, cert_watch_shutdown_tx)
    }

    pub async fn run(self) {
        let RefreshSelfSigned { http_control, mut shutdown_rx, cert_dir } = self;

        loop {
            match check_gen_certificates(&cert_dir) {
                Err(_) => {
                    error!("Failed to re-generate certificates");
                    break;
                }
                Ok((cert_generated, duration)) => {
                    if cert_generated {
                        http_control.restart().await;
                    }
                    if let Ok(_) = timeout(duration, shutdown_rx.recv()).await {
                        // Shutdown signal received
                        break;
                    }
                    continue;
                }
            }
        }
    }
}

fn create_cert(cert_dir: &Path, now: DateTime<Utc>) -> Result<Certificate, Error> {
    let cert = Certificate::from_params(create_cert_params(now)?).context(RCGen {})?;
    fs::write(cert_dir.join(PUBLIC_FILENAME), &cert.serialize_pem().context(RCGen {})?.as_bytes())
        .context(IOError { path: cert_dir.join(PUBLIC_FILENAME) })?;
    fs::write(cert_dir.join(PUBLIC_FILENAME_DER), &cert.serialize_der().context(RCGen {})?)
        .context(IOError { path: cert_dir.join(PUBLIC_FILENAME_DER) })?;
    fs::write(cert_dir.join(KEY_FILENAME), &cert.serialize_private_key_pem().as_bytes())
        .context(IOError { path: cert_dir.join(KEY_FILENAME) })?;
    fs::write(cert_dir.join(KEY_FILENAME_DER), &cert.serialize_private_key_der())
        .context(IOError { path: cert_dir.join(KEY_FILENAME_DER) })?;
    Ok(cert)
}

fn create_cert_params(now: DateTime<Utc>) -> Result<CertificateParams, Error> {
    let mut params: CertificateParams = Default::default();
    params.alg = &PKCS_ECDSA_P256_SHA256;
    params.key_pair = Some(KeyPair::generate(&PKCS_ECDSA_P256_SHA256).context(RCGen {})?);
    params.not_before = date_time_ymd(1970, 01, 01);
    params.not_after = now.with_year(now.year() + 1).expect("Year to be in range");
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(DnType::OrganizationName, "OHX Community");
    params.distinguished_name.push(DnType::CommonName, "OHX Smarthome");
    params.subject_alt_names = vec![SanType::DnsName("ohx.local".to_string()),
                                    SanType::DnsName("localhost".to_string())];
    Ok(params)
}

/// Return true if a valid certificate has been found
pub fn check_existing(cert_dir: &Path) -> Result<Duration, Error> {
    use x509_parser::parse_x509_der;

    let cert_file = cert_dir.join(PUBLIC_FILENAME_DER);
    if !cert_file.exists() { return Err(Error::CertificateExpired); }

    let validity = File::open(&cert_file).and_then(|file| {
        let mut buffer = Vec::new();
        BufReader::new(file).read_to_end(&mut buffer)?;
        let p = parse_x509_der(&buffer).map_err(|_| std::io::Error::new(ErrorKind::Other, "Failed to parse x509_der cert"))?;
        Ok(p.1.tbs_certificate.validity)
    }).context(IOError { path: cert_file.to_path_buf() })?;

    if let Some(duration) = validity.time_to_expiration() {
        // Return true if the certificate is still valid for at least 2 weeks
        if duration.as_secs() > 60 * 60 * 24 * 14 {
            return Ok(duration);
        }
    }
    return Err(Error::CertificateExpired);
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use super::{KEY_FILENAME_DER, PUBLIC_FILENAME_DER};

    #[test]
    fn check_existing() {
        let dir = tempfile::tempdir().expect("Create temp dir");
        let path = dir.path();

        let _ = std::fs::remove_file(path.join(KEY_FILENAME_DER));
        let _ = std::fs::remove_file(path.join(PUBLIC_FILENAME_DER));

        assert!(super::check_existing(path).is_err());
        let now = chrono::Utc::now();
        super::create_cert(path, now).expect("create_cert");
        assert!(super::check_existing(path).is_ok());
    }

    #[test]
    fn check_valid() {
        let dir = tempfile::tempdir().expect("Create temp dir");
        let path = dir.path();

        let _ = std::fs::remove_file(dir.path().join(KEY_FILENAME_DER));
        let _ = std::fs::remove_file(dir.path().join(PUBLIC_FILENAME_DER));

        let now = chrono::Utc::now();
        let now = now.with_year(now.year() - 1).unwrap();
        super::create_cert(path, now).expect("create_cert");
        assert!(super::check_existing(path).is_err());
    }
}