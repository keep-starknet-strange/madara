from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.bitwise import bitwise_and
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin
from starkware.cairo.common.cairo_secp.bigint import BASE, BigInt3, UnreducedBigInt3, bigint_mul
from starkware.cairo.common.cairo_secp.ec import EcPoint
from starkware.cairo.common.math import assert_nn, assert_nn_le, assert_not_zero, unsigned_div_rem
from starkware.cairo.common.math_cmp import RC_BOUND
from starkware.cairo.common.uint256 import Uint256

from src.accounts.braavos.lib.bigint import nondet_bigint3
from src.accounts.braavos.lib.constants import (
    N0,
    N1,
    N2,
    B0,
    B1,
    B2,
    A0,
    A1,
    A2,
    GX0,
    GX1,
    GX2,
    GY0,
    GY1,
    GY2,
)
from src.accounts.braavos.lib.ec import ec_add, ec_mul
from src.accounts.braavos.lib.ec_mulmuladd_secp256r1 import ec_mulmuladdW_bg3
from src.accounts.braavos.lib.field import unreduced_mul, unreduced_sqr, verify_zero

func get_generator_point() -> (point: EcPoint) {
    return (point=EcPoint(BigInt3(GX0, GX1, GX2), BigInt3(GY0, GY1, GY2)));
}

// Computes a * b^(-1) modulo the size of the elliptic curve (N).
//
// Prover assumptions:
// * All the limbs of a are in the range (-2 ** 210.99, 2 ** 210.99).
// * All the limbs of b are in the range (-2 ** 124.99, 2 ** 124.99).
// * b is in the range [0, 2 ** 256).
//
// Soundness assumptions:
// * The limbs of a are in the range (-2 ** 249, 2 ** 249).
// * The limbs of b are in the range (-2 ** 159.83, 2 ** 159.83).
func div_mod_n{range_check_ptr}(a: BigInt3, b: BigInt3) -> (res: BigInt3) {
    %{ from starkware.cairo.common.cairo_secp.secp256r1_utils import SECP256R1_N as N %}
    %{
        from starkware.cairo.common.cairo_secp.secp_utils import pack
        from starkware.python.math_utils import div_mod, safe_div

        a = pack(ids.a, PRIME)
        b = pack(ids.b, PRIME)
        value = res = div_mod(a, b, N)
    %}
    let (res) = nondet_bigint3();

    %{ value = k_plus_one = safe_div(res * b - a, N) + 1 %}
    let (k_plus_one) = nondet_bigint3();
    let k = BigInt3(d0=k_plus_one.d0 - 1, d1=k_plus_one.d1, d2=k_plus_one.d2);

    let (res_b) = bigint_mul(res, b);
    let n = BigInt3(N0, N1, N2);
    let (k_n) = bigint_mul(k, n);

    // We should now have res_b = k_n + a. Since the numbers are in unreduced form,
    // we should handle the carry.

    tempvar carry1 = (res_b.d0 - k_n.d0 - a.d0) / BASE;
    assert [range_check_ptr + 0] = carry1 + 2 ** 127;

    tempvar carry2 = (res_b.d1 - k_n.d1 - a.d1 + carry1) / BASE;
    assert [range_check_ptr + 1] = carry2 + 2 ** 127;

    tempvar carry3 = (res_b.d2 - k_n.d2 - a.d2 + carry2) / BASE;
    assert [range_check_ptr + 2] = carry3 + 2 ** 127;

    tempvar carry4 = (res_b.d3 - k_n.d3 + carry3) / BASE;
    assert [range_check_ptr + 3] = carry4 + 2 ** 127;

    assert res_b.d4 - k_n.d4 + carry4 = 0;

    let range_check_ptr = range_check_ptr + 4;

    return (res=res);
}

// Verifies that val is in the range [1, N) and that the limbs of val are in the range [0, BASE).
func validate_signature_entry{range_check_ptr}(val: BigInt3) {
    assert_nn_le(val.d2, N2);
    assert_nn_le(val.d1, BASE - 1);
    assert_nn_le(val.d0, BASE - 1);

    if (val.d2 == N2) {
        if (val.d1 == N1) {
            assert_nn_le(val.d0, N0 - 1);
            return ();
        }
        assert_nn_le(val.d1, N1 - 1);
        return ();
    }

    // Check that val > 0.
    if (val.d2 == 0) {
        if (val.d1 == 0) {
            assert_not_zero(val.d0);
            return ();
        }
    }
    return ();
}

// Verifies a Secp256r1 ECDSA signature - public_key is expected to be on secp256r1 curve.
// Also verifies that r and s are in the range (0, N), that their limbs are in the range
// [0, BASE)
func verify_secp256r1_signature{range_check_ptr}(
    msg_hash: BigInt3, r: BigInt3, s: BigInt3, public_key: EcPoint
) {
    alloc_locals;

    with_attr error_message("Signature out of range.") {
        validate_signature_entry(r);
        validate_signature_entry(s);
    }

    with_attr error_message("Invalid signature.") {
        let (generator_point: EcPoint) = get_generator_point();

        let (u1: BigInt3) = div_mod_n(msg_hash, s);
        let (u2: BigInt3) = div_mod_n(r, s);

        let (point3) = ec_mulmuladdW_bg3(generator_point, public_key, u1, u2);

        let (x_mod_N) = div_mod_n(point3.x, BigInt3(d0=1, d1=0, d2=0));
        // We already validated r in [1, N) so no need to mod N it
        assert x_mod_N = r;
    }
    return ();
}
