use anyhow::Result;
use flatbox_core::math::transform::Transform;
use flatbox_ecs::{SubWorld, Write};
use flatbox_render::{
    pbr::{
        material::Material, 
        model::Model, camera::Camera,
    }, 
    renderer::{Renderer, PrepareModelCommand, DrawModelCommand, ClearCommand, RenderCameraCommand}, 
    error::RenderError,
};

pub fn clear_screen(mut renderer: Write<Renderer>) -> Result<()> {
    renderer.execute(&mut ClearCommand(0.1, 0.1, 0.1))?;
    
    Ok(())
}

pub fn bind_material<M: Material>(mut renderer: Write<Renderer>) {
    renderer.bind_material::<M>();
}

pub fn render_material<M: Material>(
    model_world: SubWorld<(&mut Model, &M, &Transform)>,
    camera_world: SubWorld<(&mut Camera, &Transform)>,
    mut renderer: Write<Renderer>,
) -> Result<()> {
    let mut found_active_camera = false;

    for (_, (mut camera, transform)) in &mut camera_world.query::<(&mut Camera, &Transform)>() {
        if camera.is_active() {
            if found_active_camera {
                Err(RenderError::MultipleActiveCameras)?;
            } else {
                found_active_camera = true;

                renderer.execute(&mut RenderCameraCommand::<M>::new(&mut camera, transform))?;
                for (_, (mut model, material, transform)) in &mut model_world.query::<(&mut Model, &M, &Transform)>() {
                    renderer.execute(&mut PrepareModelCommand::new(&mut model, material))?;
                    renderer.execute(&mut DrawModelCommand::new(&model, material, transform))?;
                }
            }
        }
    }

    Ok(())
}