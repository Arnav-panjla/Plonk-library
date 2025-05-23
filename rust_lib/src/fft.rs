use ark_ff::Field;
use ark_std::{Zero, One};
use ark_bls12_381::Fr as ScalarField;

use ark_poly::polynomial::{Polynomial, DenseUVPolynomial};
use ark_poly::polynomial::univariate::DensePolynomial;


#[derive(Debug, Clone)]
pub struct EvaluationDomain<F: Field> {
    pub size: usize, // size of the domain
    pub omega: F,// genrator
    pub omega_inv: F, // inverse of the generator
}

impl<F: Field> EvaluationDomain<F> {
    pub fn new(size: usize, omega: F) -> Self {
        let omega_inv = omega.inverse().unwrap();
        Self {
            size,
            omega,
            omega_inv,
        }
    }
}

/// FFT usingCooley-Tukey algorithm
pub fn fft<F: Field>(poly_coeffs: &mut [F], omega: F) {
    let n = poly_coeffs.len();
    assert!(n.is_power_of_two(), "Length must be a power of 2");

    for i in 0..n {
        let j = reverse_bits(i, n.trailing_zeros() as usize);
        if i < j {
            poly_coeffs.swap(i, j);
        }
    }

    let mut m = 1;
    while m < n {
        let half_m = m;
        m *= 2;
        let w_m = omega.pow(&[(n / m) as u64]);
        
        for k in (0..n).step_by(m) {
            let mut w = F::one();
            for j in 0..half_m {
                let t = w * poly_coeffs[k + j + half_m];
                poly_coeffs[k + j + half_m] = poly_coeffs[k + j] - t;
                poly_coeffs[k + j] = poly_coeffs[k + j] + t;
                w *= w_m;
            }
        }
    }
}

pub fn ifft<F: Field>(evals: &mut [F], omega_inv: F) {
    let n = evals.len();
    fft(evals, omega_inv);

    let n_inv = F::from(n as u64).inverse().unwrap();
    evals.iter_mut().for_each(|eval| *eval *= n_inv);
}

pub fn interpolate<F: Field>(evals: &[F], domain: &[F]) -> DensePolynomial<F> {
    assert_eq!(evals.len(), domain.len(), "Evaluation and domain size mismatch");
    let n = evals.len();
    
    let mut coeffs = evals.to_vec();
    let omega_inv = domain[1].pow(&[n as u64 - 1]);

    ifft(&mut coeffs, omega_inv);
    
    DensePolynomial::from_coefficients_vec(coeffs)
}

fn reverse_bits(mut num: usize, bits: usize) -> usize {
    let mut result = 0;
    for i in 0..bits {
        if num & (1 << i) != 0 {
            result |= 1 << (bits - 1 - i);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft_ifft() {
        let mut coeffs = vec![
            ScalarField::one(),
            ScalarField::one(),
            ScalarField::zero(),
            ScalarField::zero(),
        ];
        
        let omega = ScalarField::from(5u64).pow(&[
            0xc19139cb84c680a6u64,
            0x26fe7e3811dead04u64,
            0x154e9c24a5f559c7u64,
            0x8495b4e4c316u64,
        ]);
        
        let original_coeffs = coeffs.clone();
        
        fft(&mut coeffs, omega);
        
        ifft(&mut coeffs, omega.inverse().unwrap());
        
        for (a, b) in coeffs.iter().zip(original_coeffs.iter()) {
            assert_eq!(a, b);
        }
    }
}
