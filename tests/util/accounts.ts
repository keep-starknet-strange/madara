import Keyring from "@polkadot/keyring";

const keyringEd25519 = new Keyring({ type: "ed25519" });

export const alice = keyringEd25519.addFromUri("//Alice");
