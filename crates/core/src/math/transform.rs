use serde::{Serialize, Deserialize};
use nalgebra_glm as glm;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: f32,
}

impl Transform {
    pub fn identity() -> Transform {
        Transform {
            translation: glm::Vec3::identity(),
            rotation: glm::Quat::identity(),
            scale: 1.0,
        }
    }

    pub fn to_matrix(&self) -> glm::Mat4 {
        glm::Mat4::identity()
            * glm::translation(&self.translation)
            * glm::quat_cast(&self.rotation)
            * glm::scaling(&glm::vec3(self.scale, self.scale, self.scale))
    }
}