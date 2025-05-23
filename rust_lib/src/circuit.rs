use std::collections::HashMap;
use std::ops::{Add, Mul};
use ark_std::rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
struct F(u64);

impl F {
    pub fn zero() -> Self {
        F(0)
    }

    pub fn one() -> Self {
        F(1)
    }

    pub fn rand<R: Rng>(rng: &mut R) -> Self {
        F(rng.gen_range(0..=u64::MAX))
    }
}

impl Add for F {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        F(self.0.wrapping_add(other.0))
    }
}

impl Mul for F {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        F(self.0.wrapping_mul(other.0))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GateType {
    Add,
    Mul,
}

#[derive(Debug, Clone)]
pub struct Wire {
    pub index: usize,
    pub value: F,
}

#[derive(Debug)]
pub struct Gate {
    pub gate_type: GateType,
    pub left_wire: Wire,
    pub right_wire: Wire,
    pub output_wire: Wire,
}

/// Main circuit
#[derive(Debug)]
pub struct Circuit {
    pub n: usize,// Number of gates
    pub a: Vec<F>,// Left wire values
    pub b: Vec<F>, // right wire values
    pub c: Vec<F>, // output wire values
    pub gates: Vec<Gate>, // gates
    pub selectors: CircuitSelectors, // selectors
}

// elector polynomials
#[derive(Debug, Clone)]
pub struct CircuitSelectors {
    pub q_add: Vec<F>,
    pub q_mul: Vec<F>,
    pub q_c: Vec<F>,
}

impl Circuit {
    /// Creates a new empty circuit with specified size
    pub fn new(size: usize) -> Self {
        Circuit {
            n: size,
            a: Vec::with_capacity(size),
            b: Vec::with_capacity(size),
            c: Vec::with_capacity(size),
            gates: Vec::with_capacity(size),
            selectors: CircuitSelectors {
                q_add: vec![F::zero(); size],
                q_mul: vec![F::zero(); size],
                q_c: vec![F::zero(); size],
            },
        }
    }

    /// Adds a new gate to the circuit
    pub fn add_gate(&mut self, gate: Gate) {
        let idx = self.gates.len();
        
        if idx >= self.n {
            panic!("Circuit is full. Cannot add more than {} gates.", self.n);
        }
        
        match gate.gate_type {
            GateType::Add => self.selectors.q_add[idx] = F::one(),
            GateType::Mul => self.selectors.q_mul[idx] = F::one(),
        }
        
        self.a.push(gate.left_wire.value);
        self.b.push(gate.right_wire.value);
        self.c.push(gate.output_wire.value);
        self.gates.push(gate);
    }

    /// Verifies that all constraints in the circuit are satisfied
    pub fn verify_constraints(&self) -> bool {
        for (i, gate) in self.gates.iter().enumerate() {
            let a = self.a[i];
            let b = self.b[i];
            let c = self.c[i];

            // Check gate constraints
            match &gate.gate_type {
                GateType::Add => {
                    if a + b != c {
                        return false;
                    }
                }
                GateType::Mul => {
                    if a * b != c {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::rand::thread_rng;

    #[test]
    fn test_new_circuit() {
        let circuit = Circuit::new(2);
        assert_eq!(circuit.n, 2);
        assert_eq!(circuit.gates.len(), 0);
        assert_eq!(circuit.selectors.q_add.len(), 2);
        assert_eq!(circuit.selectors.q_mul.len(), 2);
    }

    #[test]
    fn test_add_gate() {
        let mut rng = thread_rng();
        let a = F::rand(&mut rng);
        let b = F::rand(&mut rng);
        let c = a + b;

        let mut circuit = Circuit::new(2);
        
        let gate = Gate {
            gate_type: GateType::Add,
            left_wire: Wire { index: 0, value: a },
            right_wire: Wire { index: 1, value: b },
            output_wire: Wire { index: 2, value: c },
        };

        circuit.add_gate(gate);
        
        assert_eq!(circuit.a[0], a);
        assert_eq!(circuit.b[0], b);
        assert_eq!(circuit.c[0], c);
        assert_eq!(circuit.selectors.q_add[0], F::one());
        assert_eq!(circuit.selectors.q_mul[0], F::zero());
    }

    #[test]
    fn test_mul_gate() {
        let mut rng = thread_rng();
        let a = F::rand(&mut rng);
        let b = F::rand(&mut rng);
        let c = a * b;

        let mut circuit = Circuit::new(2);
        
        let gate = Gate {
            gate_type: GateType::Mul,
            left_wire: Wire { index: 0, value: a },
            right_wire: Wire { index: 1, value: b },
            output_wire: Wire { index: 2, value: c },
        };

        circuit.add_gate(gate);
        
        assert_eq!(circuit.a[0], a);
        assert_eq!(circuit.b[0], b);
        assert_eq!(circuit.c[0], c);
        assert_eq!(circuit.selectors.q_add[0], F::zero());
        assert_eq!(circuit.selectors.q_mul[0], F::one());
    }

    #[test]
    fn test_verify_constraints() {
        let mut rng = thread_rng();
        let mut circuit = Circuit::new(2);
        
        // Add gate with random values
        let a1 = F::rand(&mut rng);
        let b1 = F::rand(&mut rng);
        let add_gate = Gate {
            gate_type: GateType::Add,
            left_wire: Wire { index: 0, value: a1 },
            right_wire: Wire { index: 1, value: b1 },
            output_wire: Wire { index: 2, value: a1 + b1 },
        };
        
        // Mul gate with random values
        let a2 = F::rand(&mut rng);
        let b2 = F::rand(&mut rng);
        let mul_gate = Gate {
            gate_type: GateType::Mul,
            left_wire: Wire { index: 3, value: a2 },
            right_wire: Wire { index: 4, value: b2 },
            output_wire: Wire { index: 5, value: a2 * b2 },
        };

        circuit.add_gate(add_gate);
        circuit.add_gate(mul_gate);
        
        assert!(circuit.verify_constraints());
    }

    #[test]
    fn test_invalid_constraints() {
        let mut rng = thread_rng();
        let mut circuit = Circuit::new(1);
        
        // Invalid add gate
        let a = F::rand(&mut rng);
        let b = F::rand(&mut rng);
        let invalid_gate = Gate {
            gate_type: GateType::Add,
            left_wire: Wire { index: 0, value: a },
            right_wire: Wire { index: 1, value: b },
            output_wire: Wire { index: 2, value: a * b }, // Using multiplication instead of addition
        };

        circuit.add_gate(invalid_gate);
        
        assert!(!circuit.verify_constraints());
    }
}