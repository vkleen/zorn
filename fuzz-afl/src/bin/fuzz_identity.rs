use afl::fuzz;
use zorn_core::identity::ZornIdentity;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data).map(|s| s.trim_end()) {
            if let Ok(i) = s.parse::<ZornIdentity>() {
                println!("{}", i.to_string());
            }
        }
    });
}
