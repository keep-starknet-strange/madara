import json
import subprocess

from utils.constants import BUILD_DIR, CONTRACTS, SOURCE_DIR


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
