pub mod transform;

pub mod glm {
    pub use nalgebra_glm::*;
    
    pub fn safe_quat_look_at(
        look_from: &Vec3,
        look_to: &Vec3,
        up: &Vec3,
        alternative_up: &Vec3,
    ) -> Quat {
        let mut direction: Vec3 = look_to - look_from;
        let direction_length = length(&direction);
    
        if direction_length <= 0.0001 {
            return quat(1.0, 0.0, 0.0, 0.0);
        }
    
        direction /= direction_length;
    
        let dot = dot(&direction, up);
        let abs = if dot < 0.0 { -dot } else { dot };
        if abs > 0.9999 {
            quat_look_at(&direction, alternative_up)
        }
        else {
            quat_look_at(&direction, up)
        }
    }
}