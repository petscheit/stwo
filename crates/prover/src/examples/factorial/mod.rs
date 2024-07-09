use num_traits::One;
use self::air::{FactorialAir};
use crate::core::backend::cpu::CpuCircleEvaluation;
use crate::core::channel::{BWSSha256Channel, Channel};
use crate::core::fields::m31::{BaseField};
use crate::core::fields::{IntoSlice};
use crate::core::poly::circle::{CanonicCoset};
use crate::core::poly::BitReversedOrder;
use crate::core::prover::{verify, ProvingError, StarkProof, VerificationError, prove};
use crate::core::vcs::bws_sha256_hash::BWSSha256Hasher;
use crate::core::vcs::hasher::Hasher;
use crate::examples::factorial::components::FactorialComponent;

pub mod air;
mod components;

pub struct Factorial {
    pub air: FactorialAir,
}

impl Factorial {
    pub fn new(n: u32, log_size: u32, claim: u32) -> Self {
        let component = FactorialComponent::new(n, log_size, claim);
        Self {
            air: FactorialAir::new(component),
        }
    }

    pub fn get_trace(&self) -> CpuCircleEvaluation<BaseField, BitReversedOrder> {
        // Trace.
        let trace_domain = CanonicCoset::new(self.air.component.log_size);
        let mut trace = Vec::with_capacity(trace_domain.size());

        // Fill trace with fibonacci squared.
        let mut a = BaseField::one();
        let mut n = BaseField::from(self.air.component.n);
        // Write init values to trace
        trace.push(n);
        trace.push(BaseField::from(1));

        // Write loop values to trace
        for _ in 0..(trace_domain.size() - 2) / 2 {
            n = n - BaseField::one();
            trace.push(n);
            a = a * (n + BaseField::one());
            trace.push(a);

        }

        CpuCircleEvaluation::new_canonical_ordered(trace_domain, trace)
    }

    pub fn prove(&self) -> Result<StarkProof, ProvingError> {
        let trace = self.get_trace();
        let channel =
            &mut BWSSha256Channel::new(BWSSha256Hasher::hash(BaseField::into_slice(&[self
                .air
                .component
                .claim])));
        let proof = prove(&self.air, channel, vec![trace]);
        // println!("Proof: {:?}", proof);

       proof

    }

    pub fn verify(&self, proof: StarkProof) -> Result<(), VerificationError> {
        let channel =
            &mut BWSSha256Channel::new(BWSSha256Hasher::hash(BaseField::into_slice(&[self
                .air
                .component
                .claim])));
        verify(proof, &self.air, channel)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::backend::cpu::CpuCircleEvaluation;
    use crate::core::channel::{BWSSha256Channel, Channel};
    use crate::core::fields::IntoSlice;
    use crate::core::fields::m31::{BaseField, M31};
    use crate::core::poly::circle::CanonicCoset;
    use crate::core::prover::prove;
    use crate::core::vcs::bws_sha256_hash::BWSSha256Hasher;
    use crate::core::vcs::hasher::Hasher;
    use crate::examples::factorial::air::FactorialAir;
    use crate::examples::factorial::components::FactorialComponent;
    #[test]
    pub fn test_constraints() {
        let traces = vec![
            (vec![M31(4), M31(1), M31(3), M31(4), M31(2), M31(12), M31(1), M31(24)], true),
            (vec![M31(3), M31(1), M31(3), M31(4), M31(2), M31(12), M31(1), M31(24)], false),
            (vec![M31(3), M31(1), M31(3), M31(4), M31(2), M31(12), M31(2), M31(24)], false),
            (vec![M31(3), M31(1), M31(3), M31(4), M31(2), M31(12), M31(1), M31(23)], false),
            (vec![M31(4), M31(0), M31(3), M31(4), M31(2), M31(12), M31(1), M31(24)], false),
            (vec![M31(4), M31(1), M31(3), M31(3), M31(2), M31(12), M31(1), M31(24)], false),
            (vec![M31(4), M31(1), M31(3), M31(4), M31(2), M31(11), M31(1), M31(24)], false),
            (vec![M31(4), M31(1), M31(1), M31(4), M31(2), M31(12), M31(1), M31(24)], false),
            (vec![M31(4), M31(1), M31(4), M31(4), M31(3), M31(16), M31(2), M31(48)], false),
        ];

        for (trace, valid) in traces {
            println!("Running: {:?}", trace);
            let trace_conv = CpuCircleEvaluation::new_canonical_ordered(CanonicCoset::new(3), trace);

            let channel =
            &mut BWSSha256Channel::new(BWSSha256Hasher::hash(BaseField::into_slice(&[BaseField::from(24)])));
            let proof = prove(&FactorialAir::new(FactorialComponent::new(4, 3, 24)), channel, vec![trace_conv]);
            assert_eq!(proof.is_ok(), valid);
        }
    }

    #[test]
    pub fn test_proving_and_vcerification() {
        let factorial = super::Factorial::new(8, 4, 40320);
        let proof = factorial.prove();
        assert!(proof.is_ok());

        let valid = factorial.verify(proof.unwrap());
        assert!(valid.is_ok());
    }
}
