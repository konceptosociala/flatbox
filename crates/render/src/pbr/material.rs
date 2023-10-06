pub trait Material: Send + Sync + 'static {
    fn vertex_shader() -> &'static str;

    fn fragment_shader() -> &'static str;
}

#[repr(C)]
pub struct DefaultMaterial {
    pub color: [f32; 3]
}

impl Material for DefaultMaterial {
    fn vertex_shader() -> &'static str {
        include_str!("../shaders/defaultmat.vs")
    }

    fn fragment_shader() -> &'static str {
        include_str!("../shaders/defaultmat.fs")
    }
}