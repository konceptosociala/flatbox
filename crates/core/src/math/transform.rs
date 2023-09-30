use nalgebra_glm as glm;

pub struct Transform {
    pub translation: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: f32,
}