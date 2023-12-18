//! Traits for Starknet OS program hash.
#![cfg_attr(not(feature = "std"), no_std)]

use mp_felt::Felt252Wrapper;

/// ProgramHash for Starknet OS Cairo program
///
/// How to calculate:
///     1. Get Starknet OS program sources (e.g. check keep-starknet-strange/snos)
///     2. Install Cairo-lang and run `cairo-hash-program --program <cairo-output>.json`
///     3. Install Starkli and run `starkli mont <program hash>`
///
/// Hex value: 0x41fc2a467ef8649580631912517edcab7674173f1dbfa2e9b64fbcd82bc4d79
pub const SN_OS_PROGRAM_HASH: Felt252Wrapper = Felt252Wrapper(starknet_ff::FieldElement::from_mont([
    6431315658044554931,
    6518314672963632076,
    7993178465604693533,
    95212460539797968,
]));
