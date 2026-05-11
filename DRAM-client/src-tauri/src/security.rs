use keyring::Entry;
use rand::{distributions::Alphanumeric, Rng};

pub fn get_or_create_master_key() -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let service = "com.quorthon.app.vault";
    let username = "master_key";

    let entry = Entry::new(service, username)?;

    match entry.get_password() {
        Ok(password) => {
            let bytes = hex::decode(&password)?;
            let arr: [u8; 32] = bytes.try_into().map_err(|_| "Stored key is not 32 bytes")?;
            Ok(arr)
        }
        Err(keyring::Error::NoEntry) => {
            let key: [u8; 32] = rand::thread_rng().gen();
            entry.set_password(&hex::encode(key))?;
            Ok(key)
        }
        Err(e) => Err(e.into()),
    }
}