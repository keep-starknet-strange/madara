// Basic definitions for the secp25r1 elliptic curve.
// The curve is given by the equation:
//   y^2 = x^3 + ax + b
// over the field Z/p for
//   p = secp256r1_prime = 2 ** 256 - (2**224 - 2**192 - 2**96 + 1)
// The size of the curve is
//   n = 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551 (prime).

// SECP_REM is defined by the equation:
//   secp256r1_prime = 2 ** 256 - SECP_REM.
const SECP_REM = 2**224 - 2**192 - 2**96 + 1;

const BASE = 2 ** 86;

// SECP_REM =  2**224 - 2**192 - 2**96 + 1
const SECP_REM0 = 1;
const SECP_REM1 = -2**10;
const SECP_REM2 = 0xffffffff00000;

// P = 0xFFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF
const P0 = 0x3fffffffffffffffffffff;
const P1 = 0x3ff;
const P2 = 0xffffffff0000000100000;

// A =  0xffffffff00000001000000000000000000000000fffffffffffffffffffffffc
const A0 = 0x3ffffffffffffffffffffc;
const A1 = 0x3ff;
const A2 = 0xffffffff0000000100000;

// B = 0x5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b
const B0 = 0x13b0f63bce3c3e27d2604b;
const B1 = 0x3555da621af194741ac331;
const B2 = 0x5ac635d8aa3a93e7b3ebb;

// N = 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551
const N0 = 0x179e84f3b9cac2fc632551;
const N1 = 0x3ffffffffffef39beab69c;
const N2 = 0xffffffff00000000fffff;

// G = (
//   0x6B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296,
//   0x4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5
// )
const GX0 = 0x2b33a0f4a13945d898c296;
const GX1 = 0x1b958e9103c9dc0df604b7;
const GX2 = 0x6b17d1f2e12c4247f8bce;
const GY0 = 0x315ececbb6406837bf51f5;
const GY1 = 0x2d29f03e7858af38cd5dac;
const GY2 = 0x4fe342e2fe1a7f9b8ee7e;

// Constants for unreduced_mul/sqr
const s2 = -2**76 - 2**12;
const s1 = -2**66 + 4;
const s0 = 2**56;

const r2 = 2**54 - 2**22;
const r1 = -2**12;
const r0 = 4;