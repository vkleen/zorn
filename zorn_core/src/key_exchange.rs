use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};
use zeroize::Zeroize;

use crate::identity::{ZornIdentity, ZornIdentitySecret};

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SharedSecret([u8; 32]);

const KEY_EXCHANGE_CONTEXT : &str = "zorn-encryption.org/v1 shared secret";

fn generate_ephemeral_identity() -> (EphemeralSecret, PublicKey) {
    let s = EphemeralSecret::new(OsRng);
    let pk = PublicKey::from(&s);
    (s, pk)
}

fn compute_sender_shared_secret(sender_secret: &ZornIdentitySecret, ephemeral_secret: EphemeralSecret, ephemeral_identity: &PublicKey, recipient_identity: &ZornIdentity) -> SharedSecret {
    let mut hasher = blake3::Hasher::new_derive_key(KEY_EXCHANGE_CONTEXT);

    hasher.update(sender_secret.diffie_hellman(recipient_identity).as_bytes());
    hasher.update(ephemeral_secret.diffie_hellman(recipient_identity).as_bytes());
    hasher.update(ephemeral_identity.as_bytes());
    hasher.update(ZornIdentity::from(sender_secret).as_bytes());
    hasher.update(recipient_identity.as_bytes());

    SharedSecret(hasher.finalize().into())
}

pub fn sender_exchange(sender_secret: &ZornIdentitySecret, recipient_identity: &ZornIdentity) -> (PublicKey, SharedSecret) {
    let (ephemeral_secret, ephemeral_identity) = generate_ephemeral_identity();
    (ephemeral_identity, compute_sender_shared_secret(sender_secret, ephemeral_secret, &ephemeral_identity, recipient_identity))
}

pub fn recipient_exchange(recipient_secret: &ZornIdentitySecret, sender_identity: &ZornIdentity, ephemeral_identity: &PublicKey) -> SharedSecret {
    let mut hasher = blake3::Hasher::new_derive_key(KEY_EXCHANGE_CONTEXT);

    hasher.update(recipient_secret.diffie_hellman(sender_identity).as_bytes());
    hasher.update(recipient_secret.diffie_hellman(ephemeral_identity).as_bytes());
    hasher.update(ephemeral_identity.as_bytes());
    hasher.update(sender_identity.as_bytes());
    hasher.update(ZornIdentity::from(recipient_secret).as_bytes());

    SharedSecret(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use crate::identity::{ZornIdentity, ZornIdentitySecret};
    use rand_core::{RngCore, CryptoRng, impls, OsRng};
    use x25519_dalek::{EphemeralSecret, PublicKey};

    use super::{compute_sender_shared_secret, sender_exchange, recipient_exchange};

    struct DummyRng(u64);
    impl RngCore for DummyRng {
        fn next_u64(&mut self) -> u64 {
            self.0
        }

        fn next_u32(&mut self) -> u32 {
            self.0 as u32
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            impls::fill_bytes_via_next(self, dest)
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
            Ok(self.fill_bytes(dest))
        }
    }
    impl CryptoRng for DummyRng {}

    #[test]
    fn sender_secret_test_vector() {
        let sender_secret = ZornIdentitySecret::new(DummyRng(0));
        let ephemeral_secret = EphemeralSecret::new(DummyRng(0));
        let ephemeral_identity = PublicKey::from(&ephemeral_secret);
        let recipient_identity = ZornIdentity::from(&ZornIdentitySecret::new(DummyRng(0)));
        assert_eq!(
            hex_literal::hex!("66e20c24acbc3a8bb4d803c5bf17d8f9840a2f917cda8c5c7a5878494ddb6b93"),
            compute_sender_shared_secret(&sender_secret, ephemeral_secret, &ephemeral_identity, &recipient_identity).0);
    }

    #[test]
    fn sender_recipient_exchange() {
        let sender_secret = ZornIdentitySecret::new(OsRng);
        let recipient_secret = ZornIdentitySecret::new(OsRng);

        let (pk, sender_shared) = sender_exchange(&sender_secret, &ZornIdentity::from(&recipient_secret));
        let recipient_shared = recipient_exchange(&recipient_secret, &ZornIdentity::from(&sender_secret), &pk);
        assert_eq!(sender_shared.0, recipient_shared.0);
    }
}
