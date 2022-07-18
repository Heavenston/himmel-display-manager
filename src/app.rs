use std::collections::HashSet;
use std::time::{ Instant, Duration };
use std::sync::{ Arc, Mutex, atomic::{ AtomicBool, self } };

use super::Author;

use winit::event::{ KeyboardInput, WindowEvent, ElementState, VirtualKeyCode };
use skulpin::{
    CoordinateSystemHelper,
    skia_safe,
};
use skia_safe::{
    Point, Rect,
    Color, Color4f,
    Canvas, paint, Paint,
};

#[derive(Clone)]
pub enum AppStage {
    Inputing {
        ball_red_flash_duration: Duration,
        ball_red_flash_start: Instant,
    },
    Validating {
        start: Instant,
        finished: bool,
        succeed: bool,
    },
    LoggingIn {
        start: Instant,
    }
}

impl AppStage {
    pub fn inputing() -> Self {
        AppStage::Inputing {
            ball_red_flash_duration: Duration::from_millis(500),
            ball_red_flash_start: Instant::now() - Duration::from_millis(500),
        }
    }

    #[track_caller]
    pub fn with_red_flash(&self, dur: Duration) -> Self {
        match self {
            Self::Inputing { .. } => Self::Inputing {
                ball_red_flash_duration: dur,
                ball_red_flash_start: Instant::now(),
            },
            _ => panic!("Cann of with_red_flash on non-inputing stage"),
        }
    }

    pub fn validating() -> Self {
        AppStage::Validating {
            start: Instant::now(),
            finished: false,
            succeed: false,
        }
    }

    pub fn logging_in() -> Self {
        AppStage::LoggingIn {
            start: Instant::now(),
        }
    }
}

pub struct App<F>
    where F: Fn(String, String) -> ()
{
    login_callback: F,

    last_events: Vec<WindowEvent<'static>>,
    pressed_keys: HashSet<VirtualKeyCode>,
    boxes_size: f32,
    login_username: String,
    pass_length: usize,

    ball_position: f32,
    ball_velocity: f32,
    stage: AppStage,

    last_update: Instant,
    current_input: String,
}

/// Public methods
impl<F> App<F>
    where F: Fn(String, String) -> ()
{
    pub fn new(login_callback: F, login_username: impl Into<String>, pass_length: usize) -> Self {
        Self {
            login_callback,
            
            last_events: Default::default(),
            pressed_keys: Default::default(),
            boxes_size: 100.,
            login_username: login_username.into(),
            pass_length,

            ball_position: 0.,
            ball_velocity: 0.,
            stage: AppStage::inputing(),

            last_update: Instant::now(),
            current_input: String::default(),
        }
    }

    pub fn add_window_event(&mut self, we: WindowEvent<'static>) {
        match &we {
            WindowEvent::KeyboardInput {
                input: KeyboardInput { state, virtual_keycode: Some(vkc), .. },
                .. 
            } => {
                if *state == ElementState::Pressed { 
                    self.pressed_keys.insert(*vkc);
                }
                else {
                    self.pressed_keys.remove(vkc);
                }
            }
            _ => (),
        }

        self.last_events.push(we);
    }
    
    pub fn frame(
        &mut self,
        canvas: &mut Canvas,
        coordinate_system_helper: CoordinateSystemHelper,
    ) {
        self.update(self.last_update.elapsed().as_secs_f32());
        self.draw(canvas, coordinate_system_helper);
        self.last_events.clear();

        self.last_update = Instant::now();
    }

    pub fn login_result(&mut self, s: bool) {
        match &mut self.stage {
            AppStage::Validating { finished, succeed, .. } => {
                *finished = true;
                *succeed = s;
            }
            
            _ => panic!("Called login_result with the app in the wrong stage"),
        }
    }
}

/// Private methods
impl<F> App<F>
    where F: Fn(String, String) -> (),
{
    fn is_key_just_pressed(&self, vck: VirtualKeyCode) -> bool {
        self.last_events.iter().any(|we|
            matches!(we,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode, state: ElementState::Pressed, .. },
                    .. 
                } if *virtual_keycode == Some(vck)
            )
        )
    }
    fn is_key_just_released(&self, vck: VirtualKeyCode) -> bool {
        self.last_events.iter().any(|we|
            matches!(we,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode, state: ElementState::Released, .. },
                    .. 
                } if *virtual_keycode == Some(vck)
            )
        )
    }
    fn is_key_pressed(&self, vck: VirtualKeyCode) -> bool {
        self.pressed_keys.contains(&vck)
    }

    fn update(&mut self, delta_t: f32) {
        let mut new_stage = None;
        match &mut self.stage {
            AppStage::Inputing { .. } => {
                self.ball_velocity -= 30. * delta_t;
                self.ball_position += self.ball_velocity * delta_t;

                let ball_min = self.current_input.chars().count() as f32;
                if self.ball_position < ball_min {
                    let diff = ball_min - self.ball_position;
                    let bounce = if diff < 0.2 { 0. } else { diff * 10. };

                    self.ball_position = ball_min;
                    self.ball_velocity = (self.ball_velocity.abs() / 2.) + bounce;
                }
            },

            AppStage::Validating { start, finished, succeed, .. } => {
                if start.elapsed().as_secs_f32() > 2. && *finished {
                    if *succeed {
                        new_stage = Some(AppStage::logging_in());
                    }
                    else {
                        self.current_input.clear();
                        new_stage = Some(AppStage::inputing().with_red_flash(Duration::from_millis(2000)));
                    }
                }
                else {
                    self.ball_velocity = self.ball_velocity / (1. + delta_t * 10.);
                    self.ball_position += (self.ball_velocity + 1.) * delta_t;
                }
            },

            AppStage::LoggingIn { .. } => (),
        }

        if let Some(new_stage) = new_stage {
            self.stage = new_stage;
        }
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        coordinate_system_helper: CoordinateSystemHelper,
    ) {
        let mut ball_radius = self.boxes_size / 3.;
        let boxes_gaps = 10.;
        let rect_stroke_width = 5.;
        let ball_stroke_width = 5.;

        let extents =
            coordinate_system_helper.surface_extents();
        let (width, height) = 
            (extents.width as f32, extents.height as f32);
        canvas.clear(Color::from_rgb(0, 0, 0));

        let mut fill_paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        fill_paint.set_anti_alias(true);
        fill_paint.set_style(paint::Style::Fill);

        let mut stroke_paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        stroke_paint.set_anti_alias(true);
        stroke_paint.set_style(paint::Style::Stroke);

        let full_rect_height = self.boxes_size * (self.pass_length + 1) as f32;

        /*
         * Drawing of squares and black outlines
         */
        fill_paint.set_color4f(Color4f::new(1., 1., 1., 1.), None);
        stroke_paint.set_stroke_width(boxes_gaps);
        stroke_paint.set_color4f(Color4f::new(0., 0., 0., 1.), None);
        for i in 0..self.pass_length {
            let x = width / 2. - self.boxes_size / 2.;
            let y = height / 2. + full_rect_height / 2. - (i as f32 + 1.) * self.boxes_size;
            let rect = Rect::new(
                x, y,
                x + self.boxes_size, y + self.boxes_size,
            );

            fill_paint.set_color4f(
                if i < self.current_input.chars().count() {
                    Color4f::new(1., 1., 1., 1.)
                }
                else {
                    Color4f::new(0., 0., 0., 1.)
                }
            , None);

            canvas.draw_rect(rect, &fill_paint);
            canvas.draw_line(
                Point::new(x, y),
                Point::new(x + self.boxes_size, y),
                &stroke_paint
            );
        }

        /*
         * Drawing of the white outline around everything
         */
        stroke_paint.set_stroke_width(rect_stroke_width);
        stroke_paint.set_color4f(Color4f::new(1., 1., 1., 1.), None);
        canvas.draw_rect(
            Rect::new(
                width  / 2. - self.boxes_size  / 2. - rect_stroke_width / 2.,
                height / 2. - full_rect_height / 2. - rect_stroke_width / 2.,
                width  / 2. + self.boxes_size  / 2. + rect_stroke_width / 2.,
                height / 2. + full_rect_height / 2. + rect_stroke_width / 2.,
            ),
            &stroke_paint
        );

        /*
         * Clearing the top
         */
        stroke_paint.set_stroke_width(rect_stroke_width);
        stroke_paint.set_color4f(Color4f::new(0., 0., 0., 1.), None);
        canvas.draw_line(
            Point::new(
                width / 2. - self.boxes_size / 2.,
                height / 2. - full_rect_height / 2. - rect_stroke_width / 2.,
            ),
            Point::new(
                width / 2. + self.boxes_size / 2.,
                height / 2. - full_rect_height / 2. - rect_stroke_width / 2.,
            ),
            &stroke_paint
        );

        let ball_center = Point::new(
            width / 2.,
            height / 2. + full_rect_height / 2. - ball_radius
            - (self.boxes_size * self.ball_position) + boxes_gaps / 2.
        );

        let mut next_stage = None;
        match self.stage {
            AppStage::Inputing { ball_red_flash_start, ball_red_flash_duration, .. } => {
                // Reading inputs
                for event in &self.last_events {
                    match event {
                        WindowEvent::KeyboardInput { input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Return), ..
                        }, .. } => {
                            if self.current_input.len() < self.pass_length {
                                next_stage = Some(self.stage.with_red_flash(
                                    Duration::from_millis(500)
                                ));
                            }
                            else {
                                (self.login_callback)(
                                    self.login_username.clone(),
                                    self.current_input.clone(),
                                );
                                next_stage = Some(AppStage::validating());
                            }
                        }

                        WindowEvent::KeyboardInput { input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Back), ..
                        }, .. } => {
                            self.current_input.pop();
                        }

                        WindowEvent::ReceivedCharacter(c) if c.is_alphanumeric() || c.is_ascii_punctuation() => {
                            if self.current_input.chars().count() < self.pass_length {
                                self.current_input.push(*c);
                            }
                        },

                        _ => (),
                    }
                }

                /*
                 * Drawing the balll
                 */
                // BLACK (or flashing) FILL
                fill_paint.set_color4f(Color4f::new(0., 0., 0., 1.), None);
                let flash_elapsed = ball_red_flash_start.elapsed();
                if flash_elapsed < ball_red_flash_duration {
                    let factor = flash_elapsed.as_secs_f32() / ball_red_flash_duration.as_secs_f32();
                    fill_paint.set_color4f(Color4f::new(1. - factor, 0., 0., 1.), None);
                }
                canvas.draw_circle(ball_center, ball_radius, &fill_paint);
            }

            AppStage::Validating { .. } => {

            },

            AppStage::LoggingIn { start } => {
                let elapsed = start.elapsed().as_secs_f32();
                ball_radius += (elapsed * 3.).exp();
            }
        }

        /*
         * Continuing ball fill after flashing background is drawn
         */
        // WHITE PROGRESS FILL
        fill_paint.set_color4f(Color4f::new(1., 1., 1., 1.), None);
        canvas.draw_arc(
            Rect::new(
                ball_center.x - ball_radius,
                ball_center.y - ball_radius,
                ball_center.x + ball_radius,
                ball_center.y + ball_radius,
            ),
            0., (self.current_input.chars().count() as f32 / self.pass_length as f32) * 360.,
            true,
            &fill_paint,
        );
        // WHITE STROKE
        stroke_paint.set_color4f(Color4f::new(1., 1., 1., 1.), None);
        stroke_paint.set_stroke_width(ball_stroke_width);
        canvas.draw_circle(
            ball_center,
            ball_radius - ball_stroke_width / 2.,
            &stroke_paint,
        );

        if let Some(next) = next_stage {
            self.stage = next;
        }
    }
}
