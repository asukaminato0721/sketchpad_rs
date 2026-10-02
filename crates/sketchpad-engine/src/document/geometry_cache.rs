//! Stable query identities for derived display geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum GeometryQuery {
    CurveSamples,
    Points,
    Polygons,
    ColorizedPolygons,
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
