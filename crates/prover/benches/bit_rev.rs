#![feature(iter_array_chunks)]

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use itertools::Itertools;
use stwo_prover::core::fields::m31::BaseField;

pub fn cpu_bit_rev(c: &mut Criterion) {
    use stwo_prover::core::utils::bit_reverse;
    // TODO(andrew): Consider using same size for all.
    const SIZE: usize = 1 << 24;
    let data = (0..SIZE).map(BaseField::from).collect_vec();
    c.bench_function("cpu bit_rev 24bit", |b| {
        b.iter_batched(
            || data.clone(),
            |mut data| bit_reverse(&mut data),
            BatchSize::LargeInput,
        );
    });
}

criterion_group!(
    name = bit_rev;
    config = Criterion::default().sample_size(10);
    targets = cpu_bit_rev);
criterion_main!(bit_rev);
