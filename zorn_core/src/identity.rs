use std::ops::Deref;
use zeroize::Zeroize;

use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Debug)]
pub struct ZornIdentity(pub(crate) PublicKey);

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct ZornIdentitySecret(StaticSecret);

impl Deref for ZornIdentity {
    type Target = PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<&'a ZornIdentitySecret> for ZornIdentity {
    fn from(secret: &'a ZornIdentitySecret) -> Self {
        ZornIdentity(PublicKey::from(&secret.0))
    }
}
