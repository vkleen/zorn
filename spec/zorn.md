# zorn

`zorn` is a file encryption format supporting single recipient, streaming
decryption  with strong sender authentication.

It is philosophically based on [age](age-encryption.org/v1) but not
interoperable by design. The goal of `zorn` is to allow a UNIX style pipeline
```
cat message.zorn | zorn --from <sender-identity> --decrypt | consumer
```
to proceed in constant memory while never outputting data to `consumer` which
is not cryptographically tied to `<sender-identity>`. Further, a pipeline
```
zorn --from <sender-identity> --decrypt message.zorn | consumer
```
should be able to proceed in constant memory and ensure that `message.zorn` has
not been truncated before outputting data to `consumer`.

## Conventions

ABNF syntax follows [RFC 5234][] and [RFC 7405][] and references the core rules
in [RFC 5234][], Appendix B.1.

The operator `||` denotes concatenation. A prefix`0x` followed by two
hexadecimal digits denotes an octet value in the 0-255 range.

The expression `LE64(n)` denotes the little endian encoding of an unsigned
integer `n` between 0 and 2^64 - 1 as 8 octets.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this
document are to be interpreted as described in [BCP 14][], [RFC 2119][],
[RFC 8174][] when, and only when, they appear in all capitals, as shown here.

## XChaCha20-BLAKE3

The `XChaCha20-BLAKE3` authenticated encryption scheme is obtained as the
Encrypt-then-MAC composition of the `XChaCha20` unauthenticated stream cipher
with the `BLAKE3-keyed_hash` MAC. Under the assumption that `XChaCha20` is
IND-CPA secure and `BLAKE3-keyed_hash` is collision resistant and a good PRF,
the results of [BH22][] imply that `XChaCha20-BLAKE3` will be CMT-4 secure.
Similarly, [GLR17][] implies that `XChaCha20-BLAKE3` is a committing AEAD
scheme.

### XChaCha20

`XChaCha20` is an unauthenticated stream cipher constructed as a nonce
extension of the `ChaCha20` stream cipher as defined in [RFC 7539][]. The
construction proceeds analogously to the construction of `XSalsa20` from
`Salsa20` in [Bernstein11][]. It is implemented in [libsodium][] and documented in
internet draft [draft-irtf-cfrg-xchacha][].

`XChaCha20` is assumed to implement an interface
```
XChaCha20-encrypt(key, nonce, plaintext)
XChaCha20-decrypt(key, nonce, ciphertext)
```
with inputs
  * `key`: the secret key, length MUST be 32 octets.
  * `nonce`: the public nonce, length MUST be 24 octets.
  * `plaintext`: the plaintext to be encrypted, length MUST be less than 274,877,906,880 octets.
  * `ciphertext`: the ciphertext to be decrypted, length MUST be less than 274,877,906,880 octets.

The output of `XChaCha20-encrypt` will be the `XChaCha20` ciphertext with the
same length as the plaintext or an error if the inputs do not meet the length
requirements.

The output of `XChaCha20-decrypt` will be the `XChaCha20` decryption of the
given ciphertext or an error if the inputs do not meet the length requirements.

### BLAKE3

`BLAKE3` is a cryptographic hash function specified in [blake3][]. It is
assumed to implement interfaces
```
BLAKE3-keyed_hash(key, input)
BLAKE3-derive_key(context, key_material)
```
with inputs
  * `key`: a secret key, length MUST be 32 octets
  * `input`: arbitrary input, length MUST be less than 2^64 - 1 octets
  * `context`: A hardcoded, globally unique and application-specific context string, length MUST be less than 2^64 -1 octets
  * `key_material`: Secret material for key derivation, length MUST be less than 2^64 - 1 octets

The output of both functions is assumed to have a length of 32 octets.

### Encrypt-then-MAC Composition

The authenticated encryption scheme `XChaCha20-BLAKE3` implements the interface
```
XChaCha20-BLAKE3-encrypt(key, nonce, AD, plaintext)
XChaCha20-BLAKE3-decrypt(key, nonce, AD, ciphertext)
```
with inputs
  * `key`: the secret key, length MUST be 32 octets
  * `nonce`: the public nonce, length MUST be 24 octets
  * `AD`: the public associated data, length MUST be less than 2^64 - 1 octets
  * `plaintext`: the plaintext to be encrypted, length MUST be less than
      274,877,906,880 octets.
  * `ciphertext`: the ciphertext and appended authentication tag to be
      authenticated and decrypted, length MUST be less than 274,877,906,880 + 32
      octets.

Additionally, the combined length of `AD` and `ciphertext` MUST NOT exceed 2^64 - 73 octets.

The output consists of the `XChaCha20` ciphertext concatenated with an
authentication tag or an error if the inputs do not meet the length
requirements. The output length will be the plaintext length plus an additional
32 octets for the authentication tag.

The encryption proceeds as follows.
```
encryption-key = BLAKE3-key_derive("zorn-encryption.org/v1 XChaCha20-BLAKE3 encryption key", key || nonce)
mac-key = BLAKE3-key_derive("zorn-encryption.org/v1 XChaCha20-BLAKE3 MAC key", key || nonce)

ciphertext = XChaCha20-encrypt(encryption-key, nonce, plaintext)
tag = BLAKE3-keyed_hash(mac-key,
  key || nonce || AD || ciphertext || LE64(AD.Length) || LE64(ciphertext.Length))

return ciphertext || tag
```

The decryption  proceeds as follows.
```
tag = ciphertext[ciphertext.Length-32..]
ciphertextNoTag = ciphertext[0..ciphertext.Length-32]

encryption-key = BLAKE3-key_derive("zorn-encryption.org/v1 XChaCha20-BLAKE3 encryption key", key || nonce)
mac-key = BLAKE3-key_derive("zorn-encryption.org/v1 XChaCha20-BLAKE3 MAC key", key || nonce)

computed-tag = BLAKE3-keyed_hash(mac-key,
  key || nonce || AD || ciphertext || LE64(AD.Length) || LE64(ciphertext.Length))

if not ConstantTimeEquals(tag, computedTag)
  return error
else
  return XChaCha20-decrypt(encryption-key, nonce, ciphertextNoTag)
```

## Encrypted file format

The `zorn` file format consists of a header containing an ephemeral identity
for establishing a shared secret, followed by a payload encrypted with the
shared secret.

### Identities

A sender or recipient is identified by an `X25519` public key. Such an identity
is generated as
```
secret-key = read(CSRNG, 32)
identity = X25519(secret-key, basepoint)
```
where the function `X25519` is specified in [RFC 7748][], Section 5, and
`basepoint` denotes the Curve25519 base point specified in [RFC 7748][],
Section 4.1.

An identity is encoded as Bech32 as specified in [BIP 0173][] with human
readable prefix `zornv1`.

Identities and associated secret keys MUST NOT be reused across different
versions of the zorn encryption format or shared with different encryption
formats. If a stable identity across multiple formats is desired, an
implementation MAY generate `secret-key` as 
```
secret-key = BLAKE3-derive_key(application context, stable identity secret)
```
with a hardcoded, globally unique, implementation-specific `application
context` which MUST include the targeted `zorn` version.

### Header

The header consists of a version line followed by 32 octects `ephemeral identity`.

#### Version Line

The version line always starts with `zorn-encryption.org/`, is followed by an
arbitrary version string, and ends with a line feed `0x0a`; in ABNF:

```
version-line = %s"zorn-encryption.org/" version LF
version = 1*VCHAR
```

This document specifies the `v1` format with version line
```
v1-version-line = %s"zorn-encryption.org/v1" LF
```
Future version may change anything following the version line.

#### Shared Secret

The sender chooses an ephemeral identity and computes a shared secret with
recipient as follows. The `ephemeral secret` MUST be drawn from high entropy
source and MUST NOT be reused across files.

```
ephemeral secret = read(CSRNG, 32)
ephemeral identity = X25519(ephemeral secret, basepoint)

dh1 = X25519(sender secret, recipient identity)
dh2 = X25519(ephemeral secret, recipient identity)
shared secret = BLAKE3-derive_key("zorn-encryption.org/v1 shared secret",
  dh1 || dh2 || ephemeral identity || sender identity || recipient identity)
```

The recipient, possessing the secret key `recipient secret` corresponding to
its recipient identity, computes `shared secret` after receiving `ephemeral
identity` as follows.

```
dh1 = X25519(recipient secret, sender identity)
dh2 = X25519(recipient secret, ephemeral identity)
shared secret = BLAKE3-derive_key("zorn-encryption/v1 shared secret",
  dh1 || dh2 || ephemeral identity || sender identity || recipient identity)
```

If the received file is too short to extract 32 octects of `ephemeral identity`
after the version line, the recipient MUST abort before attempting any
cryptographic operations.

### Payload

The payload starts immediately following the header. The plaintext is split
into `N` chunks of 2^16 octets. The last chunk MAY contain less than 2^16
octets but MUST NOT be empty unless the entire plaintext is empty. The number
`N` of chunks MUST NOT exceed 2^64.

Each plaintext chunk `P_n` except the last one with 0-based index `n` is
encrypted using `XChaCha20-BLAKE3` as follows.
```
C_n = XChaCha20-BLAKE3-encrypt(
  key = shared secret,
  nonce = LE64(0) || LE64(0) || LE64(n),
  AD = ephemeral identity || sender identity || receiver identity,
  plaintext = P_n)
```
The last plaintext chunk `P_(N-1)` is encrypted using:
```
C_(N-1) = XChaCha20-BLAKE3-encrypt(
  key = shared secret,
  nonce = LE64(1) || LE64(0) || LE64(N-1),
  AD = ephemeral identity || sender identity || receiver identity,
  plaintext = P_(N-1))
```
The payload consists of the concatenation of all ciphertext chunks:
```
payload = C_0 || ... || C_(N-1)
```

For decryption the recipient splits the received `payload` into `N` chunks
`C_n` of 2^16 + 32 octets with 0-based index `n`. The last chunk MAY be shorter
than 2^16 + 32 octets but MUST NOT be shorter than 32 octects. Decryption then
proceeds as
```
P_n = XChaCha20-BLAKE3-decrypt(
  key = shared secret,
  nonce = LE64(0) || LE64(0) || LE64(n),
  AD = ephemeral identity || sender identity || receiver identity,
  ciphertext = C_n)

P_(N-1) = XChaCha20-BLAKE3-decrypt(
  key = shared secret,
  nonce = LE64(1) || LE64(0) || LE64(N-1),
  AD = ephemeral identity || sender identity || receiver identity,
  ciphertext = C_(N-1))
```

If the receiver is presented with a seekable payload, for instance as an
encrypted file, she MUST verify that all ciphertext chunks decrypt correctly
before outputting any data.


[BIP 0173]: https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki
[BCP 14]: https://www.rfc-editor.org/info/bcp14
[RFC 2119]: https://www.rfc-editor.org/rfc/rfc2119.html
[RFC 8174]: https://www.rfc-editor.org/rfc/rfc8174.html
[RFC 2104]: https://www.rfc-editor.org/rfc/rfc2104.html
[RFC 5234]: https://www.rfc-editor.org/rfc/rfc5234.html
[RFC 7405]: https://www.rfc-editor.org/rfc/rfc7405.html
[RFC 4648]: https://www.rfc-editor.org/rfc/rfc4648.html
[RFC 7468]: https://www.rfc-editor.org/rfc/rfc7468.html
[RFC 7748]: https://www.rfc-editor.org/rfc/rfc7748.html
[RFC 7539]: https://www.rfc-editor.org/rfc/rfc7539.html
[BH22]: https://eprint.iacr.org/2022/268
[GLR17]: https://eprint.iacr.org/2017/664
[Bernstein11]: https://cr.yp.to/snuffle/xsalsa-20110204.pdf
[draft-irtf-cfrg-xchacha]: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha
[blake3]: https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf
[libsodium]: https://doc.libsodium.org
