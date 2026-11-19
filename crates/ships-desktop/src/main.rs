use std::{
    fs,
    path::{Path, PathBuf},
};

use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};
use ships_core::Stage;
use ships_render::View;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let mut app = View::preview(2026);
    write_png(
        &output.join("ships.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, false),
    )?;
    assert_eq!(app.input(Input::Tap(600.0, 655.0)), Change::Full);
    assert_eq!(app.input(Input::Tap(600.0, 1280.0)), Change::Partial);
    write_png(
        &output.join("ships-fleet.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, false),
    )?;
    assert_eq!(app.input(Input::Tap(600.0, 1440.0)), Change::Full);
    assert_eq!(app.game.as_ref().unwrap().stage, Stage::Battle(0));
    assert_eq!(app.input(Input::Tap(260.0, 388.0)), Change::Partial);
    write_png(
        &output.join("ships-target-dark.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, true),
    )?;
    let mut hotseat = View::preview(2026);
    assert_eq!(hotseat.input(Input::Tap(600.0, 960.0)), Change::Full);
    assert_eq!(hotseat.input(Input::Tap(600.0, 1280.0)), Change::Partial);
    assert_eq!(hotseat.input(Input::Tap(600.0, 1440.0)), Change::Full);
    assert_eq!(hotseat.game.as_ref().unwrap().stage, Stage::PassSetup);
    write_png(
        &output.join("ships-pass.png"),
        WIDTH,
        HEIGHT,
        &frame(&hotseat, &mut canvas, false),
    )?;
    Ok(())
}
