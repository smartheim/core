fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/grpc")
        .compile(
            &["proto/addon_registration.proto","proto/request_access_token.proto"],
            &["proto"],
        )?;
    Ok(())
}