use std::collections::HashSet;
use std::time::Instant;

use winit::event::{ ScanCode, KeyboardInput, WindowEvent, ElementState, VirtualKeyCode };
use skulpin::{
     CoordinateSystemHelper,
    skia_safe,
    rafx::api::RafxExtents2D 
};
use skia_safe::{
    Point, Rect,
    Color, Color4f,
    Canvas, paint, Paint,
};

const SPACE_SCANCODE: ScanCode = 57;

pub struct App<'a> {
    last_events: Vec<WindowEvent<'a>>,
    pressed_keys: HashSet<VirtualKeyCode>,
    boxes_size: f32,

    ball_position: f32,
    ball_velocity: f32,

    last_update: Instant,
    pass_length: usize,
    current_input: String,
}

/// Public methods
impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            last_events: Default::default(),
            pressed_keys: Default::default(),
            boxes_size: 100.,

            ball_position: 0.,
            ball_velocity: 0.,

            last_update: Instant::now(),
            pass_length: 4,
            current_input: String::default(),
        }
    }

    pub fn add_window_event(&mut self, we: WindowEvent<'a>) {
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
}

/// Private methods
impl<'a> App<'a> {
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
        let ball_min = self.current_input.chars().count() as f32;

        self.ball_velocity -= 30. * delta_t;
        self.ball_position += self.ball_velocity * delta_t;

        if self.ball_position < ball_min {
            let diff = ball_min - self.ball_position;
            let bounce = if diff < 0.2 { 0. } else { diff * 10. };

            self.ball_position = ball_min;
            self.ball_velocity = (self.ball_velocity.abs() / 2.) + bounce;
        }
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        coordinate_system_helper: CoordinateSystemHelper,
    ) {
        let ball_radius = self.boxes_size / 3.;
        let boxes_gaps = 5.;
        let rect_stroke_width= 5.;

        for event in &self.last_events {
            match event {
                WindowEvent::ReceivedCharacter(c) if c.is_alphanumeric() || c.is_ascii_punctuation() => {
                    if self.current_input.chars().count() < self.pass_length {
                        self.current_input.push(*c);
                    }
                },
                WindowEvent::KeyboardInput { input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Back), ..
                }, .. } => {
                    self.current_input.pop();
                }
                _ => (),
            }
        }

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
                width / 2. - self.boxes_size / 2. - rect_stroke_width / 2.,
                height / 2. - full_rect_height / 2. - rect_stroke_width / 2.,
                width / 2. + self.boxes_size / 2. + rect_stroke_width / 2.,
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

        /*
         * Drawing the balll
         */
        fill_paint.set_color4f(Color4f::new(1., 1., 1., 1.), None);
        canvas.draw_circle(
            Point::new(
                width / 2.,
                height / 2. + full_rect_height / 2. - ball_radius - (self.boxes_size * self.ball_position)
            ),
            ball_radius,
            &fill_paint,
        );
    }
}
