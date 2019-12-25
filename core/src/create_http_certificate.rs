use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, SanType, KeyPair, PKCS_ED25519, ExtendedKeyUsagePurpose, CustomExtension, PKCS_RSA_SHA256, PKCS_ECDSA_P256_SHA256};
use std::fs;

use log::{info, error};

use std::fs::File;
use std::io::{BufReader, Read, ErrorKind};
use chrono::{Datelike, DateTime, Utc};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;

pub use snafu::{ResultExt, Snafu};
use std::net::IpAddr;
use std::str::FromStr;

use libohxcore::{FileFormat, key_filename, cert_filename};

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

    // TODO Add IP?
    let domains = ["ohx.local", "localhost"];

    let domains = domains
        .iter()
        .map(|dom| {
            if let Ok(ip) = IpAddr::from_str(dom) {
                SanType::IpAddress(ip)
            } else {
                SanType::DnsName(dom.to_string())
            }
        })
        .collect();

    std::fs::create_dir_all(&cert_dir).context(IOError { path: cert_dir })?;
    let now = chrono::Utc::now();
    let cert = create_cert(&cert_dir, now, domains)?;
    use x509_parser::parse_x509_der;
    let cert = cert.serialize_der().context(RCGen {})?;
    let x509 = parse_x509_der(cert.as_slice())
        .map_err(|_e| Error::X509Parser)?;
    let duration = x509.1.tbs_certificate.validity.time_to_expiration()
        .ok_or(Error::CertificateExpired)?;
    Ok((true, duration))
}

pub struct RefreshSelfSigned {
    shutdown_rx: tokio::sync::mpsc::Receiver<()>,
    cert_dir: PathBuf,
}

impl RefreshSelfSigned {
    pub fn new(cert_dir: PathBuf) -> (Self, tokio::sync::mpsc::Sender<()>) {
        let (cert_watch_shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);
        (RefreshSelfSigned { shutdown_rx, cert_dir }, cert_watch_shutdown_tx)
    }

    pub async fn run(self) {
        let RefreshSelfSigned { mut shutdown_rx, cert_dir } = self;

        loop {
            match check_gen_certificates(&cert_dir) {
                Err(_) => {
                    error!("Failed to re-generate certificates");
                    break;
                }
                Ok((cert_generated, duration)) => {
                    if cert_generated {
                        info!("A new self-signed certificate has been created");
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

fn create_cert(cert_dir: &Path, now: DateTime<Utc>, domains: Vec<SanType>) -> Result<Certificate, Error> {
    let cert = Certificate::from_params(create_cert_params(now, domains, true)?).context(RCGen {})?;
    let path = cert_filename(cert_dir, FileFormat::PEM);
    fs::write(&path, &cert.serialize_pem().context(RCGen {})?.as_bytes()).context(IOError { path })?;
    let path = cert_filename(cert_dir, FileFormat::DER);
    fs::write(&path, &cert.serialize_der().context(RCGen {})?).context(IOError { path })?;
    let path = key_filename(cert_dir, FileFormat::PEM);
    fs::write(&path, &cert.serialize_private_key_pem().as_bytes()).context(IOError { path })?;
    let path = key_filename(cert_dir, FileFormat::DER);
    fs::write(&path, &cert.serialize_private_key_der()).context(IOError { path })?;
    Ok(cert)
}

pub fn create_cert_params(now: DateTime<Utc>, domains: Vec<SanType>, use_ed25519: bool) -> Result<CertificateParams, Error> {
    use rand::Rng;
    const OID_ORG_UNIT: &[u64] = &[2, 5, 4, 11];

    let mut params: CertificateParams = Default::default();
    if use_ed25519 {
        params.alg = &PKCS_ED25519;
        params.key_pair = Some(KeyPair::generate(&PKCS_ED25519).context(RCGen {})?);
    } else {
        params.alg = &PKCS_ECDSA_P256_SHA256;
        params.key_pair = Some(KeyPair::generate(&PKCS_ECDSA_P256_SHA256).context(RCGen {})?);
    }
    params.serial_number = Some(rand::thread_rng().gen());
    params.not_before = now;
    params.not_after = now.with_year(now.year() + 1).expect("Year to be in range");
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(DnType::CountryName, "DE");
    params.distinguished_name.push(DnType::OrganizationName, "OHX Community");
    params.distinguished_name.push(DnType::CommonName, "OHX Smarthome");
    params.distinguished_name.push(DnType::from_oid(OID_ORG_UNIT), "ohx.local");
    params.subject_alt_names = domains;
    params.extended_key_usages.push(ExtendedKeyUsagePurpose::ServerAuth);
    params.custom_extensions.push(not_ca());
    params.custom_extensions.push(key_usage());
    Ok(params)
}

const KEY_USAGE: &[bool] = &[
    true, // digitalSignature
    true, // nonRepudiation/contentCommitment
    true, // keyEncipherment
    false,
    false,
    false, // keyCertSign
    false, // cRLSign
    false,
    false,
];

fn key_usage() -> CustomExtension {
    let der = yasna::construct_der(|writer| {
        writer.write_bitvec(&KEY_USAGE.iter().map(bool::to_owned).collect());
    });

    const OID_KEY_USAGE: &[u64] = &[2, 5, 29, 15];

    let mut key_usage = CustomExtension::from_oid_content(OID_KEY_USAGE, der);
    key_usage.set_criticality(true);
    key_usage
}

fn not_ca() -> CustomExtension {
    let der = yasna::construct_der(|writer| {
        writer.write_sequence(|writer| {
            writer.next().write_bool(false);
        });
    });

    const OID_BASIC: &[u64] = &[2, 5, 29, 19];
    CustomExtension::from_oid_content(OID_BASIC, der)
}

/// Return true if a valid certificate has been found
pub fn check_existing(cert_dir: &Path) -> Result<Duration, Error> {
    use x509_parser::parse_x509_der;

    let cert_file = key_filename(cert_dir, FileFormat::DER);
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
    use rcgen::SanType;
    use libohxcore::{cert_filename, FileFormat, key_filename};

    #[test]
    fn check_existing() {
        let dir = tempfile::tempdir().expect("Create temp dir");
        let path = dir.path();

        let _ = std::fs::remove_file(cert_filename(path, FileFormat::DER));
        let _ = std::fs::remove_file(key_filename(path, FileFormat::DER));

        let domains = vec![SanType::DnsName("localhost".to_owned())];

        assert!(super::check_existing(path).is_err());
        let now = chrono::Utc::now();
        super::create_cert(path, now, domains).expect("create_cert");
        assert!(super::check_existing(path).is_ok());
    }

    #[test]
    fn check_valid() {
        let dir = tempfile::tempdir().expect("Create temp dir");
        let path = dir.path();

        let _ = std::fs::remove_file(cert_filename(path, FileFormat::DER));
        let _ = std::fs::remove_file(key_filename(path, FileFormat::DER));

        let domains = vec![SanType::DnsName("localhost".to_owned())];

        let now = chrono::Utc::now();
        let now = now.with_year(now.year() - 1).unwrap();
        super::create_cert(path, now, domains).expect("create_cert");
        assert!(super::check_existing(path).is_err());
    }
}