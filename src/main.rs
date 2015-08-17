#![feature(slice_patterns)]

extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate uuid;
extern crate gfx_device_gl;

use std::cmp;
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

use gfx_device_gl::Resources;
use piston_window::*;
use sprite::{Animation, Ease, EaseFunction, FadeOut, MoveBy, RotateTo, ScaleTo, Scene, Sprite};
use ai_behavior::{ Action, Behavior, Sequence, Wait};
use uuid::Uuid;

const WIDTH: u32 = 300;
const HEIGHT: u32 = 300;

fn main() {
    let window: PistonWindow =
        WindowSettings::new("piston: sprite", (WIDTH, HEIGHT))
        .exit_on_esc(true)
        .opengl(OpenGL::V3_2)
        .build()
        .unwrap();

    let assets: PathBuf = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    run_intro(&window, &assets);
    run_game(&window, &assets);
}

fn run_intro(window: &PistonWindow, assets: &PathBuf) {
    let tex: Rc<Texture<Resources>> = Rc::new(Texture::from_path(
            &mut *window.factory.borrow_mut(),
            assets.join("rust-sydney.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap());
    let mut sprite: Sprite<Texture<Resources>> = Sprite::from_texture(tex.clone());
    sprite.set_position(WIDTH as f64 / 2.0, HEIGHT as f64 / 2.0);

    let mut scene: Scene<Texture<Resources>> = Scene::new();
    let id: Uuid = scene.add_child(sprite);

    // Run a sequence of animations, using ai_behavior + animations from the sprite crate
    let seq: Behavior<Animation> = Sequence(vec![
        Action(Ease(EaseFunction::CubicOut, Box::new(ScaleTo(2.0, 0.5, 0.5)))),
        Action(Ease(EaseFunction::BounceOut, Box::new(MoveBy(1.0, 0.0, 80.0)))),
        Wait(0.5),
        Action(FadeOut(0.3)),
    ]);
    scene.run(id, &seq);

    // run another animation in parallel (could also use ai_behavior to do this instead)
    let rotate: Behavior<Animation> =
        Action(Ease(EaseFunction::QuarticInOut, Box::new(RotateTo(2.5, 1080.0))));
    scene.run(id, &rotate);

    for e in window.clone() {
        // Confusingly e is also a PistonWindow, but, in practice, treat it as an event.

        scene.event(&e); // updates animation state

        e.draw_2d(|c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            scene.draw(c.transform, g);
        });
        if e.press_args() == Some(Button::Keyboard(Key::Space)) ||
                e.press_args() == Some(Button::Keyboard(Key::Return)) ||
                scene.running() == 0 {
            return;
        }
    }
}

pub fn defined_min<T: PartialOrd>(v1: T, v2: T) -> T {
    match v1.partial_cmp(&v2) {
        Some(cmp::Ordering::Less) | Some(cmp::Ordering::Equal) => Some(v1),
        Some(cmp::Ordering::Greater) => Some(v2),
        None => None
    }.unwrap()
}

pub fn defined_max<T: PartialOrd>(v1: T, v2: T) -> T {
    match v1.partial_cmp(&v2) {
        Some(cmp::Ordering::Equal) | Some(cmp::Ordering::Less) => Some(v2),
        Some(cmp::Ordering::Greater) => Some(v1),
        None => None
    }.unwrap()
}

const GROUND_Y_POS: f64 = HEIGHT as f64 * 3.0 / 4.0;
const MAX_VELOCITY: f64 = 400.0; // per axis

fn clamp_between<T: PartialOrd>(i: T, min: T, max: T) -> T {
    match i.partial_cmp(&min) {
        Some(cmp::Ordering::Less) => min,
        _ => match i.partial_cmp(&max) {
            Some(cmp::Ordering::Greater) => max,
            _ => i,
        },
    }
}

struct PlayerCharacter {
    sprite: Sprite<Texture<Resources>>, // tracks position, rotation, size, etc
    radius: f64,
    velocity: [f64; 2],
}

impl PlayerCharacter {
    fn new(window: &PistonWindow, image: &Path) -> Self {
        let tex: Rc<Texture<Resources>> = Rc::new(Texture::from_path(
                &mut *window.factory.borrow_mut(),
                image,
                Flip::None,
                &TextureSettings::new()
            ).unwrap());
        let radius = tex.get_height() as f64 / 2.0 - 2.0; // take off 2 pixels for looks
        let mut sprite = Sprite::from_texture(tex);
        sprite.set_position(WIDTH as f64 / 8.0, GROUND_Y_POS - radius);
        PlayerCharacter {
            sprite: sprite,
            radius: radius,
            velocity: [0.0, 0.0],
        }
    }

    fn accelerate(&mut self, dvx: f64, dvy: f64) {
        let [vx, vy] = self.velocity;
        self.velocity = [vx + dvx, vy + dvy];
    }

    fn update(&mut self, dt: f64) {
        let (x, y) = self.sprite.get_position();
        let [vx, vy] = self.velocity;
        self.sprite.set_position(
            clamp_between(x + vx * dt, 0.0 + self.radius, WIDTH as f64 - self.radius),
            clamp_between(y + vy * dt, 0.0, GROUND_Y_POS - self.radius)
        );

        self.velocity = [vx * 0.9, vy * 0.9];
        if self.velocity[0] > MAX_VELOCITY { self.velocity[0] = MAX_VELOCITY; }
        if self.velocity[0] < -MAX_VELOCITY { self.velocity[0] = -MAX_VELOCITY; }
        if self.velocity[1] > MAX_VELOCITY { self.velocity[1] = MAX_VELOCITY; }
        if self.velocity[1] < -MAX_VELOCITY { self.velocity[1] = -MAX_VELOCITY; }

        let new_x = self.sprite.get_position().0;
        self.sprite.set_rotation((new_x - self.radius) / (WIDTH as f64 - self.radius * 2.0) * 360.0);
    }
}

/// Utility for keeping track of which buttons are currently pressed down
struct KeyState {
    held_keys: HashSet<Button>,
}

impl KeyState {
    fn new() -> Self {
        KeyState { held_keys: HashSet::new() }
    }

    fn update(&mut self, w: &PistonWindow) {
        if let Some(pressed) = w.press_args() {
            self.held_keys.insert(pressed);
        }
        if let Some(released) = w.release_args() {
            self.held_keys.remove(&released);
        }
    }

    fn is_down(&self, button: &Button) -> bool {
        self.held_keys.contains(button)
    }
}

fn run_game(window: &PistonWindow, assets: &PathBuf) {
    let mut pc = PlayerCharacter::new(window, assets.join("rust-lang.png").as_ref());
    let mut key_state = KeyState::new();

    for e in window.clone() {
        key_state.update(&e);

        if let Some(UpdateArgs{ dt }) = e.update_args() {
            if key_state.is_down(&Button::Keyboard(Key::Left)) {
                pc.accelerate(-250.0, 0.0);
            }
            if key_state.is_down(&Button::Keyboard(Key::Right)) {
                pc.accelerate(250.0, 0.0);
            }

            pc.update(dt);
        }

        e.draw_2d(|c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);

            Rectangle::new([0.239, 0.404, 0.224, 1.0]) // deep green
                .draw([0.0, GROUND_Y_POS, WIDTH as f64, HEIGHT as f64 / 4.0],
                     &c.draw_state, c.transform, g);

            pc.sprite.draw(c.transform, g);
        });
    }
}
