extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate uuid;
extern crate gfx_device_gl;

use std::rc::Rc;
use std::path::PathBuf;

use gfx_device_gl::Resources;
use piston_window::*;
use sprite::{Animation, Blink, Ease, EaseFunction, FadeIn, FadeOut, MoveBy, RotateTo, ScaleTo, Scene, Sprite};
use ai_behavior::{ Action, Behavior, Sequence, Wait, WaitForever, While };
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

fn run_game(window: &PistonWindow, assets: &PathBuf) {
    const GROUND_Y_POS: f64 = HEIGHT as f64 * 3.0 / 4.0;
    const RUST_CHAR_SIZE: f64 = 64.0;

    let rust_lang_tex: Rc<Texture<Resources>> = Rc::new(Texture::from_path(
            &mut *window.factory.borrow_mut(),
            assets.join("rust-lang.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap()); //FIXME transparency is not respected
    let mut rust_lang_sprite: Sprite<Texture<Resources>> = Sprite::from_texture(rust_lang_tex.clone());
    rust_lang_sprite.set_position(WIDTH as f64 / 8.0, GROUND_Y_POS - RUST_CHAR_SIZE / 2.0);


    for e in window.clone() {

        e.draw_2d(|c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);

            Rectangle::new([0.239, 0.404, 0.224, 1.0]) // deep green
                .draw([0.0, GROUND_Y_POS, WIDTH as f64, HEIGHT as f64 / 4.0],
                     &c.draw_state, c.transform, g);

            rust_lang_sprite.draw(c.transform, g);
            // or
            // image(&rust_lang_tex, c.transform, g);
        });
    }
}
