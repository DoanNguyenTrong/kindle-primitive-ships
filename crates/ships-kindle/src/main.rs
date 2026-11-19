use std::path::PathBuf;

use ships_render::View;

fn main() -> Result<(), String> {
    let directory = std::env::var_os("PRIMITIVE_APP_DIR")
        .map_or_else(|| PathBuf::from("/mnt/us/ships"), PathBuf::from);
    primitive_app::run(View::open(directory.join("ships.json")), &directory)
}
