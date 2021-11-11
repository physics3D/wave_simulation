use core::f32;
use std::usize;

use crate::{ACC_LIMIT, DAMPING, HEIGHT, HEIGHT_LIMIT, SUSTAINABILITY, VEL_LIMIT, WIDTH};

pub fn clampf(num: &mut f32, min: f32, max: f32) {
    if *num < min {
        *num = min;
    } else if *num > max {
        *num = max;
    }
}
pub fn clampi(num: &mut i32, min: i32, max: i32) {
    if *num < min {
        *num = min;
    } else if *num > max {
        *num = max;
    }
}

#[derive(Copy, Clone)]
pub struct WaterParticle {
    pub pos: mint::Point3<f32>,
    vel: f32,
    acc: f32,
    pub index_x: usize,
    pub index_y: usize,
}

impl WaterParticle {
    pub fn new() -> Self {
        WaterParticle {
            pos: mint::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vel: 0.0,
            acc: 0.0,
            index_x: 0,
            index_y: 0,
        }
    }

    pub fn update(&mut self, grid_old: &[[WaterParticle; WIDTH as usize]; HEIGHT as usize]) {
        let mut neighbors = Vec::new();

        for x in (self.index_x - 1)..=(self.index_x + 1) {
            for y in (self.index_y - 1)..=(self.index_y + 1) {
                neighbors.push(grid_old[x as usize][y as usize].pos.y);
            }
        }

        let average: f32 =
            (neighbors.iter().sum::<f32>() - self.pos.y) / (neighbors.len() - 1) as f32;
        self.acc = average - self.pos.y - (self.vel / SUSTAINABILITY);
        clampf(&mut self.acc, -ACC_LIMIT, ACC_LIMIT);
        self.acc *= DAMPING;
        self.vel += self.acc;
        clampf(&mut self.vel, -VEL_LIMIT, VEL_LIMIT);
        self.pos.y += self.vel;
        clampf(&mut self.pos.y, -HEIGHT_LIMIT, HEIGHT_LIMIT);
    }
}
