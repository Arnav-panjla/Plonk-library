use ark_ec::{AdditiveGroup, PrimeGroup};
use ark_ff::{PrimeField, Field};
// We'll use the BLS12-381 G1 curve for this example.
// This group has a prime order `r`, and is associated with a prime field `Fr`.
use ark_bls12_381::{G1Projective as G, Fr as ScalarField};
use ark_std::{Zero, UniformRand, ops::Mul};

fn main() {
    let mut rng = ark_std::test_rng();
    // Let's sample uniformly random group elements:
    let a = G::rand(&mut rng);
    let b = G::rand(&mut rng);
    
    let g = G::generator();
    // Type  of g is "ark_ec::short_weierstrass::Projective<ark_bls12_381::g1::Config>"

    // We can add elements, ...
    let c = a + b;
    // ... subtract them, ...
    let d = a - b;
    // ... and double them.
    assert_eq!(c + d, a.double());
    // We can also negate elements, ...
    let e = -a;
    // ... and check that negation satisfies the basic group law
    assert_eq!(e + a, G::zero());

    // We can also multiply group elements by elements of the corresponding scalar field
    // (an act known as *scalar multiplication*)
    let scalar = ScalarField::rand(&mut rng);
    let e = c.mul(scalar);
    let f = e.mul(scalar.inverse().unwrap());
    assert_eq!(f, c);

}