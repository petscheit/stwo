use crate::core::air::{Air, AirProver, Component, ComponentProver};
use crate::core::backend::CpuBackend;
use crate::examples::factorial::components::FactorialComponent;

pub struct FactorialAir {
    pub component: FactorialComponent,
}

impl FactorialAir {
    pub fn new(component: FactorialComponent) -> Self {
        Self { component }
    }
}
impl Air for FactorialAir {
    fn components(&self) -> Vec<&dyn Component> {
        vec![&self.component]
    }
}
impl AirProver<CpuBackend> for FactorialAir {
    fn prover_components(&self) -> Vec<&dyn ComponentProver<CpuBackend>> {
        vec![&self.component]
    }
}
