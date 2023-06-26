// contracts/MyToken.cairo

%lang starknet

from openzeppelin.token.erc20.presets.ERC20 import (
    constructor,
    name,
    symbol,
    totalSupply,
    decimals,
    balanceOf,
    allowance,
    transfer,
    transferFrom,
    approve,
    increaseAllowance,
    decreaseAllowance,
)
