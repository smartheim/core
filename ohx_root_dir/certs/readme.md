# Certificates and public keys

This directory contains the https/http2 certificate and JSon WebToken (JWT) 
signing public key in form of a JWKS (JSon WebToken Key Set) file (`ohx_system.jwks`).

The JWKS will be served by ohx-serve as `/.well-known/jwks.json`.

This directory should NEVER contain any private keys.
Those MUST be stored in the individual `config/` directories and owned by the respective service only.