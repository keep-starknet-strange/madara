// *************************************************************************************/
// /* Copyright (C) 2022 - Renaud Dubois - This file is part of Cairo_musig2 project   */
// /* License: This software is licensed under a dual BSD and GPL v2 license.          */
// /* See LICENSE file at the root folder of the project.                              */
// /* FILE: multipoint.cairo                                                           */
// /*                                                                                  */
// /*                                                                                  */
// /* DESCRIPTION: optimization of dual base multiplication                            */
// /* the algorithm combines the so called Shamir's trick with Windowing method        */
// *************************************************************************************/

// Shamir's trick:https://crypto.stackexchange.com/questions/99975/strauss-shamir-trick-on-ec-multiplication-by-scalar,
// Windowing method : https://en.wikipedia.org/wiki/Exponentiation_by_squaring, section 'sliding window'
// The implementation use a 2 bits window with trick, leading to a 16 points elliptic point precomputation

from starkware.cairo.common.cairo_secp.bigint import BigInt3

from src.accounts.braavos.lib.ec import ec_add, ec_double, ec_mul, EcPoint
from src.accounts.braavos.lib.ec_mulmuladd import Window, ec_mulmuladd_W_inner

func ec_mulmuladdW_bg3{range_check_ptr}(
    G: EcPoint, Q: EcPoint, scalar_u: BigInt3, scalar_v: BigInt3
) -> (res: EcPoint) {
    alloc_locals;
    local len_hi;  // hi 84 bits part of scalar
    local len_med;  // med 86 bits part
    local len_low;  // low bits part

    // Precompute a 4-bit window , W0=infty, W1=P, W2=Q,
    // the window is indexed by (8*v1 4*u1+ 2*v0 + u0), where (u1,u0) represents two bit of scalar u,
    // (resp for v)

    let (W3) = ec_add(G, Q);  // 3:G+Q
    let (W4) = ec_double(G);  // 4:2G
    let (W5) = ec_add(G, W4);  // 5:3G
    let (W6) = ec_add(W4, Q);  // 6:2G+Q
    let (W7) = ec_add(W5, Q);  // 7:3G+Q
    let (W8) = ec_double(Q);  // 8:2Q

    let (W9) = ec_add(W8, G);  // 9:2Q+G
    let (W10) = ec_add(W8, Q);  // 10:3Q
    let (W11) = ec_add(W10, G);  // 11:3Q+G
    let (W12) = ec_add(W8, W4);  // 12:2Q+2G
    let (W13) = ec_add(W8, W5);  // 13:2Q+3G
    let (W14) = ec_add(W10, W4);  // 14:3Q+2G
    let (W15) = ec_add(W10, W5);  // 15:3Q+3G

    local PrecPoint: Window = Window(
        G, Q, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15
    );

    // initialize R with infinity point
    local R: EcPoint = EcPoint(BigInt3(0, 0, 0), BigInt3(0, 0, 0));

    %{ ids.len_hi = max(ids.scalar_u.d2.bit_length(), ids.scalar_v.d2.bit_length())-1 %}

    assert [range_check_ptr] = len_hi;
    assert [range_check_ptr + 1] = 86 - len_hi;
    let range_check_ptr = range_check_ptr + 2;

    let (hiR) = ec_mulmuladd_W_inner(R, PrecPoint, scalar_u.d2, scalar_v.d2, len_hi);
    let (medR) = ec_mulmuladd_W_inner(hiR, PrecPoint, scalar_u.d1, scalar_v.d1, 85);
    let (lowR) = ec_mulmuladd_W_inner(medR, PrecPoint, scalar_u.d0, scalar_v.d0, 85);

    return (res=lowR);
}
