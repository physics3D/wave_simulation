// extern crate cgmath;
extern crate mint;
extern crate three;

use std::usize;

use rayon::prelude::*;

mod wave_simulation;

use crate::wave_simulation::clampi;
use crate::wave_simulation::WaterParticle;

const WIDTH: i32 = 200;
const HEIGHT: i32 = 200;
const GRID_SCALE: f32 = 10.0;
const MOUSE_SIZE: i32 = 2;
const MOUSE_HEIGHT: f32 = HEIGHT_LIMIT / 5.0;

const ACC_LIMIT: f32 = 1000.0;
const VEL_LIMIT: f32 = 1000.0;
const HEIGHT_LIMIT: f32 = 2.0;
const DAMPING: f32 = 0.995;
const SUSTAINABILITY: f32 = 100.0;

fn make_mesh(particles: &[[WaterParticle; HEIGHT as usize]; WIDTH as usize]) -> three::Geometry {
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

    three::Geometry {
        faces,
        base: three::Shape {
            vertices,
            ..three::Shape::default()
        },
        ..three::Geometry::default()
    }
}

fn main() {
    let mut win = three::Window::new("Three-rs wave simulation");
    // win.set_fullscreen(true);

    let cam = win.factory.perspective_camera(60.0, 1.0..1000.0);
    let mut controls = three::controls::Orbit::builder(&cam)
        .position([0.0, -4.0, 10.0])
        .target([0.0, 0.0, 0.0])
        .build();

    let mut particles = [[WaterParticle::new(); HEIGHT as usize]; WIDTH as usize];
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let particle = &mut particles[x as usize][y as usize];
            particle.pos = mint::Point3 {
                x: (x as f32 - (WIDTH / 2) as f32) / WIDTH as f32 * GRID_SCALE,
                y: 0.0,
                z: (y as f32 - (HEIGHT / 2) as f32) / HEIGHT as f32 * GRID_SCALE,
            };
            particle.index_x = x as usize;
            particle.index_y = y as usize;
        }
    }

    for x in (WIDTH / 2 - 1)..=(WIDTH / 2 + 1) {
        for y in (HEIGHT / 2 - 1)..=(HEIGHT / 2 + 1) {
            particles[x as usize][y as usize].pos.y = HEIGHT_LIMIT;
        }
    }

    let mut mesh = {
        let geometry = make_mesh(&particles);
        let material = three::material::Wireframe { color: 0x00FFFF };
        win.factory.mesh_dynamic(geometry, material)
    };
    win.scene.add(&mesh);

    let font = win.factory.load_font_karla();
    let mut ui_text = win.factory.ui_text(&font, "");

    #[derive(PartialEq)]
    enum Mode {
        Control,
        Camera,
    }

    let mut mode = Mode::Control;

    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        // set control or camera mode
        if win.input.hit(three::KEY_SPACE) {
            if mode == Mode::Camera {
                mode = Mode::Control;
            } else {
                mode = Mode::Camera;
            }
        }

        // Update particles
        let particles_old = particles.clone();

        // for x in 1..(WIDTH - 1) {
        //     for y in 1..(HEIGHT - 1) {
        //         particles[x as usize][y as usize].update(&particles_old, &delta_time);
        //     }
        // }

        particles
            .par_iter_mut() // this runs in parallel
            .enumerate()
            .for_each(|(index, particle_slice)| {
                if !(index == 0 || index == WIDTH as usize || index == (WIDTH - 1) as usize) {
                    for i in 1..(HEIGHT - 1) {
                        particle_slice[i as usize].update(&particles_old);
                    }
                }
            });

        // // recreate mesh
        // win.scene.remove(mesh);
        // mesh = {
        //     let geometry = make_mesh(&particles);
        //     let material = three::material::Wireframe { color: 0x00FFFF };
        //     win.factory.mesh(geometry, material)
        // };
        // win.scene.add(&mesh);

        // write particle heights to mesh
        // new scope because of mutable win borrow for vmap
        {
            let mut vmap = win.factory.map_vertices(&mut mesh);
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    let pos = particles[x as usize][y as usize].pos;
                    // println!("{}", pos.y);
                    let index = x * WIDTH + y;
                    vmap[index as usize].pos = [pos.x, pos.y, pos.z, 1.0];
                }
            }
        }

        if mode == Mode::Camera {
            controls.update(&win.input);
        } else {
            if win.input.hit(three::MouseButton::Left) {
                let mouse_pos_ndc = win.input.mouse_pos();
                let mut mouse_x = (mouse_pos_ndc.x / win.size().x * WIDTH as f32) as i32;
                clampi(&mut mouse_x, MOUSE_SIZE + 1, WIDTH - MOUSE_SIZE - 2);
                let mut mouse_y = (mouse_pos_ndc.y / win.size().y * HEIGHT as f32) as i32;
                clampi(&mut mouse_y, MOUSE_SIZE + 1, HEIGHT - MOUSE_SIZE - 2);

                // particles[mouse_x as usize][mouse_y as usize].pos.y = HEIGHT_LIMIT;

                // set multiple particles

                for x in (mouse_x - MOUSE_SIZE) as usize..=(mouse_x + MOUSE_SIZE) as usize {
                    for y in (mouse_y - MOUSE_SIZE) as usize..=(mouse_y + MOUSE_SIZE) as usize {
                        particles[x][y].pos.y = MOUSE_HEIGHT;
                    }
                }
            }
        }

        let text = "FPS: ".to_string() + &(1.0 / win.input.delta_time()).ceil().to_string();
        ui_text.set_text(text);

        win.render(&cam);
    }
}
