mod app;
mod process_starts;
mod pam_wrapper;

use pam_wrapper::Author;

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

pub enum UserEvent {
    LoginResult {
        success: bool,
        username: String,
        password: String,
        author: Author,
    },
    StartSession {
        username: String,
        author: Author,
    }
}

impl fmt::Debug for UserEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserEvent")
            .finish()
    }
}

fn main() {
    if cfg!(not(feature="debug")) {
        process_starts::start_x_server();
    }

    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::<UserEvent>::with_user_event();
    let event_loop_proxy = event_loop.create_proxy();

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
        let proxy = event_loop_proxy.clone();
        move |username: String, password: String| {
            std::thread::spawn({
                let proxy = proxy.clone();
                move || {
                    let mut author = Author::new();
                    author
                        .set_username(username.as_str())
                        .set_password(password.as_str());
                    
                    proxy.send_event(UserEvent::LoginResult {
                        success: author.open_session().is_ok(),
                        username, password,
                        author,
                    }).expect("Could not send login event");
                }
            });
        }
    };

    let mut app = app::App::new(login_callback, "malo", 4);
    let mut do_on_quit: Vec<Box<dyn FnOnce() -> ()>> = Vec::new();

    event_loop.run(move |event, _start_x_serverwindow_target, control_flow| {
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
                window.request_redraw();
            }

            winit::event::Event::UserEvent(UserEvent::LoginResult{ success, author, username, .. }) => {
                let wait_duration = app.login_result(success);
                std::thread::spawn({
                    let proxy = event_loop_proxy.clone();
                    move || {
                        std::thread::sleep(wait_duration);
                        proxy.send_event(UserEvent::StartSession { username, author }).unwrap();
                    }
                });
            }

            winit::event::Event::UserEvent(UserEvent::StartSession { username, author }) => {
                #[cfg(not(feature = "debug"))]
                {
                    let mut child = process_starts::start_session(author, username);
                    do_on_quit.push(Box::new(move || {
                        child.wait().unwrap();
                    }));
                }

                *control_flow = winit::event_loop::ControlFlow::Exit;
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

            winit::event::Event::LoopDestroyed => {
                process_starts::stop_x_server();

                for f in do_on_quit.drain(..) {
                    f();
                }
            }

            _ => {}
        }
    });
}
