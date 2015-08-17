extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate uuid;
extern crate gfx_device_gl;

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
    let opengl = OpenGL::V3_2;
    let window: PistonWindow =
        WindowSettings::new("piston: sprite", (WIDTH, HEIGHT))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let assets: PathBuf = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    // run_intro(&window, &assets);
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
        scene.event(&e); // updates animations

        e.draw_2d(|c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            scene.draw(c.transform, g);
        });
        if Some(Button::Keyboard(Key::Space)) == e.press_args() ||
                Some(Button::Keyboard(Key::Return)) == e.press_args() ||
                scene.running() == 0 {
            return;
        }
    }
}

const GROUND_Y_POS: f64 = HEIGHT as f64 * 3.0 / 4.0;

struct PlayerCharacter {
    sprite: Sprite<Texture<Resources>>, // tracks position, rotation, size, etc
}

impl PlayerCharacter {
    fn new(window: &PistonWindow, image: &Path) -> Self {
        let tex: Rc<Texture<Resources>> = Rc::new(Texture::from_path(
                &mut *window.factory.borrow_mut(),
                image,
                Flip::None,
                &TextureSettings::new()
            ).unwrap());
        PlayerCharacter {
            sprite: Sprite::from_texture(tex),
        }
    }

    fn get_height(&self) -> f64 {
        self.sprite.get_texture().get_height() as f64 // ignore any scaling on the sprite
    }

    fn translate(&mut self, dx: f64, dy: f64, dt: f64) {
        let (x, y) = self.sprite.get_position();
        // println!("Current x and y: {:?}, dt = {}", (x, y), dt);
        self.sprite.set_position(x + dx * dt, y + dy * dt);
    }
}

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
    {
        let y_pos = GROUND_Y_POS - pc.get_height() / 2.0 + 2.0;
        pc.sprite.set_position(WIDTH as f64 / 8.0, y_pos);
    }

    let mut key_state = KeyState::new();

    for e in window.clone() {
        key_state.update(&e);

        if let Some(UpdateArgs{ dt }) = e.update_args() {
            if key_state.is_down(&Button::Keyboard(Key::Left)) {
                pc.translate(-250.0, 0.0, dt);
            }
            if key_state.is_down(&Button::Keyboard(Key::Right)) {
                pc.translate(250.0, 0.0, dt);
            }
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
