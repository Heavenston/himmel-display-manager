use std::collections::HashSet;

use winit::event::{ ScanCode, KeyboardInput, WindowEvent, ElementState, VirtualKeyCode };
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
    pressed_keys: HashSet<VirtualKeyCode>,

    keys: isize,
}

/// Public methods
impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            last_events: Default::default(),
            pressed_keys: Default::default(),

            keys: 0,
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
        self.draw(canvas, coordinate_system_helper);
        self.last_events.clear();
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

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        coordinate_system_helper: CoordinateSystemHelper,
    ) {
        self.keys += self.last_events
            .iter().map(|we| -> isize { match we {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Back),
                        state: ElementState::Pressed, ..
                    }, ..
                } => -1,

                WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: ElementState::Pressed, .. }, ..
                } => 1,

                _ => 0,
            }}).sum::<isize>();

        let extents =
            coordinate_system_helper.surface_extents();
        let (width, height) = 
            (extents.width as f32, extents.height as f32);
        canvas.clear(Color::from_rgb(0, 0, 0));

        let mut paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        paint.set_anti_alias(true);
        paint.set_style(paint::Style::StrokeAndFill);
        paint.set_stroke_width(2.0);

        let w = self.keys as f32 * 25.;
        canvas.draw_circle(
            Point::new(w, height / 2.),
            50.0,
            &paint,
        );
    }
}
