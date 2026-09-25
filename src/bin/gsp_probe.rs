//! Headless probes for comparing original Sketchpad files with port output.

use sketchpad_rs::coordinate_measure_records::CoordinateMeasureKind;
use sketchpad_rs::document::{Clipboard, Document, Kind, Point, Record, TextStyleCommand, records};
use sketchpad_rs::graph::GridForm;
use sketchpad_rs::plot_points_records::PlotPointsMode;
use sketchpad_rs::render::{Viewport, svg_page};
use sketchpad_rs::style_records::{LinePattern, LineWidth, PointStyle, RgbColor};
use sketchpad_rs::value_equation_records::ValueEquationKind;
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

fn number_anchor(rest: &str) -> Result<(i16, i16), String> {
    let parts = rest.split_whitespace().collect::<Vec<_>>();
    let [x, y] = parts.as_slice() else {
        return Err("expected x y".into());
    };
    Ok((
        x.parse().map_err(|_| "invalid x")?,
        y.parse().map_err(|_| "invalid y")?,
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
                "parameter" => {
                    let id = document.add_default_parameter(number_anchor(rest)?)?;
                    println!("line {}: parameter -> object {id}", index + 1);
                }
                "named_parameter" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [name, value, x, y] = parts.as_slice() else {
                        return Err("expected named_parameter NAME VALUE X Y".into());
                    };
                    let value: f64 = value.parse().map_err(|_| "invalid parameter value")?;
                    let anchor = number_anchor(&format!("{x} {y}"))?;
                    let id = document.add_named_parameter(name, value, anchor)?;
                    println!("line {}: named_parameter -> object {id}", index + 1);
                }
                "parameter2_value" => {
                    let value: u16 = rest.parse().map_err(|_| "invalid second parameter value")?;
                    document.set_second_parameter_value(2, value)?;
                }
                "parameter_value" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [id, value] = parts.as_slice() else {
                        return Err("expected parameter_value ID VALUE".into());
                    };
                    let id: usize = id.parse().map_err(|_| "invalid parameter ID")?;
                    let value: f64 = value.parse().map_err(|_| "invalid parameter value")?;
                    document.set_parameter_value(id, value)?;
                }
                "calculate_2_plus_3" => {
                    let id = document.add_literal_calculation(number_anchor(rest)?)?;
                    println!("line {}: calculate_2_plus_3 -> object {id}", index + 1);
                }
                "tabulate" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let source: usize = parts
                        .first()
                        .ok_or("expected tabulate SOURCE [X Y]")?
                        .parse()
                        .map_err(|_| "invalid parameter ID")?;
                    let anchor = match parts.as_slice() {
                        [_] => (10, 38),
                        [_, x, y] => number_anchor(&format!("{x} {y}"))?,
                        _ => return Err("expected tabulate SOURCE [X Y]".into()),
                    };
                    let id = document.tabulate_default_parameter(source, anchor)?;
                    println!("line {}: tabulate -> object {id}", index + 1);
                }
                "tabulate_pair" if rest.is_empty() => {
                    let id = document.tabulate_default_pair((10, 71))?;
                    println!("line {}: tabulate_pair -> object {id}", index + 1);
                }
                "table_add" | "table_remove" => {
                    let id: usize = rest.parse().map_err(|_| "invalid table ID")?;
                    document.change_table_rows(id, command == "table_add")?;
                }
                "number_move" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [id, x, y] = parts.as_slice() else {
                        return Err("expected ID x y".into());
                    };
                    document.move_number(
                        id.parse().map_err(|_| "invalid number ID")?,
                        (
                            x.parse().map_err(|_| "invalid x")?,
                            y.parse().map_err(|_| "invalid y")?,
                        ),
                    )?;
                }
                "open" if !rest.is_empty() => document = Document::open(Path::new(rest))?,
                "save" if !rest.is_empty() => document.save_to(Path::new(rest))?,
                "import_png" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    let [path, x, y] = parts.as_slice() else {
                        return Err("expected PNG path x y".into());
                    };
                    let png = read(path)?;
                    let at = Point {
                        x: x.parse().map_err(|_| "invalid picture x")?,
                        y: y.parse().map_err(|_| "invalid picture y")?,
                    };
                    let id = document.import_png(&png, at)?;
                    println!("line {}: import_png -> object {id}", index + 1);
                }
                "move_image" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    let [id, x, y] = parts.as_slice() else {
                        return Err("expected picture ID x y".into());
                    };
                    document.move_image(
                        id.parse().map_err(|_| "invalid picture ID")?,
                        Point {
                            x: x.parse().map_err(|_| "invalid picture x")?,
                            y: y.parse().map_err(|_| "invalid picture y")?,
                        },
                    )?;
                }
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
                "coordinate_system_at_point" => {
                    let origin: usize = rest.parse().map_err(|_| "invalid origin point ID")?;
                    let id = document.add_coordinate_system_at_point(origin)?;
                    println!(
                        "line {}: coordinate_system_at_point -> object {id}",
                        index + 1
                    );
                }
                "coordinate_system_from_circle" => {
                    let circle: usize = rest.parse().map_err(|_| "invalid circle ID")?;
                    let id = document.add_coordinate_system_from_circle(circle)?;
                    println!(
                        "line {}: coordinate_system_from_circle -> object {id}",
                        index + 1
                    );
                }
                "coordinate_system_from_distances" => {
                    let mut origin = None;
                    let mut distances = Vec::new();
                    for value in rest.split_whitespace() {
                        if let Some(id) = value.strip_prefix("origin=") {
                            if origin.is_some() {
                                return Err("duplicate origin".into());
                            }
                            origin = Some(id.parse().map_err(|_| "invalid origin ID")?);
                        } else {
                            distances.push(value.parse().map_err(|_| "invalid distance ID")?);
                        }
                    }
                    let id = document.add_coordinate_system_from_distances(
                        origin,
                        &distances,
                        Point { x: 199., y: 125. },
                    )?;
                    println!(
                        "line {}: coordinate_system_from_distances -> object {id}",
                        index + 1
                    );
                }
                "mark_coordinate_system" => {
                    let selected: usize = rest.parse().map_err(|_| "invalid graph object ID")?;
                    let frame = document.mark_coordinate_system(selected)?;
                    println!("line {}: marked coordinate system {frame}", index + 1);
                }
                "show_grid" => {
                    let visible = match rest {
                        "on" => true,
                        "off" => false,
                        _ => return Err("expected show_grid on or off".into()),
                    };
                    document.set_grid_visible(visible);
                }
                "snap_points" => {
                    let enabled = match rest {
                        "on" => true,
                        "off" => false,
                        _ => return Err("expected snap_points on or off".into()),
                    };
                    document.set_snap_points(enabled);
                }
                "dotted_grid" => {
                    let dotted = match rest {
                        "on" => true,
                        "off" => false,
                        _ => return Err("expected dotted_grid on or off".into()),
                    };
                    document.set_grid_dotted(dotted);
                }
                "grid_form" => {
                    let form = match rest {
                        "polar" => GridForm::Polar,
                        "square" => GridForm::Square,
                        "rectangular" => GridForm::Rectangular,
                        _ => return Err("expected grid_form polar|square|rectangular".into()),
                    };
                    document.set_grid_form(form)?;
                }
                "trigonometric_axis" => {
                    let enabled = match rest {
                        "on" => true,
                        "off" => false,
                        _ => return Err("expected trigonometric_axis on or off".into()),
                    };
                    document.set_axis_trigonometric(enabled)?;
                }
                "trace" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [id, value] = parts.as_slice() else {
                        return Err("expected trace ID on|off".into());
                    };
                    let id: usize = id.parse().map_err(|_| "invalid trace object ID")?;
                    let tracing = match *value {
                        "on" => true,
                        "off" => false,
                        _ => return Err("expected trace ID on|off".into()),
                    };
                    document.selected.clear();
                    document.selected.insert(id);
                    if document.set_selected_tracing(tracing)? != 1 {
                        return Err("trace target did not change".into());
                    }
                }
                "plot_points" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let mode = match parts.first().copied().unwrap_or("") {
                        "cartesian" => PlotPointsMode::Cartesian,
                        "polar" => PlotPointsMode::Polar,
                        _ => {
                            return Err("expected plot_points cartesian|polar [X Y]".into());
                        }
                    };
                    let coordinates = match parts.as_slice() {
                        [_] => [1.0, 1.0],
                        [_, first, second] => [
                            first.parse().map_err(|_| "invalid first coordinate")?,
                            second.parse().map_err(|_| "invalid second coordinate")?,
                        ],
                        _ => {
                            return Err("expected plot_points cartesian|polar [X Y]".into());
                        }
                    };
                    let id = document.plot_points_at(mode, coordinates)?;
                    println!("line {}: plot_points -> object {id}", index + 1);
                }
                "plot_table_data" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let (table, mode) = match parts.as_slice() {
                        [table] => (
                            table.parse().map_err(|_| "invalid table ID")?,
                            PlotPointsMode::Cartesian,
                        ),
                        [table, "polar"] => (
                            table.parse().map_err(|_| "invalid table ID")?,
                            PlotPointsMode::Polar,
                        ),
                        _ => return Err("expected plot_table_data TABLE [polar]".into()),
                    };
                    let ids = document.plot_pair_table_data(table, mode)?;
                    println!("line {}: plot_table_data -> objects {ids:?}", index + 1);
                }
                "plot_axis" | "plot_path" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let source: usize = parts
                        .first()
                        .ok_or("expected plot_axis SOURCE [AXIS]")?
                        .parse()
                        .map_err(|_| "invalid plot source ID")?;
                    let axis = parts
                        .get(1)
                        .map(|part| part.parse::<usize>().map_err(|_| "invalid axis ID"))
                        .transpose()?;
                    if parts.len() > 2 {
                        return Err("expected plot_axis SOURCE [AXIS]".into());
                    }
                    let id = document.plot_parameter_on_path(source, axis)?;
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "plot_axis_value" | "plot_path_value" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.is_empty() || parts.len() > 2 {
                        return Err("expected plot_axis_value VALUE [AXIS]".into());
                    }
                    let value: f64 = parts[0].parse().map_err(|_| "invalid numeric value")?;
                    let axis = parts
                        .get(1)
                        .map(|part| part.parse::<usize>().map_err(|_| "invalid axis ID"))
                        .transpose()?;
                    if axis.is_none() && document.active_coordinate_system().is_none() {
                        document.add_default_coordinate_system(Point { x: 199., y: 125. })?;
                    }
                    let id = document.plot_literal_on_path(value, axis)?;
                    println!("line {}: {command} -> object {id}", index + 1);
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
                "point_value" => {
                    let first: usize = rest.parse().map_err(|_| "invalid constrained point ID")?;
                    let second = match document
                        .objects
                        .get(first.saturating_sub(1))
                        .map(|object| &object.kind)
                    {
                        Some(Kind::PointOnSegment(host, _) | Kind::PointOnCircle(host, _, _)) => {
                            *host
                        }
                        _ => {
                            return Err(
                                "point_value requires a point on a segment or circle".into()
                            );
                        }
                    };
                    let id = document.add(Kind::ValueEquation {
                        kind: ValueEquationKind::PointValue,
                        first,
                        second,
                    });
                    println!("line {}: point_value -> object {id}", index + 1);
                }
                "equation" => {
                    let first: usize = rest.parse().map_err(|_| "invalid line or circle ID")?;
                    let kind = match document
                        .objects
                        .get(first.saturating_sub(1))
                        .map(|object| &object.kind)
                    {
                        Some(Kind::Line(_, _)) => ValueEquationKind::LineEquation,
                        Some(Kind::Circle(_, _)) => ValueEquationKind::CircleEquation,
                        _ => return Err("equation requires a line or circle".into()),
                    };
                    let second = match document.active_coordinate_system() {
                        Some(frame) => frame,
                        None => {
                            document.add_default_coordinate_system(Point { x: 199., y: 125. })?
                        }
                    };
                    let id = document.add(Kind::ValueEquation {
                        kind,
                        first,
                        second,
                    });
                    if document.value_equation_value(id).is_none() {
                        return Err("equation is undefined for this geometry".into());
                    }
                    println!("line {}: equation -> object {id}", index + 1);
                }
                "coordinates" | "abscissa" | "ordinate" | "coordinate_distance" => {
                    let kind = match command {
                        "coordinates" => CoordinateMeasureKind::Coordinates,
                        "abscissa" => CoordinateMeasureKind::Abscissa,
                        "ordinate" => CoordinateMeasureKind::Ordinate,
                        _ => CoordinateMeasureKind::CoordinateDistance,
                    };
                    let ids = rest
                        .split_whitespace()
                        .map(|s| s.parse::<usize>().map_err(|_| "invalid point ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let (first, second) = match ids.as_slice() {
                        [first] if kind != CoordinateMeasureKind::CoordinateDistance => {
                            (*first, None)
                        }
                        [first, second] if kind == CoordinateMeasureKind::CoordinateDistance => {
                            (*first, Some(*second))
                        }
                        _ => {
                            return Err(
                                "expected one point ID or two for coordinate_distance".into()
                            );
                        }
                    };
                    if document.position(first).is_none()
                        || second.is_some_and(|id| document.position(id).is_none())
                    {
                        return Err("coordinate measurement requires point IDs".into());
                    }
                    let frame = match document.active_coordinate_system() {
                        Some(frame) => frame,
                        None => {
                            document.add_default_coordinate_system(Point { x: 199., y: 125. })?
                        }
                    };
                    let id = document.add(Kind::CoordinateMeasurement {
                        kind,
                        first,
                        second,
                        frame,
                    });
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "function" => {
                    let expression = sketchpad_rs::function_records::SimpleFunction::parse(rest)?;
                    let id = document.add_function_definition(expression)?;
                    println!("line {}: function -> object {id}", index + 1);
                }
                "plot_function" => {
                    let expression = sketchpad_rs::function_records::SimpleFunction::parse(rest)?;
                    let id = document.plot_new_function(expression, Point { x: 199., y: 125. })?;
                    println!("line {}: plot_function -> object {id}", index + 1);
                }
                "plot_existing_function" => {
                    let definition = rest.parse::<usize>().map_err(|_| "invalid function ID")?;
                    let id = document.plot_function(definition, Point { x: 199., y: 125. })?;
                    println!("line {}: plot_existing_function -> object {id}", index + 1);
                }
                "differentiate" | "differentiate_radians" | "differentiate_degrees" => {
                    let source = rest.parse::<usize>().map_err(|_| "invalid function ID")?;
                    let id = match command {
                        "differentiate_radians" => {
                            document.add_derivative_function_with_angle_unit(source, true)?
                        }
                        "differentiate_degrees" => {
                            document.add_derivative_function_with_angle_unit(source, false)?
                        }
                        _ => document.add_derivative_function(source)?,
                    };
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "edit_function" | "edit_function_radians" | "edit_function_degrees" => {
                    let (definition, formula) = rest
                        .split_once(char::is_whitespace)
                        .ok_or("expected edit_function ID EXPRESSION")?;
                    let definition = definition
                        .parse::<usize>()
                        .map_err(|_| "invalid function ID")?;
                    let expression =
                        sketchpad_rs::function_records::SimpleFunction::parse(formula)?;
                    match command {
                        "edit_function_radians" => document
                            .set_function_expression_with_angle_unit(
                                definition, expression, true,
                            )?,
                        "edit_function_degrees" => document
                            .set_function_expression_with_angle_unit(
                                definition, expression, false,
                            )?,
                        _ => document.set_function_expression(definition, expression)?,
                    }
                    println!("line {}: {command} -> object {definition}", index + 1);
                }
                "plot_parametric" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [x, y, mode, min, max] = parts.as_slice() else {
                        return Err(
                            "expected plot_parametric X_ID Y_ID rectangular|polar MIN MAX".into(),
                        );
                    };
                    let x = x.parse::<usize>().map_err(|_| "invalid X function ID")?;
                    let y = y.parse::<usize>().map_err(|_| "invalid Y function ID")?;
                    let mode = match *mode {
                        "rectangular" => PlotPointsMode::Cartesian,
                        "polar" => PlotPointsMode::Polar,
                        _ => return Err("expected rectangular or polar mode".into()),
                    };
                    let min = min.parse::<f64>().map_err(|_| "invalid domain start")?;
                    let max = max.parse::<f64>().map_err(|_| "invalid domain end")?;
                    let id = document.plot_parametric_curve(
                        x,
                        y,
                        mode,
                        min,
                        max,
                        Point { x: 199., y: 125. },
                    )?;
                    println!("line {}: plot_parametric -> object {id}", index + 1);
                }
                "set_plot_range" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [id, min, max, samples] = parts.as_slice() else {
                        return Err("expected set_plot_range ID MIN MAX SAMPLES".into());
                    };
                    let id = id.parse::<usize>().map_err(|_| "invalid plot ID")?;
                    let min = min.parse::<f64>().map_err(|_| "invalid domain start")?;
                    let max = max.parse::<f64>().map_err(|_| "invalid domain end")?;
                    let samples = samples.parse::<u32>().map_err(|_| "invalid sample count")?;
                    document.set_plot_range(id, min, max, samples)?;
                    println!("line {}: set_plot_range -> object {id}", index + 1);
                }
                "plot_function_degrees" => {
                    let expression = sketchpad_rs::function_records::SimpleFunction::parse(rest)?;
                    let id = document.plot_new_function_with_angle_unit(
                        expression,
                        Point { x: 199., y: 125. },
                        false,
                    )?;
                    println!("line {}: plot_function_degrees -> object {id}", index + 1);
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
                    let host: usize = rest.parse().map_err(|_| "invalid object ID")?;
                    let kind = if document
                        .objects
                        .get(host.saturating_sub(1))
                        .is_some_and(|o| o.kind.is_polygon())
                    {
                        Kind::PolygonArea(host)
                    } else {
                        Kind::CircleArea(host)
                    };
                    let id = document.add(kind);
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
                "animation_button" => {
                    let target: usize = rest.parse().map_err(|_| "invalid button target ID")?;
                    let id = document.add_animation_button(target)?;
                    println!("line {}: animation_button -> object {id}", index + 1);
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
                "text" => {
                    let mut parts = rest.splitn(3, char::is_whitespace);
                    let x: f64 = parts
                        .next()
                        .ok_or("expected text x")?
                        .parse()
                        .map_err(|_| "invalid text x")?;
                    let y: f64 = parts
                        .next()
                        .ok_or("expected text y")?
                        .parse()
                        .map_err(|_| "invalid text y")?;
                    let content = parts
                        .next()
                        .ok_or("expected text content")?
                        .replace("\\n", "\n");
                    let id = document.add_text(&content, Point { x, y })?;
                    println!("line {}: text -> object {id}", index + 1);
                }
                "edit_text" => {
                    let (id, content) = rest
                        .split_once(char::is_whitespace)
                        .ok_or("expected edit_text ID CONTENT")?;
                    let id = id.parse::<usize>().map_err(|_| "invalid text ID")?;
                    document.edit_text(id, &content.replace("\\n", "\n"))?;
                    println!("line {}: edit_text -> object {id}", index + 1);
                }
                "format_text" => {
                    let (id, flag) = rest
                        .split_once(char::is_whitespace)
                        .ok_or("expected format_text ID bold|italic|underline|larger|smaller")?;
                    let id = id.parse::<usize>().map_err(|_| "invalid text ID")?;
                    let command = match flag {
                        "bold" => TextStyleCommand::Bold,
                        "italic" => TextStyleCommand::Italic,
                        "underline" => TextStyleCommand::Underline,
                        "larger" => TextStyleCommand::IncreaseSize,
                        "smaller" => TextStyleCommand::DecreaseSize,
                        _ => {
                            return Err(
                                "expected bold, italic, underline, larger, or smaller".into()
                            );
                        }
                    };
                    document.selected.clear();
                    document.selected.insert(id);
                    document.toggle_selected_text_style(command)?;
                    println!("line {}: format_text -> object {id}", index + 1);
                }
                "font_text" => {
                    let (id, family) = rest
                        .split_once(char::is_whitespace)
                        .ok_or("expected font_text ID FAMILY")?;
                    let id = id.parse::<usize>().map_err(|_| "invalid text ID")?;
                    document.selected.clear();
                    document.selected.insert(id);
                    document.set_selected_text_family(family.trim())?;
                    println!("line {}: font_text -> object {id}", index + 1);
                }
                "format_range" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 4 {
                        return Err(
                            "expected format_range ID START END bold|italic|underline|larger|smaller".into()
                        );
                    }
                    let id = parts[0].parse::<usize>().map_err(|_| "invalid text ID")?;
                    let start = parts[1]
                        .parse::<usize>()
                        .map_err(|_| "invalid range start")?;
                    let end = parts[2].parse::<usize>().map_err(|_| "invalid range end")?;
                    let command = match parts[3] {
                        "bold" => TextStyleCommand::Bold,
                        "italic" => TextStyleCommand::Italic,
                        "underline" => TextStyleCommand::Underline,
                        "larger" => TextStyleCommand::IncreaseSize,
                        "smaller" => TextStyleCommand::DecreaseSize,
                        _ => {
                            return Err(
                                "expected bold, italic, underline, larger, or smaller".into()
                            );
                        }
                    };
                    document.format_text_range(id, start, end, command)?;
                    println!("line {}: format_range -> object {id}", index + 1);
                }
                "format_range_family" => {
                    let parts = rest.splitn(4, char::is_whitespace).collect::<Vec<_>>();
                    if parts.len() != 4 {
                        return Err("expected format_range_family ID START END FAMILY".into());
                    }
                    let id = parts[0].parse::<usize>().map_err(|_| "invalid text ID")?;
                    let start = parts[1]
                        .parse::<usize>()
                        .map_err(|_| "invalid range start")?;
                    let end = parts[2].parse::<usize>().map_err(|_| "invalid range end")?;
                    document.format_text_range_family(id, start, end, parts[3].trim())?;
                    println!("line {}: format_range_family -> object {id}", index + 1);
                }
                "text_edit" => {
                    let (id, content) = rest
                        .split_once(char::is_whitespace)
                        .ok_or("expected text ID and content")?;
                    let id: usize = id.parse().map_err(|_| "invalid text ID")?;
                    document.edit_text(id, &content.replace("\\n", "\n"))?;
                }
                "text_move" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected text ID x y".into());
                    }
                    let id = parts[0].parse().map_err(|_| "invalid text ID")?;
                    let x = parts[1].parse().map_err(|_| "invalid text x")?;
                    let y = parts[2].parse().map_err(|_| "invalid text y")?;
                    document.move_text(id, Point { x, y })?;
                }
                "label" => {
                    let (id, name) = rest
                        .split_once(char::is_whitespace)
                        .ok_or("expected point ID and label")?;
                    let id: usize = id.parse().map_err(|_| "invalid point ID")?;
                    document.set_point_label(id, name, true)?;
                }
                "label_show" => {
                    let (id, state) = rest
                        .split_once(char::is_whitespace)
                        .ok_or("expected point ID and on/off")?;
                    let id: usize = id.parse().map_err(|_| "invalid point ID")?;
                    let visible = match state {
                        "on" => true,
                        "off" => false,
                        _ => return Err("expected on or off".into()),
                    };
                    let name = document
                        .objects
                        .get(id.saturating_sub(1))
                        .and_then(|o| o.label.as_ref())
                        .and_then(|label| label.name_utf8().ok())
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("P{id}"));
                    document.set_point_label(id, &name, visible)?;
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
                "translate_vector" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected source, vector start, and vector end IDs".into());
                    }
                    let ids = parts
                        .iter()
                        .map(|part| part.parse::<usize>().map_err(|_| "invalid point ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let kind = if matches!(
                        document
                            .objects
                            .get(ids[0].saturating_sub(1))
                            .map(|o| &o.kind),
                        Some(Kind::Polygon(_))
                    ) {
                        Kind::VectorTranslatedPolygon(ids[0], ids[1], ids[2])
                    } else {
                        Kind::VectorTranslatedPoint(ids[0], ids[1], ids[2])
                    };
                    let id = document.add(kind);
                    println!("line {}: translate_vector -> object {id}", index + 1);
                }
                "translate_polygon" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err(
                            "expected polygon ID, angle, and distance in centimeters".into()
                        );
                    }
                    let source = parts[0].parse().map_err(|_| "invalid polygon ID")?;
                    let angle = parts[1].parse().map_err(|_| "invalid angle")?;
                    let distance = parts[2].parse().map_err(|_| "invalid distance")?;
                    let id = document.add(Kind::PolarTranslatedPolygon(
                        source,
                        angle,
                        37.79527559055118,
                        distance,
                    ));
                    println!("line {}: translate_polygon -> object {id}", index + 1);
                }
                "translate_polygon_rectangular" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err(
                            "expected polygon ID, horizontal and vertical centimeters".into()
                        );
                    }
                    let source = parts[0].parse().map_err(|_| "invalid polygon ID")?;
                    let horizontal = parts[1]
                        .parse()
                        .map_err(|_| "invalid horizontal distance")?;
                    let vertical = parts[2].parse().map_err(|_| "invalid vertical distance")?;
                    let id = document.add(Kind::FixedRectangularTranslatedPolygon(
                        source,
                        37.79527559055118,
                        horizontal,
                        37.79527559055118,
                        vertical,
                    ));
                    println!(
                        "line {}: translate_polygon_rectangular -> object {id}",
                        index + 1
                    );
                }
                "translate_segment" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err(
                            "expected segment ID, angle, and distance in centimeters".into()
                        );
                    }
                    let source = parts[0].parse().map_err(|_| "invalid segment ID")?;
                    let angle = parts[1].parse().map_err(|_| "invalid angle")?;
                    let distance = parts[2].parse().map_err(|_| "invalid distance")?;
                    let id = document.add(Kind::PolarTranslatedSegment(
                        source,
                        angle,
                        37.79527559055118,
                        distance,
                    ));
                    println!("line {}: translate_segment -> object {id}", index + 1);
                }
                "translate_circle" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err("expected circle ID, angle, and distance in centimeters".into());
                    }
                    let source = parts[0].parse().map_err(|_| "invalid circle ID")?;
                    let angle = parts[1].parse().map_err(|_| "invalid angle")?;
                    let distance = parts[2].parse().map_err(|_| "invalid distance")?;
                    let id = document.add(Kind::PolarTranslatedCircle(
                        source,
                        angle,
                        37.79527559055118,
                        distance,
                    ));
                    println!("line {}: translate_circle -> object {id}", index + 1);
                }
                "translate_circle_rectangular" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err(
                            "expected circle ID, horizontal centimeters, and vertical centimeters"
                                .into(),
                        );
                    }
                    let source = parts[0].parse().map_err(|_| "invalid circle ID")?;
                    let horizontal = parts[1]
                        .parse()
                        .map_err(|_| "invalid horizontal distance")?;
                    let vertical = parts[2].parse().map_err(|_| "invalid vertical distance")?;
                    let id = document.add(Kind::FixedRectangularTranslatedCircle(
                        source,
                        37.79527559055118,
                        horizontal,
                        37.79527559055118,
                        vertical,
                    ));
                    println!(
                        "line {}: translate_circle_rectangular -> object {id}",
                        index + 1
                    );
                }
                "translate_segment_vector" | "translate_circle_vector" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err("expected source ID, vector start ID, and vector end ID".into());
                    }
                    let source: usize = parts[0].parse().map_err(|_| "invalid source ID")?;
                    let start = parts[1].parse().map_err(|_| "invalid vector start ID")?;
                    let end = parts[2].parse().map_err(|_| "invalid vector end ID")?;
                    let kind = if command == "translate_segment_vector" {
                        Kind::VectorTranslatedSegment(source, start, end)
                    } else {
                        Kind::VectorTranslatedCircle(source, start, end)
                    };
                    let id = document.add(kind);
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "translate_segment_rectangular" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err(
                            "expected segment ID, horizontal centimeters, and vertical centimeters"
                                .into(),
                        );
                    }
                    let source = parts[0].parse().map_err(|_| "invalid segment ID")?;
                    let horizontal = parts[1]
                        .parse()
                        .map_err(|_| "invalid horizontal distance")?;
                    let vertical = parts[2].parse().map_err(|_| "invalid vertical distance")?;
                    let id = document.add(Kind::FixedRectangularTranslatedSegment(
                        source,
                        37.79527559055118,
                        horizontal,
                        37.79527559055118,
                        vertical,
                    ));
                    println!(
                        "line {}: translate_segment_rectangular -> object {id}",
                        index + 1
                    );
                }
                "translate_marked_distance" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err("expected source ID, distance value ID, and angle".into());
                    }
                    let source: usize = parts[0].parse().map_err(|_| "invalid source ID")?;
                    let distance = parts[1].parse().map_err(|_| "invalid distance ID")?;
                    let angle = parts[2].parse().map_err(|_| "invalid angle")?;
                    let kind = match document
                        .objects
                        .get(source.saturating_sub(1))
                        .map(|object| &object.kind)
                    {
                        Some(Kind::Segment(_, _)) => {
                            Kind::MarkedDistanceTranslatedSegment(source, distance, angle)
                        }
                        Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)) => {
                            Kind::MarkedDistanceTranslatedCircle(source, distance, angle)
                        }
                        Some(Kind::Polygon(_)) => {
                            Kind::MarkedDistanceTranslatedPolygon(source, distance, angle)
                        }
                        _ => Kind::MarkedDistanceTranslatedPoint(source, distance, angle),
                    };
                    let id = document.add(kind);
                    println!(
                        "line {}: translate_marked_distance -> object {id}",
                        index + 1
                    );
                }
                "translate_rectangular_fixed" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err("expected source point ID, horizontal centimeters, and vertical centimeters".into());
                    }
                    let source: usize = parts[0].parse().map_err(|_| "invalid source ID")?;
                    let horizontal = parts[1]
                        .parse()
                        .map_err(|_| "invalid horizontal distance")?;
                    let vertical = parts[2].parse().map_err(|_| "invalid vertical distance")?;
                    let id = document.add(Kind::FixedRectangularTranslatedPoint(
                        source,
                        37.79527559055118,
                        horizontal,
                        37.79527559055118,
                        vertical,
                    ));
                    println!(
                        "line {}: translate_rectangular_fixed -> object {id}",
                        index + 1
                    );
                }
                "translate_rectangular_marked" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err(
                            "expected source, horizontal distance, and vertical distance IDs"
                                .into(),
                        );
                    }
                    let ids = parts
                        .iter()
                        .map(|part| part.parse::<usize>().map_err(|_| "invalid object ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let kind = if matches!(
                        document
                            .objects
                            .get(ids[0].saturating_sub(1))
                            .map(|o| &o.kind),
                        Some(Kind::Polygon(_))
                    ) {
                        Kind::RectangularMarkedTranslatedPolygon(ids[0], ids[1], ids[2])
                    } else {
                        Kind::RectangularMarkedTranslatedPoint(ids[0], ids[1], ids[2])
                    };
                    let id = document.add(kind);
                    println!(
                        "line {}: translate_rectangular_marked -> object {id}",
                        index + 1
                    );
                }
                "translate_rectangular_mixed" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 4 {
                        return Err("expected source ID, marked distance ID, fixed centimeters, and horizontal_fixed|vertical_fixed".into());
                    }
                    let source: usize = parts[0].parse().map_err(|_| "invalid source ID")?;
                    let distance = parts[1].parse().map_err(|_| "invalid distance ID")?;
                    let fixed_cm = parts[2].parse().map_err(|_| "invalid fixed distance")?;
                    let fixed_horizontal = match parts[3] {
                        "horizontal_fixed" => true,
                        "vertical_fixed" => false,
                        _ => return Err("expected horizontal_fixed or vertical_fixed".into()),
                    };
                    let kind = if matches!(
                        document
                            .objects
                            .get(source.saturating_sub(1))
                            .map(|o| &o.kind),
                        Some(Kind::Polygon(_))
                    ) {
                        Kind::MixedRectangularTranslatedPolygon(
                            source,
                            distance,
                            fixed_cm,
                            fixed_horizontal,
                        )
                    } else {
                        Kind::MixedRectangularTranslatedPoint(
                            source,
                            distance,
                            fixed_cm,
                            fixed_horizontal,
                        )
                    };
                    let id = document.add(kind);
                    println!(
                        "line {}: translate_rectangular_mixed -> object {id}",
                        index + 1
                    );
                }
                "dilate" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected source ID, center point ID, and ratio".into());
                    }
                    let source: usize = parts[0].parse().map_err(|_| "invalid source ID")?;
                    let center = parts[1].parse().map_err(|_| "invalid center ID")?;
                    let factor = parts[2].parse().map_err(|_| "invalid ratio")?;
                    let kind = if matches!(
                        document
                            .objects
                            .get(source.saturating_sub(1))
                            .map(|o| &o.kind),
                        Some(Kind::Segment(_, _))
                    ) {
                        Kind::ScaledSegment(source, center, factor)
                    } else if document.circle_points(source).is_some() {
                        Kind::ScaledCircle(source, center, factor)
                    } else if matches!(
                        document
                            .objects
                            .get(source.saturating_sub(1))
                            .map(|o| &o.kind),
                        Some(Kind::Polygon(_))
                    ) {
                        Kind::ScaledPolygon(source, center, factor)
                    } else {
                        Kind::ScaledPoint(source, center, factor)
                    };
                    let id = document.add(kind);
                    println!("line {}: dilate -> object {id}", index + 1);
                }
                "dilate_marked_ratio" => {
                    let ids = rest
                        .split_whitespace()
                        .map(|part| part.parse::<usize>().map_err(|_| "invalid point ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    if ids.len() != 5 {
                        return Err(
                            "expected source, center, ratio origin, base, and target IDs".into(),
                        );
                    }
                    let kind = if matches!(
                        document
                            .objects
                            .get(ids[0].saturating_sub(1))
                            .map(|o| &o.kind),
                        Some(Kind::Segment(_, _))
                    ) {
                        Kind::RatioScaledSegment(ids[0], ids[1], ids[2], ids[3], ids[4])
                    } else if document.circle_points(ids[0]).is_some() {
                        Kind::RatioScaledCircle(ids[0], ids[1], ids[2], ids[3], ids[4])
                    } else if matches!(
                        document
                            .objects
                            .get(ids[0].saturating_sub(1))
                            .map(|o| &o.kind),
                        Some(Kind::Polygon(_))
                    ) {
                        Kind::RatioScaledPolygon(ids[0], ids[1], ids[2], ids[3], ids[4])
                    } else {
                        Kind::RatioScaledPoint(ids[0], ids[1], ids[2], ids[3], ids[4])
                    };
                    let id = document.add(kind);
                    println!("line {}: dilate_marked_ratio -> object {id}", index + 1);
                }
                "rotate" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected source point ID, center point ID, and angle".into());
                    }
                    let source: usize = parts[0].parse().map_err(|_| "invalid source ID")?;
                    let center = parts[1].parse().map_err(|_| "invalid center ID")?;
                    let angle = parts[2].parse().map_err(|_| "invalid angle")?;
                    let kind = match document
                        .objects
                        .get(source.saturating_sub(1))
                        .map(|o| &o.kind)
                    {
                        Some(Kind::Segment(_, _)) => Kind::RotatedSegment(source, center, angle),
                        Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)) => {
                            Kind::RotatedCircle(source, center, angle)
                        }
                        Some(Kind::Polygon(_)) => Kind::RotatedPolygon(source, center, angle),
                        _ => Kind::RotatedPoint(source, center, angle),
                    };
                    let id = document.add(kind);
                    println!("line {}: rotate -> object {id}", index + 1);
                }
                "rotate_marked_angle" => {
                    let parts = rest
                        .split_whitespace()
                        .map(|part| part.parse::<usize>().map_err(|_| "invalid point ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    if parts.len() != 5 {
                        return Err(
                            "expected source, center, angle start, vertex, and angle end IDs"
                                .into(),
                        );
                    }
                    let source: usize = parts[0];
                    let kind = match document
                        .objects
                        .get(source.saturating_sub(1))
                        .map(|o| &o.kind)
                    {
                        Some(Kind::Segment(_, _)) => Kind::AngleRotatedSegment(
                            source, parts[1], parts[2], parts[3], parts[4],
                        ),
                        Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)) => {
                            Kind::AngleRotatedCircle(source, parts[1], parts[2], parts[3], parts[4])
                        }
                        Some(Kind::Polygon(_)) => Kind::AngleRotatedPolygon(
                            source, parts[1], parts[2], parts[3], parts[4],
                        ),
                        _ => {
                            Kind::AngleRotatedPoint(source, parts[1], parts[2], parts[3], parts[4])
                        }
                    };
                    let id = document.add(kind);
                    println!("line {}: rotate_marked_angle -> object {id}", index + 1);
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
                            if matches!(
                                document.objects.get(a.saturating_sub(1)).map(|o| &o.kind),
                                Some(Kind::Polygon(_))
                            ) {
                                Kind::ReflectedPolygon(a, b)
                            } else if matches!(
                                document.objects.get(a.saturating_sub(1)).map(|o| &o.kind),
                                Some(Kind::Segment(_, _))
                            ) {
                                Kind::ReflectedSegment(a, b)
                            } else if document.circle_points(a).is_some() {
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
                "locus" => {
                    let (driver, target) = two_ids(rest)?;
                    let id = document.construct_locus(driver, target)?;
                    println!("line {}: locus -> object {id}", index + 1);
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
                        Some(
                            Kind::Free(_)
                                | Kind::GraphVerticalUnit(_, _)
                                | Kind::DistanceVerticalUnit(_, _, _)
                                | Kind::PointOnSegment(_, _)
                                | Kind::PointOnCircle(_, _, _)
                                | Kind::PointOnArc(_, _)
                        )
                    ) {
                        return Err("move requires a movable point".into());
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
                "point_style" | "line_width" | "line_pattern" | "color" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [id, value] = parts.as_slice() else {
                        return Err("expected object ID and style value".into());
                    };
                    let id: usize = id.parse().map_err(|_| "invalid object ID")?;
                    if document.objects.get(id.saturating_sub(1)).is_none() {
                        return Err("unknown object ID".into());
                    }
                    document.selected.clear();
                    document.selected.insert(id);
                    let changed = match command {
                        "point_style" => document.set_selected_point_style(match *value {
                            "dot" => PointStyle::Dot,
                            "small" => PointStyle::Small,
                            "medium" => PointStyle::Medium,
                            "thick" => PointStyle::Large,
                            _ => {
                                return Err(
                                    "point_style expects dot, small, medium, or thick".into()
                                );
                            }
                        })?,
                        "line_width" => document.set_selected_line_width(match *value {
                            "hairline" => LineWidth::Hairline,
                            "thin" => LineWidth::Thin,
                            "medium" => LineWidth::Medium,
                            "thick" => LineWidth::Thick,
                            _ => {
                                return Err(
                                    "line_width expects hairline, thin, medium, or thick".into()
                                );
                            }
                        })?,
                        "line_pattern" => document.set_selected_line_pattern(match *value {
                            "solid" => LinePattern::Solid,
                            "dashed" => LinePattern::Dashed,
                            "dotted" => LinePattern::Dotted,
                            _ => return Err("line_pattern expects solid, dashed, or dotted".into()),
                        })?,
                        "color" => {
                            let hex = value.strip_prefix('#').unwrap_or(value);
                            if hex.len() != 6 {
                                return Err("color expects six hexadecimal RGB digits".into());
                            }
                            let rgb =
                                u32::from_str_radix(hex, 16).map_err(|_| "invalid RGB color")?;
                            document.set_selected_color(RgbColor {
                                red: (rgb >> 16) as u8,
                                green: (rgb >> 8) as u8,
                                blue: rgb as u8,
                            })?
                        }
                        _ => unreachable!(),
                    };
                    if changed == 0 {
                        return Err("selected object does not support this style".into());
                    }
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
                                    || saved.visual_style != current.visual_style
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
        if let Kind::PlainText { rect, .. } = &object.kind {
            bounds.0 = bounds.0.min(rect.left.into());
            bounds.1 = bounds.1.min(rect.top.into());
            bounds.2 = bounds.2.max(rect.right.into());
            bounds.3 = bounds.3.max(rect.bottom.into());
        }
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
        if let Some(segments) = document.locus_segments(object.id, 128) {
            for (start, end) in segments {
                for point in [start, end] {
                    bounds.0 = bounds.0.min(point.x);
                    bounds.1 = bounds.1.min(point.y);
                    bounds.2 = bounds.2.max(point.x);
                    bounds.3 = bounds.3.max(point.y);
                }
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
