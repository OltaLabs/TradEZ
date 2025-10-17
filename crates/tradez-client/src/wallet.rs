use std::path::Path;

use alloy_signer_local::PrivateKeySigner;

pub struct Wallet {
    pub local_signer: PrivateKeySigner,
}

pub fn load_wallet(dirpath: &str, name: &str, password: String) -> Wallet {
    if !Path::new(dirpath).is_dir() {
        std::fs::create_dir_all(dirpath).expect("Failed to create wallet directory");
    }
    let filepath = format!("{}/{}.json", dirpath, name);
    if Path::new(&filepath).is_file() {
        // Load existing wallet from file
        Wallet{
            local_signer: PrivateKeySigner::decrypt_keystore(filepath, password)
            .expect("Failed to decrypt wallet")
        }
    } else {
        // Create a new wallet
        let signer = PrivateKeySigner::random();
        let mut rng = rand::thread_rng();
        PrivateKeySigner::new_keystore(dirpath, &mut rng, password, Some(name)).expect("Failed to create wallet");
        Wallet {
            local_signer: signer
        }
    }
}