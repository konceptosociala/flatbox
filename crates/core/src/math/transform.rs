use serde::{Serialize, Deserialize};
use nalgebra_glm as glm;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: f32,
}

impl Transform {
    pub fn new(translation: glm::Vec3, rotation: glm::Quat, scale: f32) -> Transform {
        Transform { translation, rotation, scale }
    }

    pub fn identity() -> Transform {
        Transform::default()
    }

    pub fn new_from_translation(translation: glm::Vec3) -> Transform {
        Transform { translation, ..Default::default() }
    }

    pub fn new_from_rotation(rotation: glm::Quat) -> Transform {
        Transform { rotation, ..Default::default() }
    }

    pub fn to_matrices(&self) -> (glm::Mat4, glm::Mat4) {
        let matrix = glm::Mat4::identity()
            * glm::translation(&self.translation)
            * glm::quat_cast(&self.rotation)
            * glm::scaling(&glm::vec3(self.scale, self.scale, self.scale));

        let inversed = matrix.try_inverse().unwrap();
        (matrix, inversed)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::Quat::identity(),
            scale: 1.0,
        }
    }
}