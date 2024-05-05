use std::process::exit;

use flatbox::Flatbox;
use flatbox_core::logger::debug;
use flatbox_render::{context::WindowBuilder, pbr::{color::Color, texture::{Filter, Texture, TextureDescriptor, WrapMode}}};
use flatbox_ecs::SystemStage::Setup;
use ron::ser::PrettyConfig;

fn main() {
    Flatbox::init(WindowBuilder::default())
        .add_system(Setup, texture_setup)
        .run();
}

fn texture_setup() -> anyhow::Result<()> {
    let textures = vec![
        Texture::new("assets/crate.png", None)?,
        Texture::new_from_color(16, 16, Color::Byte(172, 0, 255))?,
        Texture::new_from_raw(2, 2, 
            &[
                0, 200, 255, 255,
                0, 200, 255, 255,
                255, 200, 0, 255,
                255, 200, 0, 255,
            ], 
            Some(TextureDescriptor {
                filter: Filter::Nearest,
                wrap_mode: WrapMode::ClampToEdge,
                ..Default::default()
            }),
        )?
    ];

    let ser = textures
        .iter()
        .map(|t| ron::ser::to_string_pretty(&t, PrettyConfig::default()).unwrap())
        .collect::<Vec<_>>();

    let de = ser
        .iter()
        .map(|s| ron::from_str::<Texture>(s).unwrap())
        .collect::<Vec<_>>();

    de.iter().for_each(|t| debug!("{t:#?}"));

    exit(0);
}