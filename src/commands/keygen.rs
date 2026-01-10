use ed25519_dalek::SigningKey;
use hex::ToHex;
use rand::rngs::OsRng;
use std::fs::File;
use std::io::Write;

pub fn handle(output_file: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);

    let private_key_hex: String = signing_key.to_bytes().encode_hex();
    let public_key_hex: String = signing_key.verifying_key().to_bytes().encode_hex();

    println!("Public Key: {}", public_key_hex);
    println!("Private Key: {}", private_key_hex);

    if let Some(name) = output_file {
        let pub_path = format!("{}.pub", name);
        let priv_path = name;

        let mut file = File::create(&pub_path)?;
        writeln!(file, "{}", public_key_hex)?;
        println!("Public key written to {}", pub_path);

        let mut file = File::create(&priv_path)?;
        writeln!(file, "{}", private_key_hex)?;
        println!("Private key written to {}", priv_path);
    }

    Ok(())
}
