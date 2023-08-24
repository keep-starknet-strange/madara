import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import {
  LibraryError,
  RpcProvider,
  validateAndParseAddress,
  json,
  encode,
  CompressedProgram,
  LegacyContractClass,
} from "starknet";
import { ungzip } from "pako";
import { describeDevMadara } from "../../util/setup-dev-tests";
import {
  ACCOUNT_CONTRACT,
  ACCOUNT_CONTRACT_CLASS_HASH,
  ERC20_CONTRACT,
  TEST_CONTRACT,
  TEST_CONTRACT_ADDRESS,
  TEST_CONTRACT_CLASS_HASH,
  TOKEN_CLASS_HASH,
} from "../constants";

function atobUniversal(a: string): Uint8Array {
  return encode.IS_BROWSER
    ? stringToArrayBuffer(atob(a))
    : Buffer.from(a, "base64");
}
function stringToArrayBuffer(s: string): Uint8Array {
  return Uint8Array.from(s, (c) => c.charCodeAt(0));
}
function decompressProgram(base64: CompressedProgram) {
  if (Array.isArray(base64)) return base64;
  return encode.arrayBufferToString(ungzip(atobUniversal(base64)));
}

describeDevMadara("Starknet RPC - Contracts Test", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node
  });

  describe("call", async () => {
    it("should return calldata on return_result entrypoint", async function () {
      const call = await providerRPC.callContract(
        {
          contractAddress: TEST_CONTRACT_ADDRESS,
          entrypoint: "return_result",
          calldata: ["0x19"],
        },
        "latest",
      );

      expect(call.result).to.contain("0x19");
    });

    it("should raise with invalid entrypoint", async () => {
      const callResult = providerRPC.callContract(
        {
          contractAddress: TEST_CONTRACT_ADDRESS,
          entrypoint: "return_result_WRONG",
          calldata: ["0x19"],
        },
        "latest",
      );
      await expect(callResult)
        .to.eventually.be.rejectedWith("40: Contract error")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("getClassAt", async () => {
    it("should not be undefined", async function () {
      const contract_class = await providerRPC.getClassAt(
        TEST_CONTRACT_ADDRESS,
        "latest",
      );

      expect(contract_class).to.not.be.undefined;
      expect(contract_class.entry_points_by_type).to.deep.equal(
        TEST_CONTRACT.entry_points_by_type,
      );
    });
  });

  describe("getClassHashAt", async () => {
    it("should return correct class hashes for account and test contract", async function () {
      const account_contract_class_hash = await providerRPC.getClassHashAt(
        ACCOUNT_CONTRACT,
        "latest",
      );

      expect(account_contract_class_hash).to.not.be.undefined;
      expect(validateAndParseAddress(account_contract_class_hash)).to.be.equal(
        ACCOUNT_CONTRACT_CLASS_HASH,
      );

      const test_contract_class_hash = await providerRPC.getClassHashAt(
        TEST_CONTRACT_ADDRESS,
        "latest",
      );

      expect(test_contract_class_hash).to.not.be.undefined;
      expect(validateAndParseAddress(test_contract_class_hash)).to.be.equal(
        TEST_CONTRACT_CLASS_HASH,
      );
    });

    it("should raise with invalid block id", async () => {
      // Invalid block id
      const classHash = providerRPC.getClassHashAt(
        TEST_CONTRACT_ADDRESS,
        "0x123",
      );
      await expect(classHash)
        .to.eventually.be.rejectedWith("24: Block not found")
        .and.be.an.instanceOf(LibraryError);
    });

    it("should raise with invalid contract address", async () => {
      // Invalid/un-deployed contract address
      const classHash = providerRPC.getClassHashAt("0x123", "latest");
      await expect(classHash)
        .to.eventually.be.rejectedWith("20: Contract not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("getClass", async () => {
    it("should return ERC_20 contract at class 0x10000", async function () {
      const contract_class = (await providerRPC.getClass(
        TOKEN_CLASS_HASH,
        "latest",
      )) as LegacyContractClass;
      // https://github.com/keep-starknet-strange/madara/issues/652
      // TODO: Compare program as well
      expect(contract_class.entry_points_by_type).to.deep.equal(
        ERC20_CONTRACT.entry_points_by_type,
      );
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const program = json.parse(decompressProgram(contract_class.program));
      // starknet js parses the values in the identifiers as negative numbers (maybe it's in madara).
      // FIXME: https://github.com/keep-starknet-strange/madara/issues/664
      // expect(program).to.deep.equal(ERC20_CONTRACT.program);
    });
  });
});
