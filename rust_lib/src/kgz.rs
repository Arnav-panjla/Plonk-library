use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::{Field, PrimeField, UniformRand};
use ark_poly::{univariate::DensePolynomial, UVPolynomial};
use ark_std::rand::Rng;

pub struct KZGParams<E: Pairing> {
    pub powers_of_g: Vec<E::G1Affine>, 
    pub g2: E::G2Affine,       
    pub g2_s: E::G2Affine,       
}

impl<E: Pairing> KZGParams<E> {
    pub fn setup<R: Rng>(degree: usize, rng: &mut R) -> Self {
        let s = E::ScalarField::rand(rng);
        let g1 = E::G1::generator();
        let g2 = E::G2::generator();

        let mut powers_of_g = Vec::with_capacity(degree + 1);
        let mut power = E::ScalarField::one();

        for _ in 0..=degree {
            powers_of_g.push((g1 * power).into_affine());
            power *= s;
        }

        let g2_s = (g2 * s).into_affine();

        Self {
            powers_of_g,
            g2: g2.into_affine(),
            g2_s,
        }
    }
}

