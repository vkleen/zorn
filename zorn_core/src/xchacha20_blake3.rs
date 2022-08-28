use aead::{AeadCore, consts::{U24, U32, U0}, AeadMutInPlace, generic_array::GenericArray, Nonce, Error, KeyInit, KeySizeUser};
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

impl AeadMutInPlace for XChaCha20Blake3 {
    fn encrypt_in_place_detached(
        &mut self,
        nonce: &Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
    ) -> aead::Result<aead::Tag<Self>> {
        XChaCha20::new(&self.cipher_key, &nonce).apply_keystream(buffer);

        let mac = &mut self.mac;
        mac.update(nonce);
        mac.update(associated_data);
        mac.update(buffer);
        mac.update(&associated_data.len().to_le_bytes());
        mac.update(&buffer.len().to_le_bytes());

        Ok(GenericArray::clone_from_slice(mac.finalize().as_bytes()))
    }

    fn decrypt_in_place_detached(
        &mut self,
        nonce: &Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
        tag: &aead::Tag<Self>,
    ) -> aead::Result<()> {
        let mac = &mut self.mac;
        mac.update(nonce);
        mac.update(associated_data);
        mac.update(buffer);
        mac.update(&associated_data.len().to_le_bytes());
        mac.update(&buffer.len().to_le_bytes());

        // blake3::Hash implements a constant time Eq for comparisons with [u8; 32]
        if mac.finalize() == *tag.as_slice() {
            XChaCha20::new(&self.cipher_key, &nonce).apply_keystream(buffer);
            Ok(())
        } else {
            Err(Error)
        }
    }
}
