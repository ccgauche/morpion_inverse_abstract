use std::ops::{Add, Mul};

use nalgebra::SMatrix;
use rand::Rng;

pub type LayerResult<const CSIZE: usize> = SMatrix<f32, 1, CSIZE>;

/* impl<const CSIZE: usize> From<[f32; CSIZE]> for LayerResult<CSIZE> {
    fn from(a: [f32; CSIZE]) -> Self {
        LayerResult(SMatrix::from_column_slice(&a))
    }
} */

/* impl<const CSIZE: usize> Into<[f32; CSIZE]> for LayerResult<CSIZE> {
    fn into(self) -> [f32; CSIZE] {
        self.0
            .into_iter()
            .copied()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
} */

#[derive(Clone)]
pub struct Layer<const PVS: usize, const CSIZE: usize> {
    pub neurons: SMatrix<f32, 1, CSIZE>,
    pub weights: SMatrix<f32, PVS, CSIZE>,
}

impl<const PVS: usize, const CSIZE: usize> Layer<PVS, CSIZE> {
    pub fn execute(&self, values: LayerResult<PVS>) -> LayerResult<CSIZE> {
        values
            .mul(self.weights)
            .add(self.neurons)
            .map(|x| x.max(0.))
    }

    pub fn random() -> Self {
        let mut rnd = rand::thread_rng();
        Self {
            neurons: SMatrix::<f32, 1, CSIZE>::zeros().map(|_| rnd.gen_range(-1.0..=1.0)),
            weights: SMatrix::<f32, PVS, CSIZE>::zeros().map(|_| rnd.gen_range(-1.0..=1.0)),
        }
    }

    pub fn mutate(&self) -> Self {
        Self {
            neurons: self.neurons.map(mutval),
            weights: self.weights.map(mutval),
        }
    }
}

fn mutval(val: f32) -> f32 {
    if rand::thread_rng().gen_bool(0.2) {
        val
    } else {
        (val + rand::thread_rng().gen_range(-1.0..1.))
            .max(0.)
            .min(1.)
    }
}
