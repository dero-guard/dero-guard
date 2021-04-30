use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::io::{Write, Read, Error as IoError};

use failure::Fail;

use crate::command::{execute_with, execute, ExecutionError};

pub const DEVICE_NAME: &str = "dero-guard";

pub struct WireguardKeys {
    pub private_key: String,
    pub public_key: String
}

pub struct WireguardConfig {
    pub keys: WireguardKeys,
    pub listen_port: u16,
    pub peers: Vec<WireguardPeer>
}

pub struct WireguardPeer {
    pub public_key: String,
    pub allowed_ips: String,
    pub endpoint: Option<String>
}

pub fn load_keys() -> Result<WireguardKeys, WireguardError> {
    let private_key = get_folder()?.join("private_key");
    let private_key = if !private_key.exists() {
        println!(" - Generating private key at '{:?}'", private_key);

        let key = execute(vec!["wg", "genkey"])?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o660)
            .open(&private_key)?;

        file.write(key.as_bytes())?;

        key
    } else {
        println!(" - Found private key at '{:?}'", private_key);

        let mut file = File::open(&private_key)?;
        let mut key = String::new();

        file.read_to_string(&mut key)?;

        key
    };

    let public_key = execute_with(vec!["wg", "pubkey"], &private_key)?;
    println!(" - Using public key '{}'", public_key);

    Ok(WireguardKeys { private_key, public_key })
}

pub fn setup_interface(address: &str) -> Result<(), WireguardError> {
    execute(vec!["ip", "link", "del", "dev", DEVICE_NAME])?;
    execute(vec!["ip", "link", "add", "dev", DEVICE_NAME, "type", "wireguard"])?;
    execute(vec!["ip", "address", "add", "dev", DEVICE_NAME, address])?;
    execute(vec!["ip", "link", "set", "up", "dev", DEVICE_NAME])?;

    println!(" - Set up interface with local address '{}'", address);

    Ok(())
}

pub fn apply_configuration(config: &WireguardConfig) -> Result<(), WireguardError> {
    let mut result = format!("\
[Interface]
ListenPort = {}
PrivateKey = {}", config.listen_port, config.keys.private_key);

    for peer in &config.peers {
        result += &format!("\n
[Peer]
PublicKey = {}
AllowedIPs = {}
PersistentKeepalive = 25", peer.public_key, peer.allowed_ips);

        if let Some(endpoint) = &peer.endpoint {
            result += &format!("\nEndpoint = {}", endpoint);
        }
    }

    let file = get_folder()?.join("generated.conf");
    let mut config = OpenOptions::new()
        .create(!file.exists())
        .write(true)
        .open(&file)?;

    config.write_all(result.as_bytes())?;

    println!(" - Generated wireguard configuration at '{:?}'", file);

    execute(vec!["wg", "setconf", DEVICE_NAME, &format!("{}", file.display())])?;

    Ok(())
}

fn get_folder() -> Result<PathBuf, IoError> {
    let folder = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(format!("{}/.config/dero-guard", home)),
        Err(_) => PathBuf::from(".dero-guard"),
    };

    if !folder.exists() {
        std::fs::create_dir_all(&folder)?;
    }

    Ok(folder)
}

#[derive(Debug, Fail)]
pub enum WireguardError {
    #[fail(display = "I/O error while manipulating key files: {}", inner)]
    IoError {
        inner: IoError
    },

    #[fail(display = "Error during command execution: {}", inner)]
    ExecError {
        inner: ExecutionError
    }
}

impl From<IoError> for WireguardError {
    fn from(err: IoError) -> Self {
        WireguardError::IoError { inner: err }
    }
}

impl From<ExecutionError> for WireguardError {
    fn from(err: ExecutionError) -> Self {
        WireguardError::ExecError { inner: err }
    }
}
