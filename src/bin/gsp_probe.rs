//! Headless probes for comparing original Sketchpad files with port output.

use sketchpad_rs::document::{Clipboard, Document, Kind, Point, Record, records};
use sketchpad_rs::render::{Viewport, svg_page};
use std::{env, fs, path::Path};

fn usage() -> String {
    "Usage: gsp_probe inspect FILE | diff BEFORE AFTER | verify FILE... | demo OUTPUT.gsp [1..9] | replay SCRIPT | svg INPUT.gsp OUTPUT.svg"
        .into()
}

fn read(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("{path}: {error}"))
}

fn inspect(path: &str) -> Result<(), String> {
    let bytes = read(path)?;
    let entries = records(&bytes)?;
    println!("{path}: {} bytes, {} records", bytes.len(), entries.len());
    for (index, entry) in entries.iter().enumerate() {
        println!(
            "{index:4} {:#08x} kind={:#06x} payload={}",
            entry.start,
            entry.kind,
            entry.data.len()
        );
    }
    let document = Document::from_bytes(&bytes, None)?;
    println!(
        "{} page(s), {} object(s), partial_view={}",
        document.page_count,
        document.objects.len(),
        document.partial_view
    );
    for object in &document.objects {
        println!(
            "object {:4} page {:2} visible={} {:?}",
            object.id,
            object.page + 1,
            object.visible,
            object.kind
        );
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    let mut out = bytes
        .iter()
        .take(24)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    if bytes.len() > 24 {
        out.push_str(" ...");
    }
    out
}

fn compare_payload(before: &[u8], after: &[u8], offset: usize) {
    if before.len() != after.len() {
        println!("    length {} -> {}", before.len(), after.len());
        println!("    before: {}", hex(before));
        println!("    after:  {}", hex(after));
        return;
    }
    let mut i = 0;
    let mut ranges = 0;
    while i < before.len() {
        if before[i] == after[i] {
            i += 1;
            continue;
        }
        let start = i;
        while i < before.len() && before[i] != after[i] {
            i += 1;
        }
        println!(
            "    payload +{start:#x} (file {:#x}), {} byte(s): {} -> {}",
            offset + start,
            i - start,
            hex(&before[start..i]),
            hex(&after[start..i])
        );
        ranges += 1;
        if ranges == 32 {
            println!("    ... further byte ranges omitted");
            break;
        }
    }
}

fn record_payload<'a>(bytes: &'a [u8], record: &Record) -> &'a [u8] {
    &bytes[record.data.clone()]
}

fn diff(before_path: &str, after_path: &str) -> Result<(), String> {
    let before = read(before_path)?;
    let after = read(after_path)?;
    let left = records(&before)?;
    let right = records(&after)?;
    println!(
        "{before_path} ({}, {} records) -> {after_path} ({}, {} records)",
        before.len(),
        left.len(),
        after.len(),
        right.len()
    );
    let mut changed = 0;
    for index in 0..left.len().max(right.len()) {
        match (left.get(index), right.get(index)) {
            (Some(a), Some(b)) => {
                let old = record_payload(&before, a);
                let new = record_payload(&after, b);
                if a.kind == b.kind && old == new {
                    continue;
                }
                println!(
                    "record {index}: {:#06x} at {:#x} -> {:#06x} at {:#x}",
                    a.kind, a.start, b.kind, b.start
                );
                if a.kind == b.kind {
                    compare_payload(old, new, a.data.start);
                } else {
                    println!("    before: {}", hex(old));
                    println!("    after:  {}", hex(new));
                }
                changed += 1;
            }
            (Some(a), None) => {
                println!("record {index}: removed {:#06x} at {:#x}", a.kind, a.start);
                changed += 1;
            }
            (None, Some(b)) => {
                println!("record {index}: added {:#06x} at {:#x}", b.kind, b.start);
                changed += 1;
            }
            (None, None) => unreachable!(),
        }
    }
    println!("{changed} changed record slot(s)");
    Ok(())
}

fn verify(paths: &[String]) -> Result<(), String> {
    if paths.is_empty() {
        return Err(usage());
    }
    let mut failures = 0;
    for path in paths {
        let result = (|| -> Result<(usize, bool), String> {
            let bytes = read(path)?;
            let document = Document::from_bytes(&bytes, None)?;
            if document.encode()? != bytes {
                return Err("round trip changed source bytes".into());
            }
            Ok((document.objects.len(), document.partial_view))
        })();
        match result {
            Ok((count, partial)) => println!("PASS {path}: {count} objects, partial={partial}"),
            Err(error) => {
                eprintln!("FAIL {path}: {error}");
                failures += 1;
            }
        }
    }
    if failures == 0 {
        Ok(())
    } else {
        Err(format!("{failures} document(s) failed"))
    }
}

fn demo(path: &str, steps: usize) -> Result<(), String> {
    if !(1..=9).contains(&steps) {
        return Err("demo steps must be from 1 to 9".into());
    }
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 220., y: 230. }));
    if steps >= 2 {
        let b = document.add(Kind::Free(Point { x: 480., y: 360. }));
        if steps >= 3 {
            document.add(Kind::Segment(a, b));
        }
        if steps >= 4 {
            document.add(Kind::Circle(a, b));
        }
        if steps >= 5 {
            document.add(Kind::Midpoint(a, b));
        }
        if steps >= 6 {
            document.add(Kind::Line(a, b));
        }
        if steps >= 7 {
            document.add(Kind::Ray(a, b));
        }
        if steps >= 8 {
            document.add(Kind::Parallel(a, 3));
        }
        if steps >= 9 {
            document.add(Kind::Perpendicular(b, 3));
        }
    }
    document.save_to(Path::new(path))?;
    println!("{path}: wrote {} objects", document.objects.len());
    Ok(())
}

fn two_ids(rest: &str) -> Result<(usize, usize), String> {
    let parts: Vec<_> = rest.split_whitespace().collect();
    if parts.len() != 2 {
        return Err("expected two object IDs".into());
    }
    Ok((
        parts[0].parse().map_err(|_| "invalid first object ID")?,
        parts[1].parse().map_err(|_| "invalid second object ID")?,
    ))
}

fn replay(path: &str) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    let mut document = Document::new();
    let mut clipboard: Option<Clipboard> = None;
    for (index, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if index >= 10_000 {
            return Err("replay is limited to 10,000 lines".into());
        }
        let (command, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let rest = rest.trim();
        let result = (|| -> Result<(), String> {
            match command {
                "new" if rest.is_empty() => document = Document::new(),
                "open" if !rest.is_empty() => document = Document::open(Path::new(rest))?,
                "save" if !rest.is_empty() => document.save_to(Path::new(rest))?,
                "length" => {
                    let segment: usize = rest.parse().map_err(|_| "invalid segment ID")?;
                    let id = document.add(Kind::SegmentLength(segment));
                    println!("line {}: length -> object {id}", index + 1);
                }
                "distance" => {
                    let (a, b) = two_ids(rest)?;
                    let id = document.add(Kind::PointDistance(a, b));
                    println!("line {}: distance -> object {id}", index + 1);
                }
                "coordinate_system" => {
                    let values = rest.split_whitespace().collect::<Vec<_>>();
                    let origin = match values.as_slice() {
                        [] => Point { x: 199., y: 125. },
                        [x, y] => Point {
                            x: x.parse().map_err(|_| "invalid origin x")?,
                            y: y.parse().map_err(|_| "invalid origin y")?,
                        },
                        _ => return Err("expected optional origin x y".into()),
                    };
                    let id = document.add_default_coordinate_system(origin)?;
                    println!("line {}: coordinate_system -> object {id}", index + 1);
                }
                "show_grid" => {
                    let visible = match rest {
                        "on" => true,
                        "off" => false,
                        _ => return Err("expected show_grid on or off".into()),
                    };
                    document.set_grid_visible(visible);
                }
                "slope" => {
                    let line: usize = rest.parse().map_err(|_| "invalid line ID")?;
                    let frame = match document.active_coordinate_system() {
                        Some(frame) => frame,
                        None => {
                            document.add_default_coordinate_system(Point { x: 199., y: 125. })?
                        }
                    };
                    let id = document.add(Kind::SlopeMeasurement(line, frame));
                    println!("line {}: slope -> object {id}", index + 1);
                }
                "ratio" => {
                    let ids = rest
                        .split_whitespace()
                        .map(|part| part.parse::<usize>().map_err(|_| "invalid point ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let [a, b, c] = ids.as_slice() else {
                        return Err("expected origin, base, and target point IDs".into());
                    };
                    if document.ratio_for_points(*a, *b, *c).is_none() {
                        return Err("ratio requires a nonzero base between two points".into());
                    }
                    let id = document.add(Kind::PointRatio(*a, *b, *c));
                    println!("line {}: ratio -> object {id}", index + 1);
                }
                "circumference" => {
                    let circle: usize = rest.parse().map_err(|_| "invalid circle ID")?;
                    let id = document.add(Kind::CircleCircumference(circle));
                    println!("line {}: circumference -> object {id}", index + 1);
                }
                "radius" => {
                    let circle: usize = rest.parse().map_err(|_| "invalid circle ID")?;
                    let id = document.add(Kind::CircleRadius(circle));
                    println!("line {}: radius -> object {id}", index + 1);
                }
                "area" => {
                    let circle: usize = rest.parse().map_err(|_| "invalid circle ID")?;
                    let id = document.add(Kind::CircleArea(circle));
                    println!("line {}: area -> object {id}", index + 1);
                }
                "polygon" => {
                    let vertices = rest
                        .split_whitespace()
                        .map(|part| {
                            part.parse::<usize>()
                                .map_err(|_| "invalid polygon vertex ID")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if vertices.len() < 3 {
                        return Err("polygon requires at least three point IDs".into());
                    }
                    let id = document.add(Kind::Polygon(vertices));
                    println!("line {}: polygon -> object {id}", index + 1);
                }
                "perimeter" => {
                    let polygon: usize = rest.parse().map_err(|_| "invalid polygon ID")?;
                    let id = document.add(Kind::PolygonPerimeter(polygon));
                    println!("line {}: perimeter -> object {id}", index + 1);
                }
                "hide_show_button" => {
                    let target: usize = rest.parse().map_err(|_| "invalid button target ID")?;
                    let id = document.add_hide_show_button(target)?;
                    println!("line {}: hide_show_button -> object {id}", index + 1);
                }
                "activate_button" => {
                    let id: usize = rest.parse().map_err(|_| "invalid button ID")?;
                    document.activate_hide_show_button(id)?;
                    println!("line {}: activate_button {id}", index + 1);
                }
                "arc_angle" => {
                    let arc: usize = rest.parse().map_err(|_| "invalid arc ID")?;
                    let id = document.add(Kind::ArcAngle(arc));
                    println!("line {}: arc_angle -> object {id}", index + 1);
                }
                "arc_length" => {
                    let arc: usize = rest.parse().map_err(|_| "invalid arc ID")?;
                    let id = document.add(Kind::ArcLength(arc));
                    println!("line {}: arc_length -> object {id}", index + 1);
                }
                "angle" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected three point IDs".into());
                    }
                    let a = parts[0].parse().map_err(|_| "invalid first point ID")?;
                    let b = parts[1].parse().map_err(|_| "invalid vertex point ID")?;
                    let c = parts[2].parse().map_err(|_| "invalid third point ID")?;
                    let id = document.add(Kind::AngleMeasurement(a, b, c));
                    println!("line {}: angle -> object {id}", index + 1);
                }
                "point" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 2 {
                        return Err("expected x y".into());
                    }
                    let x = parts[0].parse().map_err(|_| "invalid x coordinate")?;
                    let y = parts[1].parse().map_err(|_| "invalid y coordinate")?;
                    let id = document.add(Kind::Free(Point { x, y }));
                    println!("line {}: point -> object {id}", index + 1);
                }
                "point_on" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected host ID and x y coordinates".into());
                    }
                    let host = parts[0].parse().map_err(|_| "invalid host ID")?;
                    let x = parts[1].parse().map_err(|_| "invalid x coordinate")?;
                    let y = parts[2].parse().map_err(|_| "invalid y coordinate")?;
                    let kind = document
                        .constrained_point(host, Point { x, y })
                        .ok_or("host must be a nondegenerate segment or circle")?;
                    let id = document.add(kind);
                    println!("line {}: point_on -> object {id}", index + 1);
                }
                "translate" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err(
                            "expected source point ID, angle in degrees, distance in centimeters"
                                .into(),
                        );
                    }
                    let source = parts[0].parse().map_err(|_| "invalid source point ID")?;
                    let angle = parts[1].parse().map_err(|_| "invalid angle")?;
                    let distance = parts[2].parse().map_err(|_| "invalid distance")?;
                    let id = document.add(Kind::PolarTranslatedPoint(
                        source,
                        angle,
                        37.79527559055118,
                        distance,
                    ));
                    println!("line {}: translate -> object {id}", index + 1);
                }
                "dilate" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected source ID, center point ID, and ratio".into());
                    }
                    let source = parts[0].parse().map_err(|_| "invalid source ID")?;
                    let center = parts[1].parse().map_err(|_| "invalid center ID")?;
                    let factor = parts[2].parse().map_err(|_| "invalid ratio")?;
                    let kind = if document.circle_points(source).is_some() {
                        Kind::ScaledCircle(source, center, factor)
                    } else {
                        Kind::ScaledPoint(source, center, factor)
                    };
                    let id = document.add(kind);
                    println!("line {}: dilate -> object {id}", index + 1);
                }
                "rotate" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected source point ID, center point ID, and angle".into());
                    }
                    let source = parts[0].parse().map_err(|_| "invalid source ID")?;
                    let center = parts[1].parse().map_err(|_| "invalid center ID")?;
                    let angle = parts[2].parse().map_err(|_| "invalid angle")?;
                    let id = document.add(Kind::RotatedPoint(source, center, angle));
                    println!("line {}: rotate -> object {id}", index + 1);
                }
                "segment" | "circle" | "circle_segment" | "midpoint" | "line" | "ray"
                | "reflect" | "parallel" | "intersection" | "perpendicular" => {
                    let (a, b) = two_ids(rest)?;
                    let kind = match command {
                        "segment" => Kind::Segment(a, b),
                        "circle" => Kind::Circle(a, b),
                        "circle_segment" => Kind::CircleBySegment(a, b),
                        "midpoint" => Kind::Midpoint(a, b),
                        "line" => Kind::Line(a, b),
                        "ray" => Kind::Ray(a, b),
                        "parallel" => Kind::Parallel(a, b),
                        "intersection" => Kind::Intersection(a, b),
                        "perpendicular" => Kind::Perpendicular(a, b),
                        "reflect" => {
                            if document.circle_points(a).is_some() {
                                Kind::ReflectedCircle(a, b)
                            } else {
                                Kind::ReflectedPoint(a, b)
                            }
                        }
                        _ => unreachable!(),
                    };
                    let id = document.add(kind);
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "bisector" | "arc3" | "arc_on_circle" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected start, vertex, and end point IDs".into());
                    }
                    let start = parts[0].parse().map_err(|_| "invalid start point ID")?;
                    let vertex = parts[1].parse().map_err(|_| "invalid vertex point ID")?;
                    let end = parts[2].parse().map_err(|_| "invalid end point ID")?;
                    let kind = match command {
                        "bisector" => Kind::Bisector(start, vertex, end),
                        "arc3" => Kind::ThreePointArc(start, vertex, end),
                        _ => Kind::ArcOnCircle(start, vertex, end),
                    };
                    let id = document.add(kind);
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "line_circle_intersection" | "circle_circle_intersection" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected two object IDs and root 0 or 1".into());
                    }
                    let a = parts[0].parse().map_err(|_| "invalid first object ID")?;
                    let b = parts[1].parse().map_err(|_| "invalid second object ID")?;
                    let variant: usize =
                        parts[2].parse().map_err(|_| "invalid intersection root")?;
                    if variant > 1 {
                        return Err("intersection root must be 0 or 1".into());
                    }
                    let kind = if command == "line_circle_intersection" {
                        Kind::LineCircleIntersection(a, b, variant)
                    } else {
                        Kind::CircleCircleIntersection(a, b, variant)
                    };
                    let id = document.add(kind);
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "move" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected object ID, x, y".into());
                    }
                    let id: usize = parts[0].parse().map_err(|_| "invalid object ID")?;
                    let x = parts[1].parse().map_err(|_| "invalid x coordinate")?;
                    let y = parts[2].parse().map_err(|_| "invalid y coordinate")?;
                    if !matches!(
                        document.objects.get(id.saturating_sub(1)).map(|o| &o.kind),
                        Some(Kind::Free(_))
                    ) {
                        return Err("move requires a free point".into());
                    }
                    document.checkpoint();
                    if !document.move_point(id, Point { x, y }) {
                        return Err("invalid point coordinates".into());
                    }
                }
                "hide" => {
                    let id: usize = rest.parse().map_err(|_| "invalid object ID")?;
                    if document.objects.get(id.saturating_sub(1)).is_none() {
                        return Err("unknown object ID".into());
                    }
                    document.selected.clear();
                    document.selected.insert(id);
                    document.set_selected_visibility(false);
                }
                "clear" => {
                    let ids: Result<Vec<usize>, _> = rest
                        .split_whitespace()
                        .map(|part| part.parse().map_err(|_| "invalid object ID"))
                        .collect();
                    let ids = ids?;
                    document.selected = ids.into_iter().collect();
                    let removed = document.clear_selected()?;
                    println!("line {}: clear -> {} object(s)", index + 1, removed.len());
                }
                "copy" | "cut" => {
                    let ids: Result<Vec<usize>, _> = rest
                        .split_whitespace()
                        .map(|part| part.parse().map_err(|_| "invalid object ID"))
                        .collect();
                    document.selected = ids?.into_iter().collect();
                    let copied = document.copy_selected()?;
                    println!(
                        "line {}: {command} -> {} object(s)",
                        index + 1,
                        copied.len()
                    );
                    if command == "cut" {
                        document.clear_selected()?;
                    }
                    clipboard = Some(copied);
                }
                "paste" if rest.is_empty() => {
                    let copied = clipboard.as_ref().ok_or("Clipboard is empty")?;
                    let ids = document.paste(copied)?;
                    println!("line {}: paste -> objects {ids:?}", index + 1);
                }
                "show_all" if rest.is_empty() => document.show_all_hidden(),
                "undo" if rest.is_empty() => document.undo(),
                "redo" if rest.is_empty() => document.redo(),
                "check" if rest.is_empty() => {
                    let bytes = document.encode()?;
                    let reopened = Document::from_bytes(&bytes, None)?;
                    if reopened.objects.len() != document.objects.len()
                        || reopened
                            .objects
                            .iter()
                            .zip(&document.objects)
                            .any(|(saved, current)| {
                                saved.kind != current.kind
                                    || saved.page != current.page
                                    || saved.visible != current.visible
                            })
                    {
                        return Err("reopened objects differ".into());
                    }
                }
                _ => return Err(format!("unknown or malformed command: {command}")),
            }
            Ok(())
        })();
        result.map_err(|error| format!("{path}:{}: {error}", index + 1))?;
        if env::var_os("SKETCHPAD_TRACE").is_some() {
            println!("line {}: {command}: {:?}", index + 1, document.objects);
        }
    }
    document.can_save()?;
    println!("replay complete: {} objects", document.objects.len());
    Ok(())
}

fn svg(input: &str, output: &str) -> Result<(), String> {
    let document = Document::open(Path::new(input))?;
    let mut bounds = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for object in document
        .objects
        .iter()
        .filter(|object| object.visible && object.page == document.active_page)
    {
        if let Some(point) = document.position(object.id) {
            bounds.0 = bounds.0.min(point.x);
            bounds.1 = bounds.1.min(point.y);
            bounds.2 = bounds.2.max(point.x);
            bounds.3 = bounds.3.max(point.y);
        }
        if let Some((center, radius)) = document.circle_points(object.id) {
            bounds.0 = bounds.0.min(center.x - radius);
            bounds.1 = bounds.1.min(center.y - radius);
            bounds.2 = bounds.2.max(center.x + radius);
            bounds.3 = bounds.3.max(center.y + radius);
        }
        if let Some(points) = document.arc_points(object.id, 128) {
            for point in points {
                bounds.0 = bounds.0.min(point.x);
                bounds.1 = bounds.1.min(point.y);
                bounds.2 = bounds.2.max(point.x);
                bounds.3 = bounds.3.max(point.y);
            }
        }
    }
    if !bounds.0.is_finite() {
        bounds = (0., 0., 800., 600.);
    }
    let center = Point {
        x: (bounds.0 + bounds.2) / 2.,
        y: (bounds.1 + bounds.3) / 2.,
    };
    let scale = (900. / (bounds.2 - bounds.0).max(1.))
        .min(660. / (bounds.3 - bounds.1).max(1.))
        .min(8.) as f32;
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024., 768.)),
        scale,
        center,
    };
    let content = svg_page(&document, &viewport)?;
    fs::write(output, content).map_err(|error| error.to_string())?;
    println!("exported {output}");
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [command, path] if command == "inspect" => inspect(path),
        [command, before, after] if command == "diff" => diff(before, after),
        [command, paths @ ..] if command == "verify" => verify(paths),
        [command, path] if command == "demo" => demo(path, 9),
        [command, path, steps] if command == "demo" => demo(
            path,
            steps.parse().map_err(|_| "demo steps must be an integer")?,
        ),
        [command, path] if command == "replay" => replay(path),
        [command, input, output] if command == "svg" => svg(input, output),
        _ => Err(usage()),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
