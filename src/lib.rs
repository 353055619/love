use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Window};

const CARD_TEXTS: &[&str] = &[
    "好好爱自己",
    "天天开心",
    "多喝热水",
    "早点睡觉",
    "今天也很棒",
    "一直陪你",
    "记得吃饭",
    "别太累啦",
    "保持可爱",
    "我在这里",
    "想你了",
    "万事顺意",
    "平安喜乐",
    "慢慢来",
    "你最特别",
    "一起加油",
    "每天想你",
    "好运常在",
    "笑口常开",
    "有我在呢",
];

const CARD_COLORS: &[(&str, &str)] = &[
    ("#fff8fb", "#2f2730"),
    ("#ff9fd0", "#361625"),
    ("#8ef8db", "#123c35"),
    ("#b9f7a3", "#183814"),
    ("#bcecff", "#143342"),
    ("#fff3a6", "#413817"),
];

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
    let background_canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    let background_context = background_canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("missing background canvas context"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    let scene = Rc::new(RefCell::new(Scene::new(
        window,
        canvas,
        context,
        background_canvas,
        background_context,
    )));
    Scene::schedule(scene);
    Ok(())
}

struct Scene {
    window: Window,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    background_canvas: HtmlCanvasElement,
    background_context: CanvasRenderingContext2d,
    cards: Vec<Card>,
    layout: CardLayout,
    start_time: f64,
    last_width: u32,
    last_height: u32,
    render_ratio: f64,
}

impl Scene {
    fn new(
        window: Window,
        canvas: HtmlCanvasElement,
        context: CanvasRenderingContext2d,
        background_canvas: HtmlCanvasElement,
        background_context: CanvasRenderingContext2d,
    ) -> Self {
        Self {
            window,
            canvas,
            context,
            background_canvas,
            background_context,
            cards: Vec::new(),
            layout: CardLayout::new(0, 0),
            start_time: js_sys::Date::now(),
            last_width: 0,
            last_height: 0,
            render_ratio: 1.0,
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

        let width = self.canvas.width() as f64 / self.render_ratio;
        let height = self.canvas.height() as f64 / self.render_ratio;
        let elapsed = ((js_sys::Date::now() - self.start_time) / 1000.0).max(timestamp / 1000.0);
        let cycle = elapsed % 12.0;

        let _ = self
            .context
            .draw_image_with_html_canvas_element_and_dw_and_dh(
                &self.background_canvas,
                0.0,
                0.0,
                width,
                height,
            );

        for card in &self.cards {
            let frame = card.frame(cycle, width, height);
            self.paint_card(frame);
        }

        self.paint_caption(width, height, cycle);
    }

    fn resize_canvas(&mut self) {
        let ratio = effective_device_pixel_ratio(self.window.device_pixel_ratio());
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
        let layout = card_layout_for_width(css_width);

        if self.layout != layout {
            self.cards = cards(layout);
            self.layout = layout;
        }

        if self.last_width == width && self.last_height == height {
            return;
        }

        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.background_canvas.set_width(width);
        self.background_canvas.set_height(height);
        self.last_width = width;
        self.last_height = height;
        self.render_ratio = ratio;
        let _ = self.context.set_transform(ratio, 0.0, 0.0, ratio, 0.0, 0.0);
        let _ = self
            .background_context
            .set_transform(ratio, 0.0, 0.0, ratio, 0.0, 0.0);
        Self::paint_code_editor(&self.background_context, css_width, css_height);
    }

    fn paint_code_editor(context: &CanvasRenderingContext2d, width: f64, height: f64) {
        context.set_fill_style_str("#15181f");
        context.fill_rect(0.0, 0.0, width, height);
        context.set_fill_style_str("#0f1117");
        context.fill_rect(0.0, 0.0, 56.0, height);
        context.set_fill_style_str("rgba(255, 255, 255, 0.05)");
        context.fill_rect(56.0, 0.0, 1.0, height);

        context.set_font("12px 'SFMono-Regular', Menlo, Consolas, monospace");
        context.set_text_align("left");
        context.set_text_baseline("top");
        let snippets = [
            "fn main() {",
            "  let love = Card::new(\"you\");",
            "  loop {",
            "    heart.render(&love);",
            "  }",
            "}",
        ];

        for row in 0..32 {
            let y = 22.0 + row as f64 * 20.0;
            if y > height {
                break;
            }
            context.set_fill_style_str("rgba(155, 164, 180, 0.34)");
            let _ = context.fill_text(&(row + 1).to_string(), 18.0, y);
            context.set_fill_style_str("rgba(220, 232, 255, 0.28)");
            let _ = context.fill_text(snippets[row % snippets.len()], 74.0, y);
        }

        context.set_fill_style_str("rgba(255, 78, 156, 0.06)");
        context.begin_path();
        let _ = context.ellipse(
            width * 0.54,
            height * 0.45,
            width.min(height) * 0.42,
            width.min(height) * 0.28,
            0.0,
            0.0,
            PI * 2.0,
        );
        context.fill();
    }

    fn paint_card(&self, frame: CardFrame) {
        let w = frame.width;
        let h = frame.height;
        let x = frame.x - w / 2.0;
        let y = frame.y - h / 2.0;
        let radius = 5.0 * frame.scale;
        let shadow = 7.0 * frame.scale;

        self.context.save();
        self.context.set_global_alpha(frame.alpha);
        let _ = self.context.translate(frame.x, frame.y);
        let _ = self.context.rotate(frame.rotation);
        let _ = self.context.translate(-frame.x, -frame.y);

        self.context
            .set_fill_style_str(&format!("rgba(0, 0, 0, {})", 0.18 * frame.alpha));
        self.context.fill_rect(x + shadow, y + shadow, w, h);

        self.context.set_fill_style_str(frame.fill);
        rounded_rect(&self.context, x, y, w, h, radius);
        self.context.fill();

        self.context.set_fill_style_str("rgba(255, 255, 255, 0.45)");
        self.context.fill_rect(x + 5.0, y + 5.0, w - 10.0, 2.0);

        self.context.set_fill_style_str(frame.text_color);
        self.context.set_text_align("center");
        self.context.set_text_baseline("middle");
        self.context.set_font(&format!(
            "700 {}px -apple-system, BlinkMacSystemFont, 'PingFang SC', 'Microsoft YaHei', sans-serif",
            (11.0 * frame.scale).clamp(8.5, 15.0)
        ));
        let _ = self.context.fill_text(frame.text, frame.x, frame.y + 2.0);
        self.context.restore();
    }

    fn paint_caption(&self, width: f64, height: f64, cycle: f64) {
        let fade = if cycle < 7.2 {
            1.0
        } else {
            (1.0 - (cycle - 7.2) / 1.8).clamp(0.0, 1.0)
        };
        self.context.set_global_alpha(fade);
        self.context
            .set_font("800 22px 'SFMono-Regular', Menlo, Consolas, monospace");
        self.context.set_text_align("center");
        self.context.set_text_baseline("middle");
        self.context.set_fill_style_str("rgba(255, 242, 248, 0.94)");
        let _ = self
            .context
            .fill_text("LOVE YOU", width * 0.5, height * 0.82);
        self.context.set_global_alpha(1.0);
    }
}

struct Card {
    text: &'static str,
    fill: &'static str,
    text_color: &'static str,
    heart_x: f64,
    heart_y: f64,
    heart_depth: f64,
    wall_x: f64,
    wall_y: f64,
    start_x: f64,
    start_y: f64,
    start_depth: f64,
    order: f64,
    seed: f64,
}

impl Card {
    fn frame(&self, cycle: f64, width: f64, height: f64) -> CardFrame {
        let form = smoothstep(((cycle - self.order * 1.9) / 3.8).clamp(0.0, 1.0));
        let scatter = smoothstep(((cycle - 6.7 - self.order * 0.85) / 1.85).clamp(0.0, 1.0));
        let pulse = if cycle < 6.8 {
            1.0 + 0.035 * (cycle * 2.5 + self.seed).sin()
        } else {
            1.0
        };

        let heart_scale = width.min(height * 0.82) / 42.0;
        let heart_x = width * 0.52 + self.heart_x * heart_scale * pulse;
        let heart_y = height * 0.43 + self.heart_y * heart_scale * pulse;
        let start_x = self.start_x * width;
        let start_y = self.start_y * height;
        let wall_x = self.wall_x * width;
        let wall_y = self.wall_y * height;

        let formed_x = lerp(start_x, heart_x, form);
        let formed_y = lerp(start_y, heart_y, form);
        let formed_depth = lerp(self.start_depth, self.heart_depth, form);

        let x = lerp(formed_x, wall_x, scatter);
        let y = lerp(formed_y, wall_y, scatter);
        let depth = lerp(formed_depth, 0.15 + self.seed.fract() * 0.55, scatter);
        let scale = (0.66 + depth * 0.34) * lerp(0.72, 1.0, form);
        let alpha = (form * 1.25).clamp(0.0, 1.0);
        let rotation = lerp(-0.22 + self.seed.sin() * 0.18, self.seed.cos() * 0.04, form)
            + scatter * self.seed.sin() * 0.12;

        CardFrame {
            text: self.text,
            fill: self.fill,
            text_color: self.text_color,
            x,
            y,
            width: 66.0 * scale,
            height: 29.0 * scale,
            scale,
            alpha,
            rotation,
        }
    }
}

struct CardFrame {
    text: &'static str,
    fill: &'static str,
    text_color: &'static str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    scale: f64,
    alpha: f64,
    rotation: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CardLayout {
    per_layer: usize,
    layers: usize,
}

impl CardLayout {
    const fn new(per_layer: usize, layers: usize) -> Self {
        Self { per_layer, layers }
    }
}

fn effective_device_pixel_ratio(ratio: f64) -> f64 {
    ratio.clamp(1.0, 2.0)
}

fn card_layout_for_width(width: f64) -> CardLayout {
    if width <= 430.0 {
        CardLayout::new(36, 4)
    } else {
        CardLayout::new(42, 5)
    }
}

fn cards(layout: CardLayout) -> Vec<Card> {
    let mut cards = Vec::new();
    let per_layer = layout.per_layer;
    let layers = layout.layers;
    let total = per_layer * layers;

    for layer in 0..layers {
        for i in 0..per_layer {
            let index = layer * per_layer + i;
            let t = i as f64 / per_layer as f64 * PI * 2.0;
            let x = 16.0 * t.sin().powi(3);
            let y =
                -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
            let layer_offset = layer as f64 - (layers as f64 - 1.0) / 2.0;
            let color = CARD_COLORS[(index * 5 + layer) % CARD_COLORS.len()];
            let wall_column = index % 7;
            let wall_row = index / 7;
            let wall_x = 0.09 + wall_column as f64 * 0.14 + wave(index, 0.03);
            let wall_y = 0.10 + wall_row as f64 * 0.056 + wave(index + 11, 0.015);
            let start_side = if index % 2 == 0 { -0.16 } else { 1.16 };

            cards.push(Card {
                text: CARD_TEXTS[index % CARD_TEXTS.len()],
                fill: color.0,
                text_color: color.1,
                heart_x: x + layer_offset * 0.58,
                heart_y: y + layer_offset * 0.44,
                heart_depth: 0.35 + layer as f64 * 0.13 + (t.sin() + 1.0) * 0.06,
                wall_x,
                wall_y: wall_y.clamp(0.08, 0.92),
                start_x: start_side + wave(index, 0.12),
                start_y: 0.18 + ((index * 37) % total) as f64 / total as f64 * 0.68,
                start_depth: 0.05,
                order: index as f64 / total as f64,
                seed: index as f64 * 1.618_033_988_75,
            });
        }
    }

    cards.sort_by(|a, b| {
        a.heart_depth
            .total_cmp(&b.heart_depth)
            .then_with(|| a.order.total_cmp(&b.order))
    });
    cards
}

fn rounded_rect(context: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, r: f64) {
    context.begin_path();
    context.move_to(x + r, y);
    context.line_to(x + w - r, y);
    context.quadratic_curve_to(x + w, y, x + w, y + r);
    context.line_to(x + w, y + h - r);
    context.quadratic_curve_to(x + w, y + h, x + w - r, y + h);
    context.line_to(x + r, y + h);
    context.quadratic_curve_to(x, y + h, x, y + h - r);
    context.line_to(x, y + r);
    context.quadratic_curve_to(x, y, x + r, y);
    context.close_path();
}

fn smoothstep(value: f64) -> f64 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(start: f64, end: f64, amount: f64) -> f64 {
    start + (end - start) * amount
}

fn wave(seed: usize, amount: f64) -> f64 {
    ((seed as f64 * 12.9898).sin() * 43758.5453).fract() * amount
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_high_device_pixel_ratio_for_mobile_performance() {
        assert_eq!(effective_device_pixel_ratio(0.5), 1.0);
        assert_eq!(effective_device_pixel_ratio(1.5), 1.5);
        assert_eq!(effective_device_pixel_ratio(3.0), 2.0);
    }

    #[test]
    fn uses_fewer_cards_on_narrow_mobile_viewports() {
        assert_eq!(card_layout_for_width(390.0), CardLayout::new(36, 4));
        assert_eq!(card_layout_for_width(430.0), CardLayout::new(36, 4));
        assert_eq!(card_layout_for_width(431.0), CardLayout::new(42, 5));
    }
}
