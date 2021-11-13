use std::usize;

use mint::Point3;

use crate::{
    wave_simulation::ParticleGrid, ACC_LIMIT, DAMPING, HEIGHT_LIMIT, OSZILLATOR_SPEED,
    SUSTAINABILITY, VEL_LIMIT,
};

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum ParticleType {
    // normal water
    Fluid,
    // particle swinging in a sine wave (the f32 parameter is the time)
    Oszillator(f32),
    // reflective border
    Solid,
    // border simulating an infinite ocean
    Infinity,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: Point3<f32>,
    vel: f32,
    acc: f32,
    pub index_x: usize,
    pub index_y: usize,
    pub particle_type: ParticleType,
}

impl Particle {
    pub fn new() -> Self {
        Particle {
            pos: Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vel: 0.0,
            acc: 0.0,
            index_x: 0,
            index_y: 0,
            particle_type: ParticleType::Fluid,
        }
    }

    fn get_neighbors_average(&self, grid_old: &ParticleGrid) -> f32 {
        let mut neighbors = Vec::new();

        // we have to use i32 for x and y for negative numbers
        for x in (self.index_x as i32 - 1)..=(self.index_x as i32 + 1) {
            for y in (self.index_y as i32 - 1)..=(self.index_y as i32 + 1) {
                let slice = grid_old.get(x as usize);
                if slice.is_some() {
                    let particle = slice.unwrap().get(y as usize);
                    if particle.is_some() {
                        let height = particle.unwrap().pos.y;
                        neighbors.push(height);
                    }
                }
            }
        }

        let average: f32 =
            (neighbors.iter().sum::<f32>() - self.pos.y) / (neighbors.len() - 1) as f32;

        average
    }

    // using a reference for grid_old makes this VERY much faster
    pub fn update(&mut self, grid_old: &ParticleGrid) {
        if self.particle_type == ParticleType::Fluid {
            let average = self.get_neighbors_average(grid_old);

            self.acc = average - self.pos.y - (self.vel / SUSTAINABILITY);
            self.acc = self.acc.max(-ACC_LIMIT).min(ACC_LIMIT);
            self.acc *= DAMPING;

            self.vel += self.acc;
            self.vel = self.vel.max(-VEL_LIMIT).min(VEL_LIMIT);

            self.pos.y += self.vel;
            self.pos.y = self.pos.y.max(-HEIGHT_LIMIT).min(HEIGHT_LIMIT);
        } else if let ParticleType::Oszillator(time) = self.particle_type {
            self.pos.y = time.sin() * HEIGHT_LIMIT;
            self.particle_type = ParticleType::Oszillator(time + OSZILLATOR_SPEED);
        } else if self.particle_type == ParticleType::Infinity {
            self.pos.y = self.get_neighbors_average(grid_old);
        }
    }
}
