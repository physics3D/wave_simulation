use rayon::prelude::*;

use three::{material::Wireframe, DynamicMesh, Factory, Geometry, Shape};

use crate::{
    particle::{Particle, ParticleType},
    GRID_SCALE, HEIGHT, WIDTH,
};

pub type ParticleGrid = [[Particle; WIDTH as usize]; HEIGHT as usize];

pub struct WaveSimulation {
    pub particles: ParticleGrid,
    pub mesh: DynamicMesh,
}

impl WaveSimulation {
    pub fn new(mut factory: &mut Factory) -> Self {
        let mut particles = [[Particle::new(); WIDTH as usize]; HEIGHT as usize];

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let mut particle = &mut particles[x as usize][y as usize];
                particle.pos.x = (x as f32 - (WIDTH / 2) as f32) / WIDTH as f32 * GRID_SCALE as f32;
                particle.pos.z =
                    (y as f32 - (HEIGHT / 2) as f32) / HEIGHT as f32 * GRID_SCALE as f32;

                particle.index_x = x as usize;
                particle.index_y = y as usize;
            }
        }

        let mesh = Self::make_mesh(&mut factory, particles);

        Self { particles, mesh }
    }

    fn make_mesh(factory: &mut Factory, particles: ParticleGrid) -> DynamicMesh {
        let geometry = Self::construct_geometry(particles);
        let material = Wireframe { color: 0x00FFFF };
        factory.mesh_dynamic(geometry, material)
    }

    fn construct_geometry(particles: ParticleGrid) -> Geometry {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        // vertices
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let pos = particles[x as usize][y as usize].pos;
                vertices.push(pos);
            }
        }

        // faces (indeces)
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if 0 < x && 0 < y {
                    let a = (x * HEIGHT + y) as u32;
                    let b = (x * HEIGHT + y - 1) as u32;
                    let c = ((x - 1) * HEIGHT + y) as u32;
                    let arr = [a, b, c];
                    faces.push(arr);
                }

                if x < WIDTH && y < HEIGHT - 1 {
                    let a = (x * HEIGHT + y) as u32;
                    let b = (x * HEIGHT + y + 1) as u32;
                    let c = ((x + 1) * HEIGHT + y) as u32;
                    let arr = [a, b, c];
                    faces.push(arr);
                }
            }
        }

        Geometry {
            faces,
            base: Shape {
                vertices,
                ..Shape::default()
            },
            ..Geometry::default()
        }
    }

    pub fn update(&mut self) {
        let particles_old = self.particles.clone();

        // for x in 1..(WIDTH - 1) {
        //     for y in 1..(HEIGHT - 1) {
        //         particles[x as usize][y as usize].update(&particles_old, &delta_time);
        //     }
        // }

        self.particles
            .par_iter_mut() // this runs in parallel
            .for_each(|particle_slice| {
                for particle in particle_slice {
                    particle.update(&particles_old);
                }
            });
    }

    pub fn update_mesh(&mut self, factory: &mut Factory) {
        // // recreate mesh
        // win.scene.remove(mesh);
        // mesh = {
        //     let geometry = make_mesh(&particles);
        //     let material = three::material::Wireframe { color: 0x00FFFF };
        //     win.factory.mesh(geometry, material)
        // };
        // win.scene.add(&mesh);

        // write particle heights to mesh
        let mut vmap = factory.map_vertices(&mut self.mesh);
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let pos = self.particles[x as usize][y as usize].pos;
                // println!("{}", pos.y);
                let index = x * WIDTH + y;
                vmap[index as usize].pos = [pos.x, pos.y, pos.z, 1.0];
            }
        }
    }

    pub fn set_borders(&mut self, border_type: ParticleType) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if x == 0 || x == WIDTH - 1 || y == 0 || y == HEIGHT - 1 {
                    self.particles[x as usize][y as usize].particle_type = border_type;
                }
            }
        }
    }
}
