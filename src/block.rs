use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct BlockCoordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(
    Debug, PartialEq, Copy, Clone, Hash, Eq, Default, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct BlockIndex {
    pub i: usize,
    pub j: usize,
    pub k: usize,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct BlockSize {
    pub x_size: f32,
    pub y_size: f32,
    pub z_size: f32,
}
pub trait BlockInterface: Clone + PartialEq + for<'a> Deserialize<'a> {
    //coordinates of block in space
    fn coordinates(&self) -> BlockCoordinates;

    //dimensions of block
    fn size(&self) -> BlockSize;

    //index
    fn index(&self) -> BlockIndex;
    fn set_index(&mut self, ind: BlockIndex);
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum BlockAttributes {
    FLOAT(f32),
    INT(i32),
    LABEL(String),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Block {
    #[serde(default)]
    pub ind: BlockIndex,
    pub coords: BlockCoordinates,
    pub size: BlockSize,
    pub attributes: HashMap<String, BlockAttributes>,
}

impl Block {
    pub fn new(
        ind: BlockIndex,
        coords: BlockCoordinates,
        size: BlockSize,
        attributes: HashMap<String, BlockAttributes>,
    ) -> Self {
        Self {
            ind,
            coords,
            size,
            attributes,
        }
    }
}

impl BlockInterface for Block {
    fn coordinates(&self) -> BlockCoordinates {
        self.coords
    }

    fn size(&self) -> BlockSize {
        self.size
    }

    fn index(&self) -> BlockIndex {
        self.ind
    }

    fn set_index(&mut self, ind: BlockIndex) {
        self.ind = ind;
    }
}
