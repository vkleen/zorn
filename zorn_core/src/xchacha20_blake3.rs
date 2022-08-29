use aead::{AeadCore, consts::{U24, U32, U0}, AeadInPlace, generic_array::GenericArray, Nonce, Error, KeyInit, KeySizeUser};
use blake3::Hasher;
use chacha20::XChaCha20;
use cipher::{Key, KeyIvInit, StreamCipher};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone)]
pub struct XChaCha20Blake3 {
    cipher_key: Key<XChaCha20>,
    mac: blake3::Hasher,
}

impl Zeroize for XChaCha20Blake3 {
    fn zeroize(&mut self) {
        self.cipher_key.zeroize();
    }
}

impl Drop for XChaCha20Blake3 {
    fn drop(&mut self) {
        self.zeroize()
    }
}

const CIPHER_KEY_CONTEXT: &str = "zorn-encryption.org/v1 XChaCha20-BLAKE3 encryption key";
const MAC_KEY_CONTEXT: &str = "zorn-encryption.org/v1 XChaCha20-BLAKE3 MAC key";

impl AeadCore for XChaCha20Blake3 {
    type NonceSize = U24;
    type TagSize = U32;
    type CiphertextOverhead = U0;
}

impl XChaCha20Blake3 {
    fn derive_cipher_key(key: &[u8]) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(blake3::derive_key(CIPHER_KEY_CONTEXT, key))
    }

    fn derive_mac_key(key: &[u8]) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(blake3::derive_key(MAC_KEY_CONTEXT, key))
    }
}

impl KeySizeUser for XChaCha20Blake3 {
    type KeySize = U32;
}

impl KeyInit for XChaCha20Blake3 {
    fn new(key: &Key<XChaCha20Blake3>) -> Self {
        let mut h = Hasher::new_keyed(&*Self::derive_mac_key(key.as_slice()));
        h.update(key);
        XChaCha20Blake3 {
            cipher_key: GenericArray::clone_from_slice(&*Self::derive_cipher_key(key.as_slice())),
            mac: h,
        }
    }
}

impl AeadInPlace for XChaCha20Blake3 {
    fn encrypt_in_place_detached(
        &self,
        nonce: &Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
    ) -> aead::Result<aead::Tag<Self>> {
        XChaCha20::new(&self.cipher_key, &nonce).try_apply_keystream(buffer).map_err(|_| Error)?;

        let mut mac = self.mac.clone();
        mac.update(nonce);
        mac.update(associated_data);
        mac.update(buffer);
        mac.update(&associated_data.len().to_le_bytes());
        mac.update(&buffer.len().to_le_bytes());

        Ok(GenericArray::clone_from_slice(mac.finalize().as_bytes()))
    }

    fn decrypt_in_place_detached(
        &self,
        nonce: &Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
        tag: &aead::Tag<Self>,
    ) -> aead::Result<()> {
        let mut mac = self.mac.clone();
        mac.update(nonce);
        mac.update(associated_data);
        mac.update(buffer);
        mac.update(&associated_data.len().to_le_bytes());
        mac.update(&buffer.len().to_le_bytes());

        // blake3::Hash implements a constant time Eq for comparisons with [u8; 32]
        if mac.finalize() == *tag.as_slice() {
            XChaCha20::new(&self.cipher_key, &nonce).try_apply_keystream(buffer).map_err(|_| Error)
        } else {
            Err(Error)
        }
    }
}

#[cfg(test)]
mod test {
    use aead::{KeyInit, AeadInPlace, Nonce, generic_array::GenericArray};
    use std::assert_matches::assert_matches;

    use super::XChaCha20Blake3;

    use proptest::{proptest, prelude::any, collection::vec, prop_assume};

    proptest! {
        #[test]
        fn roundtrip(
                sk in any::<[u8; 32]>(),
                nonce in any::<[u8; 24]>(),
                msg in vec(any::<u8>(), 0..=512),
                ad in vec(any::<u8>(), 0..=512)) {
            let cipher = XChaCha20Blake3::new(&GenericArray::clone_from_slice(&sk));

            let mut encrypted_message = msg.clone();
            let tag = cipher.encrypt_in_place_detached(
                &Nonce::<XChaCha20Blake3>::from(nonce),
                ad.as_slice(),
                encrypted_message.as_mut_slice()).expect("Impossibru");

            assert_matches!(cipher.decrypt_in_place_detached(
                &Nonce::<XChaCha20Blake3>::from(nonce),
                ad.as_slice(),
                encrypted_message.as_mut_slice(),
                &tag
            ), Ok(()));
            assert_eq!(msg, encrypted_message);
        }

        #[test]
        fn checks_ad(
                sk in any::<[u8; 32]>(),
                nonce in any::<[u8; 24]>(),
                msg in vec(any::<u8>(), 0..=512),
                ad in vec(any::<u8>(), 0..=10),
                ad2 in vec(any::<u8>(), 0..=10)) {
            prop_assume!(ad != ad2);

            let cipher = XChaCha20Blake3::new(&GenericArray::clone_from_slice(&sk));

            let mut encrypted_message = msg.clone();
            let tag = cipher.encrypt_in_place_detached(
                &Nonce::<XChaCha20Blake3>::from(nonce),
                ad.as_slice(),
                encrypted_message.as_mut_slice()).expect("Impossibru");

            assert_matches!(cipher.decrypt_in_place_detached(
                &Nonce::<XChaCha20Blake3>::from(nonce),
                ad2.as_slice(),
                encrypted_message.as_mut_slice(),
                &tag
            ), Err(_));
        }

        #[test]
        fn checks_nonce(
                sk in any::<[u8; 32]>(),
                nonce in any::<[u8; 24]>(),
                nonce2 in any::<[u8; 24]>(),
                msg in vec(any::<u8>(), 0..=512),
                ad in vec(any::<u8>(), 0..=512)) {
            prop_assume!(nonce != nonce2);

            let cipher = XChaCha20Blake3::new(&GenericArray::clone_from_slice(&sk));

            let mut encrypted_message = msg.clone();
            let tag = cipher.encrypt_in_place_detached(
                &Nonce::<XChaCha20Blake3>::from(nonce),
                ad.as_slice(),
                encrypted_message.as_mut_slice()).expect("Impossibru");

            assert_matches!(cipher.decrypt_in_place_detached(
                &Nonce::<XChaCha20Blake3>::from(nonce2),
                ad.as_slice(),
                encrypted_message.as_mut_slice(),
                &tag
            ), Err(_));
        }

        #[test]
        fn checks_key(
                sk in any::<[u8; 32]>(),
                sk2 in any::<[u8; 32]>(),
                nonce in any::<[u8; 24]>(),
                msg in vec(any::<u8>(), 0..=512),
                ad in vec(any::<u8>(), 0..=512)) {
            prop_assume!(sk != sk2);

            let cipher = XChaCha20Blake3::new(&GenericArray::clone_from_slice(&sk));
            let cipher2 = XChaCha20Blake3::new(&GenericArray::clone_from_slice(&sk2));

            let mut encrypted_message = msg.clone();
            let tag = cipher.encrypt_in_place_detached(
                &Nonce::<XChaCha20Blake3>::from(nonce),
                ad.as_slice(),
                encrypted_message.as_mut_slice()).expect("Impossibru");

            assert_matches!(cipher2.decrypt_in_place_detached(
                &Nonce::<XChaCha20Blake3>::from(nonce),
                ad.as_slice(),
                encrypted_message.as_mut_slice(),
                &tag
            ), Err(_));
        }
    }
}
