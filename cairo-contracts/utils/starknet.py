import functools
import json
import logging
import subprocess
import time
from pathlib import Path

import requests
from starknet_py.common import create_compiled_contract
from starknet_py.contract import Contract
from starknet_py.hash.class_hash import compute_class_hash
from starknet_py.net.account.account import Account
from starknet_py.net.client_models import TransactionStatus
from starknet_py.net.signer.stark_curve_signer import KeyPair
from utils.constants import (
    ACCOUNT_ADDRESS,
    BLOCK_EXPLORER_URL,
    BUILD_DIR,
    CHAIN_ID,
    CONTRACTS,
    DEPLOYMENTS_DIR,
    PRIVATE_KEY,
    RPC_CLIENT,
    SOURCE_DIR,
)

logging.basicConfig()
logger = logging.getLogger(__name__)
logger.setLevel(logging.INFO)


async def get_starknet_account(
    address=None,
    private_key=None,
) -> Account:
    address = address or ACCOUNT_ADDRESS
    if address is None:
        raise ValueError("address was not given in arg nor in env variable")
    address = int(address, 16)
    private_key = private_key or PRIVATE_KEY
    if private_key is None:
        raise ValueError("private_key was not given in arg nor in env variable")
    key_pair = KeyPair.from_private_key(int(private_key, 16))
    return Account(
        address=address,
        client=RPC_CLIENT,
        chain=CHAIN_ID,
        key_pair=key_pair,
    )


async def get_contract(contract_name) -> Contract:
    return await Contract.from_address(
        get_deployments()[contract_name]["address"],
        await get_starknet_account(),
    )


def dump_declarations(declarations):
    json.dump(
        {name: hex(class_hash) for name, class_hash in declarations.items()},
        open(DEPLOYMENTS_DIR / "declarations.json", "w"),
        indent=2,
    )


def get_declarations():
    return {
        name: int(class_hash, 16)
        for name, class_hash in json.load(
            open(DEPLOYMENTS_DIR / "declarations.json")
        ).items()
    }


def dump_deployments(deployments):
    json.dump(
        {
            name: {
                **deployment,
                "address": hex(deployment["address"]),
                "tx": hex(deployment["tx"]),
                "artifact": str(deployment["artifact"]),
            }
            for name, deployment in deployments.items()
        },
        open(DEPLOYMENTS_DIR / "deployments.json", "w"),
        indent=2,
    )


def get_deployments():
    return json.load(open(DEPLOYMENTS_DIR / "deployments.json", "r"))


def get_artifact(contract_name):
    return BUILD_DIR / f"{contract_name}.json"


def get_tx_url(tx_hash: int) -> str:
    return f"{BLOCK_EXPLORER_URL}/tx/0x{tx_hash:064x}"


def compile_contract(contract_name: str):
    output = subprocess.run(
        [
            "starknet-compile-deprecated",
            CONTRACTS[contract_name],
            "--output",
            BUILD_DIR / f"{contract_name}.json",
            "--cairo_path",
            str(SOURCE_DIR),
            "--no_debug_info",
            *(["--account_contract"] if "account" in contract_name.lower() else []),
        ],
        capture_output=True,
    )
    if output.returncode != 0:
        raise RuntimeError(output.stderr)

    def _convert_offset_to_hex(obj):
        if isinstance(obj, list):
            for i in range(len(obj)):
                obj[i] = _convert_offset_to_hex(obj[i])
        elif isinstance(obj, dict):
            for key in obj:
                if obj.get(key) is not None:
                    obj[key] = _convert_offset_to_hex(obj[key])
        elif isinstance(obj, int) and obj >= 0:
            obj = hex(obj)
        return obj

    contract = json.loads((BUILD_DIR / f"{contract_name}.json").read_text())
    json.dump(
        {
            **contract,
            "entry_points_by_type": _convert_offset_to_hex(
                contract["entry_points_by_type"]
            ),
        },
        open(BUILD_DIR / f"{contract_name}.json", "w"),
        indent=2,
    )


def class_hash(contract_name: str):
    artifact = get_artifact(contract_name)
    compiled_contract = create_compiled_contract(Path(artifact).read_text())
    return compute_class_hash(compiled_contract)


async def declare(contract_name: str):
    logger.info(f"ℹ️  Declaring {contract_name}")
    account = await get_starknet_account()
    artifact = get_artifact(contract_name)
    declare_transaction = await account.sign_declare_transaction(
        compiled_contract=Path(artifact).read_text(), max_fee=int(1e17)
    )
    resp = await account.client.declare(transaction=declare_transaction)
    logger.info(f"⏳ Waiting for tx {get_tx_url(resp.transaction_hash)}")
    await wait_for_transaction(resp.transaction_hash)
    logger.info(f"✅ {contract_name} class hash: {hex(resp.class_hash)}")
    return resp.class_hash


async def deploy(contract_name, *args):
    logger.info(f"ℹ️  Deploying {contract_name}")
    abi = json.loads(Path(get_artifact(contract_name)).read_text())["abi"]
    account = await get_starknet_account()
    deploy_result = await Contract.deploy_contract(
        account=account,
        class_hash=get_declarations()[contract_name],
        abi=abi,
        constructor_args=list(args),
        max_fee=int(1e17),
    )
    logger.info(f"⏳ Waiting for tx {get_tx_url(deploy_result.hash)}")
    await wait_for_transaction(deploy_result.hash)
    logger.info(
        f"✅ {contract_name} deployed at: {hex(deploy_result.deployed_contract.address)}"
    )
    return {
        "address": deploy_result.deployed_contract.address,
        "tx": deploy_result.hash,
        "artifact": get_artifact(contract_name),
    }


async def invoke(contract_name, function_name, *inputs, address=None):
    account = await get_starknet_account()
    deployments = get_deployments()
    contract = Contract(
        deployments[contract_name]["address"] if address is None else address,
        json.load(open(get_artifact(contract_name)))["abi"],
        account,
    )
    call = contract.functions[function_name].prepare(*inputs, max_fee=int(1e17))
    logger.info(f"ℹ️  Invoking {contract_name}.{function_name}({json.dumps(inputs)})")
    response = await account.execute(call, max_fee=int(1e17))
    logger.info(f"⏳ Waiting for tx {get_tx_url(response.transaction_hash)}")
    await wait_for_transaction(response.transaction_hash)
    logger.info(
        f"✅ {contract_name}.{function_name} invoked at tx: %s",
        hex(response.transaction_hash),
    )
    return response.transaction_hash


async def call(contract_name, function_name, *inputs, address=None):
    deployments = get_deployments()
    account = await get_starknet_account()
    contract = Contract(
        deployments[contract_name]["address"] if address is None else address,
        json.load(open(get_artifact(contract_name)))["abi"],
        account,
    )
    return await contract.functions[function_name].call(*inputs)


# TODO: use RPC_CLIENT when RPC wait_for_tx is fixed, see https://github.com/kkrt-labs/kakarot/issues/586
@functools.wraps(RPC_CLIENT.wait_for_tx)
async def wait_for_transaction(*args, **kwargs):
    check_interval = kwargs.get("check_interval", 3)
    transaction_hash = args[0] if args else kwargs["tx_hash"]
    status = TransactionStatus.NOT_RECEIVED
    while status not in [TransactionStatus.ACCEPTED_ON_L2, TransactionStatus.REJECTED]:
        logger.info(f"ℹ️  Sleeping for {check_interval}s")
        time.sleep(check_interval)
        response = requests.post(
            RPC_CLIENT.url,
            json={
                "jsonrpc": "2.0",
                "method": f"starknet_getTransactionReceipt",
                "params": {"transaction_hash": hex(transaction_hash)},
                "id": 0,
            },
        )
        status = json.loads(response.text).get("result", {}).get("status")
        if status is not None:
            status = TransactionStatus(status)
            logger.info(f"ℹ️  Current status: {status.value}")
