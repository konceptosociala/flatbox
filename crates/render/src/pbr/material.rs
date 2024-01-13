use std::fmt::Debug;

use serde::{Serialize, Deserialize};
use flatbox_assets::{impl_ser_component, typetag};
use flatbox_core::math::glm;

use crate::hal::shader::GraphicsPipeline;

use super::texture::{Texture, Order};

#[typetag::serde(tag = "material")]
pub trait Material: Debug + Send + Sync + 'static {
    fn vertex_shader() -> &'static str
    where 
        Self: Sized;

    fn fragment_shader() -> &'static str
    where 
        Self: Sized;

    fn setup_pipeline(&self, _pipeline: &GraphicsPipeline) {}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DefaultMaterial {
    pub color: glm::Vec3,
    pub diffuse_map: Texture,
    pub specular_map: Texture,
    pub shininess: f32,
}

impl Default for DefaultMaterial {
    fn default() -> Self {
        DefaultMaterial {
            color: glm::vec3(1.0, 1.0, 1.0),
            diffuse_map: Texture::default(),
            specular_map: Texture::default(),
            shininess: 32.0,
        }
    }
}

#[typetag::serde]
impl Material for DefaultMaterial {
    fn vertex_shader() -> &'static str {
        include_str!("../shaders/defaultmat.vs")
    }

    fn fragment_shader() -> &'static str {
        include_str!("../shaders/defaultmat.fs")
    }

    fn setup_pipeline(&self, pipeline: &GraphicsPipeline) {
        pipeline.set_vec3("material.color", &self.color);
        pipeline.set_float("material.shininess", self.shininess);

        pipeline.set_int("material.diffuse_map", 0);
        self.diffuse_map.activate(Order::Texture0);

        pipeline.set_int("material.specular_map", 1);
        self.specular_map.activate(Order::Texture1);

        let point_light_positions = [
            glm::vec3( 0.7,  0.2,  2.0),
            glm::vec3( 2.3, -3.3, -4.0),
            glm::vec3(-4.0,  2.0, -12.0),
            glm::vec3( 0.0,  0.0, -3.0)
        ];

        // Light
        pipeline.set_vec3("light.position", &glm::vec3(0.0, 0.0, 0.0));
        pipeline.set_vec3("light.ambient", &glm::vec3(0.2, 0.2, 0.2));
        pipeline.set_vec3("light.diffuse", &glm::vec3(0.5, 0.5, 0.5));
        pipeline.set_vec3("light.specular", &glm::vec3(1.0, 1.0, 1.0));
        // directional light
        pipeline.set_vec3("dirLight.direction", &glm::vec3(-0.2, -1.0, -0.3));
        pipeline.set_vec3("dirLight.ambient", &glm::vec3(0.05, 0.05, 0.05));
        pipeline.set_vec3("dirLight.diffuse", &glm::vec3(0.4, 0.4, 0.4));
        pipeline.set_vec3("dirLight.specular", &glm::vec3(0.5, 0.5, 0.5));
        // point light 1
        pipeline.set_vec3("pointLights[0].position", &point_light_positions[0]);
        pipeline.set_vec3("pointLights[0].ambient", &glm::vec3(0.05, 0.05, 0.05));
        pipeline.set_vec3("pointLights[0].diffuse", &glm::vec3(0.8, 0.8, 0.8));
        pipeline.set_vec3("pointLights[0].specular", &glm::vec3(1.0, 1.0, 1.0));
        pipeline.set_float("pointLights[0].constant", 1.0);
        pipeline.set_float("pointLights[0].linear", 0.09);
        pipeline.set_float("pointLights[0].quadratic", 0.032);
        // point light 2
        pipeline.set_vec3("pointLights[1].position", &point_light_positions[1]);
        pipeline.set_vec3("pointLights[1].ambient", &glm::vec3(0.05, 0.05, 0.05));
        pipeline.set_vec3("pointLights[1].diffuse", &glm::vec3(0.8, 0.8, 0.8));
        pipeline.set_vec3("pointLights[1].specular", &glm::vec3(1.0, 1.0, 1.0));
        pipeline.set_float("pointLights[1].constant", 1.0);
        pipeline.set_float("pointLights[1].linear", 0.09);
        pipeline.set_float("pointLights[1].quadratic", 0.032);
        // point light 3
        pipeline.set_vec3("pointLights[2].position", &point_light_positions[2]);
        pipeline.set_vec3("pointLights[2].ambient", &glm::vec3(0.05, 0.05, 0.05));
        pipeline.set_vec3("pointLights[2].diffuse", &glm::vec3(0.8, 0.8, 0.8));
        pipeline.set_vec3("pointLights[2].specular", &glm::vec3(1.0, 1.0, 1.0));
        pipeline.set_float("pointLights[2].constant", 1.0);
        pipeline.set_float("pointLights[2].linear", 0.09);
        pipeline.set_float("pointLights[2].quadratic", 0.032);
        // point light 4
        pipeline.set_vec3("pointLights[3].position", &point_light_positions[3]);
        pipeline.set_vec3("pointLights[3].ambient", &glm::vec3(0.05, 0.05, 0.05));
        pipeline.set_vec3("pointLights[3].diffuse", &glm::vec3(0.8, 0.8, 0.8));
        pipeline.set_vec3("pointLights[3].specular", &glm::vec3(1.0, 1.0, 1.0));
        pipeline.set_float("pointLights[3].constant", 1.0);
        pipeline.set_float("pointLights[3].linear", 0.09);
        pipeline.set_float("pointLights[3].quadratic", 0.032);
        // spotLight
        pipeline.set_vec3("spotLight.position", &glm::vec3(0.0, 0.0, -3.0));
        pipeline.set_vec3("spotLight.direction", &glm::vec3(0.0, 0.0, 0.0));
        pipeline.set_vec3("spotLight.ambient", &glm::vec3(0.0, 0.0, 0.0));
        pipeline.set_vec3("spotLight.diffuse", &glm::vec3(1.0, 1.0, 1.0));
        pipeline.set_vec3("spotLight.specular", &glm::vec3(1.0, 1.0, 1.0));
        pipeline.set_float("spotLight.constant", 1.0);
        pipeline.set_float("spotLight.linear", 0.09);
        pipeline.set_float("spotLight.quadratic", 0.032);
        pipeline.set_float("spotLight.cutOff", f32::cos(15.0f32.to_radians()));
        pipeline.set_float("spotLight.outerCutOff", f32::cos(15.0f32.to_radians()));
    }
}

impl_ser_component!(DefaultMaterial);