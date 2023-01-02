use eframe::epaint::Hsva;
use egui::{
    lerp, pos2, remap_clamp, vec2, Color32, Mesh, Painter, Rect, Response, Rgba, Sense, Shape,
    Stroke, Ui, Vec2,
};

pub fn sigmoid(times: u32) -> f32 {
    let times: f64 = times as f64 / 20.;
    let res = ((1. / (1. + (-times).exp()) - 0.5) * 2.) as f32;
    res
}

/// a function
pub fn get_color(hue: f32, times: u32) -> Color32 {
    // let h = 220. / 360.;
    let k = (0.3 - 0.98) / 1.;
    let s = sigmoid(times);
    let v = k * s * s * s * s + 0.98;
    let srgb = Hsva::new(hue, s, v, 1.).to_srgb();
    Color32::from_rgb(srgb[0], srgb[1], srgb[2])
}

pub fn get_strike_color(color: Color32) -> Color32 {
    let mut hsv = Hsva::from_srgb([color.r(), color.g(), color.b()]);
    hsv.v -= 0.12;
    let srgb = hsv.to_srgb();
    Color32::from_rgb(srgb[0], srgb[1], srgb[2])
}

#[test]
fn test_get_color() {
    let times_vec: Vec<u32> = vec![0, 1, 10, 100, 1000];
    times_vec.iter().for_each(|&times| {
        let _color = get_color(210. / 360., times);
    });
}

/// copy from egui color_pickers
const N: u32 = 6 * 6;
pub fn color_slider_1d(
    ui: &mut Ui,
    value: &mut f32,
    color_at: impl Fn(f32) -> Color32,
) -> Response {
    #![allow(clippy::identity_op)]

    let desired_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    if let Some(mpos) = response.interact_pointer_pos() {
        *value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        background_checkers(ui.painter(), rect); // for alpha:

        {
            // fill color:
            let mut mesh = Mesh::default();
            for i in 0..=N {
                let t = i as f32 / (N as f32);
                let color = color_at(t);
                let x = lerp(rect.left()..=rect.right(), t);
                mesh.colored_vertex(pos2(x, rect.top()), color);
                mesh.colored_vertex(pos2(x, rect.bottom()), color);
                if i < N {
                    mesh.add_triangle(2 * i + 0, 2 * i + 1, 2 * i + 2);
                    mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
                }
            }
            ui.painter().add(Shape::mesh(mesh));
        }

        ui.painter().rect_stroke(rect, 0.0, visuals.bg_stroke); // outline

        {
            // Show where the slider is at:
            let x = lerp(rect.left()..=rect.right(), *value);
            let r = rect.height() / 4.0;
            let picked_color = color_at(*value);
            ui.painter().add(Shape::convex_polygon(
                vec![
                    pos2(x, rect.center().y),   // tip
                    pos2(x + r, rect.bottom()), // right bottom
                    pos2(x - r, rect.bottom()), // left bottom
                ],
                picked_color,
                Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
            ));
        }
    }

    response
}

fn background_checkers(painter: &Painter, rect: Rect) {
    let rect = rect.shrink(0.5); // Small hack to avoid the checkers from peeking through the sides
    if !rect.is_positive() {
        return;
    }

    let dark_color = Color32::from_gray(32);
    let bright_color = Color32::from_gray(128);

    let checker_size = Vec2::splat(rect.height() / 2.0);
    let n = (rect.width() / checker_size.x).round() as u32;

    let mut mesh = Mesh::default();
    mesh.add_colored_rect(rect, dark_color);

    let mut top = true;
    for i in 0..n {
        let x = lerp(rect.left()..=rect.right(), i as f32 / (n as f32));
        let small_rect = if top {
            Rect::from_min_size(pos2(x, rect.top()), checker_size)
        } else {
            Rect::from_min_size(pos2(x, rect.center().y), checker_size)
        };
        mesh.add_colored_rect(small_rect, bright_color);
        top = !top;
    }
    painter.add(Shape::mesh(mesh));
}

fn contrast_color(color: impl Into<Rgba>) -> Color32 {
    if color.into().intensity() < 0.5 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}
