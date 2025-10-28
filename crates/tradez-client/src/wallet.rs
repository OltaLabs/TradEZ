use std::path::Path;

use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;

pub struct Wallet {
    pub local_signer: PrivateKeySigner,
}

impl Wallet {
    pub fn load_wallet(dirpath: &str, name: &str, password: String) -> Self {
        if !Path::new(dirpath).is_dir() {
            std::fs::create_dir_all(dirpath).expect("Failed to create wallet directory");
        }
        let filepath = format!("{}/{}", dirpath, name);
        if Path::new(&filepath).is_file() {
            // Load existing wallet from file
            Self {
                local_signer: PrivateKeySigner::decrypt_keystore(filepath, password)
                    .expect("Failed to decrypt wallet"),
            }
        } else {
            // Create a new wallet
            let signer = PrivateKeySigner::random();
            let mut rng = rand::thread_rng();
            PrivateKeySigner::new_keystore(dirpath, &mut rng, password, Some(name))
                .expect("Failed to create wallet");
            Self {
                local_signer: signer,
            }
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, String> {
        println!("Signing with key: {:?}", self.local_signer.public_key());
        let signature = self
            .local_signer
            .sign_message_sync(message)
            .map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&signature.as_bytes());
        Ok(bytes)
    }
}
