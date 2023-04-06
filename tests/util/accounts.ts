import Keyring from "@polkadot/keyring";

const keyringSr25519 = new Keyring({ type: "sr25519" });

export const alice = keyringSr25519.addFromUri("//Alice");
