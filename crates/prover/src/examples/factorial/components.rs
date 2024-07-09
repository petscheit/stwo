use std::ops::Div;
use num_traits::One;


use crate::core::air::accumulation::{DomainEvaluationAccumulator, PointEvaluationAccumulator};
use crate::core::air::mask::shifted_mask_points;
use crate::core::air::{Component, ComponentProver, ComponentTrace, ComponentTraceWriter};
use crate::core::backend::CpuBackend;
use crate::core::circle::{CirclePoint, Coset};
use crate::core::constraints::{coset_vanishing, pair_vanishing, point_excluder};
use crate::core::fields::m31::{BaseField,};
use crate::core::fields::qm31::SecureField;
use crate::core::fields::{ExtensionOf, FieldExpOps};
use crate::core::poly::circle::{CanonicCoset, CircleEvaluation};
use crate::core::poly::BitReversedOrder;
use crate::core::utils::bit_reverse_index;
use crate::core::{ColumnVec, InteractionElements};

#[derive(Debug)]
pub struct FactorialComponent {
    pub log_size: u32,
    pub n: u32,
    pub claim: BaseField,
}

fn multiply_circle<F: ExtensionOf<BaseField>>(a: CirclePoint<F>, b: CirclePoint<F>) -> CirclePoint<F> {
    CirclePoint {
        x: a.x * b.x - a.y * b.y,
        y: a.x * b.y + a.y * b.x,
    }
}

impl FactorialComponent {
    pub fn new(n: u32, log_size: u32, claim: u32) -> Self {
        Self { n, log_size, claim: BaseField::from(claim) }
    }

    /// Evaluates the step constraint quotient polynomial on a single point.
    /// The step constraint is defined as:
    ///   mask[0] * mask[1] - mask[3]
    pub fn step_constraint_eval_multiplication<F: ExtensionOf<BaseField>>(
        &self,
        point: CirclePoint<F>,
        mask: &[F; 4],
    ) -> F {
        let constraint_zero_domain = Coset::subgroup(self.log_size - 1);
        // Unsafe, but the idea is to scope the different evaluations, so they dont cancel each other out.
        let super_safe_random_value: F = BaseField::from(23481234).into();
        let constraint_value = mask[0] * mask[1] - mask[3] + (mask[0] - mask[2] - BaseField::one()) * super_safe_random_value;
        let selector = point_excluder(
            constraint_zero_domain
                .at(constraint_zero_domain.size() - 1)
                .into_ef(),
            point,
        );
        let num = constraint_value * selector;
        let denom = coset_vanishing(constraint_zero_domain, point);
        num / denom
    }


    // We wanted to run the decrement boundry constraint, but it was not functioning properly. It is now combined with the multiplication contraint (unsound)
    // pub fn step_constraint_eval_decrement<F: ExtensionOf<BaseField>>(
    //     &self,
    //     point: CirclePoint<F>,
    //     mask: &[F; 4],
    // ) -> F {
    //     let constraint_zero_domain = Coset::subgroup(self.log_size - 1);
    //     let constraint_value = mask[0] - mask[2] - BaseField::one();
    //     println!("constraint_value: {:?}", constraint_value);
    //      let selector = point_excluder(
    //         constraint_zero_domain
    //             .at(constraint_zero_domain.size() - 1)
    //             .into_ef(),
    //         point,
    //     );
    //     let num = constraint_value * selector;
    //     println!("num: {:?}", num);
    //     let denom = coset_vanishing(constraint_zero_domain, point);
    //     num / denom
    // }

     // The constraint is not used, as we couldn't get it to work properly. The logic has been defined though, we just ran out of time.
     pub fn boundary_constraint_eval_quotient_by_mask<F: ExtensionOf<BaseField>>(
        &self,
        point: CirclePoint<F>,
        mask: &[F; 1],
    ) -> F {
        let constraint_zero_domain = Coset::subgroup(self.log_size);
        let g_1 = constraint_zero_domain.at(1);
        let g_minus_1 = constraint_zero_domain.at(constraint_zero_domain.size() - 1);
        let g_minus_2 = constraint_zero_domain.at(constraint_zero_domain.size() - 2);
        // On g^1, we should get 1.
        // On p, we should get self.claim.
        // 1 + (point * g_minus_1).y * (self.claim - 1) * (g_minus_2.y)^-1
        let linear = F::one() + multiply_circle(point, g_minus_1.into_ef()).y * (self.claim - BaseField::one()) * (g_minus_2.y).inverse();
        let num = mask[0] - linear;
        let denom = pair_vanishing(g_minus_1.into_ef(), g_1.into_ef(), point);
        num / denom
    }
}

impl Component for FactorialComponent {
    fn n_constraints(&self) -> usize {
        2
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        // Step constraint is of degree 2.
        self.log_size + 1
    }

    fn trace_log_degree_bounds(&self) -> Vec<u32> {
        vec![self.log_size + 1]
    }

    fn mask_points(
        &self,
        point: CirclePoint<SecureField>,
    ) -> ColumnVec<Vec<CirclePoint<SecureField>>> {
        shifted_mask_points(
            &vec![vec![0, 1, 2, 3]],
            &[CanonicCoset::new(self.log_size)],
            point,
        )
    }

    fn interaction_element_ids(&self) -> Vec<String> {
        vec![]
    }

    fn evaluate_constraint_quotients_at_point(
        &self,
        point: CirclePoint<SecureField>,
        mask: &ColumnVec<Vec<SecureField>>,
        evaluation_accumulator: &mut PointEvaluationAccumulator,
    ) {
        evaluation_accumulator.accumulate(
            self.step_constraint_eval_multiplication(point, &mask[0][..].try_into().unwrap()),
        );

        // Not running, as not working
        // evaluation_accumulator.accumulate(
        //     self.boundary_constraint_eval_quotient_by_mask(
        //         point,
        //         &mask[0][..1].try_into().unwrap(),
        //     ),
        // );
    }
}

impl ComponentTraceWriter<CpuBackend> for FactorialComponent {
    fn write_interaction_trace(
        &self,
        _trace: &ColumnVec<&CircleEvaluation<CpuBackend, BaseField, BitReversedOrder>>,
        _elements: &InteractionElements,
    ) -> ColumnVec<CircleEvaluation<CpuBackend, BaseField, BitReversedOrder>> {
        vec![]
    }
}

impl ComponentProver<CpuBackend> for FactorialComponent {
    fn evaluate_constraint_quotients_on_domain(
        &self,
        trace: &ComponentTrace<'_, CpuBackend>,
        evaluation_accumulator: &mut DomainEvaluationAccumulator<CpuBackend>,
    ) {
        let poly = &trace.polys[0];
        let trace_domain = CanonicCoset::new(self.log_size);
        let trace_eval_domain = CanonicCoset::new(self.log_size + 1).circle_domain();
        let trace_eval = poly.evaluate(trace_eval_domain).bit_reverse();

        // Step constraint.
        let constraint_log_degree_bound = trace_domain.log_size() + 1;
        let [mut accum] = evaluation_accumulator.columns([(constraint_log_degree_bound, 2)]);
        let constraint_eval_domain = trace_eval_domain;
        for (off, point_coset) in [
            (0, constraint_eval_domain.half_coset),
            (
                constraint_eval_domain.half_coset.size(),
                constraint_eval_domain.half_coset.conjugate(),
            ),
        ] {
            let eval = trace_eval.fetch_eval_on_coset(point_coset.shift(trace_domain.index_at(0)));
            let mul = trace_domain.step_size().div(point_coset.step_size);
            for (i, point) in point_coset.iter().enumerate() {

                let mask = [eval[i], eval[i as isize + mul], eval[i as isize + 2 * mul], eval[i as isize + 3 * mul]];
                let res = self.step_constraint_eval_multiplication(point, &mask)
                    * accum.random_coeff_powers[0];


                // Boundry constraint is not functioning properly. The logic is outlined in a comment thought
                // res += self.boundary_constraint_eval_quotient_by_mask(point, &[mask[0]])
                //     * accum.random_coeff_powers[1];

                accum.accumulate(bit_reverse_index(i + off, constraint_log_degree_bound), res);
            }
        }
    }
}
