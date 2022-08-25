# zorn

`zorn` is a minimally featured file encryption format for public key authenticated encryption. There is a [draft specification](./spec/zorn.md) of the format.

## Key features
* Constant memory operation in pipelines `cat message.zorn | zorn decrypt --from <sender-identity> | consumer` without ever releasing unauthenticated data to `consumer`
* Cryptographic sender authentication while maintaining repudiability

## Caveats
* This is a new format and has not been formally audited by anyone with credentials.
* Message truncation  in a pipeline `cat message.zorn | zorn decrypt --from <sender-identity> | consumer` is not prevented and I conjecture that it is not preventable while keeping constant memory operation. This can be an issue, for example if `consumer` is `bash` and the `message.zorn` contains a script. In this case the script could be terminated prematurely, at least with a granularity of 64 kibibytes. Any truncation will however be detected by `zorn` after the fact.
* A `zorn` message will leak the length of the plaintext.
* Key compromise impersonation to the receiver is always possible. That is, if the receiver's private key is compromised the cryptographic proof of origin loses all meaning. This property is necessary for repudiability.
