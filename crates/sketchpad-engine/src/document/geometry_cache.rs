//! Stable query identities for derived display geometry.
use super::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum GeometryQuery {
    CurveSamples,
    Points,
    Polygons,
    ColorizedPolygons,
    ColorizedPolygonBatch,
    Disks,
    ColorizedDisks,
    BranchSegments,
    Lines,
    Segments,
    Rays,
    Arcs,
    Circles,
    MappedStrokes,
    MappedSurfaces,
    LocusSurfaces,
    SampledStrokes,
    LocusStrokes,
}

impl GeometryQuery {
    /// Reject only kinds that the corresponding evaluator cannot consume.
    /// Binding source types and construction validity stay in the evaluator.
    pub(super) fn accepts(self, kind: &Kind) -> bool {
        match self {
            Self::ColorizedPolygonBatch => matches!(kind, Kind::MappedPointIteration { .. }),
            Self::CurveSamples => true,
            Self::Points => matches!(
                kind,
                Kind::IterationBinding(..)
                    | Kind::PointIteration(..)
                    | Kind::SegmentIteration(..)
                    | Kind::CircleIteration(..)
                    | Kind::PointParameterIteration(..)
                    | Kind::RayIteration(..)
            ),
            Self::Segments => matches!(kind, Kind::SegmentIteration(..)),
            Self::Rays => matches!(kind, Kind::IterationBinding(..) | Kind::RayIteration(..)),
            Self::LocusStrokes | Self::LocusSurfaces => {
                matches!(kind, Kind::Locus(..) | Kind::ParameterLocus(..))
            }
            Self::SampledStrokes => matches!(kind, Kind::SampledMapped(..)),
            Self::Polygons
            | Self::ColorizedPolygons
            | Self::Disks
            | Self::ColorizedDisks
            | Self::BranchSegments
            | Self::Lines
            | Self::Arcs
            | Self::Circles
            | Self::MappedStrokes
            | Self::MappedSurfaces => matches!(kind, Kind::IterationBinding(..)),
        }
    }
}
