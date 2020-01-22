# OHX Serve

## Command Line Options

The **OHX Root directory** is by default the working directory.
You may change this by starting with `ohx-serve -r your_directory`.

If you provide an https certificate (x509 in *der* format) via `OHX-ROOT/certs/key.der` and `OHX-ROOT/certs/cert.der`,
`ohx-serve` will use it.
If no certificate exist, a self-signed certificate valid for one year will be created which is refreshed 14 days before expiry.

**Ports** are set with `ohx-serve -p 80 -s 443` for http and https.
