use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    spatial, ColorScalar, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing};
use crate::draw::{
    primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon},
    svg_renderer::SvgRenderContext,
};
use crate::geom::{self, Point2, Vector2};
use crate::math::{BaseFloat, ElementWise};
use crate::{color::conv::IntoLinSrgba, draw::svg_renderer::color_string};
use lyon::tessellation::StrokeOptions;
use palette::named::BLACK;
use svg::node::element::{path::Data, Path as SVGPath};

/// Properties related to drawing a **Quad**.
#[derive(Clone, Debug)]
pub struct Quad<S = geom::scalar::Default> {
    quad: geom::Quad<Point2<S>>,
    polygon: PolygonInit<S>,
    dimensions: spatial::dimension::Properties<S>,
}

/// The drawing context for a `Quad`.
pub type DrawingQuad<'a, S = geom::scalar::Default> = Drawing<'a, Quad<S>, S>;

// Quad-specific methods.

impl<S> Quad<S> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }

    /// Use the given four points as the vertices (corners) of the quad.
    pub fn points<P>(mut self, a: P, b: P, c: P, d: P) -> Self
    where
        P: Into<Point2<S>>,
    {
        let a = a.into();
        let b = b.into();
        let c = c.into();
        let d = d.into();
        self.quad = geom::Quad([a, b, c, d]);
        self
    }
}

// Trait implementations.

impl draw::renderer::RenderPrimitive for Quad<f32> {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
        let Quad {
            mut quad,
            polygon,
            dimensions,
        } = self;

        // If dimensions were specified, scale the points to those dimensions.
        let (maybe_x, maybe_y, _maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        if maybe_x.is_some() || maybe_y.is_some() {
            let cuboid = quad.bounding_rect();
            let centroid = quad.centroid();
            let x_scale = maybe_x.map(|x| x / cuboid.w()).unwrap_or(1.0);
            let y_scale = maybe_y.map(|y| y / cuboid.h()).unwrap_or(1.0);
            let scale = Vector2 {
                x: x_scale,
                y: y_scale,
            };
            let (a, b, c, d) = quad.into();
            let translate = |v: Point2| centroid + ((v - centroid).mul_element_wise(scale));
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            let new_d = translate(d);
            quad = geom::Quad([new_a, new_b, new_c, new_d]);
        }

        let points = quad.vertices();
        polygon::render_points_themed(
            polygon.opts,
            points,
            ctxt,
            &draw::theme::Primitive::Quad,
            mesh,
        );

        draw::renderer::PrimitiveRender::default()
    }
}

impl draw::svg_renderer::SvgRenderPrimitive<SVGPath> for Quad<f32> {
    fn render_svg_element(self, ctx: SvgRenderContext) -> SVGPath {
        let Quad {
            mut quad,
            polygon,
            dimensions,
        } = self;

        let color = polygon.opts.color.unwrap_or(BLACK.into_lin_srgba());
        let col_string = color_string(color);
        let global_transform = ctx.transform;
        let local_transform =
            polygon.opts.position.transform() * polygon.opts.orientation.transform();
        let transform = global_transform * local_transform;

        let transform_point =
            |v: Vector2<f32>| cgmath::Transform::transform_point(&transform, v.extend(0.0).into());

        let (maybe_x, maybe_y, _maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        if maybe_x.is_some() || maybe_y.is_some() {
            let cuboid = quad.bounding_rect();
            let centroid = quad.centroid();
            let x_scale = maybe_x.map(|x| x / cuboid.w()).unwrap_or(1.0);
            let y_scale = maybe_y.map(|y| y / cuboid.h()).unwrap_or(1.0);
            let scale = Vector2 {
                x: x_scale,
                y: y_scale,
            };
            let (a, b, c, d) = quad.into();
            let translate = |v: Point2| centroid + ((v - centroid).mul_element_wise(scale));
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            let new_d = translate(d);
            quad = geom::Quad([new_a, new_b, new_c, new_d]);
        }

        let mut points = quad.vertices();

        let mut data = Data::new();
        // TODO: handle unwrap
        let first = transform_point(points.next().unwrap());
        data = data.move_to((first.x, -first.y));
        for p in points {
            let tp = transform_point(p);
            data = data.line_to((tp.x, -tp.y));
        }
        data = data.line_to((first.x, -first.y));
        data = data.close();

        let mut el = SVGPath::new().set("fill", col_string).set("d", data);
        if let Some(stroke) = polygon.opts.stroke {
            el = el.set("stroke-width", stroke.line_width);
        }
        if let Some(stroke_color) = polygon.opts.stroke_color {
            el = el.set("stroke", color_string(stroke_color));
        }

        el
    }
}

impl<S> From<geom::Quad<Point2<S>>> for Quad<S>
where
    S: BaseFloat,
{
    fn from(quad: geom::Quad<Point2<S>>) -> Self {
        let polygon = Default::default();
        let dimensions = Default::default();
        Quad {
            polygon,
            dimensions,
            quad,
        }
    }
}

impl<S> Default for Quad<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        // Create a quad pointing towards 0.0 radians.
        let fifty = S::from(50.0).unwrap();
        let left = -fifty;
        let bottom = -fifty;
        let right = fifty;
        let top = fifty;
        let a = Point2 { x: left, y: bottom };
        let b = Point2 { x: left, y: top };
        let c = Point2 { x: right, y: top };
        let d = Point2 {
            x: right,
            y: bottom,
        };
        Quad::from(geom::Quad([a, b, c, d]))
    }
}

impl<S> SetOrientation<S> for Quad<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl<S> SetPosition<S> for Quad<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.polygon)
    }
}

impl<S> SetDimensions<S> for Quad<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl<S> SetColor<ColorScalar> for Quad<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.polygon)
    }
}

impl<S> SetStroke for Quad<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl<S> SetPolygon<S> for Quad<S> {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions<S> {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversions.

impl<S> From<Quad<S>> for Primitive<S> {
    fn from(prim: Quad<S>) -> Self {
        Primitive::Quad(prim)
    }
}

impl<S> Into<Option<Quad<S>>> for Primitive<S> {
    fn into(self) -> Option<Quad<S>> {
        match self {
            Primitive::Quad(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'a, S> DrawingQuad<'a, S>
where
    S: BaseFloat,
{
    /// Use the given points as the vertices (corners) of the quad.
    pub fn points<P>(self, a: P, b: P, c: P, d: P) -> Self
    where
        P: Into<Point2<S>>,
    {
        self.map_ty(|ty| ty.points(a, b, c, d))
    }
}
