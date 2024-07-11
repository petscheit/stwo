use itertools::{zip_eq, Itertools};

use super::component::FactorialComponent;
use crate::core::air::{Air, AirProver, Component, ComponentProver};
use crate::core::backend::CpuBackend;
use crate::core::fields::m31::BaseField;

pub struct FibonacciAir {
    pub component: FactorialComponent,
}

impl FibonacciAir {
    pub fn new(component: FactorialComponent) -> Self {
        Self { component }
    }
}
impl Air for FibonacciAir {
    fn components(&self) -> Vec<&dyn Component> {
        vec![&self.component]
    }
}
impl AirProver<CpuBackend> for FibonacciAir {
    fn prover_components(&self) -> Vec<&dyn ComponentProver<CpuBackend>> {
        vec![&self.component]
    }
}
