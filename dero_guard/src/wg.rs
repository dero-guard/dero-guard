use std::fs::{File, OpenOptions};
use std::io::{Error as IoError, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use failure::Fail;

use crate::command::{execute, execute_with, ExecutionError};
use std::str::FromStr;

pub const DEVICE_NAME: &str = "dero-guard";

pub struct WireguardKeys {
    pub private_key: String,
    pub public_key: String,
}

pub struct WireguardConfig {
    pub keys: WireguardKeys,
    pub listen_port: u16,
    pub peers: Vec<WireguardPeer>,
}

pub struct WireguardPeer {
    pub public_key: String,
    pub allowed_ips: String,
    pub endpoint: Option<String>,
}

#[derive(Debug)]
pub struct BandwidthUsage {
    pub download: u64,
    pub upload: u64,
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

    Ok(WireguardKeys {
        private_key,
        public_key,
    })
}

pub fn setup_interface(address: &str) -> Result<(), WireguardError> {
    remove_interface()?;

    execute(vec![
        "ip",
        "link",
        "add",
        "dev",
        DEVICE_NAME,
        "type",
        "wireguard",
    ])?;
    execute(vec!["ip", "address", "add", "dev", DEVICE_NAME, address])?;
    execute(vec!["ip", "link", "set", "up", "dev", DEVICE_NAME])?;

    println!(" - Set up interface with local address '{}'", address);

    Ok(())
}

pub fn remove_interface() -> Result<(), WireguardError> {
    execute(vec!["ip", "link", "del", "dev", DEVICE_NAME])?;
    Ok(())
}

pub fn apply_configuration(config: &WireguardConfig) -> Result<(), WireguardError> {
    let mut result = format!(
        "\
[Interface]
ListenPort = {}
PrivateKey = {}",
        config.listen_port, config.keys.private_key
    );

    for peer in &config.peers {
        result += &format!(
            "\n
[Peer]
PublicKey = {}
AllowedIPs = {}
PersistentKeepalive = 25",
            peer.public_key, peer.allowed_ips
        );

        if let Some(endpoint) = &peer.endpoint {
            result += &format!("\nEndpoint = {}", endpoint);
        }
    }

    let file = get_folder()?.join("generated.conf");
    let mut config = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&file)?;

    config.write_all(result.as_bytes())?;

    println!(" - Generated wireguard configuration at '{:?}'", file);

    execute(vec![
        "wg",
        "setconf",
        DEVICE_NAME,
        &format!("{}", file.display()),
    ])?;

    Ok(())
}

pub fn get_bandwidth(public_key: &str) -> Result<BandwidthUsage, WireguardError> {
    use WireguardError::ContentParsingError;

    let output = execute(vec!["wg", "show", DEVICE_NAME])?;
    let expr = format!(
        "interface: {}\\n( {{2}}[^\\n]+\\n){{3}}\\npeer: {}\\n( {{2}}[^\\n]+\\n){{3}} {{2}}transfer: (\\d+\\.\\d+) ([PTKMG]?iB) received, (\\d+\\.\\d+) ([PTKMG]?iB) sent",
        DEVICE_NAME,
        regex::escape(public_key)
    );
    let re = regex::Regex::new(&expr).unwrap();
    let captures = re.captures(&output).ok_or(ContentParsingError)?;

    let len = captures.len();
    if len < 4 {
        return Err(ContentParsingError);
    }

    let capture = |s: isize| {
        captures
            .get((len as isize + s) as usize)
            .ok_or(ContentParsingError)
    };
    let read_float = |m: regex::Match| f64::from_str(m.as_str()).map_err(|_| ContentParsingError);
    let read_unit = |m: regex::Match| {
        let mut pow = 1usize;
        let unit = m.as_str().chars().nth(0).ok_or(ContentParsingError)?;

        for u in &['B', 'K', 'M', 'G', 'T', 'P'] {
            if unit == *u {
                return Ok(pow);
            }

            pow *= 1000;
        }

        Err(ContentParsingError)
    };

    let dl_amount = read_float(capture(-4)?)?;
    let dl_unit = read_unit(capture(-3)?)?;
    let up_amount = read_float(capture(-2)?)?;
    let up_unit = read_unit(capture(-1)?)?;

    Ok(BandwidthUsage {
        download: (dl_amount * dl_unit as f64) as u64,
        upload: (up_amount * up_unit as f64) as u64,
    })
}

pub fn get_folder() -> Result<PathBuf, IoError> {
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
    IoError { inner: IoError },

    #[fail(display = "Error during command execution: {}", inner)]
    ExecError { inner: ExecutionError },

    #[fail(display = "Cannot parse wireguard command output")]
    ContentParsingError,
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
