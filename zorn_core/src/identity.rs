use std::ops::{Deref, DerefMut};
use rand_core::{RngCore, CryptoRng};
use zeroize::Zeroize;

use x25519_dalek::{PublicKey, StaticSecret};

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

impl Deref for ZornIdentitySecret {
    type Target = StaticSecret;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl From<&ZornIdentitySecret> for ZornIdentity {
    fn from(secret: &ZornIdentitySecret) -> Self {
        ZornIdentity(PublicKey::from(&secret.0))
    }
}

impl From<[u8;32]> for ZornIdentitySecret {
    fn from(bytes: [u8;32]) -> Self {
        ZornIdentitySecret(StaticSecret::from(bytes))
    }
}

impl ZornIdentitySecret {
    pub fn new<T: RngCore + CryptoRng>(csprng: T) -> ZornIdentitySecret {
        ZornIdentitySecret(StaticSecret::new(csprng))
    }
}
