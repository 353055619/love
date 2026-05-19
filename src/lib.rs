use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Window};

const MESSAGE: &str = "LOVE YOU";
const HEART_CHARS: &[u8] = b"LOVE<RUST>";
const GRID_COLUMNS: usize = 62;
const GRID_ROWS: usize = 34;

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("missing window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("missing document"))?;
    let canvas = document
        .get_element_by_id("heart-canvas")
        .ok_or_else(|| JsValue::from_str("missing #heart-canvas"))?
        .dyn_into::<HtmlCanvasElement>()?;
    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("missing 2d canvas context"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    let scene = Rc::new(RefCell::new(Scene::new(window, canvas, context)));
    Scene::schedule(scene);
    Ok(())
}

struct Scene {
    window: Window,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    points: Vec<HeartPoint>,
    start_time: f64,
    last_width: u32,
    last_height: u32,
}

impl Scene {
    fn new(window: Window, canvas: HtmlCanvasElement, context: CanvasRenderingContext2d) -> Self {
        Self {
            window,
            canvas,
            context,
            points: heart_points(),
            start_time: js_sys::Date::now(),
            last_width: 0,
            last_height: 0,
        }
    }

    fn schedule(scene: Rc<RefCell<Self>>) {
        let callback = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
        let callback_ref = Rc::clone(&callback);
        let scene_ref = Rc::clone(&scene);

        *callback_ref.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
            scene_ref.borrow_mut().render(timestamp);

            let window = scene_ref.borrow().window.clone();
            if let Some(callback) = callback.borrow().as_ref() {
                let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
            }
        }) as Box<dyn FnMut(f64)>));

        if let Some(callback) = callback_ref.borrow().as_ref() {
            let _ = scene
                .borrow()
                .window
                .request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }

    fn render(&mut self, timestamp: f64) {
        self.resize_canvas();

        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;
        let elapsed = ((js_sys::Date::now() - self.start_time) / 1000.0).max(timestamp / 1000.0);
        let reveal = (elapsed / 4.2).clamp(0.0, 1.0);
        let pulse = 1.0 + 0.035 * (elapsed * 2.2).sin();
        let cell = (width / 76.0).min(height / 46.0).max(8.0);
        let center_x = width * 0.5;
        let center_y = height * 0.47;

        self.paint_background(width, height, elapsed);
        self.paint_scanlines(width, height);
        self.paint_prompt(width, cell);

        self.context.set_font(&format!(
            "700 {}px 'SFMono-Regular', Consolas, monospace",
            cell
        ));
        self.context.set_text_align("center");
        self.context.set_text_baseline("middle");

        for point in &self.points {
            if point.order > reveal {
                continue;
            }

            let age = ((reveal - point.order) * 9.0).clamp(0.0, 1.0);
            let flicker = 0.78 + 0.22 * (elapsed * 7.0 + point.seed).sin().abs();
            let alpha = (age * flicker).clamp(0.0, 1.0);
            let x = center_x + point.x * cell * pulse;
            let y = center_y + point.y * cell * pulse;

            self.context
                .set_fill_style_str(&format!("rgba(255, 74, 134, {alpha})"));
            let _ = self.context.fill_text(&point.ch.to_string(), x, y);
        }

        self.context.set_font(&format!(
            "800 {}px 'SFMono-Regular', Consolas, monospace",
            cell * 1.25
        ));
        self.context.set_fill_style_str("rgba(255, 231, 240, 0.95)");
        let _ = self
            .context
            .fill_text(MESSAGE, center_x, center_y + cell * 16.4);
    }

    fn resize_canvas(&mut self) {
        let ratio = self.window.device_pixel_ratio().max(1.0);
        let css_width = self
            .window
            .inner_width()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(390.0);
        let css_height = self
            .window
            .inner_height()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(720.0);
        let width = (css_width * ratio).round() as u32;
        let height = (css_height * ratio).round() as u32;

        if self.last_width == width && self.last_height == height {
            return;
        }

        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.last_width = width;
        self.last_height = height;
        let _ = self.context.set_transform(ratio, 0.0, 0.0, ratio, 0.0, 0.0);
    }

    fn paint_background(&self, width: f64, height: f64, elapsed: f64) {
        let width = width / self.window.device_pixel_ratio().max(1.0);
        let height = height / self.window.device_pixel_ratio().max(1.0);
        self.context.set_fill_style_str("#05060a");
        self.context.fill_rect(0.0, 0.0, width, height);

        self.context.set_fill_style_str("rgba(255, 42, 116, 0.08)");
        let glow = 90.0 + 12.0 * (elapsed * 1.8).sin();
        self.context.begin_path();
        let _ = self.context.ellipse(
            width * 0.5,
            height * 0.47,
            glow * 1.6,
            glow,
            0.0,
            0.0,
            PI * 2.0,
        );
        self.context.fill();
    }

    fn paint_scanlines(&self, width: f64, height: f64) {
        let ratio = self.window.device_pixel_ratio().max(1.0);
        let width = width / ratio;
        let height = height / ratio;
        self.context
            .set_fill_style_str("rgba(255, 255, 255, 0.025)");
        let mut y = 0.0;
        while y < height {
            self.context.fill_rect(0.0, y, width, 1.0);
            y += 5.0;
        }
    }

    fn paint_prompt(&self, width: f64, cell: f64) {
        let width = width / self.window.device_pixel_ratio().max(1.0);
        self.context.set_font(&format!(
            "600 {}px 'SFMono-Regular', Consolas, monospace",
            cell * 0.9
        ));
        self.context.set_text_align("left");
        self.context.set_text_baseline("top");
        self.context.set_fill_style_str("rgba(98, 255, 178, 0.78)");
        let _ = self
            .context
            .fill_text("cargo run --target wasm32-love", 18.0, 18.0);
        self.context.set_text_align("right");
        self.context.set_fill_style_str("rgba(98, 255, 178, 0.42)");
        let _ = self
            .context
            .fill_text("status: always yours", width - 18.0, 18.0);
    }
}

struct HeartPoint {
    x: f64,
    y: f64,
    ch: char,
    order: f64,
    seed: f64,
}

fn heart_points() -> Vec<HeartPoint> {
    let mut points = Vec::new();
    let mut index = 0usize;

    for row in 0..GRID_ROWS {
        for column in 0..GRID_COLUMNS {
            let x = (column as f64 / (GRID_COLUMNS - 1) as f64) * 34.0 - 17.0;
            let y = (row as f64 / (GRID_ROWS - 1) as f64) * 30.0 - 15.0;
            let normalized_x = x / 16.0;
            let normalized_y = -y / 14.0;

            let curve = (normalized_x * normalized_x + normalized_y * normalized_y - 1.0).powi(3)
                - normalized_x * normalized_x * normalized_y.powi(3);
            if curve > 0.0 {
                continue;
            }

            let distance = ((x / 17.0).powi(2) + (y / 15.0).powi(2)).sqrt();
            let order = (distance * 0.72 + row as f64 / GRID_ROWS as f64 * 0.28).clamp(0.0, 1.0);
            let ch = HEART_CHARS[index % HEART_CHARS.len()] as char;
            points.push(HeartPoint {
                x,
                y,
                ch,
                order,
                seed: (index as f64 * 1.618_033_988_75) % 12.0,
            });
            index += 1;
        }
    }

    points.sort_by(|a, b| a.order.total_cmp(&b.order));
    points
}
