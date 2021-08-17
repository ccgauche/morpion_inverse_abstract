#![feature(array_methods)]
#![feature(crate_visibility_modifier)]

pub use self::layer_topology::*;

use self::{layer::*, neuron::*};
use rand::{prelude::ThreadRng, Rng, RngCore};
use std::iter::once;

mod layer;
mod layer_topology;
mod neuron;
pub mod nlib;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Network {
    layers: Vec<Layer>,
}

impl Network {
    crate fn new(layers: Vec<Layer>) -> Self {
        Self { layers }
    }

    pub fn save(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn load(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }

    pub fn random(rng: &mut ThreadRng, layers: &[LayerTopology]) -> Self {
        assert!(layers.len() > 1);

        let layers = layers
            .windows(2)
            .map(|layers| Layer::random(rng, layers[0].0, layers[1].0))
            .collect();

        Self::new(layers)
    }

    pub fn from_weights(layers: &[LayerTopology], weights: impl IntoIterator<Item = f32>) -> Self {
        assert!(layers.len() > 1);

        let mut weights = weights.into_iter();

        let layers = layers
            .windows(2)
            .map(|layers| Layer::from_weights(layers[0].0, layers[1].0, &mut weights))
            .collect();

        if weights.next().is_some() {
            panic!("got too many weights");
        }

        Self::new(layers)
    }

    pub fn propagate(&self, inputs: Vec<f32>) -> Vec<f32> {
        self.layers
            .iter()
            .fold(inputs, |inputs, layer| layer.propagate(inputs))
    }

    pub fn mutate(&self, mutation_ratio: f64) -> Self {
        Self {
            layers: self
                .layers
                .iter()
                .map(|x| x.mutate(mutation_ratio))
                .collect(),
        }
    }

    pub fn weights(&self) -> impl Iterator<Item = f32> + '_ {
        self.layers
            .iter()
            .flat_map(|layer| layer.neurons.iter())
            .flat_map(|neuron| once(&neuron.bias).chain(&neuron.weights))
            .cloned()
    }
}
