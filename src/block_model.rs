use bevy::{render::primitives::Aabb, utils::HashMap};
use colorgrad::Gradient;
use itertools::izip;
use ordered_float::OrderedFloat;
use polars::datatypes::DataType;
use polars::prelude::DataFrame;

use bevy::prelude::*;
use bevy::render::color::Color;
use bevy_aabb_instancing::{Cuboid, Cuboids};

#[derive(Resource, Default, Clone)]
pub struct BlockModelResource {
    pub block_model: Option<BlockModel>,
}

#[derive(Resource, Default)]
pub struct BlockModelDB {
    pub block_models: HashMap<String, BlockModel>,
}

#[derive(Clone)]
pub struct BlockModel {
    pub name: String,
    pub df: DataFrame,
    pub columns: Vec<String>,
    pub x: String,
    pub y: String,
    pub z: String,
    pub x_size: String,
    pub y_size: String,
    pub z_size: String,
}

impl BlockModel {
    pub fn new(
        name: String,
        df: DataFrame,
        x: String,
        y: String,
        z: String,
        x_size: String,
        y_size: String,
        z_size: String,
    ) -> Self {
        let columns = df
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        Self {
            name,
            df,
            columns,
            x,
            y,
            z,
            x_size,
            y_size,
            z_size,
        }
    }

    pub fn mesh_material(&self, column: String, cmap: Gradient) -> Vec<(Mesh, Color)> {
        fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
            to_range.0
                + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
        }
        let mut bundles = Vec::new();

        let binding = self
            .df
            .column(self.x.as_str())
            .expect("Missing X column")
            .cast(&DataType::Float32)
            .expect("X of incorrect datatype");
        let x_values = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.y.as_str())
            .expect("Missing Y column")
            .cast(&DataType::Float32)
            .expect("Y of incorrect datatype");
        let y_values = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.z.as_str())
            .expect("Missing Z column")
            .cast(&DataType::Float32)
            .expect("Z of incorrect datatype");
        let z_values = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.x_size.as_str())
            .expect("Missing X size column")
            .cast(&DataType::Float32)
            .expect("X size of incorrect datatype");
        let x_size = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.y_size.as_str())
            .expect("Missing Y size column")
            .cast(&DataType::Float32)
            .expect("Y size of incorrect datatype");
        let y_size = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.z_size.as_str())
            .expect("Missing Z size column")
            .cast(&DataType::Float32)
            .expect("Z size of incorrect datatype");
        let z_size = binding.f32().unwrap();

        let binding = self
            .df
            .column(column.as_str())
            .expect(format!("Missing {} column", column).as_str())
            .cast(&DataType::Float32)
            .expect("column of incorrect datatype");
        let column_values = binding.f32().unwrap();

        let min_value = column_values
            .into_iter()
            .filter_map(|v| {
                if v.is_none() {
                    return None;
                }
                Some(OrderedFloat(v.unwrap() as f64))
            })
            .min()
            .unwrap();
        let max_value = column_values
            .into_iter()
            .filter_map(|v| {
                if v.is_none() {
                    return None;
                }
                Some(OrderedFloat(v.unwrap() as f64))
            })
            .max()
            .unwrap();

        for (x, y, z, x_size, y_size, z_size, value) in izip!(
            x_values,
            y_values,
            z_values,
            x_size,
            y_size,
            z_size,
            column_values
        ) {
            if x.is_none()
                || y.is_none()
                || z.is_none()
                || x_size.is_none()
                || y_size.is_none()
                || z_size.is_none()
                || value.is_none()
            {
                continue;
            }

            let x = x.unwrap() as f32;
            let y = y.unwrap() as f32;
            let z = z.unwrap() as f32;
            let x_size = x_size.unwrap() as f32;
            let y_size = y_size.unwrap() as f32;
            let z_size = z_size.unwrap() as f32;
            let value = value.unwrap() as f32;

            let mapped_value = map_range((min_value.0, max_value.0), (0.0, 1.0), value as f64);

            let color = cmap.at(mapped_value);

            bundles.push((
                Mesh::from(shape::Box {
                    min_x: x,
                    max_x: x + x_size,
                    min_y: y,
                    max_y: y + y_size,
                    min_z: z,
                    max_z: z + z_size,
                }),
                Color::rgb(color.r as f32, color.g as f32, color.b as f32).into(),
            ));
        }

        bundles
    }

    pub fn aabb_instances(
        &self,
        column: String,
        cmap: Gradient,
        patch_size: usize,
    ) -> Vec<(Cuboids, Aabb)> {
        fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
            to_range.0
                + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
        }

        let binding = self
            .df
            .column(self.x.as_str())
            .expect("Missing X column")
            .cast(&DataType::Float32)
            .expect("X of incorrect datatype");
        let x_values = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.y.as_str())
            .expect("Missing Y column")
            .cast(&DataType::Float32)
            .expect("Y of incorrect datatype");
        let y_values = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.z.as_str())
            .expect("Missing Z column")
            .cast(&DataType::Float32)
            .expect("Z of incorrect datatype");
        let z_values = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.x_size.as_str())
            .expect("Missing X size column")
            .cast(&DataType::Float32)
            .expect("X size of incorrect datatype");
        let x_size = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.y_size.as_str())
            .expect("Missing Y size column")
            .cast(&DataType::Float32)
            .expect("Y size of incorrect datatype");
        let y_size = binding.f32().unwrap();

        let binding = self
            .df
            .column(self.z_size.as_str())
            .expect("Missing Z size column")
            .cast(&DataType::Float32)
            .expect("Z size of incorrect datatype");
        let z_size = binding.f32().unwrap();

        let binding = self
            .df
            .column(column.as_str())
            .expect(format!("Missing {} column", column).as_str())
            .cast(&DataType::Float32)
            .expect("column of incorrect datatype");
        let column_values = binding.f32().unwrap();

        let min_value = column_values
            .into_iter()
            .filter_map(|v| {
                if v.is_none() {
                    return None;
                }
                Some(OrderedFloat(v.unwrap() as f64))
            })
            .min()
            .unwrap();
        let max_value = column_values
            .into_iter()
            .filter_map(|v| {
                if v.is_none() {
                    return None;
                }
                Some(OrderedFloat(v.unwrap() as f64))
            })
            .max()
            .unwrap();

        let mut all_cuboids = Vec::new();
        let mut instances = Vec::with_capacity(patch_size);
        for (x, y, z, x_size, y_size, z_size, value) in izip!(
            x_values,
            y_values,
            z_values,
            x_size,
            y_size,
            z_size,
            column_values
        ) {
            if x.is_none()
                || y.is_none()
                || z.is_none()
                || x_size.is_none()
                || y_size.is_none()
                || z_size.is_none()
                || value.is_none()
            {
                continue;
            }

            let x = x.unwrap() as f32;
            let y = y.unwrap() as f32;
            let z = z.unwrap() as f32;
            let x_size = x_size.unwrap() as f32;
            let y_size = y_size.unwrap() as f32;
            let z_size = z_size.unwrap() as f32;
            let value = value.unwrap() as f32;

            let mapped_value = map_range((min_value.0, max_value.0), (0.0, 1.0), value as f64);

            let color = cmap.at(mapped_value);

            let encoded_color =
                Color::rgb(color.r as f32, color.g as f32, color.b as f32).as_rgba_u32();

            let minimum = Vec3::new(x, y, z);
            let maximum = Vec3::new(x + x_size, y + y_size, z + z_size);

            let cuboid = Cuboid::new(minimum, maximum, encoded_color);

            instances.push(cuboid);

            if instances.len() % patch_size == 0 {
                let cuboids = Cuboids::new(instances.clone());
                let aabb = cuboids.aabb();
                all_cuboids.push((cuboids, aabb));
            }
        }

        all_cuboids
    }
}
