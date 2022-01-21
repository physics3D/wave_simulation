use std::usize;

mod particle;
mod wave_simulation;

use mint::{Point2, Vector2};
use three::{controls::Orbit, Key::B, Key::R, MouseButton, Window, KEY_ESCAPE, KEY_SPACE};

use crate::{particle::ParticleType, wave_simulation::WaveSimulation};

const HELP_TEXT: &str = r"Controls:
LEFT MOUSE / MOUSE WHEEL - move camera
LEFT MOUSE - draw on the fluid mesh
RIGHT MOUSE - set a fluid particle as an oszillator
SPACE - switch between camera and mouse mode
B - switch between reflective and infinite borders
R - reset all oszillators to fluid";

const WIDTH: i32 = 200;
const HEIGHT: i32 = 200;
const GRID_SCALE: i32 = 10;
const MOUSE_SIZE: i32 = 2;
const MOUSE_HEIGHT: f32 = HEIGHT_LIMIT / 5.0;

const ACC_LIMIT: f32 = 1000.0;
const VEL_LIMIT: f32 = 1000.0;
const HEIGHT_LIMIT: f32 = 2.0;
const DAMPING: f32 = 0.995;
const SUSTAINABILITY: f32 = 100.0;

const OSZILLATOR_SPEED: f32 = 0.2;

fn get_normalised_mouse_pos(mouse_pos: Point2<f32>, win_size: Vector2<f32>) -> (i32, i32) {
    let mut mouse_x = WIDTH - (mouse_pos.x / win_size.x * WIDTH as f32) as i32;
    mouse_x = mouse_x.max(MOUSE_SIZE).min(WIDTH - MOUSE_SIZE - 1);
    let mut mouse_y = HEIGHT - (mouse_pos.y / win_size.y * HEIGHT as f32) as i32;
    mouse_y = mouse_y.max(MOUSE_SIZE).min(HEIGHT - MOUSE_SIZE - 1);

    (mouse_x, mouse_y)
}

fn main() {
    let mut win = Window::new("Three-rs wave simulation");
    // win.set_fullscreen(true);

    let cam = win.factory.perspective_camera(60.0, 1.0..1000.0);
    let mut controls = Orbit::builder(&cam)
        .position([0.0, 4.0, -10.0])
        .target([0.0, 0.0, 0.0])
        .build();

    let mut simulation = WaveSimulation::new(&mut win.factory);

    // raise a few initial particles
    for x in (WIDTH / 2 - 1)..=(WIDTH / 2 + 1) {
        for y in (HEIGHT / 2 - 1)..=(HEIGHT / 2 + 1) {
            simulation.particles[x as usize][y as usize].pos.y = HEIGHT_LIMIT;
        }
    }

    win.scene.add(&simulation.mesh);

    let font = win.factory.load_font_karla();
    let mut ui_text = win.factory.ui_text(&font, "");

    #[derive(PartialEq)]
    enum Mode {
        Control,
        Camera,
    }

    let mut mode = Mode::Control;
    let mut border_type = ParticleType::Fluid;

    while win.update() && !win.input.hit(KEY_ESCAPE) {
        // set control or camera mode
        if win.input.hit(KEY_SPACE) {
            if mode == Mode::Camera {
                mode = Mode::Control;
            } else {
                mode = Mode::Camera;
            }
        }

        // set borders
        if win.input.hit(B) {
            if border_type == ParticleType::Fluid {
                border_type = ParticleType::Infinity;
            } else {
                border_type = ParticleType::Fluid
            }

            simulation.set_borders(border_type);
        }

        // reset oszillators
        if win.input.hit(R) {
            for x in 1..(WIDTH - 1) {
                for y in 1..(HEIGHT - 1) {
                    simulation.particles[x as usize][y as usize].particle_type =
                        ParticleType::Fluid;
                }
            }
        }

        // update simulation
        simulation.update();
        simulation.update_mesh(&mut win.factory);

        if mode == Mode::Camera {
            controls.update(&win.input);
        } else {
            if win.input.hit(MouseButton::Left) {
                // raise particles
                let (mouse_x, mouse_y) =
                    get_normalised_mouse_pos(win.input.mouse_pos(), win.size());

                // particles[mouse_x as usize][mouse_y as usize].pos.y = HEIGHT_LIMIT;

                // set multiple particles
                for x in (mouse_x - MOUSE_SIZE) as usize..=(mouse_x + MOUSE_SIZE) as usize {
                    for y in (mouse_y - MOUSE_SIZE) as usize..=(mouse_y + MOUSE_SIZE) as usize {
                        simulation.particles[x][y].pos.y = MOUSE_HEIGHT;
                        // make sure they're a fluid
                        simulation.particles[x][y].particle_type = ParticleType::Fluid;
                    }
                }
            } else if win.input.hit(MouseButton::Right) {
                let (x, y) = get_normalised_mouse_pos(win.input.mouse_pos(), win.size());

                // or make them oszillators
                simulation.particles[x as usize][y as usize].particle_type =
                    ParticleType::Oszillator(0.0);
            }
        }

        let text = "FPS: ".to_string()
            + &(1.0 / win.input.delta_time()).round().to_string()
            + "\n"
            + HELP_TEXT;
        ui_text.set_text(text);

        win.render(&cam);
    }
}
