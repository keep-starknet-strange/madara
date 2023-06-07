from starkware.cairo.common.cairo_secp.bigint import BigInt3, UnreducedBigInt3

from src.accounts.braavos.lib.bigint import nondet_bigint3
from src.accounts.braavos.lib.constants import (
    BASE,
    P0,
    P1,
    P2,
    SECP_REM,
    SECP_REM0,
    SECP_REM1,
    SECP_REM2,
    s0,
    s1,
    s2,
    r0,
    r1,
    r2,
)

// Adapt from starkware.cairo.common.math's assert_250_bit
func assert_165_bit{range_check_ptr}(value) {
    const UPPER_BOUND = 2 ** 165;
    const SHIFT = 2 ** 128;
    const HIGH_BOUND = SHIFT - UPPER_BOUND / SHIFT;

    let low = [range_check_ptr];
    let high = [range_check_ptr + 1];

    %{
        from starkware.cairo.common.math_utils import as_int

        # Correctness check.
        value = as_int(ids.value, PRIME) % PRIME
        assert value < ids.UPPER_BOUND, f'{value} is outside of the range [0, 2**250).'

        # Calculation for the assertion.
        ids.high, ids.low = divmod(ids.value, ids.SHIFT)
    %}

    assert [range_check_ptr + 2] = high + HIGH_BOUND;

    assert value = high * SHIFT + low;

    let range_check_ptr = range_check_ptr + 3;
    return ();
}

// Computes the multiplication of two big integers, given in BigInt3 representation, modulo the
// secp256r1 prime.
//
// Arguments:
//   x, y - the two BigInt3 to operate on.
//
// Returns:
//   x * y in an UnreducedBigInt3 representation (the returned limbs may be above 3 * BASE).
//
// This means that if unreduced_mul is called on the result of nondet_bigint3, or the difference
// between two such results, we have:
//   Soundness guarantee: the limbs are in the range (-2**249, 2**249).
//   Completeness guarantee: the limbs are in the range (-2**250, 2**250).
func unreduced_mul(a: BigInt3, b: BigInt3) -> (res_low: UnreducedBigInt3) {
    tempvar twice_d2 = a.d2 * b.d2;
    tempvar d1d2 = a.d2 * b.d1 + a.d1 * b.d2;
    return (
        UnreducedBigInt3(
            d0=a.d0 * b.d0 + s0 * twice_d2 + r0 * d1d2,
            d1=a.d1 * b.d0 + a.d0 * b.d1 + s1 * twice_d2 + r1 * d1d2,
            d2=a.d2 * b.d0 + a.d1 * b.d1 + a.d0 * b.d2 + s2 * twice_d2 + r2 * d1d2,
        ),
    );
}

// Computes the square of a big integer, given in BigInt3 representation, modulo the
// secp256r1 prime.
//
// Has the same guarantees as in unreduced_mul(a, a).
func unreduced_sqr(a: BigInt3) -> (res_low: UnreducedBigInt3) {
    tempvar twice_d2 = a.d2 * a.d2;
    tempvar twice_d1d2 = a.d2 * a.d1 + a.d1 * a.d2;
    tempvar d1d0 = a.d1 * a.d0;
    return (
        UnreducedBigInt3(
            d0=a.d0 * a.d0 + s0 * twice_d2 + r0 * twice_d1d2,
            d1=d1d0 + d1d0 + s1 * twice_d2 + r1 * twice_d1d2,
            d2=a.d2 * a.d0 + a.d1 * a.d1 + a.d0 * a.d2 + s2 * twice_d2 + r2 * twice_d1d2,
        ),
    );
}

// Verifies that the given unreduced value is equal to zero modulo the secp256r1 prime.
//
// Completeness assumption: val's limbs are in the range (-2**249, 2**249).
// Soundness assumption: val's limbs are in the range (-2**250, 2**250).
func verify_zero{range_check_ptr}(val: UnreducedBigInt3) {
    alloc_locals;
    local q;
    %{ from starkware.cairo.common.cairo_secp.secp256r1_utils import SECP256R1_P as SECP_P %}
    %{
        from starkware.cairo.common.cairo_secp.secp_utils import pack

        q, r = divmod(pack(ids.val, PRIME), SECP_P)
        assert r == 0, f"verify_zero: Invalid input {ids.val.d0, ids.val.d1, ids.val.d2}."
        ids.q = q % PRIME
    %}

    assert_165_bit(q + 2 ** 164);
    // q in [-2**164, 2**164)

    tempvar r1 = (val.d0 + q * SECP_REM0) / BASE;
    assert_165_bit(r1 + 2 ** 164);
    // r1 in [-2**164, 2**164) also meaning
    // numerator divides BASE which is the case when val divides secp256r1
    // so r1 * BASE = val.d0 + q*SECP_REM0 in the integers

    tempvar r2 = (val.d1 + q * SECP_REM1 + r1) / BASE;
    assert_165_bit(r2 + 2 ** 164);
    // r2 in [-2**164, 2**164) following the same reasoning
    // so r2 * BASE = val.d1 + q*SECP_REM1 + r1 in the integers
    // so r2 * BASE ** 2 = val.d1 * BASE + q*SECP_REM1 * BASE + r1 * BASE

    assert val.d2 + q * SECP_REM2 = q * (BASE / 4) - r2;
    // both lhs and rhs are in (-2**250, 2**250) so assertion valid in the integers
    // multiply both sides by BASE**2
    // val.d2*BASE**2 + q * SECP_REM2*BASE**2
    //     = q * (2**256) - val.d1 * BASE + q*SECP_REM1 * BASE + val.d0 + q*SECP_REM0
    //  collect val on one side and all the rest on the other =>
    //  val = q*(2**256 - SECP_REM) = q * secp256r1 = 0 mod secp256r1

    return ();
}

// Returns 1 if x == 0 (mod secp256r1_prime), and 0 otherwise.
//
// Completeness assumption: x's limbs are in the range (-BASE, 2*BASE).
// Soundness assumption: x's limbs are in the range (-2**107.49, 2**107.49).
func is_zero{range_check_ptr}(x: BigInt3) -> (res: felt) {
    %{ from starkware.cairo.common.cairo_secp.secp256r1_utils import SECP256R1_P as SECP_P %}
    %{
        from starkware.cairo.common.cairo_secp.secp_utils import pack
        x = pack(ids.x, PRIME) % SECP_P
    %}
    if (nondet %{ x == 0 %} != 0) {
        verify_zero(UnreducedBigInt3(d0=x.d0, d1=x.d1, d2=x.d2));
        return (res=1);
    }

    %{
        from starkware.python.math_utils import div_mod

        value = x_inv = div_mod(1, x, SECP_P)
    %}
    let (x_inv) = nondet_bigint3();
    let (x_x_inv) = unreduced_mul(x, x_inv);

    // Check that x * x_inv = 1 to verify that x != 0.
    verify_zero(UnreducedBigInt3(d0=x_x_inv.d0 - 1, d1=x_x_inv.d1, d2=x_x_inv.d2));
    return (res=0);
}

// Receives an unreduced number, and returns a number that is equal to the original number mod
// SECP_P and in reduced form.
// Soundness guarantee: the limbs are in the range (-2**249, 2**249).
// Completeness guarantee: the limbs are in the range (-2**250, 2**250).
func reduce{range_check_ptr}(x: UnreducedBigInt3) -> (reduced_x: BigInt3) {
    let orig_x = x;
    %{ from starkware.cairo.common.cairo_secp.secp256r1_utils import SECP256R1_P as SECP_P %}
    %{
        from starkware.cairo.common.cairo_secp.secp_utils import pack
        x = pack(ids.x, PRIME) % SECP_P
    %}
    // WORKAROUND: assign x into value for nondet_bigint3 until hint is fixed by Starkware
    %{
        from starkware.python.math_utils import div_mod

        value = x_inv = div_mod(1, x, SECP_P)
    %}
    let (x_inv: BigInt3) = nondet_bigint3();
    tempvar x = UnreducedBigInt3(d0=x_inv.d0, d1=x_inv.d1, d2=x_inv.d2);
    %{
        from starkware.cairo.common.cairo_secp.secp_utils import pack
        x = pack(ids.x, PRIME) % SECP_P
    %}
    %{
        from starkware.python.math_utils import div_mod

        value = x_inv = div_mod(1, x, SECP_P)
    %}
    // WORKAROUND END

    let (reduced_x: BigInt3) = nondet_bigint3();

    verify_zero(
        UnreducedBigInt3(
            d0=orig_x.d0 - reduced_x.d0, d1=orig_x.d1 - reduced_x.d1, d2=orig_x.d2 - reduced_x.d2
        ),
    );
    return (reduced_x=reduced_x);
}
