use nannou::{draw::svg_renderer::to_svg, prelude::*};

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App, frame: Frame) {
    let draw = app.draw();

    draw.background().color(CORNFLOWERBLUE);

    let win = app.window_rect();

    let get_pos = |index: usize| {
        let padding = 100.0;
        let w = win.w() - padding * 2.0;
        let h = win.h() - padding * 2.0;
        let cols = 4;
        let rows = 3;
        let col = index % cols;
        let row = index / cols;
        (
            col as f32 * w / cols as f32 - w / 2.0 + padding,
            -(row as f32 * h / rows as f32 - h / 2.0 + padding),
        )
    };

    let pos = get_pos(0);
    draw.arrow()
        .x_y(pos.0, pos.1)
        .weight(10.0)
        .points(pt2(0.0, -60.0), pt2(0.0, 60.0))
        .color(RED);

    let pos = get_pos(1);
    draw.ellipse()
        .x_y(pos.0, pos.1)
        .stroke_color(BLUE)
        .stroke_weight(5.0)
        .radius(100.0)
        .height(60.0)
        // .z_radians(TAU * 0.35)
        .color(RED);

    let pos = get_pos(2);
    draw.line()
        .x_y(pos.0, pos.1)
        .weight(10.0 + (0.5 * 0.5 + 0.5) * 90.0)
        .caps_round()
        .color(PALEGOLDENROD)
        .points(vec2(0.0, -50.0), vec2(0.0, 50.0));

    let pos = get_pos(3);
    //  MESH

    let pos = get_pos(4);
    // PATH

    let pos = get_pos(5);
    // POLYGON

    let pos = get_pos(6);
    draw.quad()
        .x_y(pos.0, pos.1)
        .color(DARKGREEN)
        .stroke_color(YELLOW)
        .stroke_weight(5.0)
        .rotate(TAU * 0.4);

    let pos = get_pos(7);
    draw.rect()
        .x_y(pos.0, pos.1)
        .w(78.0)
        .hsv(0.3, 1.0, 1.0)
        .rotate(TAU * 0.4);

    let pos = get_pos(8);
    // TEXT

    let pos = get_pos(9);
    // TEXTURE

    let pos = get_pos(10);
    draw.tri()
        .x_y(pos.0, pos.1)
        .points(vec2(-80.0, 50.0), vec2(-50.0, -60.0), vec2(75.0, 3.0))
        .stroke_color(RED)
        .stroke_weight(1.0)
        .color(VIOLET);

    if app.elapsed_frames() == 1 {
        let document = to_svg(app, &draw, &frame);
        svg::save("svg_primitives.svg", &document).unwrap();
    }

    draw.to_frame(app, &frame).unwrap();
}
