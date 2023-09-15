use bevy::prelude::Resource;
use ndarray::Array3;

use mining_width_maintainer::mining_width_maintainer::MiningWidthMaintainer;

#[derive(Default, Resource)]
pub struct OptimizeParams {
    pub grade_col: String,
    pub tonnage_col: String,
    pub discount_rate: f32,
    pub cutoff: f32,
    pub min_life: f32,
    pub proc_cap: f32,
    pub mining_rate: f32,
    pub metal_price: f32,
    pub mining_cost: f32,
    pub processing_cost: f32,
    pub num_iters: usize,
}

pub struct Optimizer {
    pub params: OptimizeParams,
    pub tonnage: Array3<f32>,
    pub grade: Array3<f32>,
    pub sched: MiningWidthMaintainer,
}
