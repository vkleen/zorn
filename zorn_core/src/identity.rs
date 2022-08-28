use std::ops::{Deref, DerefMut};
use bech32::{ToBase32, FromBase32};
use rand_core::{RngCore, CryptoRng};
use zeroize::Zeroize;
use thiserror::Error;

use x25519_dalek::{PublicKey, StaticSecret, SharedSecret};

#[cfg(test)]
use proptest::{arbitrary::Arbitrary, strategy::{BoxedStrategy, Strategy}};

//const ZORN_SECRET_APPLICATION_CONTEXT: &str = "zorn-encryption.org/cli 2022-08-28T15:31:50+00:00 ZornIdentitySecret key derivation";
const ZORN_IDENTITY_HRP: &str = "zornv1-";

#[derive(Debug, PartialEq, Eq)]
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

impl ZornIdentity {
    pub fn to_string(&self) -> String {
        bech32::encode(ZORN_IDENTITY_HRP, self.to_bytes().to_base32(), bech32::Variant::Bech32m)
            .expect("The HRP is valid")
    }
}

#[derive(Error, Debug)]
pub enum ZornIdentityDecodeError {
    #[error("incorrect byte length {0} for a public key")]
    IncorrectPubKeyLength(usize),
    #[error("string has an incorrect HRP for zornv1")]
    IncorrectHRP,
    #[error("string is Bech32 instead of Bech32m")]
    IncorrectBech32Variant,
    #[error(transparent)]
    InvalidBech32mEncoding(#[from] bech32::Error),
}

impl std::str::FromStr for ZornIdentity {
    type Err = ZornIdentityDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hrp, data32, variant) = bech32::decode(s)?;
        let data = match (hrp.as_str(), variant) {
            (ZORN_IDENTITY_HRP, bech32::Variant::Bech32m) => Vec::from_base32(&data32).map_err(ZornIdentityDecodeError::from),
            (ZORN_IDENTITY_HRP, _) => Err(ZornIdentityDecodeError::IncorrectBech32Variant),
            (&_, _) => Err(ZornIdentityDecodeError::IncorrectHRP),
        }?;
        TryInto::<[u8; 32]>::try_into(&data[..])
            .map_err(|_| ZornIdentityDecodeError::IncorrectPubKeyLength(data.len()))
            .map(|pk| ZornIdentity(PublicKey::from(pk)))
    }
}

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct ZornIdentitySecret(StaticSecret);

#[cfg(test)] opaque_debug::implement!(ZornIdentitySecret);
#[cfg(test)] impl Arbitrary for ZornIdentitySecret {
    type Parameters = <u8 as Arbitrary>::Parameters;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        <[u8; 32] as Arbitrary>::arbitrary_with(args).prop_map(|k| ZornIdentitySecret(StaticSecret::from(k))).boxed()
    }
}

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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hex_literal::hex;
    use x25519_dalek::StaticSecret;

    use crate::identity::{ZornIdentity, ZornIdentitySecret};

    use proptest::{proptest, prelude::any};

    const TEST_SK: [u8; 32] = hex!("00b575f5689a44612c4c8b4f6fb257623bd24b53838b10a50e84ef340bed057d");
    const TEST_ID: &str = "zornv1-1gjfs6r7x5fmydhgrz9cnwrdkdnnvt3w7zhwya6dwvrp528qjmd3s04fc4w";

    #[test]
    fn zorn_identity_bech32m_test_vector() {
        let sk = ZornIdentitySecret(StaticSecret::from(TEST_SK));
        assert_eq!(ZornIdentity::from(&sk).to_string(), TEST_ID);
        assert_eq!(ZornIdentity::from(&sk), ZornIdentity::from_str(TEST_ID).expect("TEST_ID is valid"));
    }

    proptest! {
        #[test]
        fn zorn_identity_bech32m_roundtrip(sk in any::<ZornIdentitySecret>()) {
            let id = ZornIdentity::from(&sk);
            assert_eq!(id, ZornIdentity::from_str(id.to_string().as_str()).expect("Encoding should be valid"));
        }
    }
}
