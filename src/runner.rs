use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::core::Core;

pub fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    pollster::block_on(async {
        let core = Core::new(&event_loop, &window).await;
        core.block_loop(event_loop, window);
    });
}
