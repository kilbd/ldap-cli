use crate::config::config_path;
use color_eyre::eyre::Result;
use rand::{rngs::OsRng, RngCore};
use std::fs;

pub fn generate_key(password: String) -> Result<(Vec<u8>, [u8; 32])> {
    let argon_config = argon2::Config {
        variant: argon2::Variant::Argon2id,
        hash_length: 32,
        lanes: 8,
        mem_cost: 16 * 1024,
        time_cost: 8,
        ..Default::default()
    };
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    let key = argon2::hash_raw(password.as_bytes(), &salt, &argon_config)?;
    fs::write(config_path()?.join("ldap.key"), &key)?;
    Ok((key, salt))
}
