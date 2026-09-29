//! Presentation geometry for native three-point angle marks.
use super::{Document, Kind, Point};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AngleMarkerStyle {
    pub arrow: bool,
    pub opacity: u16,
}

impl AngleMarkerStyle {
    pub(super) fn for_kind(kind: &Kind) -> Option<Self> {
        matches!(
            kind,
            Kind::AngleMarker(..)
                | Kind::SignedAngleValue(..)
                | Kind::PolarAngleValue(..)
                | Kind::VertexAngleValue(..)
        )
        .then_some(Self {
            arrow: !matches!(kind, Kind::AngleMarker(..)),
            opacity: 0x3fff,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AngleMarkerGeometry {
    pub strokes: Vec<Vec<Point>>,
    pub sector: Vec<Point>,
    pub arrows: Vec<[Point; 3]>,
    pub label_anchor: Point,
}

impl Document {
    pub(super) fn patch_angle_marker_styles(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        if !self.objects.iter().any(|o| o.angle_marker_style.is_some()) {
            return Ok(bytes);
        }
        let rs = super::records(&bytes)?;
        let mut objects = self.objects.iter().filter(|o| !o.deleted);
        let mut style = None;
        for record in super::embedded_tools::document_records(&rs) {
            if record.kind == 0x07d0 {
                style = objects.next().and_then(|o| o.angle_marker_style);
                if let Some(style) = style {
                    bytes[record.data.start + 16..record.data.start + 18]
                        .copy_from_slice(&style.opacity.to_le_bytes());
                }
            } else if record.kind == 0x090e {
                if let Some(style) = style {
                    bytes[record.data.start + 2..record.data.start + 4]
                        .copy_from_slice(&u16::from(style.arrow).to_le_bytes());
                }
            } else if record.kind == 0x07d7 {
                style = None;
            }
        }
        Ok(bytes)
    }

    /// Native markers use a 24-unit radius, independent of their arm lengths.
    /// Additional marks are spaced six units apart; right angles use squares.
    pub fn angle_marker_geometry(&self, id: usize) -> Option<AngleMarkerGeometry> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let (first, vertex, last, count) = match object.kind {
            Kind::AngleMarker(a, b, c, count) => (a, b, c, count),
            Kind::SignedAngleValue(a, b, c)
            | Kind::PolarAngleValue(a, b, c)
            | Kind::VertexAngleValue(a, b, c) => (a, b, c, 1),
            _ => return None,
        };
        let (first, vertex, last) = (
            self.position(first)?,
            self.position(vertex)?,
            self.position(last)?,
        );
        let (ax, ay) = (first.x - vertex.x, first.y - vertex.y);
        let (bx, by) = (last.x - vertex.x, last.y - vertex.y);
        let (al, bl) = (ax.hypot(ay), bx.hypot(by));
        if !al.is_finite() || !bl.is_finite() || al <= 1e-9 || bl <= 1e-9 {
            return None;
        }
        let (ax, ay, bx, by) = (ax / al, ay / al, bx / bl, by / bl);
        let start = ay.atan2(ax);
        let signed = (ax * by - ay * bx).atan2(ax * bx + ay * by);
        let sweep = match object.kind {
            Kind::PolarAngleValue(..) => -(-signed).rem_euclid(std::f64::consts::TAU),
            Kind::VertexAngleValue(..) => signed.rem_euclid(std::f64::consts::TAU),
            _ => {
                (signed + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                    - std::f64::consts::PI
            }
        };
        if !sweep.is_finite() {
            return None;
        }
        let square = (sweep.abs() - std::f64::consts::FRAC_PI_2).abs() < 1e-9;
        let point = |x, y| Point {
            x: vertex.x + x,
            y: vertex.y + y,
        };
        // 07d5's double locates the label along the angle bisector: native
        // parameters 0, 1, 2 place it at radii -24, 48, 120 respectively.
        let label_parameter = object.label.as_ref().map_or(0., |label| {
            f64::from_le_bytes(label.payload()[14..22].try_into().unwrap())
        });
        let label_radius = 24. * (3. * label_parameter - 1.);
        let mut geometry = AngleMarkerGeometry {
            strokes: Vec::new(),
            sector: vec![vertex],
            arrows: Vec::new(),
            label_anchor: point(
                label_radius * (start + sweep / 2.).cos(),
                label_radius * (start + sweep / 2.).sin(),
            ),
        };
        if sweep.abs() <= 1e-9 {
            return Some(geometry);
        }
        for ring in 0..count {
            let radius = 24. + f64::from(ring) * 6.;
            let path: Vec<Point> = if square {
                let side = 24. / std::f64::consts::SQRT_2 + f64::from(ring) * 6.;
                vec![
                    point(ax * side, ay * side),
                    point((ax + bx) * side, (ay + by) * side),
                    point(bx * side, by * side),
                ]
            } else {
                let samples = (sweep.abs() * 24.).ceil() as usize;
                (0..=samples)
                    .map(|sample| {
                        let angle = start + sweep * sample as f64 / samples as f64;
                        point(radius * angle.cos(), radius * angle.sin())
                    })
                    .collect()
            };
            if ring == 0 {
                geometry.sector.extend_from_slice(&path);
            }
            if object.angle_marker_style?.arrow {
                let tip = *path.last()?;
                let (tx, ty) = if square {
                    (-ax, -ay)
                } else {
                    (-by * sweep.signum(), bx * sweep.signum())
                };
                geometry.arrows.push([
                    tip,
                    Point {
                        x: tip.x - tx * 6. - ty * 3.,
                        y: tip.y - ty * 6. + tx * 3.,
                    },
                    Point {
                        x: tip.x - tx * 6. + ty * 3.,
                        y: tip.y - ty * 6. - tx * 3.,
                    },
                ]);
            }
            geometry.strokes.push(path);
        }
        Some(geometry)
    }

    pub fn angle_marker_strokes(&self, id: usize) -> Option<Vec<Vec<Point>>> {
        Some(self.angle_marker_geometry(id)?.strokes)
    }

    pub fn angle_marker_label_anchor(&self, id: usize) -> Option<Point> {
        Some(self.angle_marker_geometry(id)?.label_anchor)
    }
}
