use ark_ec::{AdditiveGroup, PrimeGroup, AffineRepr, CurveGroup};
use ark_ff::{PrimeField, Field};
use ark_std::{Zero, One, UniformRand, ops::Mul};
use ark_ec::pairing::Pairing;
use ark_std::rand::Rng;

use ark_bls12_381::{
    Bls12_381,
    G1Projective as G1, 
    G2Projective as G2, 
    Fr as ScalarField
    };

use ark_poly::polynomial::{Polynomial, DenseUVPolynomial};
use ark_poly::polynomial::univariate::DensePolynomial;


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

        //constructing g1*s^{i}
        for _ in 0..=degree {
            powers_of_g.push((g1 * power).into_affine());
            power *= s;
        }

        Self {
            powers_of_g,
            g2: g2.into_affine(),
            g2_s: (g2 * s).into_affine(), //g2*s for verification
        }
    }

    pub fn commit(&self, poly: &DensePolynomial<E::ScalarField>) -> E::G1Affine {
        let mut result = E::G1::zero();
        
        for (i, coeff) in poly.coeffs().iter().enumerate() {
            if i >= self.powers_of_g.len() {
                panic!("Polynomial degree too large for parameters");
            }
            result += self.powers_of_g[i].mul(*coeff);
        }
        
        result.into_affine()
    }

    /// Creates an evaluation proof for the polynomial at point z
    /// Returns (proof, value) where:
    /// - proof is the KZG proof (commitment to quotient polynomial)
    /// - value is the evaluation of polynomial at z
    pub fn open(
        &self,
        poly: &DensePolynomial<E::ScalarField>,
        z: E::ScalarField,
    ) -> (E::G1Affine, E::ScalarField) {
        // Compute polynomial value at z
        let value = poly.evaluate(&z);
        
        // Compute quotient polynomial q(X) = (p(X) - p(z)) / (X - z)
        let numerator = poly - &DensePolynomial::from_coefficients_vec(vec![value]);
        let divisor = DensePolynomial::from_coefficients_vec(vec![-z, E::ScalarField::one()]);
        
        // Divide to get the quotient polynomial

        let quotient = numerator / &divisor ;
        
        // Commit to quotient polynomial
        let proof = self.commit(&quotient);
        
        (proof, value)
    }

    /// Verifies a KZG proof
    /// Returns true if the proof is valid for the claimed evaluation
    pub fn verify(
        &self,
        commitment: &E::G1Affine,
        proof: &E::G1Affine,
        z: E::ScalarField,
        value: E::ScalarField,
    ) -> bool {
        // (proof, [x]₂ - [z]₂) = e(commitment - [value]₁, [1]₂)
        let g1_value = self.powers_of_g[0].mul(value);
        let commitment_minus_value = commitment.into_group() - g1_value;
        
        let g2_z = self.g2.mul(z);
        let g2_s_minus_z = self.g2_s.into_group() - g2_z;
        
        let pairing1 = E::pairing(proof.into_group(), g2_s_minus_z);
        let pairing2 = E::pairing(commitment_minus_value, self.g2.into_group());
        
        pairing1 == pairing2
    }
}

#[test]
fn test_kgz_setup() {
    let mut rng = ark_std::test_rng();
    let params : KZGParams<Bls12_381> = KZGParams::setup(10, &mut rng);
    assert_eq!(params.powers_of_g.len(), 11);
    assert_eq!(params.g2, G2::generator().into_affine());
}


#[test]
fn test_kgz_g2_relationship() {
    let mut rng = ark_std::test_rng();
    let params: KZGParams<Bls12_381> = KZGParams::setup(3, &mut rng);
    
    let g1_s = params.powers_of_g[1];
    let g1_gen = params.powers_of_g[0];
    
    let pairing1 = Bls12_381::pairing(g1_s, params.g2);
    let pairing2 = Bls12_381::pairing(g1_gen, params.g2_s);
    assert_eq!(pairing1, pairing2);
}

#[test]
fn test_kgz_edge_cases() {
    let mut rng = ark_std::test_rng();
    
    // Test degree 0
    let params: KZGParams<Bls12_381> = KZGParams::setup(0, &mut rng);
    
    assert_eq!(params.powers_of_g.len(), 1);
    assert_eq!(params.powers_of_g[0], G1::generator().into_affine());
    
    // Test degree 1
    let params: KZGParams<Bls12_381> = KZGParams::setup(1, &mut rng);
    assert_eq!(params.powers_of_g.len(), 2);
}

#[test]
fn test_kzg_commit_verify() {
    
    let mut rng = ark_std::test_rng();
    let params: KZGParams<Bls12_381> = KZGParams::setup(5, &mut rng);
    
    // Create a test polynomial x^2 + 2x + 3
    let poly = DensePolynomial::from_coefficients_vec(
        vec![
            ScalarField::from(3u64),
            ScalarField::from(2u64),
            ScalarField::from(1u64),
        ]
    );
    
    // Create commitment
    let commitment = params.commit(&poly);
    
    // Create proof for evaluation at z = 2
    let z = ScalarField::from(2u64);
    let (proof, value) = params.open(&poly, z);
    
    // Verify the proof
    assert!(params.verify(&commitment, &proof, z, value));
    
    // Verify that wrong value fails
    let wrong_value = value + ScalarField::one();
    assert!(!params.verify(&commitment, &proof, z, wrong_value));
}


