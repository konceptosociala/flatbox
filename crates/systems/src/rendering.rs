use anyhow::Result;
use flatbox_core::math::transform::Transform;
use flatbox_ecs::{SubWorld, Write};
use flatbox_render::{
    pbr::{
        material::Material, 
        model::Model,
    }, 
    renderer::{Renderer, PrepareModelCommand, DrawModelCommand, ClearCommand}
};

pub fn clear_screen(mut renderer: Write<Renderer>) -> Result<()> {
    renderer.execute(&mut ClearCommand)?;
    Ok(())
}

pub fn render_material<M: Material>(
    world: SubWorld<(&mut Model, &M, &Transform)>,
    mut renderer: Write<Renderer>,
) -> Result<()> {
    for (_, (mut model, material, transform)) in &mut world.query::<(&mut Model, &M, &Transform)>() {
        renderer.execute(&mut PrepareModelCommand::new(&mut model, material))?;
        renderer.execute(&mut DrawModelCommand::new(&model, material, transform))?;
    }

    Ok(())
}