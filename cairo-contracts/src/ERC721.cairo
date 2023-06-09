// contracts/MyToken.cairo

%lang starknet

from openzeppelin.token.erc721.presets.ERC721MintableBurnable import (
    constructor,
    supportsInterface,
    name,
    symbol,
    balanceOf,
    ownerOf,
    getApproved,
    isApprovedForAll,
    tokenURI,
    owner,
    approve,
    setApprovalForAll,
    transferFrom,
    safeTransferFrom,
    mint,
    burn,
    setTokenURI,
    transferOwnership,
    renounceOwnership,
)
