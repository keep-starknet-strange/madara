# %% Imports
import logging
from asyncio import run
from math import ceil, log

from utils.constants import CHAIN_ID, RPC_CLIENT
from utils.starknet import (
    declare,
    deploy,
    dump_declarations,
    dump_deployments,
    get_declarations,
    get_starknet_account,
)

logging.basicConfig()
logger = logging.getLogger(__name__)
logger.setLevel(logging.INFO)


# %% Main
async def main():
    # %% Declarations
    logger.info(
        f"ℹ️  Connected to CHAIN_ID {CHAIN_ID.value.to_bytes(ceil(log(CHAIN_ID.value, 256)), 'big')} "
        f"with RPC {RPC_CLIENT.url}"
    )
    account = await get_starknet_account()
    logger.info(f"ℹ️  Using account {hex(account.address)} as deployer")

    class_hash = {"erc20": await declare("erc20")}
    dump_declarations(class_hash)

    # %% Deployments
    class_hash = get_declarations()

    deployments = {}
    deployments["madara_token"] = await deploy(
        "erc20",
        "Madara Token",  # name
        "MDT",  # symbol
        18,  # decimals
        int(1e10),  # initial_supply: Uint256,
        account.address,  # recipient
    )
    dump_deployments(deployments)


# %% Run
if __name__ == "__main__":
    run(main())
