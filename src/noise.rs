use bevy::{ecs::reflect, prelude::Resource, reflect::Reflect};
use bevy_inspector_egui::prelude::*;
use noise::{Fbm, NoiseFn, Perlin};

use crate::rtin::PlaneSampler;

#[derive(Debug, Resource)]
pub struct NoiseSampler {
    layers: Vec<Fbm<Perlin>>,
}

impl PlaneSampler for NoiseSampler {
    fn get(&self, x: f32, y: f32) -> f32 {
        let mut res = 0.0;
        for layer in self.layers.iter() {
            res += layer.get([x as f64, y as f64]);
        }
        res as f32
    }
}

impl NoiseSampler {
    pub fn single_layer(layer: Fbm<Perlin>) -> Self {
        Self {
            layers: vec![layer],
        }
    }
    pub fn new(layers: Vec<Fbm<Perlin>>) -> Self {
        Self { layers }
    }
    pub fn add_layer(&mut self, layer: Fbm<Perlin>) {
        self.layers.push(layer)
    }
}
