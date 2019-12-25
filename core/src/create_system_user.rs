use std::path::{PathBuf, Path};
use std::io::{BufWriter, BufReader, Read};
use std::fs::File;
use serde_json::json;
use log::info;
use std::io::Write;

pub use snafu::{ResultExt, Snafu};
use ring::signature::{KeyPair, EcdsaSigningAlgorithm, ECDSA_P256_SHA256_FIXED_SIGNING, ECDSA_P256_SHA256_FIXED};
use libohxcore::{system_user_jwks, system_user_key_der, biscuit};
use rcgen::{Certificate, SanType};
use crate::create_http_certificate::create_cert_params;
use chrono::{DateTime, Utc};
use snafu::OptionExt;
use std::net::IpAddr;
use std::str::FromStr;
use biscuit::jwa::SignatureAlgorithm;
use biscuit::jws::{Secret, Header, Compact, RegisteredHeader};
use std::sync::Arc;
use ring::signature;
use serde::{Serialize, Deserialize};
use biscuit::{ClaimsSet, RegisteredClaims, SingleOrMultiple, Empty};
use biscuit::jwk::{AlgorithmParameters, JWK, JWKSet, CommonParameters, PublicKeyUse, EllipticCurveKeyParameters, EllipticCurve};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to create cert path {}: {}", path.display(), source))]
    CreateCertPathError { source: std::io::Error, path: PathBuf },

    #[snafu(display("Failed to use the just generated certificate for Ring"))]
    UseGeneratedCert {},

    #[snafu(display("TODO: {}", source))]
    GenericIOError { source: std::io::Error },

    #[snafu(display("Serde error: {}", source))]
    SerdeError { source: serde_json::Error },

    #[snafu(display("Biscuit error: {}", source))]
    BiscuitError { source: biscuit::errors::Error },

    #[snafu(display("Failed to create system auth certificate parameters: {}", source))]
    CertificateParamsFailed { source: crate::create_http_certificate::Error },

    #[snafu(display("RcgenError: {}", source))]
    RCGen { source: rcgen::RcgenError },
}

pub fn create_check_system_user(cert_path: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(&cert_path).context(CreateCertPathError { path: cert_path.to_path_buf() })?;

    let system_user_jwks_file = system_user_jwks(cert_path);
    let system_user_key_der_file = system_user_key_der(cert_path);
    if system_user_jwks_file.exists() && system_user_key_der_file.exists() {
        if let Ok(_) = check_jwks(&system_user_jwks_file, &system_user_key_der_file) {
            return Ok(());
        }
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

    let now = chrono::Utc::now();

    create_jwks(&system_user_jwks_file, &system_user_key_der_file, now, domains)?;
    Ok(())
}

fn check_jwks(jwks_file: &Path, key_der: &Path) -> Result<(), Error> {
    let jwks: biscuit::jwk::JWKSet<biscuit::Empty> = serde_json::from_reader(BufReader::new(File::open(&jwks_file).context(GenericIOError {})?)).context(SerdeError {})?;

    let mut key_pair_der = BufReader::new(File::open(&key_der).context(GenericIOError {})?);
    let mut buffer_key = Vec::new();
    key_pair_der.read_to_end(&mut buffer_key).context(GenericIOError {})?;
    let secret = Secret::EcdsaKeyPair(Arc::new(signature::EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &buffer_key).ok().context(UseGeneratedCert {})?));

    let key = match jwks.keys.get(0) {
        Some(key) => key,
        None => { return Err(Error::UseGeneratedCert {}); }
    };

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    struct PrivateClaims {
        company: String,
        department: String,
    }

    let expected_claims = ClaimsSet::<PrivateClaims> {
        registered: RegisteredClaims {
            issuer: Some(FromStr::from_str("https://openhabx.com").expect("")),
            subject: Some(FromStr::from_str("John Doe").expect("")),
            audience: Some(SingleOrMultiple::Single(FromStr::from_str("https://openhabx.com").expect(""))),
            not_before: Some(1234.into()),
            ..Default::default()
        },
        private: PrivateClaims {
            department: "Toilet Cleaning".to_string(),
            company: "OHX".to_string(),
        },
    };

    let header = Header {
        registered: RegisteredHeader {
            algorithm: SignatureAlgorithm::ES256,
            ..Default::default()
        },
        private: biscuit::Empty {},
    };

    let expected_jwt = Compact::new_decoded(header.clone(), expected_claims);
    let token = expected_jwt.into_encoded(&secret).context(BiscuitError {})?;

    let _decoded = token.into_decoded(&secret, SignatureAlgorithm::ES256).context(BiscuitError {})?;

    return Ok(());
}

fn create_jwks(jwks_file: &Path, key_der: &Path, now: DateTime<Utc>, domains: Vec<SanType>) -> Result<(), Error> {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = ring::signature::EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).ok().context(UseGeneratedCert {})?;
    let key_pair = signature::EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8_bytes.as_ref()).ok().context(UseGeneratedCert {})?;

    x509_parser::parse_subject_public_key_info();

    let mut key_pair_der = BufWriter::new(File::create(&key_der).context(GenericIOError {})?);
    key_pair_der.write(pkcs8_bytes.as_ref()).context(GenericIOError {})?;

    let uid = now.timestamp_millis();

    let jwk_set: JWKSet<Empty> = JWKSet {
        keys: vec![
            JWK {
                common: CommonParameters {
                    public_key_use: Some(PublicKeyUse::Signature),
                    key_operations: None,
                    algorithm: Some(biscuit::jwa::Algorithm::Signature(SignatureAlgorithm::ES256)),
                    key_id: Some(uid.to_string()),
                    ..Default::default()
                },
                algorithm: AlgorithmParameters::EllipticCurve(EllipticCurveKeyParameters {
                    key_type: Default::default(),
                    curve: EllipticCurve::P256,
                    x: vec![
                        48, 160, 66, 76, 210, 28, 41, 68, 131, 138, 45, 117, 201, 43, 55, 231,
                        110, 162, 13, 159, 0, 137, 58, 59, 78, 238, 138, 60, 10, 175, 236, 62,
                    ],
                    y: vec![
                        224, 75, 101, 233, 36, 86, 217, 136, 139, 82, 179, 121, 189, 251, 213,
                        30, 232, 105, 239, 31, 15, 198, 91, 102, 89, 105, 91, 108, 206, 8, 23,
                        35,
                    ],
                    d: None,
                }),
                additional: Default::default(),
            },
        ],
    };

    let mut jwks = BufWriter::new(File::create(&jwks_file).context(GenericIOError {})?);
    serde_json::to_writer(&mut jwks, &jwk_set).context(SerdeError {})?;

    Ok(())
}