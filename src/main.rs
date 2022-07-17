use skulpin::{
    CoordinateSystem, CoordinateSystemHelper,
    skia_safe,
    rafx::api::RafxExtents2D 
};
use skia_safe::{
    Point, 
    Color, Color3f, Color4f,
    Canvas, paint, Paint,
};
use winit::window::Fullscreen;

const CANVAS_WIDTH: f32 = 960.;
const CANVAS_HEIGHT: f32 = 600.;

fn main() {
    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    let logical_size = winit::dpi::LogicalSize::new(CANVAS_WIDTH, CANVAS_HEIGHT);
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: logical_size.width as f32,
        top: 0.0,
        bottom: logical_size.height as f32,
    };
    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Center;

    let monitor = event_loop.primary_monitor().or(event_loop.available_monitors().next());
    let fullscreen = monitor.as_ref()
        .and_then(|m| m.video_modes().next())
        .map(|vm| Fullscreen::Exclusive(vm));
    println!("Using fullscreen: {:?}", fullscreen.is_some());

    // Create a single window
    let window = winit::window::WindowBuilder::new()
        .with_title("Skulpin")
        .with_resizable(false)
        .with_inner_size(logical_size)
        .with_fullscreen(fullscreen)
        .build(&event_loop)
        .expect("Failed to create window");

    let window_size = window.inner_size();
    let window_extents =
        monitor.as_ref()
        .map(|m| m.size())
        .map(|s| RafxExtents2D {
            width: s.width,
            height: s.height,
        })
        .unwrap_or(RafxExtents2D {
            width: window_size.width,
            height: window_size.height,
        });

    // Create the renderer, which will draw to the window
    let renderer = skulpin::RendererBuilder::new()
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(&window, window_extents);

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }
    let mut renderer = renderer.unwrap();

    // Increment a frame count so we can render something that moves
    let mut frame_count = 0;

    // Start the window event loop. Winit will not return once run is called. We will get notified
    // when important events happen.
    event_loop.run(move |event, _window_target, control_flow| {
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,

            winit::event::Event::WindowEvent {
                event:
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,


            winit::event::Event::MainEventsCleared => {
                // Queue a RedrawRequested event.
                window.request_redraw();
            }

            winit::event::Event::RedrawRequested(_window_id) => {
                let window_size = window.inner_size();
                let window_extents = RafxExtents2D {
                    width: window_size.width,
                    height: window_size.height,
                };

                if let Err(e) = renderer.draw(
                    window_extents,
                    window.scale_factor(),
                    |canvas, coordinate_system_helper| {
                        draw(canvas, coordinate_system_helper, frame_count);
                        frame_count += 1;
                    },
                ) {
                    println!("Error during draw: {:?}", e);
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
            }

            _ => {}
        }
    });
}

fn draw(
    canvas: &mut Canvas,
    coordinate_system_helper: CoordinateSystemHelper,
    frame_count: i32,
) {
    let t = (frame_count as f32) / 60.;
    canvas.clear(Color::from_rgb(0, 0, 0));

    let mut paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
    paint.set_anti_alias(true);
    paint.set_style(paint::Style::StrokeAndFill);
    paint.set_stroke_width(2.0);

    canvas.draw_circle(
        Point::new(CANVAS_WIDTH / 2. + t.cos() * 50., CANVAS_HEIGHT / 2. + t.sin() * 50.),
        50.0,
        &paint,
    );
}
