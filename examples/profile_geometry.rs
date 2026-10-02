//! CPU geometry benchmark using the renderer's sampling resolutions.
//! cargo run --release --example profile_geometry -- file.gsp [frames]
use sketchpad_engine::document::{Document, EvaluationCache, Kind, Point};
use std::{hint::black_box, path::Path, time::Instant};

fn geometry(doc: &Document, detail: bool) {
    for object in doc.objects.iter().filter(|o| doc.is_displayed(o.id)) {
        let id = object.id;
        let start = Instant::now();
        black_box(doc.position(id));
        black_box(doc.line_points(id));
        black_box(doc.circle_points(id));
        black_box(doc.arc_points(id, 128));
        if let Some(polygons) = doc.colorized_polygon_iteration_images(id) {
            black_box(polygons);
        } else {
            black_box(doc.polygon_iteration_images(id));
        }
        black_box(doc.colorized_disk_iteration_images(id));
        black_box(doc.branch_segment_iteration_images(id));
        black_box(doc.circle_iteration_images(id));
        black_box(doc.segment_iteration_images(id));
        black_box(doc.line_iteration_images(id));
        black_box(doc.arc_iteration_images(id, 96));
        black_box(doc.ray_iteration_images(id));
        black_box(doc.point_iteration_images(id));
        let strokes = doc
            .locus_styled_strokes(id, 384)
            .or_else(|| doc.mapped_collection_strokes(id, 384))
            .or_else(|| doc.sampled_mapped_styled_strokes(id, 384));
        if strokes.is_none() {
            black_box(
                doc.mapped_collection_surfaces(id, 384)
                    .or_else(|| doc.locus_surfaces(id, 384)),
            );
        }
        black_box(strokes);
        black_box(doc.plot_sample_points(id, 384));
        black_box(doc.point_numeric_label(id));
        let elapsed = start.elapsed().as_secs_f64() * 1000.;
        if detail && elapsed > 0.5 {
            let kind = format!("{:?}", object.kind);
            let name = kind.split(['(', '{', ' ']).next().unwrap();
            println!("object\t{id}\t{name}\t{elapsed:.3}");
        }
    }
    black_box(doc.visible_mapped_numeric_label_images());
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let path = args.get(1).ok_or("expected file.gsp [frames]")?;
    let frames: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
    let start = Instant::now();
    let mut doc = Document::open(Path::new(path))?;
    println!(
        "open\t{:.3}\tobjects={}\tvisible={}",
        start.elapsed().as_secs_f64() * 1000.,
        doc.objects.len(),
        doc.objects
            .iter()
            .filter(|o| o.visible && !o.deleted && o.page == doc.active_page)
            .count()
    );
    if std::env::var_os("SKETCHPAD_PROFILE_KINDS").is_some() {
        for object in &doc.objects {
            println!("kind\t{}\t{:?}", object.id, object.kind);
        }
    }
    let mut cache = EvaluationCache::default();
    let start = Instant::now();
    let evaluated = cache.evaluate(&doc);
    println!("evaluate\t{:.3}", start.elapsed().as_secs_f64() * 1000.);
    for frame in 0..frames {
        let start = Instant::now();
        geometry(&evaluated, frame == 0);
        println!(
            "geometry\t{frame}\t{:.3}",
            start.elapsed().as_secs_f64() * 1000.
        );
    }
    let driver = doc.objects.iter().find_map(|object| match object.kind {
        Kind::Free(point) => Some((object.id, point)),
        _ => None,
    });
    if let Some((id, point)) = driver {
        for frame in 0..frames {
            let start = Instant::now();
            doc.move_point(
                id,
                Point {
                    x: point.x + frame as f64 + 1.,
                    y: point.y + 2.,
                },
            );
            geometry(&cache.evaluate(&doc), false);
            println!(
                "update\t{id}\t{frame}\t{:.3}",
                start.elapsed().as_secs_f64() * 1000.
            );
        }
    }
    Ok(())
}
