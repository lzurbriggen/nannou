use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::orientation;
use crate::draw::properties::spatial::{dimension, position};
use crate::draw::properties::{
    spatial, ColorScalar, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::Drawing;
use crate::draw::{self, svg_renderer::SvgRenderContext};
use crate::geom::{self, Vector2};
use crate::math::{rad_to_deg, BaseFloat, Zero};
use crate::{color::conv::IntoLinSrgba, draw::svg_renderer::color_string};
use cgmath::{Euler, Matrix3, Matrix4, Point3, Quaternion, Vector3};
use lyon::tessellation::StrokeOptions;
use palette::{named::BLACK, Alpha};
use svg::{
    node::element::{Element, Ellipse as SVGEllipse},
    Node,
};

/// Properties related to drawing an **Ellipse**.
#[derive(Clone, Debug)]
pub struct Ellipse<S = geom::scalar::Default> {
    dimensions: spatial::dimension::Properties<S>,
    resolution: Option<usize>,
    polygon: PolygonInit<S>,
}

/// The drawing context for an ellipse.
pub type DrawingEllipse<'a, S = geom::scalar::Default> = Drawing<'a, Ellipse<S>, S>;

// Ellipse-specific methods.

impl<S> Ellipse<S>
where
    S: BaseFloat,
{
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }

    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: S) -> Self {
        let side = radius * (S::one() + S::one());
        self.w_h(side, side)
    }

    /// The number of sides used to draw the ellipse.
    ///
    /// By default, ellipse does not use a resolution, but rather uses a stroke tolerance to
    /// determine how many vertices to use during tessellation.
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = Some(resolution);
        self
    }
}

// Trait implementations.

impl draw::renderer::RenderPrimitive for Ellipse<f32> {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
        let Ellipse {
            dimensions,
            polygon,
            resolution,
        } = self;

        // First get the dimensions of the ellipse.
        let (maybe_x, maybe_y, maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        assert!(
            maybe_z.is_none(),
            "z dimension support for ellipse is unimplemented"
        );

        let w = maybe_x.map(f32::abs).unwrap_or(100.0);
        let h = maybe_y.map(f32::abs).unwrap_or(100.0);
        match resolution {
            None => {
                // Determine the transform to apply to all points.
                let radii = lyon::math::vector(w * 0.5, h * 0.5);
                if radii.square_length() > 0.0 {
                    let centre = lyon::math::point(0.0, 0.0);
                    let mut builder = lyon::path::Path::builder();
                    let sweep_angle = lyon::math::Angle::radians(std::f32::consts::PI * 2.0);
                    let x_rotation = lyon::math::Angle::radians(0.0);
                    let start = lyon::math::point(w * 0.5, 0.0);
                    builder.move_to(start);
                    builder.arc(centre, radii, sweep_angle, x_rotation);
                    let path = builder.build();
                    polygon::render_events_themed(
                        polygon.opts,
                        || (&path).into_iter(),
                        ctxt,
                        &draw::theme::Primitive::Ellipse,
                        mesh,
                    );
                }
            }
            Some(resolution) => {
                let rect = geom::Rect::from_wh(Vector2 { x: w, y: h });
                let ellipse = geom::Ellipse::new(rect, resolution);
                let points = ellipse.circumference();
                polygon::render_points_themed(
                    polygon.opts,
                    points,
                    ctxt,
                    &draw::theme::Primitive::Ellipse,
                    mesh,
                );
            }
        }

        draw::renderer::PrimitiveRender::default()
    }
}

impl draw::svg_renderer::SvgRenderPrimitive<SVGEllipse> for Ellipse<f32> {
    fn render_svg_element(self, ctx: SvgRenderContext) -> SVGEllipse {
        let Ellipse {
            dimensions,
            resolution: _,
            polygon,
        } = self;

        // TODO: let color = fill
        //             .0
        //             .unwrap_or_else(|| ctx.theme.fill_lin_srgba(&theme_prim));
        let color = polygon.opts.color.unwrap_or(BLACK.into_lin_srgba());
        let col_string = color_string(color);
        let global_transform = ctx.transform;
        let local_transform =
            polygon.opts.position.transform() * polygon.opts.orientation.transform();
        let transform = global_transform * local_transform;

        // TODO: other rotations using skew?
        let orientation = match polygon.opts.orientation {
            orientation::Properties::Axes(v) => cgmath::Euler {
                x: cgmath::Rad(v.x),
                y: cgmath::Rad(v.y),
                z: cgmath::Rad(v.z),
            },
            orientation::Properties::LookAt(p) => {
                // TODO
                cgmath::Euler {
                    x: cgmath::Rad(0.0),
                    y: cgmath::Rad(0.0),
                    z: cgmath::Rad(0.0),
                }
            }
        };
        println!("{:?}", orientation);
        let pos = cgmath::Transform::transform_point(&transform, Point3::new(0.0, 0.0, 0.0));
        let mut el = SVGEllipse::new()
            .set("fill", col_string)
            .set("cx", pos.x)
            .set("cy", -pos.y)
            // TODO: better way to set radii
            .set("rx", dimensions.x.unwrap_or(100.0) / 2.0)
            .set("ry", dimensions.y.unwrap_or(100.0) / 2.0)
            // TODO: figure out rotation
            .set(
                "transform",
                format!("rotate({})", -rad_to_deg(orientation.z.0)),
            );
        if let Some(stroke) = polygon.opts.stroke {
            el = el.set("stroke-width", stroke.line_width);
        }
        if let Some(stroke_color) = polygon.opts.stroke_color {
            el = el.set("stroke", color_string(stroke_color));
        }

        el
    }
}

impl<S> Default for Ellipse<S>
where
    S: Zero,
{
    fn default() -> Self {
        let dimensions = Default::default();
        let polygon = Default::default();
        let resolution = Default::default();
        Ellipse {
            dimensions,
            polygon,
            resolution,
        }
    }
}

impl<S> SetOrientation<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl<S> SetPosition<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.polygon)
    }
}

impl<S> SetDimensions<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl<S> SetColor<ColorScalar> for Ellipse<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.polygon)
    }
}

impl<S> SetStroke for Ellipse<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl<S> SetPolygon<S> for Ellipse<S> {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions<S> {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversion.

impl<S> From<Ellipse<S>> for Primitive<S> {
    fn from(prim: Ellipse<S>) -> Self {
        Primitive::Ellipse(prim)
    }
}

impl<S> Into<Option<Ellipse<S>>> for Primitive<S> {
    fn into(self) -> Option<Ellipse<S>> {
        match self {
            Primitive::Ellipse(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'a, S> DrawingEllipse<'a, S>
where
    S: BaseFloat,
{
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: S) -> Self {
        self.map_ty(|ty| ty.radius(radius))
    }

    /// The number of sides used to draw the ellipse.
    pub fn resolution(self, resolution: usize) -> Self {
        self.map_ty(|ty| ty.resolution(resolution))
    }
}
