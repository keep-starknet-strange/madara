import logging
from pathlib import Path

from dotenv import load_dotenv

load_dotenv()

logging.basicConfig()
logger = logging.getLogger(__name__)
logger.setLevel(logging.INFO)


BUILD_DIR = Path("build")
BUILD_DIR.mkdir(exist_ok=True, parents=True)
SOURCE_DIR = Path("src")
CONTRACTS = {p.stem: p for p in list(SOURCE_DIR.glob("**/*.cairo"))}

DEPLOYMENTS_DIR = Path("deployments")
DEPLOYMENTS_DIR.mkdir(exist_ok=True, parents=True)
