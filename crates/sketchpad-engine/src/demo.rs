//! Small, self-contained documents for interactive demonstrations.

use crate::document::{Document, Kind, Point};

pub const GEOMETRY_STEPS: usize = 12;

fn reopen(document: &Document) -> Result<Document, String> {
    // Start with no construction-time undo history. The first drag is then the
    // first undo, just as it would be after opening a saved demonstration.
    Document::from_bytes(&document.encode()?, None)
}

/// Build the staged geometry construction used by the desktop demo and probe.
pub fn geometry_document(steps: usize) -> Result<Document, String> {
    if !(1..=GEOMETRY_STEPS).contains(&steps) {
        return Err(format!("demo steps must be from 1 to {GEOMETRY_STEPS}"));
    }

    let mut document = Document::new();
    // Keep the construction centered in the editor's default viewport so the
    // generated file is useful as a live demo, not only as a writer probe.
    let a = document.add(Kind::Free(Point { x: 300., y: 270. }));
    document.set_object_label(a, "A", true)?;
    if steps == 1 {
        return reopen(&document);
    }

    let b = document.add(Kind::Free(Point { x: 380., y: 320. }));
    document.set_object_label(b, "B", true)?;
    if steps == 2 {
        return reopen(&document);
    }

    let segment = document.add(Kind::Segment(a, b));
    if steps == 3 {
        return reopen(&document);
    }

    document.add(Kind::Circle(a, b));
    if steps == 4 {
        return reopen(&document);
    }

    let midpoint = document.add(Kind::Midpoint(a, b));
    document.set_object_label(midpoint, "M", true)?;
    if steps == 5 {
        return reopen(&document);
    }

    let c = document.add(Kind::Free(Point { x: 390., y: 190. }));
    document.set_object_label(c, "C", true)?;
    if steps == 6 {
        return reopen(&document);
    }

    document.add(Kind::Line(a, c));
    if steps == 7 {
        return reopen(&document);
    }

    document.add(Kind::Ray(b, c));
    if steps == 8 {
        return reopen(&document);
    }

    document.add(Kind::Parallel(c, segment));
    if steps == 9 {
        return reopen(&document);
    }

    document.add(Kind::Perpendicular(midpoint, segment));
    if steps == 10 {
        return reopen(&document);
    }

    let length = document.add(Kind::SegmentLength(segment));
    document.move_measurement_caption(length, (20, 20))?;
    if steps == 11 {
        return reopen(&document);
    }

    document.add_text("Drag A, B, or C", Point { x: 255., y: 100. })?;
    reopen(&document)
}
