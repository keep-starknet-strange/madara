# Genesis

The genesis of the chain can be found in the [node]
(<https://github.com/keep-starknet-strange/madara/tree/main/crates/node/src/genesis_assets>)
crate. The genesis is defined in the form of a JSON file containing the
following:

- contract_classes: list of tuples containing the class hash and the class. The
  class can be provided in two formats:
  - An object containing a field "path" with the path to the compiled class from
    the root of the repository and a field "version" to indicate which cairo
    version this class belongs to (0 or 1). Example:
    `{ "path": "./cairo-contracts/build/NoValidateAccount.json", "version": 0 }`
  - The whole serialized class
- contracts: list of tuples containing the contract address and the associated
  class hash.
- storage: list of tuples containing the storage key and the storage value.
  Please note that the storage key is itself a tuple, containing the contract
  address for which storage is set and the
  [Starknet storage key](https://docs.starknet.io/documentation/architecture_and_concepts/Contracts/contract-storage/#storage_variables).

The below defines all hardcoded values set in the geneses:

## Node genesis [link](https://github.com/keep-starknet-strange/madara/tree/main/crates/node/src/genesis_assets/genesis.json)

### Contract classes node genesis

<!-- markdownlint-disable MD013 -->

| Class hash                                                         | Definition                                     |
| :----------------------------------------------------------------- | :--------------------------------------------- |
| 0x0279d77db761fba82e0054125a6fdb5f6baa6286fa3fb73450cc44d193c2d37f | No validation account class hash               |
| 0x35ccefcf9d5656da623468e27e682271cd327af196785df99e7fee1436b6276  | No validation account class hash cairo 1       |
| 0x06f0d6f6ae72e1a507ff4b65181291642889742dbf8f1a53e9ec1c595d01ba7d | Argent account class hash                      |
| 0x0424b7f61e3c5dfd74400d96fdea7e1f0bf2757f31df04387eaa957f095dd7b9 | Proxy class hash                               |
| 0x2c2b8f559e1221468140ad7b2352b1a5be32660d0bf1a3ae3a054a4ec5254e4  | Braavos account class hash                     |
| 0x5aa23d5bb71ddaa783da7ea79d405315bafa7cf0387a74f4593578c3e9e6570  | Braavos account base implementation class hash |
| 0x07db5c2c2676c2a5bfc892ee4f596b49514e3056a0eee8ad125870b4fb1dd909 | Braavos account call aggregator class hash     |
| 0x3131fa018d520a037686ce3efddeab8f28895662f019ca3ca18a626650f7d1e  | Proxy class hash                               |
| 0x006280083f8c2a2db9f737320d5e3029b380e0e820fe24b8d312a6a34fdba0cd | Openzeppelin account class hash                |
| 0x1000                                                             | Test contract class hash                       |
| 0x10000                                                            | ERC20 class hash                               |
| 0x20000                                                            | ERC20 class hash                               |
| 0x80000                                                            | ERC721 class hash                              |
| 0x90000                                                            | Universal deployer class hash                  |

<!-- markdownlint-disable MD013 -->

### Contracts node genesis

<!-- markdownlint-disable MD013 -->

| Contract address                                                   | Class hash                                                         |
| :----------------------------------------------------------------- | :----------------------------------------------------------------- |
| 0x1                                                                | 0x0279d77db761fba82e0054125a6fdb5f6baa6286fa3fb73450cc44d193c2d37f |
| 0x2                                                                | 0x06f0d6f6ae72e1a507ff4b65181291642889742dbf8f1a53e9ec1c595d01ba7d |
| 0x3                                                                | 0x006280083f8c2a2db9f737320d5e3029b380e0e820fe24b8d312a6a34fdba0cd |
| 0x4                                                                | 0x35ccefcf9d5656da623468e27e682271cd327af196785df99e7fee1436b6276  |
| 0x1111                                                             | 0x1000                                                             |
| 0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00 | 0x10000                                                            |
| 0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d02 | 0x80000                                                            |
| 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7  | 0x20000                                                            |
| 0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf | 0x90000                                                            |

<!-- markdownlint-disable MD013 -->

### Storage node genesis

The node storage is prefilled using the genesis in order to allow access to
prefunded accounts. Available accounts with unlimited funds are 0x1, 0x2, 0x3
and 0x4 (hence why the storage value we write is
0xffffffffffffffffffffffffffffffff for U256 low and U256 high).

Additionally, a public key
(0x3603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2) is set for
accounts 0x2 and 0x3, for which the signature will be checked during the
validation phases of the execution. Accounts 0x1 and 0x4 include an empty
validation phases, meaning no signature is required to execute any transactions
going through them.

Finally, 0x1 is set as the contract owner of contract
0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d02, which is
deployed as a ERC721 contract (given the class hash of 0x80000).

<!-- markdownlint-disable MD013 -->

| Contract address                                                   | Storage key                                                                                  | Storage value                                                                         |
| :----------------------------------------------------------------- | :------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7  | 0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09 (ERC20_balances(0x1).low)  | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7  | 0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f0a (ERC20_balances(0x1).high) | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7  | 0x1d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f2146f (ERC20_balances(0x2).low)  | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7  | 0x1d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f21470 (ERC20_balances(0x2).high) | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7  | 0x262e096a838c0d8f34f641ff917d47d7dcb345c69efe61d9ab6b675e7340fc6 (ERC20_balances(0x3).low)  | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7  | 0x262e096a838c0d8f34f641ff917d47d7dcb345c69efe61d9ab6b675e7340fc7 (ERC20_balances(0x3).high) | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7  | 0x45abe05a3e7fb0c2ae1fa912be22a7dbc4832915e00562e2783dee710b9e4bc (ERC20_balances(0x4).low)  | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7  | 0x45abe05a3e7fb0c2ae1fa912be22a7dbc4832915e00562e2783dee710b9e4bd (ERC20_balances(0x4).high) | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00 | 0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09 (ERC20_balances(0x1).low)  | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00 | 0x7b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f0a (ERC20_balances(0x1).high) | 0xffffffffffffffffffffffffffffffff (U128::MAX)                                        |
| 0x2                                                                | 0x1ccc09c8a19948e048de7add6929589945e25f22059c7345aaf7837188d8d05 (\_signer)                 | 0x3603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2 (Signer public key) |
| 0x3                                                                | 0x1379ac0624b939ceb9dede92211d7db5ee174fe28be72245b0a1a2abd81c98f (Account_public_key)       | 0x3603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2 (Signer public key) |
| 0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d02 | 0x2bd557f4ba80dfabefabe45e9b2dd35db1b9a78e96c72bc2b69b655ce47a930 (Ownable_owner)            | 0x1 (Owner)                                                                           |

<!-- markdownlint-disable MD013 -->

## Mock genesis [link](https://github.com/keep-starknet-strange/madara/tree/main/crates/pallets/starknet/src/tests/mock/genesis.json)

### Contract classes mock genesis

<!-- markdownlint-disable MD013 -->

| Class hash                                                         | Definition                                 |
| :----------------------------------------------------------------- | :----------------------------------------- |
| 0x03bcec8de953ba8e305e2ce2db52c91504aefa7c56c91211873b4d6ba36e8c32 | No validation account class hash           |
| 0x0279d77db761fba82e0054125a6fdb5f6baa6286fa3fb73450cc44d193c2d37f | No validation account class hash           |
| 0x35ccefcf9d5656da623468e27e682271cd327af196785df99e7fee1436b6276  | No validation account class hash cairo 1   |
| 0x071aaf68d30c3e52e1c4b7d1209b0e09525939c31bb0275919dffd4cd53f57c4 | Unauthorized inner call account class hash |
| 0x06f0d6f6ae72e1a507ff4b65181291642889742dbf8f1a53e9ec1c595d01ba7d | Argent account class hash                  |
| 0x06a89ae7bd72c96202c040341c1ee422474b562e1d73c6848f08cae429c33262 | Proxy class hash                           |
| 0x0244ca3d9fe8b47dd565a6f4270d979ba31a7d6ff2c3bf8776198161505e8b52 | Braavos account class hash                 |
| 0x006280083f8c2a2db9f737320d5e3029b380e0e820fe24b8d312a6a34fdba0cd | Openzeppelin account class hash            |
| 0x00000000000000000000000000000000000000000000000000000000DEADBEEF | Test contract class hash                   |
| 0x01cb5d0b5b5146e1aab92eb9fc9883a32a33a604858bb0275ac0ee65d885bba8 | L1 handler class hash                      |
| 0x06232eeb9ecb5de85fc927599f144913bfee6ac413f2482668c9f03ce4d07922 | ERC20 class hash                           |
| 0x91000                                                            | Single event emitter class hash            |
| 0x92000                                                            | Multiple event emitter class hash          |

<!-- markdownlint-disable MD013 -->

### Contracts mock genesis

<!-- markdownlint-disable MD013 -->

| Contract address                                                   | Class hash                                                         |
| :----------------------------------------------------------------- | :----------------------------------------------------------------- |
| 0x1                                                                | 0x01cb5d0b5b5146e1aab92eb9fc9883a32a33a604858bb0275ac0ee65d885bba8 |
| 0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7 | 0x00000000000000000000000000000000000000000000000000000000DEADBEEF |
| 0x02356b628d108863baf8644c945d97bad70190af5957031f4852d00d0f690a77 | 0x03bcec8de953ba8e305e2ce2db52c91504aefa7c56c91211873b4d6ba36e8c32 |
| 0x06e2616a2dceff4355997369246c25a78e95093df7a49e5ca6a06ce1544ffd50 | 0x006280083f8c2a2db9f737320d5e3029b380e0e820fe24b8d312a6a34fdba0cd |
| 0x02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5 | 0x06f0d6f6ae72e1a507ff4b65181291642889742dbf8f1a53e9ec1c595d01ba7d |
| 0x05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122 | 0x0244ca3d9fe8b47dd565a6f4270d979ba31a7d6ff2c3bf8776198161505e8b52 |
| 0x04e7b41e2d628e6ab91d6c805bd22fbdb186d4e581266640663bd0094b3ef98b | 0x06a89ae7bd72c96202c040341c1ee422474b562e1d73c6848f08cae429c33262 |
| 0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0 | 0x0279d77db761fba82e0054125a6fdb5f6baa6286fa3fb73450cc44d193c2d37f |
| 0x0642a8b9e2c6cc3a9ddb84575123f262a21415f78db453b0625d889e1e06ac32 | 0x35ccefcf9d5656da623468e27e682271cd327af196785df99e7fee1436b6276  |
| 0x0764d66462958b670b4dbd46e00eb3d60100f329dc0365d9b059e0549a4c6f58 | 0x071aaf68d30c3e52e1c4b7d1209b0e09525939c31bb0275919dffd4cd53f57c4 |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x06232eeb9ecb5de85fc927599f144913bfee6ac413f2482668c9f03ce4d07922 |
| 0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02cf | 0x91000                                                            |
| 0x051a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02cf | 0x92000                                                            |

<!-- markdownlint-disable MD013 -->

### Storage mock genesis

<!-- markdownlint-disable MD013 -->

| Contract address                                                   | Storage key                                                                                                                                                 | Storage value                                                                         |
| :----------------------------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x3701645da930cd7f63318f7f118a9134e72d64ab73c72ece81cae2bd5fb403f (ERC20_balances(0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0).low)  | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x3701645da930cd7f63318f7f118a9134e72d64ab73c72ece81cae2bd5fb4040 (ERC20_balances(0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0).high) | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x25aa869465e1c3ac7ed6e933ef1af43f4d9126339b8f453f692d631c4a40d24 (ERC20_balances(0x0642a8b9e2c6cc3a9ddb84575123f262a21415f78db453b0625d889e1e06ac32).low)  | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x25aa869465e1c3ac7ed6e933ef1af43f4d9126339b8f453f692d631c4a40d25 (ERC20_balances(0x0642a8b9e2c6cc3a9ddb84575123f262a21415f78db453b0625d889e1e06ac32).high) | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x6afaa15cba5e9ea552a55fec494d2d859b4b73506794bf5afbb3d73c1fb00aa (ERC20_balances(0x02356b628d108863baf8644c945d97bad70190af5957031f4852d00d0f690a77).low)  | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x6afaa15cba5e9ea552a55fec494d2d859b4b73506794bf5afbb3d73c1fb00ab (ERC20_balances(0x02356b628d108863baf8644c945d97bad70190af5957031f4852d00d0f690a77).high) | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x2231fbd06f0277a2cbcd41f94a6d6cf930a586168e7faa4d62281f554934236 (ERC20_balances(0x06e2616a2dceff4355997369246c25a78e95093df7a49e5ca6a06ce1544ffd50).low)  | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x2231fbd06f0277a2cbcd41f94a6d6cf930a586168e7faa4d62281f554934237 (ERC20_balances(0x06e2616a2dceff4355997369246c25a78e95093df7a49e5ca6a06ce1544ffd50).high) | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x60b6ac06a42730e54bfd5d389ca51256c926bc9317adb44f7c1029711f8bf8e (ERC20_balances(0x02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5).high) | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x60b6ac06a42730e54bfd5d389ca51256c926bc9317adb44f7c1029711f8bf8f (ERC20_balances(0x02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5).high) | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x78f9a7bb317327b7ad49232784f8e6acfa88269879253bbf780c5bc7a18149a (ERC20_balances(0x05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122).high) | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x00000000000000000000000000000000000000000000000000000000000000AA | 0x78f9a7bb317327b7ad49232784f8e6acfa88269879253bbf780c5bc7a18149b (ERC20_balances(0x05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122).high) | 0xffffffffffffffffffffffffffffffff                                                    |
| 0x06e2616a2dceff4355997369246c25a78e95093df7a49e5ca6a06ce1544ffd50 | 0x1379ac0624b939ceb9dede92211d7db5ee174fe28be72245b0a1a2abd81c98f (Account_public_key)                                                                      | 0x3603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2 (Signer public key) |
| 0x02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5 | 0x1ccc09c8a19948e048de7add6929589945e25f22059c7345aaf7837188d8d05 (\_signer)                                                                                | 0x3603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2 (Signer public key) |
| 0x05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122 | 0x1f23302c120008f28b62f70efc67ccd75cfe0b9631d77df231d78b0538dcd8f (Account_signers(0x0))                                                                    | 0x3603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2 (Signer public key) |
| 0x051a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02cf | 0x238cf5ef6d6264a50d29a47fdf07ec9b7a8e9873214fa58179c5bb40933fdcb (external_contract_addr)                                                                  | 0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02cf (Target)           |

<!-- markdownlint-disable MD013 -->
