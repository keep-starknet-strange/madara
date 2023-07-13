import { startMadaraDevNode } from "../util/dev-node";
import { type ChildProcess } from "child_process";
import process from "process";

let madaraProcess: ChildProcess;

exports.mochaGlobalSetup = async function () {
  const { p2pPort, rpcPort, runningNode } = await startMadaraDevNode();

  madaraProcess = runningNode;
  process.env.P2P_PORT = `${p2pPort}`;
  process.env.RPC_PORT = `${rpcPort}`;
};

exports.mochaGlobalTeardown = async function () {
  // end madara server
  await new Promise((resolve) => {
    madaraProcess.once("exit", resolve);
    madaraProcess.kill();
    madaraProcess = null;
  });
};
