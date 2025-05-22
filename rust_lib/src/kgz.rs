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


