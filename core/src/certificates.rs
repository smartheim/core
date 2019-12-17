use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, SanType, date_time_ymd, KeyPair, PKCS_ECDSA_P256_SHA256};
use std::fs;

use log::warn;

use std::fs::File;
use std::io::{BufReader, Read, ErrorKind};
use chrono::{Datelike, DateTime, Utc};
use std::path::{Path, PathBuf};

const KEY_FILENAME: &'static str = "https_key.pem";
const KEY_FILENAME_DER: &'static str = "https_key.der";
const PUBLIC_FILENAME: &'static str = "https_cert.pem";
const PUBLIC_FILENAME_DER: &'static str = "https_cert.der";

pub enum FileFormat {
    DER,
    PEM
}

pub fn key_filename(cert_dir: &Path,format: FileFormat) -> PathBuf {
    match format {
        FileFormat::DER => cert_dir.join(KEY_FILENAME_DER),
        FileFormat::PEM => cert_dir.join(KEY_FILENAME),
    }
}

pub fn cert_filename(cert_dir: &Path,format: FileFormat) -> PathBuf {
    match format {
        FileFormat::DER => cert_dir.join(PUBLIC_FILENAME_DER),
        FileFormat::PEM => cert_dir.join(PUBLIC_FILENAME),
    }
}

pub fn check_gen_certificates(cert_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if check_existing(&cert_dir) {
        return Ok(());
    }

    std::fs::create_dir_all(&cert_dir)?;
    let now = chrono::Utc::now();
    create_cert(&cert_dir, now)?;
    Ok(())
}

fn create_cert(cert_dir: &Path, now: DateTime<Utc>) -> Result<(), Box<dyn std::error::Error>> {
    let cert = Certificate::from_params(create_cert_params(now)?)?;
    fs::write(cert_dir.join(PUBLIC_FILENAME), &cert.serialize_pem().unwrap().as_bytes())?;
    fs::write(cert_dir.join(PUBLIC_FILENAME_DER), &cert.serialize_der().unwrap())?;
    fs::write(cert_dir.join(KEY_FILENAME), &cert.serialize_private_key_pem().as_bytes())?;
    fs::write(cert_dir.join(KEY_FILENAME_DER), &cert.serialize_private_key_der())?;
    Ok(())
}

fn create_cert_params(now: DateTime<Utc>) -> Result<CertificateParams, Box<dyn std::error::Error>> {
    let mut params: CertificateParams = Default::default();
    params.alg = &PKCS_ECDSA_P256_SHA256;
    params.key_pair=Some(KeyPair::generate(&PKCS_ECDSA_P256_SHA256)?);
    params.not_before = date_time_ymd(1970, 01, 01);
    params.not_after = now.with_year(now.year()+1).expect("Year to be in range");
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(DnType::OrganizationName, "OHX Community");
    params.distinguished_name.push(DnType::CommonName, "OHX Smarthome");
    params.subject_alt_names = vec![SanType::DnsName("ohx.local".to_string()),
                                    SanType::DnsName("localhost".to_string())];
    Ok(params)
}

/// Return true if a valid certificate has been found
fn check_existing(cert_dir: &Path) -> bool {
    use x509_parser::parse_x509_der;

    let cert_file = cert_dir.join(PUBLIC_FILENAME_DER);
    if !cert_file.exists() { return false; }

    let public_cert = File::open(cert_file).and_then(|file| {
        let mut buffer = Vec::new();
        BufReader::new(file).read_to_end(&mut buffer)?;
        let p = parse_x509_der(&buffer).map_err(|_| std::io::Error::new(ErrorKind::Other, "Failed to parse x509_der cert"))?;
        Ok(p.1.tbs_certificate.validity)
    });

    match public_cert {
        Ok(validity) => {
            if let Some(duration) = validity.time_to_expiration() {
                // Return true if the certificate is still valid for at least 2 weeks
                return duration.as_secs() > 60 * 60 * 24 * 14;
            }
        }
        Err(e) => warn!("{}", e),
    }

    false
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

        assert_eq!(super::check_existing(path), false);
        let now = chrono::Utc::now();
        super::create_cert(path, now).expect("create_cert");
        assert_eq!(super::check_existing(path), true);
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
        assert_eq!(super::check_existing(path), false);
    }
}