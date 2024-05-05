use std::{time::{Instant, Duration}, sync::Arc, fmt::Debug};
use flatbox_core::logger::LoggerLevel;
use glutin::{
    dpi::{LogicalSize, PhysicalSize, Size}, event::Event, event_loop::{ControlFlow as WinitControlFlow, EventLoop, EventLoopWindowTarget}, platform::run_return::EventLoopExtRunReturn, window::{Fullscreen, Icon, Window, WindowBuilder as GlutinWindowBuilder}, Api, ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent 
};
use parking_lot::{Mutex, MutexGuard};
use crate::renderer::WindowExtent;

pub use glutin::event::WindowEvent;
pub use glutin::event::VirtualKeyCode;
pub use glutin::event::ElementState;

pub type GlContext = ContextWrapper<PossiblyCurrent, Window>;

#[derive(Clone)]
pub struct Display(Arc<Mutex<GlContext>>);

impl Display {
    pub fn new(context: GlContext) -> Display {

        #[allow(clippy::arc_with_non_send_sync)]
        Display(Arc::new(Mutex::new(context)))
    }

    pub fn set_fullscreen(&self, fullscreen: bool) {
        self.lock().window().set_fullscreen(match fullscreen {
            true => Some(Fullscreen::Borderless(None)),
            false => None,
        });
    }

    pub fn lock(&self) -> MutexGuard<GlContext> {
        self.0.lock()
    }
}

unsafe impl Send for Display {}
unsafe impl Sync for Display {}

impl From<PhysicalSize<u32>> for WindowExtent {
    fn from(size: PhysicalSize<u32>) -> Self {
        WindowExtent { 
            x: 0.0,
            y: 0.0,
            width: size.width as f32, 
            height: size.height as f32,
        }
    }
}

#[derive(Default, Clone)]
pub struct ControlFlow {
    inner: Arc<Mutex<WinitControlFlow>>,
    repaint_after: Duration,
}  

impl ControlFlow {
    pub fn new() -> ControlFlow {
        ControlFlow::default()
    }

    pub fn repaint_after(&self) -> Duration {
        self.repaint_after
    }

    pub fn set_repaint_after(&mut self, repaint_after: Duration) {
        self.repaint_after = repaint_after;
    }

    pub fn set_poll(&self) {
        *(self.inner.lock()) = WinitControlFlow::Poll;
    }

    pub fn set_wait(&self) {
        *(self.inner.lock()) = WinitControlFlow::Wait;
    }

    pub fn set_wait_until(&self, instant: Instant) {
        *(self.inner.lock()) = WinitControlFlow::WaitUntil(instant);
    }

    pub fn exit(&self) {
        *(self.inner.lock()) = WinitControlFlow::Exit;
    }
}

impl Debug for ControlFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

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
        match event_loop {
            Self::NotPresent => panic!("EventLoop is not present"),
            Self::Present(e) => e,
        }
    }
}

impl AsRef<EventLoop<()>> for EventLoopWrapper {
    fn as_ref(&self) -> &EventLoop<()> {
        match self {
            Self::NotPresent => panic!("EventLoop is not present"),
            Self::Present(e) => e,
        }
    }
}

pub enum ContextEvent {
    Setup(Display),
    Resize(WindowExtent),
    Update,
    Render(Display, ControlFlow),
    Window(Display, WindowEvent<'static>),
}

pub struct Context {
    event_loop: EventLoopWrapper,
    display: Display,
    control_flow: ControlFlow,
    max_frame_time: Duration,
    exit_next_iteration: bool,
    window_occluded: bool,
    fixed_time_step: f64,
    number_of_updates: u32,
    number_of_renders: u32,
    last_frame_time: f64,
    running_time: f64,
    accumulated_time: f64,
    blending_factor: f64,
    previous_instant: Instant,
    current_instant: Instant,
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
            display: Display::new(gl_context),
            control_flow: ControlFlow::default(),
            max_frame_time: Duration::from_secs_f64(builder.max_frame_time),
            window_occluded: false,
            exit_next_iteration: false,
            fixed_time_step: 1.0 / builder.updates_per_second as f64,
            number_of_updates: 0,
            number_of_renders: 0,
            running_time: 0.0,
            accumulated_time: 0.0,
            blending_factor: 0.0,
            previous_instant: Instant::now(),
            current_instant: Instant::now(),
            last_frame_time: 0.0,
        }
    }

    pub fn display(&self) -> Display {
        self.display.clone()
    }

    pub fn event_loop_target(&self) -> &EventLoopWindowTarget<()> {
        self.event_loop.as_ref()
    }

    pub fn get_proc_address(&self, addr: &str) -> *const core::ffi::c_void {
        self.display.lock().get_proc_address(addr)
    }

    pub fn next_frame<F: FnMut(ContextEvent)>(&mut self, mut runner: F) {
        if self.exit_next_iteration { return; }

        self.current_instant = Instant::now();

        let mut elapsed = self.current_instant.duration_since(self.previous_instant);
        if elapsed > self.max_frame_time { elapsed = self.max_frame_time; }

        self.last_frame_time = elapsed.as_secs_f64();
        self.running_time += elapsed.as_secs_f64();
        self.accumulated_time += elapsed.as_secs_f64();

        while self.accumulated_time >= self.fixed_time_step {
            (runner)(ContextEvent::Update);

            self.accumulated_time -= self.fixed_time_step;
            self.number_of_updates += 1;
        }

        self.blending_factor = self.accumulated_time / self.fixed_time_step;

        if self.window_occluded {
            std::thread::sleep(Duration::from_secs_f64(self.fixed_time_step));
        } else {
            (runner)(ContextEvent::Render(
                self.display.clone(), 
                self.control_flow.clone(),
            ));

            self.number_of_renders += 1;
        }

        self.previous_instant = self.current_instant;        
    }

    pub fn run<F: FnMut(ContextEvent)>(&mut self, mut runner: F) {
        (runner)(ContextEvent::Setup(self.display.clone()));

        self.event_loop.take().run_return(move |event, _, control_flow|{
            match event {
                Event::LoopDestroyed => (),
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = WinitControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            let size = WindowExtent::from(physical_size);
                            (runner)(ContextEvent::Resize(size));
                            self.display.lock().resize(physical_size);
                        },
                        WindowEvent::Occluded(occluded) => self.window_occluded = occluded,
                        _ => {},
                    }

                    (runner)(ContextEvent::Window(
                        self.display.clone(),
                        event.to_static().unwrap_or(WindowEvent::Focused(true)), 
                    ));
                },
                Event::RedrawRequested(_) => {
                    self.next_frame(&mut runner);
                    
                    *control_flow = *(self.control_flow.inner.lock());
                    self.display.lock().swap_buffers().unwrap();
                },
                Event::MainEventsCleared => {
                    self.display.lock().window().request_redraw();
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
    /// Specifies logger level and whether it must be initialized
    pub logger_level: LoggerLevel,
    /// 
    pub updates_per_second: u32, 
    ///
    pub max_frame_time: f64
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
            #[cfg(not(debug_assertions))]
            logger_level: LoggerLevel::Info, 
            #[cfg(debug_assertions)]
            logger_level: LoggerLevel::Debug,
            updates_per_second: 240,
            max_frame_time: 0.1,
        }
    }
}