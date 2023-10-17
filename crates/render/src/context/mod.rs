/*
 * Based on `game-loop` (c) tuzz
 * Licensed under MIT License
 */
use std::time::{Instant, Duration};
use flatbox_core::logger::LoggerLevel;
use glutin::{
    platform::run_return::EventLoopExtRunReturn,
    event_loop::{EventLoop, ControlFlow}, 
    window::{Window, Icon, WindowBuilder as GlutinWindowBuilder},
    dpi::{Size, LogicalSize, PhysicalSize},
    ContextWrapper, PossiblyCurrent, ContextBuilder, GlRequest, Api, event::{Event, WindowEvent},
};

use crate::renderer::WindowExtent;

impl From<PhysicalSize<u32>> for WindowExtent {
    fn from(size: PhysicalSize<u32>) -> Self {
        WindowExtent { 
            width: size.width as f32, 
            height: size.height as f32,
        }
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

pub enum ContextEvent {
    ResizeEvent(WindowExtent),
    UpdateEvent,
    RenderEvent,
}

pub struct Context {
    event_loop: EventLoopWrapper,
    ctx: ContextWrapper<PossiblyCurrent, Window>,
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
            ctx: gl_context,
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

    pub fn get_proc_address(&self, addr: &str) -> *const core::ffi::c_void {
        self.ctx.get_proc_address(addr)
    }

    pub fn next_frame<F: FnMut(ContextEvent)>(&mut self, mut runner: F) -> bool {
        if self.exit_next_iteration { return false; }

        self.current_instant = Instant::now();

        let mut elapsed = self.current_instant.duration_since(self.previous_instant);
        if elapsed > self.max_frame_time { elapsed = self.max_frame_time; }

        self.last_frame_time = elapsed.as_secs_f64();
        self.running_time += elapsed.as_secs_f64();
        self.accumulated_time += elapsed.as_secs_f64();

        while self.accumulated_time >= self.fixed_time_step {
            (runner)(ContextEvent::UpdateEvent);

            self.accumulated_time -= self.fixed_time_step;
            self.number_of_updates += 1;
        }

        self.blending_factor = self.accumulated_time / self.fixed_time_step;

        if self.window_occluded {
            std::thread::sleep(Duration::from_secs_f64(self.fixed_time_step));
        } else {
            (runner)(ContextEvent::RenderEvent);
            self.number_of_renders += 1;
        }

        self.previous_instant = self.current_instant;

        true
    }

    pub fn run<F: FnMut(ContextEvent)>(&mut self, mut runner: F) {
        self.event_loop.take().run_return(move |event, _, control_flow|{  
            match event {
                Event::LoopDestroyed => (),
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        let size = WindowExtent::from(physical_size);
                        unsafe { gl::Viewport(0, 0, size.width as i32, size.height as i32); }
                        (runner)(ContextEvent::ResizeEvent(size));
                        self.ctx.resize(physical_size);
                    },
                    WindowEvent::Occluded(occluded) => self.window_occluded = occluded,
                    _ => {},
                },
                Event::RedrawRequested(_) => {
                    if !self.next_frame(&mut runner) {
                        *control_flow = ControlFlow::Exit;
                    }

                    self.ctx.swap_buffers().unwrap();
                },
                Event::MainEventsCleared => {
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