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
from starkware.cairo.common.cairo_secp.bigint import BigInt3

from src.accounts.braavos.lib.ec import EcPoint, ec_add, ec_mul, ec_double

// Structure storing all aP+b.Q for (a,b) in [0..3]x[0..3]
struct Window {
    G: EcPoint,
    Q: EcPoint,
    W3: EcPoint,
    W4: EcPoint,
    W5: EcPoint,
    W6: EcPoint,
    W7: EcPoint,
    W8: EcPoint,
    W9: EcPoint,
    W10: EcPoint,
    W11: EcPoint,
    W12: EcPoint,
    W13: EcPoint,
    W14: EcPoint,
    W15: EcPoint,
}

// https://crypto.stackexchange.com/questions/99975/strauss-shamir-trick-on-ec-multiplication-by-scalar,
// * Internal call for recursion of point multiplication via Shamir's trick */
func ec_mulmuladd_inner{range_check_ptr}(
    R: EcPoint, G: EcPoint, Q: EcPoint, H: EcPoint, scalar_u: felt, scalar_v: felt, m: felt
) -> (res: EcPoint) {
    alloc_locals;

    // this means if m=-1, beware if felt definition changes
    if (m == -1) {
        return (res=R);
    }

    let (double_point) = ec_double(R);

    let mm1 = m - 1;
    local dibit;
    // extract MSB values of both exponents
    %{ ids.dibit = ((ids.scalar_u >> ids.m) & 1) + 2 * ((ids.scalar_v >> ids.m) & 1) %}

    // set R:=R+R
    if (dibit == 0) {
        let (res) = ec_mulmuladd_inner(double_point, G, Q, H, scalar_u, scalar_v, mm1);
        return (res=res);
    }
    // if ui=1 and vi=0, set R:=R+G
    if (dibit == 1) {
        let (res10) = ec_add(double_point, G);
        let (res) = ec_mulmuladd_inner(res10, G, Q, H, scalar_u, scalar_v, mm1);
        return (res=res);
    }
    // (else) if ui=0 and vi=1, set R:=R+Q
    if (dibit == 2) {
        let (res01) = ec_add(double_point, Q);
        let (res) = ec_mulmuladd_inner(res01, G, Q, H, scalar_u, scalar_v, mm1);
        return (res=res);
    }
    // (else) if ui=1 and vi=1, set R:=R+Q
    if (dibit == 3) {
        let (res11) = ec_add(double_point, H);
        let (res) = ec_mulmuladd_inner(res11, G, Q, H, scalar_u, scalar_v, mm1);
        return (res=res);
    }

    // you shall never end up here
    return (res=R);
}

// https://crypto.stackexchange.com/questions/99975/strauss-shamir-trick-on-ec-multiplication-by-scalar,
// * Internal call for recursion of point multiplication via Shamir's trick+Windowed method */
func ec_mulmuladd_W_inner{range_check_ptr}(
    R: EcPoint, Prec: Window, scalar_u: felt, scalar_v: felt, m: felt
) -> (res: EcPoint) {
    alloc_locals;
    let mm2 = m - 2;

    // (8*v1 4*u1+ 2*v0 + u0), where (u1,u0) represents two bit at index m of scalar u, (resp for v)
    local quad_bit;

    if (m == -1) {
        return (res=R);
    }

    let (double_point) = ec_double(R);

    // still have to make the last addition over 1 bit (initial length was odd)
    if (m == 0) {
        let (res) = ec_mulmuladd_inner(R, Prec.G, Prec.Q, Prec.W3, scalar_u, scalar_v, m);
        return (res=res);
    }

    let (quadruple_point) = ec_double(double_point);

    // compute quadruple (8*v1 4*u1+ 2*v0 + u0)
    %{
        ids.quad_bit = (
            8 * ((ids.scalar_v >> ids.m) & 1)
            + 4 * ((ids.scalar_u >> ids.m) & 1)
            + 2 * ((ids.scalar_v >> (ids.m - 1)) & 1)
            + ((ids.scalar_u >> (ids.m - 1)) & 1)
        )
    %}

    if (quad_bit == 0) {
        let (res) = ec_mulmuladd_W_inner(quadruple_point, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 1) {
        let (ecTemp) = ec_add(quadruple_point, Prec.G);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 2) {
        let (ecTemp) = ec_add(quadruple_point, Prec.Q);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }

    if (quad_bit == 3) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W3);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 4) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W4);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 5) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W5);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 6) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W6);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 7) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W7);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 8) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W8);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 9) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W9);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 10) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W10);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 11) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W11);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 12) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W12);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 13) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W13);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 14) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W14);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }
    if (quad_bit == 15) {
        let (ecTemp) = ec_add(quadruple_point, Prec.W15);
        let (res) = ec_mulmuladd_W_inner(ecTemp, Prec, scalar_u, scalar_v, mm2);
        return (res=res);
    }

    // shall not be reach
    return (res=R);
}
