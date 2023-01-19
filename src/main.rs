use nannou::{image::{DynamicImage, ImageBuffer}, prelude::*, wgpu::Texture, color::FromColor};
use num_complex::Complex64;
use rayon::prelude::*;
use std::ops::Range;

const STEP_DIV: usize = 100;
const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;
const WH_SIZE: u32 = WIDTH * HEIGHT;
const WIDTH64: f64 = WIDTH as f64;
const HEIGHT64: f64 = HEIGHT as f64;
const RANGE_X: Range<f64> = -2.00..0.47;
const RANGE_Y: Range<f64> = -1.12..1.12;

struct Model {
    dragging: bool,
    invalid: bool,
    iterations: usize,
    mouse_pos0: Option<Point2>,
    mouse_pos1: Option<Point2>,
    offset: usize,
    range_x: Range<f64>,
    range_y: Range<f64>,
    running: bool,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(WIDTH, HEIGHT)
        .resizable(false)
        .view(view)
        .key_pressed(key_pressed)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .mouse_released(mouse_released)
        .build()
        .unwrap();
    Model {
        dragging: false,
        invalid: true,
        iterations: 0,
        mouse_pos0: None,
        mouse_pos1: None,
        offset: 0,
        range_x: RANGE_X.clone(),
        range_y: RANGE_Y.clone(),
        running: true,
    }
}

fn key_pressed(_: &App, model: &mut Model, key: Key) {
    match &key {
        Key::R => {
            model.offset = model.iterations;
        },
        Key::Space => {
            model.running = !model.running;
        }
        _ => ()
    }
}

fn mouse_pressed(app: &App, model: &mut Model, button: MouseButton) {
    if button == MouseButton::Left {
        model.dragging = true;
        model.mouse_pos0 = app.mouse.position().into();
        model.mouse_pos1 = model.mouse_pos0.clone();
    }
}

fn mouse_moved(app: &App, model: &mut Model, pos: Point2) {
    if model.dragging {
        // maintain same aspect ratio as window
        let rect = app.window_rect();
        let pos0 = model.mouse_pos0.unwrap();
        let y = model.mouse_pos0.unwrap().y - (pos.x - pos0.x) * rect.h() * rect.w().recip();
        model.mouse_pos1 = Some(Point2::new(pos.x, y));
    }
}

fn mouse_released(app: &App, model: &mut Model, button: MouseButton) {
    match &button {
        MouseButton::Left => {
            model.invalid = true;
            model.dragging = false;

            let pos0 = model.mouse_pos0.unwrap();
            let pos1 = model.mouse_pos1.unwrap();
            if pos0 != pos1 {
                let rect = app.window_rect();
                let x0 = map_range(pos0.x, rect.left(), rect.left() + rect.w(), model.range_x.start, model.range_x.end);
                let x1 = map_range(pos1.x, rect.left(), rect.left() + rect.w(), model.range_x.start, model.range_x.end);
                let y0 = map_range(pos0.y, rect.top(), rect.top() - rect.h(), model.range_y.start, model.range_y.end);
                let y1 = map_range(pos1.y, rect.top(), rect.top() - rect.h(), model.range_y.start, model.range_y.end);
                model.range_x = if x0 < x1 { x0..x1 } else { x1..x0 };
                model.range_y = if y0 < y1 { y0..y1 } else { y1..y0 };
            }
        }
        MouseButton::Right => {
            model.offset = model.iterations;
            model.range_x = RANGE_X.clone();
            model.range_y = RANGE_Y.clone();
            model.invalid = true;
        },
        _ => ()
    }
}

fn update(_: &App, model: &mut Model, update: Update) {
    let d = update.since_start.as_millis() as usize / STEP_DIV;
    if model.invalid && model.running {
        model.invalid = false; 
    }
    if d != model.iterations {
        model.invalid = true;
        model.iterations = d;
        if !model.running {
            model.offset = model.iterations;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    if !model.running || !(model.invalid || model.dragging) { return }
    let iterations = model.iterations - model.offset;
    // let now = Instant::now();
    
    let buf = (0..WH_SIZE)
        .into_par_iter()
        .map(|i| to_color(mandelbrot(
            i as f64,
            iterations,
            &model.range_x,
            &model.range_y
        ), iterations))
        .collect::<Vec<[u8;3]>>()
        .into_iter()
        .flat_map(|v| v)
        .collect();

    let img = ImageBuffer::from_raw(WIDTH, HEIGHT, buf).unwrap();
    let view = Texture::from_image(app, &DynamicImage::ImageRgb8(img));

    let draw = app.draw();
    draw.texture(&view);

    if model.dragging {
        let pos0 = model.mouse_pos0.unwrap();
        let pos1 = model.mouse_pos1.unwrap();
        draw.rect()
            .xy((pos0 + pos1) * 0.5)
            .wh(pos1 - pos0)
            .stroke_color(GREEN)
            .stroke_weight(1.0)
            .no_fill();
    }

    draw.to_frame(app, &frame).unwrap();
}

fn mandelbrot(i: f64, n: usize, rx: &Range<f64>, ry: &Range<f64>) -> usize {
    let c = Complex64::new(
        map_rrange(i % WIDTH64, WIDTH64, &rx),
        map_rrange(i / WIDTH64, HEIGHT64, &ry)
    );
    let mut z = Complex64::new(0.0, 0.0);
    let mut j = 0;
    while j < n && z.norm() <= 2.0 {
        z = z * z + c;
        j += 1;
    }
    j
}

// fn _julia(i: f64, n: usize, rx: &Range<f64>, ry: &Range<f64>) -> usize {
//     // let c = Complex64::new(0.285, 0.01);
//     let c = Complex64::new(-0.7269, 0.1889);
//     let mut z = Complex64::new(
//         map_rrange(i % WIDTH64, WIDTH64, &rx),
//         map_rrange(i / WIDTH64, HEIGHT64, &ry)
//     );
//     let mut j = 0;
//     while j < n && z.norm() < 2.0 {
//         z = z * z + c;
//         j += 1;
//     }
//     j
// }

fn to_color(value: usize, limit: usize) -> [u8; 3] {
    let hue = value as f32 / limit as f32;
    let hsv = hsv(hue, 1.0, if value < limit { 1.0 } else { 0.0 });
    let rgb = Rgb::from_hsv(hsv);
    let u8_max = u8::MAX as f32;
    [
        (u8_max * rgb.red) as u8,
        (u8_max * rgb.green) as u8,
        (u8_max * rgb.blue) as u8
    ]
}

fn map_rrange(val: f64, in_max: f64, out_range: &Range<f64>) -> f64 {
    out_range.start + (out_range.end - out_range.start) * in_max.recip() * val
}