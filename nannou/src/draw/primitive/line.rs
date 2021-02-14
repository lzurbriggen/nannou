use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{ColorScalar, SetColor, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use crate::draw::{primitive::path, svg_renderer::color_string};
use crate::geom::{self, pt2, Point2};
use crate::math::{BaseFloat, Zero};
use crate::{color::LinSrgba, draw::svg_renderer::SvgRenderContext};
use crate::{
    draw::primitive::{PathStroke, Primitive},
    prelude::Vector2,
};
use lyon::tessellation::StrokeOptions;
use svg::node::element::{path::Data, Line as SVGLine};

/// A path containing only two points - a start and end.
///
/// The usage of this type is almost identical to `PathStroke` but provides `start`, `end` and
/// `points(a, b)` methods.
#[derive(Clone, Debug)]
pub struct Line<S = geom::scalar::Default> {
    pub path: PathStroke<S>,
    pub start: Option<Point2<S>>,
    pub end: Option<Point2<S>>,
}

/// The drawing context for a line.
pub type DrawingLine<'a, S = geom::scalar::Default> = Drawing<'a, Line<S>, S>;

impl<S> Line<S> {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_path(|p| p.stroke_weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_path(|p| p.stroke_tolerance(tolerance))
    }

    /// Specify the start point of the line.
    pub fn start(mut self, start: Point2<S>) -> Self {
        self.start = Some(start);
        self
    }

    /// Specify the end point of the line.
    pub fn end(mut self, end: Point2<S>) -> Self {
        self.end = Some(end);
        self
    }

    /// Specify the start and end points of the line.
    pub fn points(self, start: Point2<S>, end: Point2<S>) -> Self {
        self.start(start).end(end)
    }

    // Map the inner `PathStroke<S>` using the given function.
    fn map_path<F>(self, map: F) -> Self
    where
        F: FnOnce(PathStroke<S>) -> PathStroke<S>,
    {
        let Line { path, start, end } = self;
        let path = map(path);
        Line { path, start, end }
    }
}

impl<'a, S> DrawingLine<'a, S>
where
    S: BaseFloat,
{
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_ty(|ty| ty.weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.tolerance(tolerance))
    }

    /// Specify the start point of the line.
    pub fn start(self, start: Point2<S>) -> Self {
        self.map_ty(|ty| ty.start(start))
    }

    /// Specify the end point of the line.
    pub fn end(self, end: Point2<S>) -> Self {
        self.map_ty(|ty| ty.end(end))
    }

    /// Specify the start and end points of the line.
    pub fn points(self, start: Point2<S>, end: Point2<S>) -> Self {
        self.map_ty(|ty| ty.points(start, end))
    }
}

impl<S> SetStroke for Line<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.path)
    }
}

impl<S> SetOrientation<S> for Line<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.path)
    }
}

impl<S> SetPosition<S> for Line<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.path)
    }
}

impl<S> SetColor<ColorScalar> for Line<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.path)
    }
}

impl<S> From<Line<S>> for Primitive<S> {
    fn from(prim: Line<S>) -> Self {
        Primitive::Line(prim)
    }
}

impl<S> Into<Option<Line<S>>> for Primitive<S> {
    fn into(self) -> Option<Line<S>> {
        match self {
            Primitive::Line(prim) => Some(prim),
            _ => None,
        }
    }
}

impl draw::renderer::RenderPrimitive for Line<f32> {
    fn render_primitive(
        self,
        mut ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
        let Line { path, start, end } = self;
        let start = start.unwrap_or(pt2(0.0, 0.0));
        let end = end.unwrap_or(pt2(0.0, 0.0));
        if start == end {
            return draw::renderer::PrimitiveRender::default();
        }
        let close = false;
        let points = [start, end];
        let points = points.iter().cloned().map(Into::into);
        let events = lyon::path::iterator::FromPolyline::new(close, points);

        // Determine the transform to apply to all points.
        let global_transform = ctxt.transform;
        let local_transform = path.position.transform() * path.orientation.transform();
        let transform = global_transform * local_transform;

        path::render_path_events(
            events,
            path.color,
            transform,
            path::Options::Stroke(path.opts),
            &ctxt.theme,
            &draw::theme::Primitive::Line,
            &mut ctxt.fill_tessellator,
            &mut ctxt.stroke_tessellator,
            mesh,
        );

        draw::renderer::PrimitiveRender::default()
    }
}

impl draw::svg_renderer::SvgRenderPrimitive<SVGLine> for Line<f32> {
    fn render_svg_element(self, ctx: SvgRenderContext) -> SVGLine {
        let Line { path, start, end } = self;

        let start = start.unwrap_or(pt2(0.0, 0.0));
        let end = end.unwrap_or(pt2(0.0, 0.0));
        // TODO if start == end {
        //     return draw::renderer::PrimitiveRender::default();
        // }

        let global_transform = ctx.transform;
        let local_transform = path.position.transform() * path.orientation.transform();
        let transform = global_transform * local_transform;

        let transform_point =
            |v: Vector2<f32>| cgmath::Transform::transform_point(&transform, v.extend(0.0).into());

        let close = false;
        let points = [start, end];
        let points = points.iter().cloned().map(Into::into);
        let events = lyon::path::iterator::FromPolyline::new(close, points);

        let mut el = SVGLine::new();
        let cap = match path.opts.start_cap {
            lyon::lyon_tessellation::LineCap::Butt => "butt",
            lyon::lyon_tessellation::LineCap::Square => "square",
            lyon::lyon_tessellation::LineCap::Round => "round",
        };
        let color = path.color.unwrap();
        let col_string = color_string(color);
        el = el.set("stroke", col_string);
        el = el.set("stroke-linecap", cap);
        el = el.set("stroke-width", path.opts.line_width);

        println!("{:?}", path);
        println!("{:?}", start);
        println!("{:?}", end);

        for e in events {
            println!("{:?}", e);
            match e {
                lyon::path::Event::Begin { at } => {}
                lyon::path::Event::Line { from, to } => {
                    let from = transform_point(Vector2::new(from.x, from.y));
                    let to = transform_point(Vector2::new(to.x, to.y));

                    el = el
                        .set("x1", from.x)
                        .set("y1", -from.y)
                        .set("x2", to.x)
                        .set("y2", -to.y);
                }
                lyon::path::Event::Quadratic { from, ctrl, to } => {}
                lyon::path::Event::Cubic {
                    from,
                    ctrl1,
                    ctrl2,
                    to,
                } => {}
                lyon::path::Event::End { last, first, close } => {}
            }
        }

        el
    }
}

impl<S> Default for Line<S>
where
    S: Zero,
{
    fn default() -> Self {
        Line {
            path: Default::default(),
            start: Default::default(),
            end: Default::default(),
        }
    }
}
