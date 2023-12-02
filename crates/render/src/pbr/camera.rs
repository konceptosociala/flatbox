use std::f32::consts::FRAC_PI_3;

use serde::{Serialize, Deserialize};
use flatbox_core::{
    math::{
        glm, 
        transform::Transform,
    },
    logger::error,
};

use crate::hal::shader::GraphicsPipeline;

#[derive(Clone, Default, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub enum CameraType {
    #[default]
    FirstPerson,
    LookAt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    camera_type: CameraType,
    projection_matrix: glm::Mat4,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    is_active: bool,
}

impl Camera {
    pub fn new() -> Self {
        Camera::default()
    }
    
    pub fn builder() -> CameraBuilder {
        CameraBuilder {
            camera_type: CameraType::LookAt,
            fovy: FRAC_PI_3,
            aspect: 800.0 / 600.0,
            near: 0.1,
            far: 100.0,
            is_active: false,
        }
    }
    
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    pub fn set_active(&mut self, is_active: bool){
        self.is_active = is_active;
    }
    
    pub fn camera_type(&self) -> CameraType {
        self.camera_type.clone()
    }
    
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.update_projection_matrix();
    }
    
    pub(crate) fn update_buffer(
        &self,
        pipeline: &GraphicsPipeline,
        transform: &Transform,
    ) {     
        let rotation_matrix = glm::quat_cast(&transform.rotation);
        let translation_matrix = glm::translation(&transform.translation);
        
        let view_matrix = {
            if self.camera_type == CameraType::FirstPerson {
                rotation_matrix * translation_matrix
            } else {
                translation_matrix * rotation_matrix
            }
        };
        
        pipeline.apply();
        pipeline.set_mat4("view", &view_matrix);
        pipeline.set_mat4("projection", &self.projection_matrix);
        pipeline.set_vec3("viewPos", &transform.translation);
    }
    
    fn update_projection_matrix(&mut self) {
        self.projection_matrix = glm::perspective(self.aspect, self.fovy, self.near, self.far);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::builder().build()
    }
}

pub struct CameraBuilder {
    camera_type: CameraType,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    is_active: bool,
}

impl CameraBuilder {
    pub fn build(self) -> Camera {
        if self.far < self.near {
            error!("Far plane (at {}) is closer than near plane (at {})!", self.far, self.near);
        }
        
        let mut cam = Camera {
            camera_type: self.camera_type,
            fovy: self.fovy,
            aspect: self.aspect,
            near: self.near,
            far: self.far,
            projection_matrix: glm::Mat4::identity(),
            is_active: self.is_active,
        };

        cam.update_projection_matrix();
        cam
    }
    
    pub fn camera_type(mut self, camera_type: CameraType) -> CameraBuilder {
        self.camera_type = camera_type;
        self
    }
    
    pub fn fovy(mut self, fovy: f32) -> CameraBuilder {
        self.fovy = fovy.max(0.01).min(std::f32::consts::PI - 0.01);
        self
    }
    
    pub fn aspect(mut self, aspect: f32) -> CameraBuilder {
        self.aspect = aspect;
        self
    }
    
    pub fn near(mut self, near: f32) -> CameraBuilder {
        if near <= 0.0 {
            error!("Near plane ({}) can't be negative!", near);
        }
        self.near = near;
        self
    }
    
    pub fn far(mut self, far: f32) -> CameraBuilder {
        if far <= 0.0 {
            error!("Far plane ({}) can't be negative!", far);
        }
        self.far = far;
        self
    }
    
    pub fn is_active(mut self, is_active: bool) -> CameraBuilder {
        self.is_active = is_active;
        self
    }
}