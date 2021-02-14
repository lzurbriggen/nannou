use crate::draw;

use crate::{
    draw::{primitive::Primitive, DrawCommand},
    Draw,
};
use crate::{App, Frame};
use palette::{LinSrgba, Srgb};
use svg::node::element::{Ellipse as SVGEllipse, Line as SVGLine, Path, Rectangle as SVGRectangle};
use svg::Document;
use svg::{
    node::element::{path::Data, Element},
    Node,
};

/// Draw API primitives that may be rendered via the **Renderer** type.
pub trait SvgRenderPrimitive<T> {
    /// Render self into the given mesh.
    fn render_svg_element(self, ctx: SvgRenderContext) -> T;
}

pub struct SvgRenderContext<'a> {
    pub transform: &'a crate::math::Matrix4<f32>,
    pub theme: &'a draw::Theme,
    // pub intermediary_mesh: &'a draw::Mesh,
    // pub path_event_buffer: &'a [PathEvent],
    // pub path_points_colored_buffer: &'a [(Point2, Color)],
    // pub path_points_textured_buffer: &'a [(Point2, Point2)],
    // pub text_buffer: &'a str,
    // pub theme: &'a draw::Theme,
    // pub glyph_cache: &'a mut GlyphCache,
    // pub fill_tessellator: &'a mut FillTessellator,
    // pub stroke_tessellator: &'a mut StrokeTessellator,
    // pub output_attachment_size: Vector2, // logical coords
    // pub output_attachment_scale_factor: f32,
}

pub fn color_string(color: LinSrgba) -> String {
    let fromlin = Srgb::from_linear(color.color);
    format!(
        "rgba({}, {}, {}, {})",
        fromlin.red * 255.0,
        fromlin.green * 255.0,
        fromlin.blue * 255.0,
        color.alpha as f32 * 255.0
    )
}

pub fn to_svg(app: &App, draw: &Draw, frame: &Frame) -> Document {
    let window_id = frame.window_id();
    let window = app
        .window(window_id)
        .expect("no window to draw to for `Draw`'s window_id");
    let win_size = window.inner_size_pixels();

    let draw_cmds: Vec<_> = draw.drain_commands().collect();
    let draw_state = draw.state.borrow();

    let mut document = Document::new().set(
        "viewBox",
        (
            -(win_size.0 as i32) / 2,
            -(win_size.1 as i32) / 2,
            win_size.0,
            win_size.1,
        ),
    );

    if let Some(bg_color) = draw_state.background_color {
        let background = SVGRectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("x", "-50%")
            .set("y", "-50%")
            .set("fill", color_string(bg_color));
        document = document.add(background);
    }

    let mut curr_ctxt = draw::Context::default();

    for draw_cmd in draw_cmds {
        // Track the prev index and vertex counts.
        // let prev_index_count = self.mesh.indices().len() as u32;
        // let prev_vert_count = self.mesh.vertex_count();

        // Info required during rendering.
        let ctx = SvgRenderContext {
            // intermediary_mesh: &intermediary_state.intermediary_mesh,
            // path_event_buffer: &intermediary_state.path_event_buffer,
            // path_points_colored_buffer: &intermediary_state.path_points_colored_buffer,
            // path_points_textured_buffer: &intermediary_state.path_points_textured_buffer,
            // text_buffer: &intermediary_state.text_buffer,
            theme: &draw_state.theme,
            transform: &curr_ctxt.transform,
            // fill_tessellator: &mut fill_tessellator,
            // stroke_tessellator: &mut stroke_tessellator,
            // glyph_cache: &mut self.glyph_cache,
            // output_attachment_size: Vector2::new(px_to_pt(w_px), px_to_pt(h_px)),
            // output_attachment_scale_factor: scale_factor,
        };

        match draw_cmd {
            DrawCommand::Primitive(p) => match p {
                Primitive::Arrow(_) => {}
                Primitive::Ellipse(e) => {
                    document = document.add(e.render_svg_element(ctx));
                }
                Primitive::Line(e) => {
                    // let color = e.path.color.unwrap();
                    // let col_string = color_string(color);
                    // let local_transform =
                    //     e.path.position.transform() * e.path.orientation.transform();

                    // let start = e.start.unwrap().extend(0.0);
                    // let start_t =
                    //     cgmath::Transform::transform_point(&local_transform, start.into());

                    // let end = e.end.unwrap().extend(0.0);
                    // let end_t = cgmath::Transform::transform_point(&local_transform, end.into());

                    // let cap = match e.path.opts.start_cap {
                    //     lyon::lyon_tessellation::LineCap::Butt => "butt",
                    //     lyon::lyon_tessellation::LineCap::Square => "square",
                    //     lyon::lyon_tessellation::LineCap::Round => "round",
                    // };

                    // let el = SVGLine::new()
                    //     .set("stroke", col_string)
                    //     .set("x1", start_t.x)
                    //     .set("y1", -start_t.y)
                    //     .set("x2", end_t.x)
                    //     .set("y2", -end_t.y)
                    //     .set("stroke-width", e.path.opts.line_width)
                    //     .set("stroke-linecap", cap);
                    // document = document.add(el);

                    document = document.add(e.render_svg_element(ctx));
                }
                Primitive::MeshVertexless(_) => {}
                Primitive::Mesh(_) => {}
                Primitive::PathInit(_) => {}
                Primitive::PathFill(_) => {}
                Primitive::PathStroke(_) => {}
                Primitive::Path(_) => {}
                Primitive::PolygonInit(_) => {}
                Primitive::Polygon(e) => {
                    document = document.add(e.render_svg_element(ctx));
                }
                Primitive::Quad(e) => {
                    document = document.add(e.render_svg_element(ctx));
                }
                Primitive::Rect(e) => {
                    document = document.add(e.render_svg_element(ctx));
                }
                Primitive::Text(_) => {}
                Primitive::Texture(_) => {}
                Primitive::Tri(e) => {
                    document = document.add(e.render_svg_element(ctx));
                }
            },
            DrawCommand::Context(c) => {
                curr_ctxt = c;
            }
        }
    }
    document
}
