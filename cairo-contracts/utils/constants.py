import json
import logging
import os
from enum import Enum
from pathlib import Path

import requests
from dotenv import load_dotenv
from starknet_py.net.full_node_client import FullNodeClient

load_dotenv()

logging.basicConfig()
logger = logging.getLogger(__name__)
logger.setLevel(logging.INFO)


RPC_CLIENT = FullNodeClient(node_url=os.getenv("RPC_URL", "http://127.0.0.1:5050/rpc"))
BLOCK_EXPLORER_URL = "https://starknet-madara.netlify.app/#/explorer/query"


BUILD_DIR = Path("build")
BUILD_DIR.mkdir(exist_ok=True, parents=True)
SOURCE_DIR = Path("src")
CONTRACTS = {p.stem: p for p in list(SOURCE_DIR.glob("**/*.cairo"))}

ACCOUNT_ADDRESS = os.environ["ACCOUNT_ADDRESS"]
PRIVATE_KEY = os.environ["PRIVATE_KEY"]

DEPLOYMENTS_DIR = Path("deployments")
DEPLOYMENTS_DIR.mkdir(exist_ok=True, parents=True)


# TODO: Remove enum when starknet-py doesn't expect an enum as chain_id
try:
    response = requests.post(
        RPC_CLIENT.url,
        json={
            "jsonrpc": "2.0",
            "method": f"starknet_chainId",
            "params": {},
            "id": 0,
        },
    )

    class ChainId(Enum):
        chain_id = int(json.loads(response.text)["result"], 16)

    CHAIN_ID = getattr(ChainId, "chain_id")
except Exception:
    logger.info("⚠ Could not fetch CHAIN_ID from RPC")
    CHAIN_ID = ""
