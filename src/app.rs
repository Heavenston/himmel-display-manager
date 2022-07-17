use std::collections::HashSet;

use winit::event::{ ScanCode, KeyboardInput, WindowEvent, ElementState };
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

const SPACE_SCANCODE: ScanCode = 57;

pub struct App<'a> {
    last_events: Vec<WindowEvent<'a>>,
    pressed_keys: HashSet<ScanCode>,

    state: bool,
}

/// Public methods
impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            last_events: Default::default(),
            pressed_keys: Default::default(),

            state: false,
        }
    }

    pub fn add_window_event(&mut self, we: WindowEvent<'a>) {
        match &we {
            WindowEvent::KeyboardInput { input, .. } => {
                if input.state == ElementState::Pressed { 
                    self.pressed_keys.insert(input.scancode);
                }
                else {
                    self.pressed_keys.remove(&input.scancode);
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
        self.draw(canvas, coordinate_system_helper);
        self.last_events.clear();
    }
}

/// Private methods
impl<'a> App<'a> {
    fn is_key_just_pressed(&self, scan_code: ScanCode) -> bool {
        self.last_events.iter().any(|we|
            matches!(we,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { scancode, state: ElementState::Pressed, .. },
                    .. 
                } if *scancode == scan_code
            )
        )
    }
    fn is_key_just_released(&self, scan_code: ScanCode) -> bool {
        self.last_events.iter().any(|we|
            matches!(we,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { scancode, state: ElementState::Released, .. },
                    .. 
                } if *scancode == scan_code
            )
        )
    }
    fn is_key_pressed(&self, scan_code: ScanCode) -> bool {
        self.pressed_keys.contains(&scan_code)
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        coordinate_system_helper: CoordinateSystemHelper,
    ) {
        if self.is_key_just_pressed(SPACE_SCANCODE) {
            self.state = !self.state;
        }

        let extents =
            coordinate_system_helper.surface_extents();
        let (width, height) = 
            (extents.width as f32, extents.height as f32);
        canvas.clear(Color::from_rgb(0, 0, 0));

        let mut paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        paint.set_anti_alias(true);
        paint.set_style(paint::Style::StrokeAndFill);
        paint.set_stroke_width(2.0);

        let h = if self.state { 50. } else { 0. };
        canvas.draw_circle(
            Point::new(width / 2., height / 2. + h),
            50.0,
            &paint,
        );
    }
}
