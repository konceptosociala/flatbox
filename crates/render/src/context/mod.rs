use glutin::{
    event_loop::{EventLoop, ControlFlow}, 
    window::{Window, Icon, WindowBuilder as GlutinWindowBuilder},
    dpi::{Size, LogicalSize},
    ContextWrapper, PossiblyCurrent, ContextBuilder, GlRequest, Api, event::{Event, WindowEvent},
};
use flatbox_core::math::glm;

#[derive(Default)]
pub enum EventLoopWrapper {
    Present(EventLoop<()>),
    #[default]
    NotPresent,
}

impl EventLoopWrapper {
    pub fn new(event_loop: EventLoop<()>) -> EventLoopWrapper {
        EventLoopWrapper::Present(event_loop)
    }

    pub fn new_not_present() -> EventLoopWrapper {
        EventLoopWrapper::NotPresent
    }

    pub fn take(&mut self) -> EventLoop<()> {
        let event_loop = std::mem::take(self);
        *self = EventLoopWrapper::NotPresent;
        return match event_loop {
            Self::NotPresent => panic!("EventLoop is not present"),
            Self::Present(e) => e,
        };
    }
}

pub struct Context {
    event_loop: EventLoopWrapper,
    ctx: ContextWrapper<PossiblyCurrent, Window>,
}

impl Context {
    pub fn new(builder: &WindowBuilder) -> Context {
        let event_loop = EventLoop::new();

        let window = GlutinWindowBuilder::new()
            .with_inner_size(Size::from(LogicalSize::new(builder.width, builder.height)))
            .with_title(builder.title)
            .with_maximized(builder.maximized)
            .with_resizable(builder.resizable)
            .with_window_icon(builder.icon.clone())
            .with_fullscreen(match builder.fullscreen {
                true => Some(glutin::window::Fullscreen::Borderless(None)),
                false => None,
            });

        let gl_context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (4, 1)))
            .build_windowed(window, &event_loop)
            .expect("Cannot create windowed context");

        let gl_context = unsafe {
            gl_context
                .make_current()
                .expect("Failed to make context current")
        };

        Context {
            event_loop: EventLoopWrapper::new(event_loop),
            ctx: gl_context,
        }
    }

    pub fn get_proc_address(&self, addr: &str) -> *const core::ffi::c_void {
        self.ctx.get_proc_address(addr)
    }

    pub fn run<F: FnMut() + 'static>(mut self, mut runner: F) -> ! {
        self.event_loop.take().run(move |event, _, control_flow|{
            *control_flow = ControlFlow::Wait;
    
            match event {
                Event::LoopDestroyed => (),
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => self.ctx.resize(physical_size),
                    _ => {},
                },
                Event::RedrawRequested(_) => {
                    (runner)();
    
                    self.ctx.swap_buffers().unwrap();
                    self.ctx.window().request_redraw();
                },
                _ => {},
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct WindowBuilder {
    /// Title of the window
    pub title: &'static str, 
    /// Width of the window
    pub width: u32,
    /// Height of the window
    pub height: u32,
    /// Specifies whether the window should be fullscreen or windowed
    pub fullscreen: bool,
    /// Specifies whether the window is maximized on startup
    pub maximized: bool,
    /// Specifies whether the window should be resizable
    pub resizable: bool,
    /// Icon of the winit window. Requires feature `render` enabled
    pub icon: Option<Icon>,
    /// Specifies whether the logger must be initialized
    pub init_logger: bool,
    /// Window clear background color:
    pub clear_color: glm::Vec3,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        WindowBuilder { 
            title: "My Game", 
            width: 800, 
            height: 600, 
            fullscreen: false, 
            maximized: false, 
            resizable: true, 
            icon: None, 
            init_logger: true, 
            clear_color: glm::vec3(0.1, 0.1, 0.1), 
        }
    }
}