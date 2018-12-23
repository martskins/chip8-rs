use crate::cpu::CPU;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;

struct App {
    cpu: CPU,
    gl: GlGraphics,
    keypad: [bool; 16],
}

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const SCALE: f64 = 10.0;

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;
        if !self.cpu.draw_screen {
            return;
        }
        let display = self.cpu.display;
        self.gl.draw(args.viewport(), |c, gl| {
            for (y, &row) in display.iter().enumerate() {
                for (x, &cell) in row.iter().enumerate() {
                    let x = (x as f64) * SCALE;
                    let y = (y as f64) * SCALE;
                    let square = rectangle::square(x, y, SCALE);
                    let color = if cell == 0 { BLACK } else { WHITE };
                    rectangle(color, square, c.transform, gl);
                }
            }
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        self.cpu.tick(self.keypad);
    }
}

pub fn start(path: &str) {
    use super::cpu::{SCREEN_HEIGHT, SCREEN_WIDTH};
    const WIDTH: u32 = (SCREEN_WIDTH as u32) * (SCALE as u32);
    const HEIGHT: u32 = (SCREEN_HEIGHT as u32) * (SCALE as u32);

    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("test", [WIDTH, HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .unwrap();

    let mut app = App {
        cpu: CPU::new(),
        gl: GlGraphics::new(opengl),
        keypad: [false; 16],
    };

    app.cpu.load_rom(path);

    let mut events = Events::new(EventSettings::new());
    events.ups(60);
    while let Some(e) = events.next(&mut window) {
        if let Event::Input(input) = e {
            if let Input::Button(button_args) = input {
                if let Button::Keyboard(key) = button_args.button {
                    if let Some(k) = map_key(key) {
                        app.keypad[k] = true;
                    }
                }
            }
        } else {
            if let Some(r) = e.render_args() {
                app.render(&r);
            }

            if let Some(u) = e.update_args() {
                app.update(&u);
            }
        }
    }
}

fn map_key(key: Key) -> Option<usize> {
    match key {
        Key::D7 => Some(0x00),
        Key::D8 => Some(0x01),
        Key::D9 => Some(0x02),
        Key::D0 => Some(0x03),
        Key::U => Some(0x04),
        Key::I => Some(0x05),
        Key::O => Some(0x06),
        Key::P => Some(0x07),
        Key::J => Some(0x08),
        Key::K => Some(0x09),
        Key::L => Some(0x0A),
        Key::Semicolon => Some(0x0B),
        Key::M => Some(0x0C),
        Key::Comma => Some(0x0D),
        Key::Period => Some(0x0E),
        Key::Slash => Some(0x0F),
        _ => None,
    }
}
