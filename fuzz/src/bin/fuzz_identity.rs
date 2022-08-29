use afl::fuzz;
use zorn_core::identity::ZornIdentity;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = s.parse::<ZornIdentity>();
        }
    });
}
