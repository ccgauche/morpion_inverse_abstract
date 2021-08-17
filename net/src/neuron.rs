use crate::*;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Neuron {
    crate bias: f32,
    crate weights: Vec<f32>,
}

impl Neuron {
    pub fn new(bias: f32, weights: Vec<f32>) -> Self {
        assert!(!weights.is_empty());

        Self { bias, weights }
    }

    pub fn random(rng: &mut dyn RngCore, output_neurons: usize) -> Self {
        let bias = rng.gen_range(-1.0..=1.0);

        let weights = (0..output_neurons)
            .map(|_| rng.gen_range(-1.0..=1.0))
            .collect();

        Self::new(bias, weights)
    }

    pub fn from_weights(output_neurons: usize, weights: &mut dyn Iterator<Item = f32>) -> Self {
        let bias = weights.next().expect("got not enough weights");

        let weights = (0..output_neurons)
            .map(|_| weights.next().expect("got not enough weights"))
            .collect();

        Self::new(bias, weights)
    }

    pub fn propagate(&self, inputs: &[f32]) -> f32 {
        assert_eq!(inputs.len(), self.weights.len());

        let output = inputs
            .iter()
            .zip(&self.weights)
            .map(|(input, weight)| input * weight)
            .sum::<f32>();

        (self.bias + output).max(0.0)
    }

    pub fn mutate(&self, mutation_ratio: f64) -> Self {
        Self {
            bias: if rand::thread_rng().gen_bool(mutation_ratio) {
                rand::thread_rng().gen_range(-1.0..=1.0)
            } else {
                self.bias
            },
            weights: self
                .weights
                .iter()
                .map(|x| {
                    if rand::thread_rng().gen_bool(mutation_ratio) {
                        rand::thread_rng().gen_range(-1.0..=1.0)
                    } else {
                        *x
                    }
                })
                .collect(),
        }
    }
}
