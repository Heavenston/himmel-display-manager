mod app;

use std::fmt;

use skulpin::{
    CoordinateSystemHelper,
    skia_safe,
    rafx::api::RafxExtents2D 
};
use skia_safe::{
    Point, 
    Color, Color4f,
    Canvas, paint, Paint,
};
use winit::window::Fullscreen;

pub type Author = pam::Authenticator<'static, pam::PasswordConv>;

pub struct LoginResultEvent {
    pub success: bool,
    pub author: Author,
}

impl fmt::Debug for LoginResultEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoginResultEvent")
            .field("success", &self.success)
            .field("author", &())
            .finish()
    }
}

fn main() {
    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::<LoginResultEvent>::with_user_event();

    let monitor = event_loop.primary_monitor().or(event_loop.available_monitors().next());
    let fullscreen = monitor.as_ref()
        .and_then(|m| m.video_modes().next())
        .map(|vm| Fullscreen::Exclusive(vm));
    println!("Using fullscreen: {:?}", fullscreen.is_some());

    // Create a single window
    let window = winit::window::WindowBuilder::new()
        .with_title("Skulpin")
        .with_resizable(false)
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
        .coordinate_system(skulpin::CoordinateSystem::Logical)
        .build(&window, window_extents);

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }
    let mut renderer = renderer.unwrap();

    let login_callback = {
        let proxy = event_loop.create_proxy();
        move |username, password| {
            std::thread::spawn({
                let proxy = proxy.clone();
                move || {
                    let mut author = pam::Authenticator::with_password("system-auth").unwrap();
                    author.get_handler().set_credentials(username, password);
                    let auth_result = author.authenticate();
                    let r = proxy.send_event(LoginResultEvent {
                        success: auth_result.is_ok(),
                        author,
                    });
                    match r {
                        Ok(()) => (),
                        Err(e) => println!("{:?}", e),
                    }
                }
            });
        }
    };

    let mut app = app::App::new(login_callback, "malo", 4);

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

            winit::event::Event::WindowEvent { event, .. } => {
                if let Some(event) = event.to_static() {
                    app.add_window_event(event);
                }
            }

            winit::event::Event::MainEventsCleared => {
                // Queue a RedrawRequested event.
                window.request_redraw();
            }

            winit::event::Event::UserEvent(LoginResultEvent { success, author: _ }) => {
                app.login_result(success);
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
                        app.frame(canvas, coordinate_system_helper);
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
