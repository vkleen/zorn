use std::ops::{Deref, DerefMut};
use rand_core::{RngCore, CryptoRng};
use zeroize::Zeroize;

use x25519_dalek::{PublicKey, StaticSecret, SharedSecret};

//const ZORN_SECRET_APPLICATION_CONTEXT: &str = "zorn-encryption.org/cli 2022-08-28T15:31:50+00:00 ZornIdentitySecret key derivation";

#[derive(Debug)]
pub struct ZornIdentity(pub(crate) PublicKey);

impl Deref for ZornIdentity {
    type Target = PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZornIdentity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct ZornIdentitySecret(StaticSecret);

impl From<&ZornIdentitySecret> for ZornIdentity {
    fn from(secret: &ZornIdentitySecret) -> Self {
        ZornIdentity(PublicKey::from(&secret.0))
    }
}

impl ZornIdentitySecret {
    pub fn new<T: RngCore + CryptoRng>(csprng: T) -> ZornIdentitySecret {
        ZornIdentitySecret(StaticSecret::new(csprng))
    }

    pub fn diffie_hellman(&self, their_id: &PublicKey) -> SharedSecret {
        self.0.diffie_hellman(their_id)
    }
}
