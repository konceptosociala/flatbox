use std::{ptr, any::TypeId};

use flatbox::{Flatbox, error::FlatboxResult};
use flatbox_assets::{manager::AssetManager, AssetHandle};
use flatbox_core::{
    math::*,
    logger::*,
};

use flatbox_ecs::{Write, Read, CommandBuffer, SubWorld};
use flatbox_render::{
    hal::shader::*,
    pbr::{
        texture::{
            Texture,
            Filter, Order,
        },
        mesh::Mesh, material::Material,
    },
    renderer::*, 
    context::*,
};

fn main() {
    Flatbox::init(WindowBuilder {
        title:  "Learn OpenGL with Rust",
        width:  800,
        height: 800,
        ..Default::default()
    })
        .add_setup_system(setup)
        .add_system(update)
        .run();

}

#[repr(C)]
pub struct MyMaterial;

impl Material for MyMaterial {
    fn vertex_shader() -> &'static str {
        include_str!("../crates/render/src/shaders/basic.vs")
    }

    fn fragment_shader() -> &'static str {
        include_str!("../crates/render/src/shaders/basic.fs")
    }
}

struct Textures(Vec<AssetHandle>);

fn setup(
    mut renderer: Write<Renderer>,
    mut assets: Write<AssetManager>,
    mut cmd: Write<CommandBuffer>,
) -> FlatboxResult<()> {
    renderer.bind_material::<MyMaterial>();

    let program = renderer.graphics_pipelines.get(&TypeId::of::<MyMaterial>()).unwrap();

    let mut mesh = Mesh::cube();
    mesh.setup(program);

    let rust = assets.insert(Texture::new("assets/rust.png", Filter::Linear)?);
    let wall = assets.insert(Texture::new("assets/wall.jpg", Filter::Linear)?);

    program.set_int_uniform("rustTexture", 0);
    program.set_int_uniform("wallTexture", 1);    

    cmd.spawn((
        mesh, 
        Textures(vec![rust, wall]),
    ));

    Ok(())
}

fn update(
    renderer: Read<Renderer>,
    assets: Read<AssetManager>,
    world: SubWorld<(&Mesh, &Textures)>,
) -> FlatboxResult<()> {
    renderer.clear();

    for (_, (mesh, textures)) in &mut world.query::<(&Mesh, &Textures)>() {
        let program = renderer.graphics_pipelines.get(&TypeId::of::<MyMaterial>()).unwrap();
        let rust = assets.get::<Texture>(textures.0[0])?;
        let wall = assets.get::<Texture>(textures.0[1])?;

        rust.activate(Order::Texture0);
        wall.activate(Order::Texture1);
        
        let mut model = glm::Mat4::identity();
        model = glm::rotate(&model, (std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis() - 1696149300000) as f32 / 257.0, &glm::vec3(1.0, 1.0, 1.0));
        let mut view = glm::Mat4::identity();
        view = glm::translate(&view, &glm::vec3(0.0, 0.0, -3.0));
        let projection;
        projection = glm::perspective(45.0f32.to_radians(), 800.0 / 600.0, 0.1, 100.0);
        
        program.apply();
    
        let model_loc = program.get_uniform_location("model");
        let view_loc = program.get_uniform_location("view");
        let projection_loc = program.get_uniform_location("projection");

        renderer.push_uniform_matrix(model_loc, &model);
        renderer.push_uniform_matrix(view_loc, &view);
        renderer.push_uniform_matrix(projection_loc, &projection);
    
        mesh.vertex_array.bind();
        renderer.render();
    }

    Ok(())
}