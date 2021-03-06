#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::cmp::min;
use std::num::Wrapping as W;
use std::result;

type EmptyResult = result::Result<(), ()>;

static mut randombytes: fn(x: &mut [u8]) = randombytes_impl;
pub unsafe fn init(f: fn(x: &mut [u8])) {
    randombytes = f;
}
fn randombytes_impl(_: &mut [u8]) {
    panic!("randombytes() is not implemented");
}

type gf = [i64; 16];

const _0: &'static [u8] = &[0; 16];
const _9: &'static [u8] = &[9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0];
const gf0: gf = [0; 16];
const gf1: gf = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const _121665: gf = [0xDB41, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const D: gf = [0x78a3, 0x1359, 0x4dca, 0x75eb, 0xd8ab, 0x4141, 0x0a4d, 0x0070, 0xe898, 0x7779,
               0x4079, 0x8cc7, 0xfe73, 0x2b6f, 0x6cee, 0x5203];
const D2: gf = [0xf159, 0x26b2, 0x9b94, 0xebd6, 0xb156, 0x8283, 0x149a, 0x00e0, 0xd130, 0xeef3,
                0x80f2, 0x198e, 0xfce7, 0x56df, 0xd9dc, 0x2406];
const X: gf = [0xd51a, 0x8f25, 0x2d60, 0xc956, 0xa7b2, 0x9525, 0xc760, 0x692c, 0xdc5c, 0xfdd6,
               0xe231, 0xc0a4, 0x53fe, 0xcd6e, 0x36d3, 0x2169];
const Y: gf = [0x6658, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666,
               0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666];
const I: gf = [0xa0b0, 0x4a0e, 0x1b27, 0xc4ee, 0xe478, 0xad2f, 0x1806, 0x2f43, 0xd7a7, 0x3dfb,
               0x0099, 0x2b4d, 0xdf0b, 0x4fc1, 0x2480, 0x2b83];

#[inline(always)]
fn L32(x: W<u32>, c: usize) -> W<u32> {
    (x << c) | ((x & W(0xffffffff)) >> (32 - c))
}

#[inline(always)]
fn ld32(x: &[u8]) -> W<u32> {
    let mut u = x[3] as u32;
    u = (u << 8) | x[2] as u32;
    u = (u << 8) | x[1] as u32;
    W((u << 8) | x[0] as u32)
}

#[inline(always)]
fn dl64(x: &[u8]) -> W<u64> {
    let mut u = 0u64;
    for i in 0..8 {
        u = (u << 8) | x[i] as u64;
    }
    W(u)
}

#[inline(always)]
fn st32(x: &mut [u8], u: W<u32>) {
    let mut u = u;
    for i in 0..4 {
        x[i] = u.0 as u8;
        u >>= 8;
    }
}

#[inline(always)]
fn ts64(x: &mut [u8], u: W<u64>) {
    let mut u = u;
    for i in (0..8).rev() {
        x[i] = u.0 as u8;
        u >>= 8;
    }
}

#[inline(always)]
fn vn(x: &[u8], y: &[u8]) -> isize {
    assert!(x.len() == y.len(), "length of two slices must be equal");
    let n = x.len();
    let mut d = W(0);
    for i in 0..n {
        d |= W((x[i] ^ y[i]) as u32);
    }
    ((W(1) & ((d - W(1)) >> 8)) - W(1)).0 as isize /* returns 0 if equal, 0xFF..FF otherwise */
}

pub fn crypto_verify_16(x: &[u8], y: &[u8]) -> bool {
    vn(&x[..16], &y[..16]) == 0
}

pub fn crypto_verify_32(x: &[u8], y: &[u8]) -> bool {
    vn(&x[..32], &y[..32]) == 0
}

fn core(out: &mut [u8], in_: &[u8], k: &[u8], c: &[u8], h: bool) {
    let mut w = [W(0); 16];
    let mut x = [W(0); 16];
    let mut y = [W(0); 16];
    let mut t = [W(0); 4];

    for i in 0..4 {
        x[5 * i] = ld32(&c[4 * i..]);
        x[1 + i] = ld32(&k[4 * i..]);
        x[6 + i] = ld32(&in_[4 * i..]);
        x[11 + i] = ld32(&k[16 + 4 * i..]);
    }

    for i in 0..16 {
        y[i] = x[i];
    }

    for _ in 0..20 {
        for j in 0..4 {
            for m in 0..4 {
                t[m] = x[(5 * j + 4 * m) % 16];
            }
            t[1] ^= L32(t[0] + t[3], 7);
            t[2] ^= L32(t[1] + t[0], 9);
            t[3] ^= L32(t[2] + t[1], 13);
            t[0] ^= L32(t[3] + t[2], 18);
            for m in 0..4 {
                w[4 * j + (j + m) % 4] = t[m];
            }
        }
        for m in 0..16 {
            x[m] = w[m];
        }
    }

    if h {
        for i in 0..16 {
            x[i] += y[i];
        }
        for i in 0..4 {
            x[5 * i] -= ld32(&c[4 * i..]);
            x[6 + i] -= ld32(&in_[4 * i..]);
        }
        for i in 0..4 {
            st32(&mut out[4 * i..], x[5 * i]);
            st32(&mut out[16 + 4 * i..], x[6 + i]);
        }
    } else {
        for i in 0..16 {
            st32(&mut out[4 * i..], x[i] + y[i]);
        }
    }
}

pub fn crypto_core_salsa20(out: &mut [u8], in_: &[u8], k: &[u8], c: &[u8]) -> bool {
    core(out, in_, k, c, false);
    true
}

pub fn crypto_core_hsalsa20(out: &mut [u8], in_: &[u8], k: &[u8], c: &[u8]) -> bool {
    core(out, in_, k, c, true);
    true
}

const SIGMA: &'static [u8; 16] = b"expand 32-byte k";

pub fn crypto_stream_salsa20_xor(c: &mut [u8],
                                 m: Option<&[u8]>,
                                 n: &[u8],
                                 k: &[u8])
                                 -> EmptyResult {
    if let Some(m) = m {
        assert!(m.len() == c.len());
    }
    assert!(n.len() >= 8);
    let mut cpos = 0;
    let mut mpos = 0;
    let mut b = c.len();
    let mut z = [0u8; 16];
    let mut x = [0u8; 64];
    let mut u: u32;
    if c.len() == 0 {
        return Ok(());
    }
    for i in 0..8 {
        z[i] = n[i];
    }
    while b >= 64 {
        crypto_core_salsa20(&mut x, &z, k, SIGMA);
        for i in 0..64 {
            c[cpos + i] = if let Some(m) = m { m[mpos + i] } else { 0u8 } ^ x[i];
        }
        u = 1;
        for i in 8..16 {
            u += z[i] as u32;
            z[i] = u as u8;
            u >>= 8;
        }
        b -= 64;
        cpos += 64;
        if m.is_some() {
            mpos += 64;
        }
    }
    if b > 0 {
        crypto_core_salsa20(&mut x, &z, k, SIGMA);
        for i in 0..b {
            c[cpos + i] = if let Some(m) = m { m[mpos + i] } else { 0u8 } ^ x[i];
        }
    }
    Ok(())
}

pub fn crypto_stream_salsa20(c: &mut [u8], n: &[u8], k: &[u8]) -> EmptyResult {
    crypto_stream_salsa20_xor(c, None, n, k)
}

pub fn crypto_stream(c: &mut [u8], n: &[u8], k: &[u8]) -> EmptyResult {
    let mut s = [0u8; 32];
    crypto_core_hsalsa20(&mut s, &n[..16], k, SIGMA);
    crypto_stream_salsa20(c, &n[16..], &s)
}

pub fn crypto_stream_xor(c: &mut [u8], m: Option<&[u8]>, n: &[u8], k: &[u8]) -> EmptyResult {
    let mut s = [0u8; 32];
    crypto_core_hsalsa20(&mut s, &n[..16], k, SIGMA);
    crypto_stream_salsa20_xor(c, m, &n[16..], &s)
}

#[inline(always)]
fn add1305(h: &mut [W<u32>; 17], c: &[W<u32>; 17]) {
    let mut u = W(0);
    for j in 0..17 {
        u += h[j] + c[j];
        h[j] = u & W(255);
        u >>= 8;
    }
}

const minusp: [W<u32>; 17] = [W(5), W(0), W(0), W(0), W(0), W(0), W(0), W(0), W(0), W(0), W(0),
                              W(0), W(0), W(0), W(0), W(0), W(252)];

#[cfg_attr(rustfmt, rustfmt_skip)]
pub fn crypto_onetimeauth(out: &mut [u8], mut m: &[u8], k: &[u8]) {
    let mut n = m.len();

    let s: W<u32>;
    let mut u: W<u32>;
    let mut x = [W(0); 17];
    let mut r = [W(0); 17];
    let mut h = [W(0); 17];
    let mut c = [W(0); 17];
    let mut g = [W(0); 17];

    // FOR(j,17) r[j]=h[j]=0;
    for j in 0..16 {
        r[j] = W(k[j] as u32);
    }
    r[3] &= W(15);
    r[4] &= W(252);
    r[7] &= W(15);
    r[8] &= W(252);
    r[11] &= W(15);
    r[12] &= W(252);
    r[15] &= W(15);

    while n > 0 {
        for j in 0..17 {
            c[j] = W(0);
        }
        let j_end = min(16, n);
        for j in 0..j_end {
            c[j] = W(m[j] as u32);
        }
        c[j_end] = W(1);
        m = &m[j_end..];
        n -= j_end;
        add1305(&mut h, &c);
        for i in 0..17 {
            x[i] = W(0);
            for j in 0..17 {
                x[i] += h[j] * if j <= i { r[i - j] } else { W(320) * r[i + 17 - j] };
            }
        }
        for i in 0..17 {
            h[i] = x[i];
        }
        u = W(0);
        for j in 0..16 {
            u += h[j];
            h[j] = u & W(255);
            u >>= 8;
        }
        u += h[16];
        h[16] = u & W(3);
        u = W(5) * (u >> 2);
        for j in 0..16 {
            u += h[j];
            h[j] = u & W(255);
            u >>= 8;
        }
        u += h[16];
        h[16] = u;
    }

    for j in 0..17 {
        g[j] = h[j];
    }
    add1305(&mut h, &minusp);
    s = -(h[16] >> 7);
    for j in 0..17 {
        h[j] ^= s & (g[j] ^ h[j]);
    }

    for j in 0..16 {
        c[j] = W(k[j + 16] as u32);
    }
    c[16] = W(0);
    add1305(&mut h, &c);
    for j in 0..16 {
        out[j] = h[j].0 as u8;
    }
}

pub fn crypto_onetimeauth_verify(h: &[u8], m: &[u8], k: &[u8]) -> EmptyResult {
    let mut x = [0u8; 16];
    crypto_onetimeauth(&mut x, m, k);
    if crypto_verify_16(h, &x) {
        Ok(())
    } else {
        Err(())
    }
}

pub fn crypto_secretbox(c: &mut [u8], m: &[u8], n: &[u8], k: &[u8]) -> EmptyResult {
    assert!(c.len() == m.len(), "length of c and m must be equal");
    assert!(c.len() >= 32, "output must be larger than 32 bytes");
    try!(crypto_stream_xor(c, Some(m), n, k));
    let mut x = [0u8; 16];
    crypto_onetimeauth(&mut x, &c[32..], c);
    c[16..32].copy_from_slice(&x);
    for i in 0..16 {
        c[i] = 0;
    }
    Ok(())
}

pub fn crypto_secretbox_open(m: &mut [u8], c: &[u8], n: &[u8], k: &[u8]) -> EmptyResult {
    assert!(c.len() == m.len(), "length of c and m must be equal");
    assert!(c.len() >= 32, "output must be larger than 32 bytes");
    let mut x = [0u8; 32];
    try!(crypto_stream(&mut x, n, k));
    try!(crypto_onetimeauth_verify(&c[16..], &c[32..], &x));
    try!(crypto_stream_xor(m, Some(c), n, k));
    for i in 0..32 {
        m[i] = 0;
    }
    Ok(())
}

fn set25519(r: &mut gf, a: &gf) {
    for i in 0..16 {
        r[i] = a[i];
    }
}

fn car25519(o: &mut gf) {
    let mut c: i64;
    for i in 0..16 {
        o[i] += 1i64 << 16;
        c = o[i] >> 16;
        o[(i + 1) * (i < 15) as usize] += c - 1 + 37 * (c - 1) * (i == 15) as i64;
        o[i] -= c << 16;
    }
}

fn sel25519(p: &mut gf, q: &mut gf, b: i64) {
    let mut t: i64;
    let c = !(b - 1);
    for i in 0..16 {
        t = c & (p[i] ^ q[i]);
        p[i] ^= t;
        q[i] ^= t;
    }
}

fn pack25519(o: &mut [u8], n: &[i64]) {
    let mut b: i64;
    let mut m = gf0;
    let mut t = gf0;
    for i in 0..16 {
        t[i] = n[i];
    }
    car25519(&mut t);
    car25519(&mut t);
    car25519(&mut t);
    for _ in 0..2 {
        m[0] = t[0] - 0xffed;
        for i in 1..15 {
            m[i] = t[i] - 0xffff - ((m[i - 1] >> 16) & 1);
            m[i - 1] &= 0xffff;
        }
        m[15] = t[15] - 0x7fff - ((m[14] >> 16) & 1);
        b = (m[15] >> 16) & 1;
        m[14] &= 0xffff;
        sel25519(&mut t, &mut m, 1 - b);
    }
    for i in 0..16 {
        o[2 * i] = (t[i] & 0xff) as u8;
        o[2 * i + 1] = (t[i] >> 8) as u8;
    }
}

fn neq25519(a: &gf, b: &gf) -> bool {
    let mut c = [0u8; 32];
    let mut d = [0u8; 32];
    pack25519(&mut c, a);
    pack25519(&mut d, b);
    !crypto_verify_32(&c, &d)
}

fn par25519(a: &gf) -> u8 {
    let mut d = [0u8; 32];
    pack25519(&mut d, a);
    d[0] & 1
}

fn unpack25519(o: &mut [i64], n: &[u8]) {
    for i in 0..16 {
        o[i] = n[2 * i] as i64 + ((n[2 * i + 1] as i64) << 8)
    }
    o[15] &= 0x7fff;
}

#[inline(always)]
fn A(o: &mut gf, a: &gf, b: &gf) {
    for i in 0..16 {
        o[i] = a[i] + b[i];
    }
}

#[inline(always)]
fn Z(o: &mut gf, a: &gf, b: &gf) {
    for i in 0..16 {
        o[i] = a[i] - b[i];
    }
}

#[inline(always)]
fn M(o: &mut gf, a: &[i64], b: &[i64]) {
    let mut t = [0i64; 31];
    // FOR(i,31) t[i]=0;
    for i in 0..16 {
        for j in 0..16 {
            t[i + j] += a[i] * b[j];
        }
    }
    for i in 0..15 {
        t[i] += 38 * t[i + 16];
    }
    for i in 0..16 {
        o[i] = t[i];
    }
    car25519(o);
    car25519(o);
}

#[inline(always)]
fn S(o: &mut gf, a: &[i64]) {
    M(o, a, a);
}

fn inv25519(o: &mut [i64], i: &[i64]) {
    let mut c = gf0;
    let mut t = gf0; // additional copy
    for a in 0..16 {
        c[a] = i[a];
    }
    for a in (0..254).rev() {
        S(&mut t, &c);
        if a != 2 && a != 4 {
            M(&mut c, &t, i)
        } else {
            c = t;
        }
    }
    for a in 0..16 {
        o[a] = c[a];
    }
}

fn pow2523(o: &mut [i64], i: &[i64]) {
    let mut c = gf0;
    let mut t = gf0; // additional copy
    for a in 0..16 {
        c[a] = i[a];
    }
    for a in (0..251).rev() {
        S(&mut t, &c);
        if a != 1 {
            M(&mut c, &t, i);
        } else {
            c = t;
        }
    }
    for a in 0..16 {
        o[a] = c[a];
    }
}

pub fn crypto_scalarmult(q: &mut [u8], n: &[u8], p: &[u8]) -> EmptyResult {
    let mut z = [0u8; 32];
    let mut x = [0i64; 80];
    let mut r: i64;
    let mut a = gf0;
    let mut b = gf0;
    let mut c = gf0;
    let mut d = gf0;
    let mut e = gf0;
    let mut f = gf0;
    let mut t = gf0; // additional copy
    for i in 0..32 {
        z[i] = n[i];
    }
    z[31] = (n[31] & 127) | 64;
    z[0] &= 248;
    unpack25519(&mut x, p);
    for i in 0..16 {
        b[i] = x[i];
        // d[i]=a[i]=c[i]=0;
    }
    a[0] = 1;
    d[0] = 1;
    for i in (0..255).rev() {
        r = ((z[i >> 3] >> (i & 7)) & 1) as i64;
        sel25519(&mut a, &mut b, r);
        sel25519(&mut c, &mut d, r);
        A(&mut e, &a, &c);
        Z(&mut t, &a, &c);
        a = t;
        A(&mut c, &b, &d);
        Z(&mut t, &b, &d);
        b = t;
        S(&mut d, &e);
        S(&mut f, &a);
        M(&mut t, &c, &a);
        a = t;
        M(&mut c, &b, &e);
        A(&mut e, &a, &c);
        Z(&mut t, &a, &c);
        a = t;
        S(&mut b, &a);
        Z(&mut c, &d, &f);
        M(&mut a, &c, &_121665);
        A(&mut t, &a, &d);
        a = t;
        M(&mut t, &c, &a);
        c = t;
        M(&mut a, &d, &f);
        M(&mut d, &b, &x[..]);
        S(&mut b, &e);
        sel25519(&mut a, &mut b, r);
        sel25519(&mut c, &mut d, r);
    }
    for i in 0..16 {
        x[i + 16] = a[i];
        x[i + 32] = c[i];
        x[i + 48] = b[i];
        x[i + 64] = d[i];
    }
    inv25519(&mut t, &x[32..]);
    x[32..48].copy_from_slice(&t[..]);
    M(&mut t, &x[16..], &x[32..]);
    x[16..32].copy_from_slice(&t[..]);
    pack25519(q, &x[16..]);
    Ok(())
}

pub fn crypto_scalarmult_base(q: &mut [u8], n: &[u8]) -> EmptyResult {
    crypto_scalarmult(q, n, _9)
}

pub fn crypto_box_keypair(y: &mut [u8], x: &mut [u8]) -> EmptyResult {
    unsafe { randombytes(x) };
    crypto_scalarmult_base(y, x)
}

pub fn crypto_box_beforenm(k: &mut [u8], y: &[u8], x: &[u8]) -> EmptyResult {
    let mut s = [0u8; 32];
    try!(crypto_scalarmult(&mut s, x, y));
    if crypto_core_hsalsa20(k, _0, &s, SIGMA) {
        Ok(())
    } else {
        Err(())
    }
}

pub fn crypto_box_afternm(c: &mut [u8], m: &[u8], n: &[u8], k: &[u8]) -> EmptyResult {
    crypto_secretbox(c, m, n, k)
}

pub fn crypto_box_open_afternm(m: &mut [u8], c: &[u8], n: &[u8], k: &[u8]) -> EmptyResult {
    crypto_secretbox_open(m, c, n, k)
}

pub fn crypto_box(c: &mut [u8], m: &[u8], n: &[u8], y: &[u8], x: &[u8]) -> EmptyResult {
    let mut k = [0u8; 32];
    try!(crypto_box_beforenm(&mut k, y, x));
    crypto_box_afternm(c, m, n, &k)
}

pub fn crypto_box_open(m: &mut [u8], c: &[u8], n: &[u8], y: &[u8], x: &[u8]) -> EmptyResult {
    let mut k = [0u8; 32];
    try!(crypto_box_beforenm(&mut k, y, x));
    crypto_box_open_afternm(m, c, n, &k)
}

fn R(x: W<u64>, c: usize) -> W<u64> {
    (x >> c) | (x << (64 - c))
}
fn Ch(x: W<u64>, y: W<u64>, z: W<u64>) -> W<u64> {
    (x & y) ^ (!x & z)
}
fn Maj(x: W<u64>, y: W<u64>, z: W<u64>) -> W<u64> {
    (x & y) ^ (x & z) ^ (y & z)
}
fn Sigma0(x: W<u64>) -> W<u64> {
    R(x, 28) ^ R(x, 34) ^ R(x, 39)
}
fn Sigma1(x: W<u64>) -> W<u64> {
    R(x, 14) ^ R(x, 18) ^ R(x, 41)
}
fn sigma0(x: W<u64>) -> W<u64> {
    R(x, 1) ^ R(x, 8) ^ (x >> 7)
}
fn sigma1(x: W<u64>) -> W<u64> {
    R(x, 19) ^ R(x, 61) ^ (x >> 6)
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const K: [u64; 80] =
[
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817
];

fn crypto_hashblocks(x: &mut [u8], mut m: &[u8]) -> usize {
    let mut n = m.len();
    let mut z = [W(0u64); 8];
    let mut b = [W(0u64); 8];
    let mut a = [W(0u64); 8];
    let mut w = [W(0u64); 16];
    let mut t: W<u64>;

    for i in 0..8 {
        a[i] = dl64(&x[8 * i..]);
        z[i] = a[i];
    }

    while n >= 128 {
        for i in 0..16 {
            w[i] = dl64(&m[8 * i..]);
        }

        for i in 0..80 {
            for j in 0..8 {
                b[j] = a[j];
            }
            t = a[7] + Sigma1(a[4]) + Ch(a[4], a[5], a[6]) + W(K[i]) + w[i % 16];
            b[7] = t + Sigma0(a[0]) + Maj(a[0], a[1], a[2]);
            b[3] += t;
            for j in 0..8 {
                a[(j + 1) % 8] = b[j];
            }
            if i % 16 == 15 {
                for j in 0..16 {
                    w[j] += w[(j + 9) % 16] + sigma0(w[(j + 1) % 16]) + sigma1(w[(j + 14) % 16]);
                }
            }
        }

        for i in 0..8 {
            a[i] += z[i];
            z[i] = a[i]
        }

        m = &m[128..];
        n -= 128;
    }

    for i in 0..8 {
        ts64(&mut x[8 * i..], z[i]);
    }

    return n;
}

const iv: [u8; 64] = [0x6a, 0x09, 0xe6, 0x67, 0xf3, 0xbc, 0xc9, 0x08, 0xbb, 0x67, 0xae, 0x85,
                      0x84, 0xca, 0xa7, 0x3b, 0x3c, 0x6e, 0xf3, 0x72, 0xfe, 0x94, 0xf8, 0x2b,
                      0xa5, 0x4f, 0xf5, 0x3a, 0x5f, 0x1d, 0x36, 0xf1, 0x51, 0x0e, 0x52, 0x7f,
                      0xad, 0xe6, 0x82, 0xd1, 0x9b, 0x05, 0x68, 0x8c, 0x2b, 0x3e, 0x6c, 0x1f,
                      0x1f, 0x83, 0xd9, 0xab, 0xfb, 0x41, 0xbd, 0x6b, 0x5b, 0xe0, 0xcd, 0x19,
                      0x13, 0x7e, 0x21, 0x79];

pub fn crypto_hash(out: &mut [u8], mut m: &[u8]) -> EmptyResult {
    let mut n = m.len();
    let mut h = [0u8; 64];
    let mut x = [0u8; 256];
    let b = n;

    for i in 0..64 {
        h[i] = iv[i];
    }

    crypto_hashblocks(&mut h[..], m);
    m = &m[n - (n & 127)..];
    n &= 127;

    for i in 0..256 {
        x[i] = 0;
    }
    for i in 0..n {
        x[i] = m[i];
    }
    x[n] = 128;

    n = if n < 112 { 128 } else { 256 };
    x[n - 9] = (b >> 61) as u8;
    ts64(&mut x[n - 8..], W((b << 3) as u64));
    crypto_hashblocks(&mut h[..], &x[..n]);

    for i in 0..64 {
        out[i] = h[i];
    }

    Ok(())
}

#[inline(always)]
fn add(p: &mut [gf; 4], q: &[gf; 4]) {
    let mut a = gf0;
    let mut b = gf0;
    let mut c = gf0;
    let mut d = gf0;
    let mut t = gf0;
    let mut e = gf0;
    let mut f = gf0;
    let mut g = gf0;
    let mut h = gf0;
    let mut tmp = gf0;

    Z(&mut a, &p[1], &p[0]);
    Z(&mut t, &q[1], &q[0]);
    M(&mut tmp, &a, &t);
    a = tmp;
    A(&mut b, &p[0], &p[1]);
    A(&mut t, &q[0], &q[1]);
    M(&mut tmp, &b, &t);
    b = tmp;
    M(&mut c, &p[3], &q[3]);
    M(&mut tmp, &c, &D2);
    c = tmp;
    M(&mut d, &p[2], &q[2]);
    A(&mut tmp, &d, &d);
    d = tmp;
    Z(&mut e, &b, &a);
    Z(&mut f, &d, &c);
    A(&mut g, &d, &c);
    A(&mut h, &b, &a);

    M(&mut p[0], &e, &f);
    M(&mut p[1], &h, &g);
    M(&mut p[2], &g, &f);
    M(&mut p[3], &e, &h);
}

fn cswap(p: &mut [gf; 4], q: &mut [gf; 4], b: u8) {
    for i in 0..4 {
        sel25519(&mut p[i], &mut q[i], b as i64);
    }
}

fn pack(r: &mut [u8], p: &[gf; 4]) {
    let mut tx = gf0;
    let mut ty = gf0;
    let mut zi = gf0;
    inv25519(&mut zi, &p[2]);
    M(&mut tx, &p[0], &zi);
    M(&mut ty, &p[1], &zi);
    pack25519(r, &ty);
    r[31] ^= par25519(&tx) << 7;
}

fn scalarmult(p: &mut [gf; 4], q: &mut [gf; 4], s: &[u8]) {
    let mut tmp;

    set25519(&mut p[0], &gf0);
    set25519(&mut p[1], &gf1);
    set25519(&mut p[2], &gf1);
    set25519(&mut p[3], &gf0);

    for i in (0..256).rev() {
        let b: u8 = (s[i / 8] >> (i & 7)) & 1;
        cswap(p, q, b);
        add(q, p);
        tmp = *p;
        add(p, &tmp);
        cswap(p, q, b);
    }
}

fn scalarbase(p: &mut [gf; 4], s: &[u8]) {
    let mut q = [gf0; 4];
    set25519(&mut q[0], &X);
    set25519(&mut q[1], &Y);
    set25519(&mut q[2], &gf1);
    M(&mut q[3], &X, &Y);
    scalarmult(p, &mut q, s);
}

pub fn crypto_sign_keypair(pk: &mut [u8], sk: &mut [u8]) -> EmptyResult {
    let mut d = [0u8; 64];
    let mut p = [gf0; 4];

    unsafe { randombytes(&mut sk[..32]) };
    try!(crypto_hash(&mut d, &sk[..32]));
    d[0] &= 248;
    d[31] &= 127;
    d[31] |= 64;

    scalarbase(&mut p, &d[..]);
    pack(pk, &p);

    for i in 0..32 {
        sk[32 + i] = pk[i];
    }
    Ok(())
}

const L: [u64; 32] = [0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2,
                      0xde, 0xf9, 0xde, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10];

fn modL(r: &mut [u8], x: &mut [W<i64>; 64]) {
    let mut carry: W<i64>;
    for i in (32..64).rev() {
        carry = W(0);
        for j in (i - 32)..(i - 12) {
            x[j] += carry - W(16) * x[i] * W(L[j - (i - 32)] as i64);
            carry = (x[j] + W(128)) >> 8;
            x[j] -= carry << 8;
        }
        x[i - 12] += carry;
        x[i] = W(0);
    }
    carry = W(0);
    for j in 0..32 {
        x[j] += carry - (x[31] >> 4) * W(L[j] as i64);
        carry = x[j] >> 8;
        x[j] &= W(255);
    }
    for j in 0..32 {
        x[j] -= carry * W(L[j] as i64);
    }
    for i in 0..32 {
        x[i + 1] += x[i] >> 8;
        r[i] = (x[i].0 & 255) as u8;
    }
}

fn reduce(r: &mut [u8]) {
    let mut x = [W(0i64); 64];
    for i in 0..64 {
        x[i] = W(r[i] as i64);
    }
    for i in 0..64 {
        r[i] = 0;
    }
    modL(r, &mut x);
}

pub fn crypto_sign(sm: &mut [u8], smlen: &mut usize, m: &[u8], sk: &[u8]) -> EmptyResult {
    let n = m.len();
    let mut d = [0u8; 64];
    let mut h = [0u8; 64];
    let mut r = [0u8; 64];
    let mut x = [W(0i64); 64];
    let mut p = [gf0; 4];

    try!(crypto_hash(&mut d, &sk[..32]));
    d[0] &= 248;
    d[31] &= 127;
    d[31] |= 64;

    *smlen += n + 64;
    for i in 0..n {
        sm[64 + i] = m[i];
    }
    for i in 0..32 {
        sm[32 + i] = d[32 + i];
    }

    try!(crypto_hash(&mut r, &sm[32..n + 64]));
    reduce(&mut r);
    scalarbase(&mut p, &r);
    pack(sm, &p);

    for i in 0..32 {
        sm[i + 32] = sk[i + 32];
    }
    try!(crypto_hash(&mut h, &sm[..n + 64]));
    reduce(&mut h);

    for i in 0..64 {
        x[i] = W(0);
    }
    for i in 0..32 {
        x[i] = W(r[i] as i64);
    }
    for i in 0..32 {
        for j in 0..32 {
            x[i + j] += W(h[i] as i64) * W(d[j] as i64)
        }
    }
    modL(&mut sm[32..], &mut x);
    Ok(())
}

fn unpackneg(r: &mut [gf; 4], p: &[u8]) -> bool {
    let mut t = gf0;
    let mut chk = gf0;
    let mut num = gf0;
    let mut den = gf0;
    let mut den2 = gf0;
    let mut den4 = gf0;
    let mut den6 = gf0;
    let mut tmp = gf0;
    set25519(&mut r[2], &gf1);
    unpack25519(&mut r[1], &p[..]);
    S(&mut num, &r[1]);
    M(&mut den, &num, &D);
    Z(&mut tmp, &num, &r[2]);
    num = tmp;
    A(&mut tmp, &r[2], &den);
    den = tmp;

    S(&mut den2, &den);
    S(&mut den4, &den2);
    M(&mut den6, &den4, &den2);
    M(&mut t, &den6, &num);
    M(&mut tmp, &t, &den);
    t = tmp;

    pow2523(&mut tmp, &t);
    t = tmp;
    M(&mut tmp, &t, &num);
    t = tmp;
    M(&mut tmp, &t, &den);
    t = tmp;
    M(&mut tmp, &t, &den);
    t = tmp;
    M(&mut r[0], &t, &den);

    S(&mut chk, &r[0]);
    M(&mut tmp, &chk, &den);
    chk = tmp;
    if neq25519(&chk, &num) {
        M(&mut tmp, &r[0], &I);
        r[0] = tmp;
    }

    S(&mut chk, &r[0]);
    M(&mut tmp, &chk, &den);
    chk = tmp;
    if neq25519(&chk, &num) {
        return false;
    }

    if par25519(&r[0]) == (p[31] >> 7) {
        Z(&mut tmp, &gf0, &r[0]);
        r[0] = tmp;
    }

    M(&mut tmp, &r[0], &r[1]);
    r[3] = tmp;
    true
}

pub fn crypto_sign_open(m: &mut [u8], mlen: &mut isize, sm: &[u8], pk: &[u8]) -> EmptyResult {
    let mut n = sm.len();
    let mut t = [0u8; 32];
    let mut h = [0u8; 64];
    let mut p = [gf0; 4];
    let mut q = [gf0; 4];

    *mlen = -1;
    if n < 64 {
        return Err(());
    }

    if !unpackneg(&mut q, &pk) {
        return Err(());
    }

    for i in 0..n {
        m[i] = sm[i];
    }
    for i in 0..32 {
        m[i + 32] = pk[i];
    }
    try!(crypto_hash(&mut h, &m[..n]));
    reduce(&mut h);
    scalarmult(&mut p, &mut q, &h);

    scalarbase(&mut q, &sm[32..]);
    add(&mut p, &q);
    pack(&mut t, &p);

    n -= 64;
    if !crypto_verify_32(&sm, &t) {
        for i in 0..n {
            m[i] = 0;
        }
        return Err(());
    }

    for i in 0..n {
        m[i] = sm[i + 64];
    }
    *mlen = n as isize;
    Ok(())
}
