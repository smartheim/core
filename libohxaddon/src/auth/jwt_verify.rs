//! # Verifies gRCP https authentication JWT header

pub struct JWTVerifier {

}

pub enum JWTVerificationResult {
    OK,
    /// The given input is not a JWT
    InvalidJWT,
    /// The key_id cannot be found. This is similar to "Expired".
    InvalidKey,
    /// The token expired. A new one need to be requested first from ohx-auth.
    Expired
}

impl JWTVerifier {
    pub fn verify(auth_header_value:&str) ->JWTVerificationResult {
        todo!()
    }
}