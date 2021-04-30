mod vpn;

use vpn::{VPNError, VPN};

fn main() {
    if let Err(err) = start_vpn() {
        eprintln!("Error while starting VPN: {}", err);
    }
}

fn start_vpn() -> Result<(), VPNError> {
    let mut vpn = VPN::new()?;
    vpn.connect(
        "SERVER PUBLIC KEY".into(),
        "SERVER IP ADDRESS".into(),
        22350,
        "10.0.0.2/24".into()
    )?;

    Ok(())
}
