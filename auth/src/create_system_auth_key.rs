//! # OHX JWT Auth key generation
//! Interprocess communication between OHX core services and OHX Addons via gRPC (http2) and http
//! uses JWT tokens. This module is responsible for creating and rotating keys and
//! manage the corresponding JWKS (JSon Webtoken Key Set) file.
//! It is used for verifying JWT tokens of incoming requests.
//!
//! JWT Tokens may contain arbitrary additional data and will contain access claims
//! and scopes.
//!
//! There are two types of JWT tokens: Access Tokens and Refresh Tokens.
//! Refresh tokens are valid indefinitely until they are revoked. They can only
//! be used to request access tokens. Access tokens are valid for 1 hour.
//!
//! ## Startup
//! On startup this module will create JWT access tokens for a given list of services
//! and stores them in ohx_root_dir/startup/{ohx_service_name}.token.
//! The directory is cleared initially.
//! Those tokens are valid for 5 minutes and can be swapped for JWT refresh tokens.
//!
//! ## JWKS Access
//! The jwks file ("ohx_system.jwks") is public readable for all core services.
//! ohx-serve makes it http accessible for Addons.
//!
//! ## Key Rotation
//! Rotation is not and should never be configurable and is fixed to a 7 day cycle.
//! A new secret key is generated and the public part appended to the jwks file
//! 2 days before the most recent key will expire.

use std::path::{PathBuf, Path};
use std::io::{BufWriter, BufReader, Read};
use std::fs::File;
use serde_json::json;
use log::info;
use std::io::Write;

pub use snafu::{ResultExt, Snafu};
use ring::signature::{KeyPair, EcdsaSigningAlgorithm, ECDSA_P256_SHA256_FIXED_SIGNING, ECDSA_P256_SHA256_FIXED};
use libohxcore::{system_auth_jwks, biscuit};
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
use std::time::Duration;

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

    #[snafu(display("Base64 error: {}", source))]
    Base64DecodeError { source: base64::DecodeError },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct PrivateClaims {
    company: String,
    department: String,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JwkAdditional {
    pub expire: chrono::DateTime<Utc>,
}

/// Swap keys every SWAP_KEY_TIME (7 days). A new key will be generated OVERLAP_TIME (2 days) earlier before expiry.
const SWAP_KEY_TIME: Duration = Duration::from_secs(60 * 60 * 24 * 7);
const OVERLAP_TIME: Duration = Duration::from_secs(60 * 60 * 24 * 2);

pub fn check_generate(cert_path: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(&cert_path).context(CreateCertPathError { path: cert_path.to_path_buf() })?;

    let system_auth_jwks = system_auth_jwks(cert_path);
    let now = chrono::Utc::now();

    // Check jwks file writable
    if system_auth_jwks.exists() {
        let _ = File::open(&system_auth_jwks).context(GenericIOError {})?;
    }

    // Check directory writable
    {
        let dummy_file = system_auth_jwks.with_file_name("_non_");
        let _ = File::create(&dummy_file).context(GenericIOError {})?;
        std::fs::remove_file(&dummy_file).context(GenericIOError {})?;
    }

    update_jwks(&system_auth_jwks, now)
}

/// A given JWK, containing public key components like x,y for EC, is verified against
/// the given private key. Any file access errors result in a returned error and should
/// abort the application (wrong access rights!).
///
/// If the private key does not match the public key or the private key file does not exist,
/// false is returned, otherwise true.
fn is_jwk_usable(key_file: &Path, jwk: &JWK<JwkAdditional>) -> Result<bool, Error> {
    let decode_secret = if let AlgorithmParameters::EllipticCurve(p) = &jwk.algorithm {
        let mut decode_secret = Vec::with_capacity(p.x.len() + p.y.len() + 1);
        decode_secret.push(0x04);
        decode_secret.extend_from_slice(&p.x);
        decode_secret.extend_from_slice(&p.y);
        decode_secret
    } else { return Err(Error::UseGeneratedCert {}); };

    let decode_secret = Secret::PublicKey(decode_secret);

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

    let mut key_pair_der = BufReader::new(File::open(key_file).context(GenericIOError {})?);
    let mut buffer_key = Vec::new();
    key_pair_der.read_to_end(&mut buffer_key).context(GenericIOError {})?;
    let secret = Secret::EcdsaKeyPair(Arc::new(signature::EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &buffer_key).ok().context(UseGeneratedCert {})?));

    let token = expected_jwt.into_encoded(&secret).context(BiscuitError {})?;
    Ok(token.into_decoded(&decode_secret, SignatureAlgorithm::ES256).is_ok())
}

/// Create a new JWK entry (and a new EC with ECDSA_P256_SHA256).
/// Returns a tuple of (private_key, JWK_with_public_key, JWK_ID).
fn create_jwk(now: DateTime<Utc>) -> Result<(ring::pkcs8::Document, JWK<JwkAdditional>, String), Error> {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = ring::signature::EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).ok().context(UseGeneratedCert {})?;
    let key_pair = signature::EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8_bytes.as_ref()).ok().context(UseGeneratedCert {})?;
    let public = key_pair.public_key().as_ref();

    if public[0] != 4 {
        return Err(Error::UseGeneratedCert {});
    }

    let uid = now.timestamp_millis().to_string();

    let jwk = JWK {
        common: CommonParameters {
            public_key_use: Some(PublicKeyUse::Signature),
            key_operations: None,
            algorithm: Some(biscuit::jwa::Algorithm::Signature(SignatureAlgorithm::ES256)),
            key_id: Some(uid.clone()),
            ..Default::default()
        },
        algorithm: AlgorithmParameters::EllipticCurve(EllipticCurveKeyParameters {
            key_type: Default::default(),
            curve: EllipticCurve::P256,
            x: public[1..33].to_vec(),
            y: public[33..].to_vec(),
            d: None,
        }),
        additional: JwkAdditional { expire: now + chrono::Duration::from_std(SWAP_KEY_TIME).expect("") },
    };

    Ok((pkcs8_bytes, jwk, uid))
}

fn update_jwks(jwks_file: &Path, now: DateTime<Utc>) -> Result<(), Error> {
    use biscuit::jwk::JWKSet;

    let mut jwks: JWKSet<JwkAdditional> = if jwks_file.exists() {
        let file = File::open(&jwks_file).context(GenericIOError {})?;
        let jwks: JWKSet<JwkAdditional> = serde_json::from_reader(BufReader::new(file)).context(SerdeError {})?;
        jwks
    } else { JWKSet { keys: Default::default() } };

    let non_existing_file="_non_".to_owned();

    // Filter old entries. Remove old key files if existent.
    let expire_time = chrono::Utc::now() - chrono::Duration::from_std(SWAP_KEY_TIME).expect("");
    for i in jwks.keys.len() as i32 - 1..0 {
        let jwk = &jwks.keys[i as usize];
        let key_file = jwks_file.with_file_name(format!("ohx_system_key_{}.der", jwk.common.key_id.as_ref().unwrap_or(&non_existing_file)));
        if jwk.additional.expire < expire_time || !is_jwk_usable(&key_file, &jwk)? {
            if key_file.exists() {
                std::fs::remove_file(key_file).context(GenericIOError {})?;
            }
            jwks.keys.remove(i as usize);
        }
    }

    // Find latest key
    let key: Option<&JWK<JwkAdditional>> = jwks.keys.iter().fold(None, |acc, v| {
        match acc {
            Some(before) if before.additional.expire > v.additional.expire => Some(before),
            _ => Some(v)
        }
    });

    // Latest key (if any) must not be older than overlap time, otherwise a new key is generated
    let expire_time = chrono::Utc::now() - chrono::Duration::from_std(OVERLAP_TIME).expect("");
    if let Some(key) = key {
        if key.additional.expire > expire_time {
            return Ok(());
        }
    }

    let (pkcs8_document, jwk, uid) = create_jwk(now)?;

    let key_file = jwks_file.with_file_name(format!("ohx_system_key_{}.der", uid));
    let mut key_pair_der = BufWriter::new(File::create(&key_file).context(GenericIOError {})?);
    key_pair_der.write(pkcs8_document.as_ref()).context(GenericIOError {})?;

    jwks.keys.push(jwk);
    let mut jwks_file = BufWriter::new(File::create(&jwks_file).context(GenericIOError {})?);
    serde_json::to_writer_pretty(&mut jwks_file, &jwks).context(SerdeError {})?;

    Ok(())
}