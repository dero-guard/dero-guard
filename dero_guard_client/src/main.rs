use dero_guard::wg::load_keys;

mod vpn;

fn main() {
    if let Err(err) = load_keys() {
        eprintln!("Error while loading keys: {}", err);
    }
}
