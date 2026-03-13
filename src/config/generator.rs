use std::path::Path;

use rand::RngExt;
use rand::distr::Alphanumeric;
use totp_rs::{Algorithm, TOTP};

fn generate_random_bytes<const N: usize>() -> [u8; N] {
    let mut rng = rand::rng();
    let mut buf = [0u8; N];
    rng.fill(&mut buf);
    buf
}

fn generate_password() -> String {
    let mut rng = rand::rng();
    (0..16).map(|_| rng.sample(Alphanumeric) as char).collect()
}

fn generate_jwt_secret() -> String {
    let bytes = generate_random_bytes::<16>();
    hex::encode(bytes)
}

fn generate_totp_url() -> anyhow::Result<String> {
    let secret = generate_random_bytes::<32>();
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_vec(),
        None,
        "admin".to_owned(),
    )?;
    Ok(totp.get_url())
}

pub fn generate_auth_config(path: impl AsRef<Path>) -> anyhow::Result<bool> {
    let path = path.as_ref();

    // 如果文件存在，则不生成配置文件
    if path.exists() && path.is_file() {
        return Ok(false);
    }

    let config = toml::to_string_pretty(&serde_json::json!({
        "admin": {
            "password": generate_password(),
            "totp_url": generate_totp_url()?,
        },
        "jwt": { "secret": generate_jwt_secret() }
    }))?;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, config)?;

    Ok(true)
}
