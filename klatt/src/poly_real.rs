use alloc::{vec, vec::Vec};
use core::cmp::{max, min};
use core::{
    iter::Iterator, option::Option, result::Result, result::Result::Err, result::Result::Ok,
};

/// Returns `true` if two polynomials are equal.
fn compare_equal(a1: &[f64], a2: &[f64], eps: Option<f64>) -> bool {
    let eps = eps.unwrap_or(0.0);
    let n1 = a1.len() - 1;
    let n2 = a2.len() - 1;
    let n = max(n1, n2);
    for i in 0..=n {
        let v1 = if i <= n1 { a1[i] } else { 0.0 };
        let v2 = if i <= n2 { a2[i] } else { 0.0 };
        if (v1 - v2).abs() > eps {
            return false;
        }
    }
    true
}

/// Adds two real polynomials.
fn add(a1: &[f64], a2: &[f64], eps: Option<f64>) -> Result<Vec<f64>, &'static str> {
    let n1 = if a1.is_empty() { 0 } else { a1.len() - 1 };
    let n2 = if a2.is_empty() { 0 } else { a2.len() - 1 };
    let n3 = max(n1, n2);
    let mut a3 = vec![0.0; n3 + 1];
    for i in 0..=n3 {
        let v1 = if i <= n1 { a1[i] } else { 0.0 };
        let v2 = if i <= n2 { a2[i] } else { 0.0 };
        a3[i] = v1 + v2;
    }
    trim(&a3, eps)
}

/// Multiplies two real polynomials.
fn multiply(a1: &[f64], a2: &[f64], eps: Option<f64>) -> Result<Vec<f64>, &'static str> {
    if a1.is_empty() || a2.is_empty() {
        return Err("Zero len() arrays.");
    }
    if a1.len() == 1 && a1[0] == 0.0 || a2.len() == 1 && a2[0] == 0.0 {
        return Ok(vec![0.0]);
    }
    let n1 = a1.len() - 1;
    let n2 = a2.len() - 1;
    let n3 = n1 + n2;
    let mut a3 = vec![0.0; n3 + 1];
    for i in 0..=n3 {
        let mut t = 0.0;
        let p1 = i.saturating_sub(n2);
        let p2 = min(n1, i);
        for j in p1..=p2 {
            t += a1[j] * a2[i - j];
        }
        a3[i] = t;
    }
    trim(&a3, eps)
}

/// Divides two real polynomials.
/// Returns [quotient, remainder] = [a1 / a2, a1 % a2].
// fine for us because 1.0 is considered a special value (set by us)
#[allow(clippy::float_cmp)]
fn divide(a1r: &[f64], a2r: &[f64], eps: Option<f64>) -> Result<Vec<Vec<f64>>, &'static str> {
    if a1r.is_empty() || a2r.is_empty() {
        return Err("Zero len() arrays.");
    }
    let a1 = trim(a1r, eps)?;
    let a2 = trim(a2r, eps)?;
    if a2.len() == 1 {
        if a2[0] == 0.0 {
            return Err("Polynomial division by zero.");
        }
        if a2[0] == 1.0 {
            return Ok(vec![a1.clone(), vec![0.0]]);
        }
        return Ok(vec![div_by_real(&a1, a2[0]), vec![0.0]]);
    }
    let n1 = a1.len() - 1;
    let n2 = a2.len() - 1;
    if n1 < n2 {
        return Ok(vec![vec![0.0], a1.clone()]);
    }
    let mut a = a1.clone();

    let lc2 = a2[n2]; // leading coefficient of a2
    for i in (0..=(n1 - n2)).rev() {
        let r = a[n2 + i] / lc2;
        a[n2 + i] = r;
        for j in 0..n2 {
            a[i + j] -= r * a2[j];
        }
    }
    let quotient = trim(&a[n2..], eps)?;
    let remainder = trim(&a[0..n2], eps)?;
    Ok(vec![quotient, remainder])
}

/// Returns the monic GCD (greatest common divisor) of two polynomials.
fn gcd(a1: &[f64], a2: &[f64], eps: Option<f64>) -> Result<Vec<f64>, &'static str> {
    let mut r1 = trim(a1, eps)?;
    let mut r2 = trim(a2, eps)?;
    make_monic(&mut r1)?;
    make_monic(&mut r2)?;
    if r1.len() < r2.len() {
        core::mem::swap(&mut r1, &mut r2);
    }
    loop {
        if r2.len() < 2 {
            // GCD is 1
            return Ok(vec![1.0]);
        }
        let mut r = divide(&r1, &r2, eps)?[1].clone();
        if r.len() == 1 && r[0] == 0.0 {
            return Ok(r2);
        }
        make_monic(&mut r)?;
        r1 = r2;
        r2 = r;
    }
}

/// Trims top order zero coefficients.
fn trim(a: &[f64], eps: Option<f64>) -> Result<Vec<f64>, &'static str> {
    let eps = eps.unwrap_or(0.0);
    if a.is_empty() {
        return Err("Zero length array.");
    }
    if (a[a.len() - 1]).abs() > eps {
        return Ok(a.to_vec());
    }
    let mut len = a.len() - 1;
    while len > 0 && (a[len - 1]).abs() <= eps {
        len -= 1;
    }
    if len == 0 {
        return Ok(vec![0.0]);
    }
    let mut a2 = vec![0.0; len];
    for i in 0..len {
        a2[i] = a[i];
    }
    Ok(a2)
}

/// Divides the coefficients by the leading coefficient.
// fine for us because 1.0 is considered a special value (set by us)
#[allow(clippy::float_cmp)]
fn make_monic(a: &mut [f64]) -> Result<(), &'static str> {
    let len = a.len();
    if len == 0 {
        return Err("Zero length array.");
    }
    let lc = a[len - 1]; // leading coefficient
    if lc == 1.0 {
        // already monic
        return Ok(());
    }
    if lc == 0.0 {
        // not trimmed?
        return Err("Leading coefficient is zero.");
    }
    a[len - 1] = 1.0;
    for i in 0..len - 1 {
        a[i] /= lc;
    }
    Ok(())
}

fn div_by_real(a: &[f64], b: f64) -> Vec<f64> {
    let mut a2 = vec![0.0; a.len()];
    for i in 0..a.len() {
        a2[i] = a[i] / b;
    }
    a2
}

// fine for us because 1.0 is considered a special value (set by us)
#[allow(clippy::float_cmp)]
pub fn add_fractions(
    f1: &[Vec<f64>],
    f2: &[Vec<f64>],
    eps: Option<f64>,
) -> Result<Vec<Vec<f64>>, &'static str> {
    if compare_equal(&f1[1], &f2[1], eps) {
        // if same denominator add numerators
        return Ok(vec![add(&f1[0], &f2[0], eps)?, f1[1].clone()]);
    }
    let g = gcd(&f1[1], &f2[1], eps)?; // GCD of demoninators
    if g.len() == 1 && g[0] == 1.0 {
        // if GCD is 1
        let top = add(
            &multiply(&f1[0], &f2[1], eps)?,
            &multiply(&f2[0], &f1[1], eps)?,
            eps,
        )?;
        let bottom = multiply(&f1[1], &f2[1], eps)?;
        return Ok(vec![top, bottom]);
    }

    let top = vec![];
    let bottom = vec![];

    Ok(vec![top, bottom])
}

pub fn multiply_fractions(
    f1: &[Vec<f64>],
    f2: &[Vec<f64>],
    eps: Option<f64>,
) -> Result<Vec<Vec<f64>>, &'static str> {
    let top = multiply(&f1[0], &f2[0], eps)?;
    let bottom = multiply(&f1[1], &f2[1], eps)?;
    Ok(vec![top, bottom])
}
