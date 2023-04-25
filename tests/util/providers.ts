import { ApiPromise, WsProvider } from "@polkadot/api";

export const providePolkadotApi = async (port: number) => {
  return await ApiPromise.create({
    initWasm: false,
    provider: new WsProvider(`ws://localhost:${port}`),
    noInitWarn: true,
  });
};
