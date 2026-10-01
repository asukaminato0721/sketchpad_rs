//! Headless probes for comparing original Sketchpad files with port output.

use sketchpad_rs::action_records::{
    AnimationDirection, MovementSpeed, ParameterAnimationSettings, PresentationMode,
    ScrollPlacement,
};
use sketchpad_rs::coordinate_measure_records::CoordinateMeasureKind;
use sketchpad_rs::document::colorized::{ColorBinding, ColorRangeBehavior};
use sketchpad_rs::document::{
    Clipboard, Document, EditRestriction, IterationDepth, Kind, Point, Record,
    SampledImageDistortion, TextStyleCommand, records,
};
use sketchpad_rs::function_records::PlotPresentation;
use sketchpad_rs::graph::GridForm;
use sketchpad_rs::plot_points_records::PlotPointsMode;
#[cfg(feature = "desktop")]
use sketchpad_rs::render::{Viewport, svg_page};
use sketchpad_rs::style_records::{LinePattern, LineWidth, PointStyle, RgbColor};
use sketchpad_rs::value_equation_records::ValueEquationKind;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

fn usage() -> String {
    format!(
        "Usage: gsp_probe inspect FILE | diff BEFORE AFTER | verify FILE... | coverage FILE_OR_DIRECTORY... | demo OUTPUT.gsp [1..{}] | replay SCRIPT | svg INPUT.gsp OUTPUT.svg",
        sketchpad_rs::demo::GEOMETRY_STEPS
    )
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
        "{} page(s), {} object(s), partial_view={}, edit_restriction={:?}",
        document.page_count,
        document.objects.len(),
        document.partial_view,
        document.edit_restriction()
    );
    for object in &document.objects {
        let label = object.label.as_ref().and_then(|label| {
            label
                .name_utf8()
                .ok()
                .map(|name| (name, label.visible(), label.offset()))
        });
        println!(
            "object {:4} page {:2} visible={} label={label:?} {:?}",
            object.id,
            object.page + 1,
            object.visible,
            object.kind
        );
    }
    for tool in document.embedded_tools()? {
        println!(
            "embedded tool {:?}: {} prototype object(s), inputs={:?}, complete={}",
            tool.name,
            tool.object_codes.len(),
            tool.input_ids,
            tool.complete
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

// Byte retention and object decoding are separate gates. This report makes
// unsupported payload families visible even when every untouched file saves.
fn coverage(paths: &[String]) -> Result<(), String> {
    fn collect(path: &Path, files: &mut BTreeSet<PathBuf>) -> Result<(), String> {
        if path.is_dir() {
            for entry in
                fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))?
            {
                let entry = entry.map_err(|error| error.to_string())?;
                let kind = entry.file_type().map_err(|error| error.to_string())?;
                if kind.is_dir() {
                    collect(&entry.path(), files)?;
                } else if kind.is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("gsp"))
                {
                    files.insert(entry.path());
                }
            }
        } else {
            files.insert(path.to_path_buf());
        }
        Ok(())
    }
    if paths.is_empty() {
        return Err(usage());
    }
    let mut files = BTreeSet::new();
    for path in paths {
        collect(Path::new(path), &mut files)?;
    }
    if files.is_empty() {
        return Err("No .gsp files found".into());
    }
    let mut unsupported: BTreeMap<u16, (usize, BTreeSet<PathBuf>)> = BTreeMap::new();
    let mut prototype_unsupported: BTreeMap<u16, (usize, BTreeSet<PathBuf>)> = BTreeMap::new();
    let (mut full, mut partial, mut objects, mut preserved, mut failures, mut partial_views) =
        (0, 0, 0, 0, 0, 0);
    let mut restrictions: BTreeMap<&'static str, (usize, PathBuf)> = BTreeMap::new();
    let (mut tool_definitions, mut tool_documents, mut tool_slots, mut scene_full) = (0, 0, 0, 0);
    let (mut tool_known, mut tool_complete, mut tool_partial, mut tool_failed) = (0, 0, 0, 0);
    for path in &files {
        let result = (|| -> Result<Document, String> {
            let bytes = fs::read(path).map_err(|error| error.to_string())?;
            let document = Document::from_bytes(&bytes, None)?;
            if document.encode()? == bytes {
                preserved += 1;
            } else {
                return Err("round trip changed source bytes".into());
            }
            Ok(document)
        })();
        match result {
            Ok(document) => {
                partial_views += usize::from(document.partial_view);
                if let Some(restriction) = document.edit_restriction() {
                    let name = match restriction {
                        EditRestriction::UnsupportedObjects => "unsupported objects",
                        EditRestriction::ReadOnlyImages => "read-only image resources",
                        EditRestriction::MultiPageStructure => "multi-page structure",
                        EditRestriction::UnrecognizedRecords => "unrecognized records",
                    };
                    let (count, _) = restrictions
                        .entry(name)
                        .or_insert_with(|| (0, path.clone()));
                    *count += 1;
                }
                let tools = document.embedded_tools()?;
                let prototype_slots: usize = tools.iter().map(|tool| tool.object_codes.len()).sum();
                tool_definitions += tools.len();
                tool_documents += usize::from(!tools.is_empty());
                tool_slots += prototype_slots;
                let mut has_unsupported_tools = false;
                for (index, summary) in tools.iter().enumerate() {
                    let unknown_codes = match document.embedded_tool_document(index) {
                        Ok(prototype) => {
                            let codes: Vec<_> = prototype
                                .objects
                                .iter()
                                .filter_map(|object| {
                                    if let Kind::Unsupported(code) = object.kind {
                                        Some(code)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            tool_known += prototype.objects.len() - codes.len();
                            if prototype.partial_view || !codes.is_empty() {
                                tool_partial += 1;
                            } else {
                                tool_complete += 1;
                            }
                            codes
                        }
                        Err(_) => {
                            // A failed prototype decode does not invalidate the
                            // independently preserved drawing document.
                            tool_failed += 1;
                            summary.object_codes.clone()
                        }
                    };
                    has_unsupported_tools |= !unknown_codes.is_empty();
                    for code in unknown_codes {
                        let (count, examples) = prototype_unsupported.entry(code).or_default();
                        *count += 1;
                        examples.insert(path.clone());
                    }
                }
                let has_unsupported_scene = document
                    .objects
                    .iter()
                    .any(|object| matches!(object.kind, Kind::Unsupported(_)));
                scene_full += usize::from(!has_unsupported_scene);
                if has_unsupported_scene || has_unsupported_tools {
                    partial += 1;
                } else {
                    full += 1;
                }
                objects += document.objects.len() + prototype_slots;
                for object in &document.objects {
                    if let Kind::Unsupported(code) = object.kind {
                        let (count, examples) = unsupported.entry(code).or_default();
                        *count += 1;
                        examples.insert(path.clone());
                    }
                }
            }
            Err(error) => {
                eprintln!("FAIL {}: {error}", path.display());
                failures += 1;
            }
        }
    }
    println!(
        "Documents: {} total, {full} with all object kinds decoded, {partial} with unsupported objects, {failures} failed",
        files.len()
    );
    println!(
        "Byte-preserving round trips: {preserved}/{}; object slots: {objects}",
        files.len()
    );
    println!(
        "Partial views: {partial_views}/{} (includes multi-page structural edit restrictions)",
        files.len()
    );
    for (reason, (count, example)) in restrictions {
        println!("  {reason}: {count}; first example: {}", example.display());
    }
    println!(
        "Drawing objects: {} slots; {scene_full} documents with all drawing kinds decoded",
        objects - tool_slots
    );
    let tool_unknown = tool_slots - tool_known;
    println!(
        "Embedded tool prototypes: {tool_definitions} definitions in {tool_documents} documents; {tool_slots} slots, {tool_known} decoded, {tool_unknown} undecoded"
    );
    println!(
        "Isolated tool views: {tool_complete} complete, {tool_partial} partial, {tool_failed} failed (separate object and font namespaces)"
    );
    let scene_unsupported: usize = unsupported.values().map(|(count, _)| count).sum();
    println!(
        "Unsupported object slots including tool prototypes: {}",
        scene_unsupported + tool_unknown
    );
    for (heading, counts) in [
        ("Unsupported drawing object codes", unsupported),
        ("Unsupported tool prototype codes", prototype_unsupported),
    ] {
        println!("{heading} (code, objects, documents, first example):");
        let mut families: Vec<_> = counts.into_iter().collect();
        families.sort_by_key(|(code, (count, _))| (std::cmp::Reverse(*count), *code));
        for (code, (count, examples)) in families {
            println!(
                "{code:5} {count:7} {:5} {}",
                examples.len(),
                examples.first().unwrap().display()
            );
        }
    }
    println!("Object decoding does not establish editing or command parity.");
    if failures == 0 {
        Ok(())
    } else {
        Err(format!("{failures} document(s) failed"))
    }
}

fn demo(path: &str, steps: usize) -> Result<(), String> {
    let mut document = sketchpad_rs::demo::geometry_document(steps)?;
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
                    document.set_parameter_value(2, f64::from(value))?;
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
                    let id = document.add_calculation_expression("2 + 3", number_anchor(rest)?)?;
                    println!("line {}: calculate_2_plus_3 -> object {id}", index + 1);
                }
                "calculate" => {
                    let id = document.add_calculation_expression(rest, (10, 38))?;
                    println!("line {}: calculate -> object {id}", index + 1);
                }
                "calculate_at" => {
                    let parts = rest.splitn(3, ' ').collect::<Vec<_>>();
                    if parts.len() != 3 {
                        return Err("expected X Y EXPRESSION".into());
                    }
                    let x = parts[0].parse().map_err(|_| "invalid X anchor")?;
                    let y = parts[1].parse().map_err(|_| "invalid Y anchor")?;
                    let id = document.add_calculation_expression(parts[2], (x, y))?;
                    println!("line {}: calculate_at -> object {id}", index + 1);
                }
                "attach_numeric_label" => {
                    let (point, value) = two_ids(rest)?;
                    let id = document.add_point_numeric_label(point, value)?;
                    println!("line {}: attach_numeric_label -> object {id}", index + 1);
                }
                "calculate_lengths" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [first, operator, second, y] = parts.as_slice() else {
                        return Err("expected calculate_lengths FIRST +|-|*|/ SECOND Y".into());
                    };
                    let operator = match *operator {
                        "+" => 0,
                        "-" => 1,
                        "*" => 2,
                        "/" => 3,
                        _ => return Err("Use +, -, *, or / for two lengths".into()),
                    };
                    let id = document.add_length_pair_operation(
                        first.parse().map_err(|_| "Invalid first source")?,
                        second.parse().map_err(|_| "Invalid second source")?,
                        operator,
                        (10, y.parse().map_err(|_| "Invalid row")?),
                    )?;
                    println!("line {}: calculate_lengths -> object {id}", index + 1);
                }
                "calculate_length" | "divide_length" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let (source, factor, y) = match parts.as_slice() {
                        [source, factor] => (*source, *factor, 34),
                        [source, factor, y] => (
                            *source,
                            *factor,
                            y.parse::<i16>().map_err(|_| "invalid calculation row")?,
                        ),
                        _ => {
                            return Err(format!("expected {command} MEASUREMENT_ID FACTOR [Y]"));
                        }
                    };
                    let source = source.parse().map_err(|_| "invalid measurement ID")?;
                    let factor = factor
                        .parse()
                        .map_err(|_| "invalid multiplication factor")?;
                    let id = if command == "divide_length" {
                        document.add_measured_length_division(source, factor, (10, y))?
                    } else {
                        document.add_measured_length_calculation(source, factor, (10, y))?
                    };
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "measurement_precision" | "numeric_precision" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [ids, precision] = parts.as_slice() else {
                        return Err(
                            "expected measurement_precision ID,ID,... DECIMAL_PLACES".into()
                        );
                    };
                    let ids = ids
                        .split(',')
                        .map(|id| id.parse::<usize>().map_err(|_| "Invalid measurement ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let precision = precision.parse().map_err(|_| "Invalid precision")?;
                    if command == "numeric_precision" {
                        document.set_numeric_precision(&ids, precision)?;
                    } else {
                        document.set_measurement_precision(&ids, precision)?;
                    }
                }
                "calculate_multi" => {
                    let (ids, expression) = rest
                        .split_once(' ')
                        .ok_or("expected ID,ID,... EXPRESSION")?;
                    let sources = ids
                        .split(',')
                        .map(|id| {
                            id.parse::<usize>()
                                .map_err(|_| "Invalid calculation source ID")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let id = document.add_multi_calculation(
                        &sources,
                        expression,
                        document.next_calculation_anchor(),
                    )?;
                    println!("line {}: calculate_multi -> object {id}", index + 1);
                }
                "calculate_parameter" => {
                    let Some((source, expression)) = rest.split_once(char::is_whitespace) else {
                        return Err("expected calculate_parameter SOURCE EXPRESSION".into());
                    };
                    let source = source
                        .parse()
                        .map_err(|_| "invalid calculation source ID")?;
                    let anchor = document.next_calculation_anchor();
                    let id = document.add_parameter_calculation(source, expression, anchor)?;
                    println!("line {}: calculate_parameter -> object {id}", index + 1);
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
                "split_point" => {
                    let id: usize = rest.parse().map_err(|_| "invalid point ID")?;
                    document.split_point(id)?;
                    println!("line {}: split_point {id}", index + 1);
                }
                "merge_points" => {
                    let (source, target) = two_ids(rest)?;
                    document.merge_points(source, target)?;
                    println!("line {}: merge_points {source} {target}", index + 1);
                }
                "custom_map" => {
                    let ids = rest
                        .split_whitespace()
                        .map(|part| part.parse::<usize>().map_err(|_| "invalid point ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let [seed, output, replacement] = ids.as_slice() else {
                        return Err("expected custom_map SEED OUTPUT REPLACEMENT".into());
                    };
                    let carried = document.custom_transform_carried(*seed, *output)?;
                    let id = document.add_mapped_element(*output, *seed, *replacement, &carried)?;
                    println!("line {}: custom_map -> object {id}", index + 1);
                }
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
                "picture_function" => {
                    let source: usize = rest.parse().map_err(|_| "invalid picture ID")?;
                    let id = document.add_picture_function(source, Point { x: 199.0, y: 125.0 })?;
                    println!("line {}: picture_function -> object {id}", index + 1);
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
                "distance_to_line" => {
                    let (point, host) = two_ids(rest)?;
                    let id = document.add(Kind::PointLineDistance(point, host));
                    println!("line {}: distance_to_line -> object {id}", index + 1);
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
                "axis_labels" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [id, format] = parts.as_slice() else {
                        return Err("expected axis_labels ID decimal|pi".into());
                    };
                    let id = id.parse().map_err(|_| "invalid axis object ID")?;
                    let pi = match *format {
                        "decimal" => false,
                        "pi" => true,
                        _ => return Err("expected axis_labels ID decimal|pi".into()),
                    };
                    document.set_axis_uses_pi_labels(id, pi)?;
                }
                "sampled_picture" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [id, samples, distortion] = parts.as_slice() else {
                        return Err("expected sampled_picture ID SAMPLES none|some|most|all".into());
                    };
                    let id = id
                        .parse()
                        .map_err(|_| "invalid sampled-picture object ID")?;
                    let samples = samples.parse().map_err(|_| "invalid sample count")?;
                    let distortion = match *distortion {
                        "none" => SampledImageDistortion::None,
                        "some" => SampledImageDistortion::Some,
                        "most" => SampledImageDistortion::Most,
                        "all" => SampledImageDistortion::All,
                        _ => {
                            return Err(
                                "expected sampled_picture ID SAMPLES none|some|most|all".into()
                            );
                        }
                    };
                    document.set_sampled_image_properties(id, samples, distortion)?;
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
                        Some(
                            Kind::PointOnSegment(host, _)
                            | Kind::PointOnRay(host, _)
                            | Kind::PointOnLine(host, _)
                            | Kind::PointOnArc(host, _)
                            | Kind::PointOnPolygon(host, _, _)
                            | Kind::PointOnCircle(host, _, _),
                        ) => *host,
                        _ => {
                            return Err(
                                "point_value requires a point on a segment, ray, line, circle, or arc"
                                    .into(),
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
                        Some(kind) if kind.is_line() => ValueEquationKind::LineEquation,
                        Some(kind) if kind.is_circle() => ValueEquationKind::CircleEquation,
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
                "plot_properties" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [id, min, max, samples, drawing, endpoints] = parts.as_slice() else {
                        return Err("expected plot_properties ID MIN MAX SAMPLES continuous|discrete arrows|no_arrows".into());
                    };
                    let id = id.parse::<usize>().map_err(|_| "invalid plot ID")?;
                    let min = min.parse::<f64>().map_err(|_| "invalid domain start")?;
                    let max = max.parse::<f64>().map_err(|_| "invalid domain end")?;
                    let samples = samples.parse::<u32>().map_err(|_| "invalid sample count")?;
                    let discrete = match *drawing {
                        "continuous" => false,
                        "discrete" => true,
                        _ => return Err("expected continuous or discrete drawing".into()),
                    };
                    let show_arrows_and_endpoints = match *endpoints {
                        "arrows" => true,
                        "no_arrows" => false,
                        _ => return Err("expected arrows or no_arrows".into()),
                    };
                    document.set_plot_properties(
                        id,
                        min,
                        max,
                        samples,
                        PlotPresentation {
                            discrete,
                            show_arrows_and_endpoints,
                        },
                    )?;
                    println!("line {}: plot_properties -> object {id}", index + 1);
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
                "opacity" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [id, opacity] = parts.as_slice() else {
                        return Err("expected opacity OBJECT_ID 0..65535".into());
                    };
                    document.set_object_polygon_opacity(
                        id.parse().map_err(|_| "invalid object ID")?,
                        opacity.parse().map_err(|_| "invalid opacity")?,
                    )?;
                }
                "perimeter" => {
                    let polygon: usize = rest.parse().map_err(|_| "invalid polygon ID")?;
                    let id = document.add(Kind::PolygonPerimeter(polygon));
                    println!("line {}: perimeter -> object {id}", index + 1);
                }
                "hide_show_button" => {
                    let targets = rest
                        .split_whitespace()
                        .map(|part| part.parse().map_err(|_| "invalid button target ID"))
                        .collect::<Result<Vec<usize>, &str>>()?;
                    let id = document.add_hide_show_button_for_targets(&targets)?;
                    println!("line {}: hide_show_button -> object {id}", index + 1);
                }
                "sound_button" => {
                    let functions = rest
                        .split_whitespace()
                        .map(|part| part.parse().map_err(|_| "invalid Sound function ID"))
                        .collect::<Result<Vec<usize>, &str>>()?;
                    let id = document.add_sound_button_for_functions(&functions)?;
                    println!("line {}: sound_button -> object {id}", index + 1);
                }
                "animation_button" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.is_empty() || parts.len() > 4 {
                        return Err("expected target [forward|backward|bidirectional|random] [speed] [once|repeat]".into());
                    }
                    let target: usize = parts[0].parse().map_err(|_| "invalid button target ID")?;
                    let mut settings = document
                        .animation_button_default_settings(target)
                        .ok_or("invalid animation target")?;
                    if let Some(direction) = parts.get(1) {
                        settings.direction = match *direction {
                            "forward" => AnimationDirection::Forward,
                            "backward" => AnimationDirection::Backward,
                            "bidirectional" => AnimationDirection::Bidirectional,
                            "random" => AnimationDirection::Random,
                            _ => return Err("invalid animation direction".into()),
                        };
                    }
                    if let Some(speed) = parts.get(2) {
                        settings.speed = speed.parse().map_err(|_| "invalid animation speed")?;
                    }
                    if let Some(travel) = parts.get(3) {
                        settings.once = match *travel {
                            "once" => true,
                            "repeat" => false,
                            _ => return Err("expected once or repeat".into()),
                        };
                    }
                    let id = document.add_animation_button_with_settings(target, settings)?;
                    println!("line {}: animation_button -> object {id}", index + 1);
                }
                "animation_button_from" => {
                    let source: usize = rest.parse().map_err(|_| "invalid animation source")?;
                    let kind = document
                        .objects
                        .get(source.checked_sub(1).ok_or("invalid animation source")?)
                        .map(|object| &object.kind)
                        .ok_or("invalid animation source")?;
                    let label = kind
                        .animation_button_source_label()
                        .ok_or("expected a supported geometric source")?;
                    let mut targets = kind.dependencies();
                    targets.reverse();
                    let values = targets
                        .into_iter()
                        .map(|id| {
                            document
                                .animation_button_default_settings(id)
                                .map(|settings| (id, settings))
                                .ok_or("invalid animation source point".to_string())
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let id = document.add_animation_button_for_targets_with_settings_and_label(
                        &values,
                        Some(label),
                    )?;
                    println!("line {}: animation_button_from -> object {id}", index + 1);
                }
                "animation_parameter_button" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if !(7..=8).contains(&parts.len()) {
                        return Err("expected target direction rate continuous|discrete step min max [once|repeat]".into());
                    }
                    let target: usize = parts[0].parse().map_err(|_| "invalid parameter target")?;
                    let mut settings = document
                        .animation_button_default_settings(target)
                        .ok_or("invalid parameter target")?;
                    if settings.parameter.is_none() {
                        return Err("target is not a parameter".into());
                    }
                    settings.direction = match parts[1] {
                        "forward" => AnimationDirection::Forward,
                        "backward" => AnimationDirection::Backward,
                        "bidirectional" => AnimationDirection::Bidirectional,
                        "random" => AnimationDirection::Random,
                        _ => return Err("invalid animation direction".into()),
                    };
                    settings.speed = parts[2].parse().map_err(|_| "invalid parameter rate")?;
                    settings.parameter = Some(ParameterAnimationSettings {
                        discrete: match parts[3] {
                            "continuous" => false,
                            "discrete" => true,
                            _ => return Err("expected continuous or discrete".into()),
                        },
                        step: parts[4].parse().map_err(|_| "invalid parameter step")?,
                        min: parts[5].parse().map_err(|_| "invalid parameter minimum")?,
                        max: parts[6].parse().map_err(|_| "invalid parameter maximum")?,
                    });
                    if let Some(travel) = parts.get(7) {
                        settings.once = match *travel {
                            "once" => true,
                            "repeat" => false,
                            _ => return Err("expected once or repeat".into()),
                        };
                    }
                    let id = document.add_animation_button_with_settings(target, settings)?;
                    println!(
                        "line {}: animation_parameter_button -> object {id}",
                        index + 1
                    );
                }
                "animation_button_multi" => {
                    let mut targets = Vec::new();
                    for spec in rest.split_whitespace() {
                        let parts = spec.split(':').collect::<Vec<_>>();
                        if parts.len() > 4 {
                            return Err("expected target[:direction[:speed[:once|repeat]]]".into());
                        }
                        let target: usize =
                            parts[0].parse().map_err(|_| "invalid animation target")?;
                        let mut settings = document
                            .animation_button_default_settings(target)
                            .ok_or("invalid animation target")?;
                        if let Some(direction) = parts.get(1) {
                            settings.direction = match *direction {
                                "forward" => AnimationDirection::Forward,
                                "backward" => AnimationDirection::Backward,
                                "bidirectional" => AnimationDirection::Bidirectional,
                                "random" => AnimationDirection::Random,
                                _ => return Err("invalid animation direction".into()),
                            };
                        }
                        if let Some(speed) = parts.get(2) {
                            settings.speed =
                                speed.parse().map_err(|_| "invalid animation speed")?;
                        }
                        if let Some(travel) = parts.get(3) {
                            settings.once = match *travel {
                                "once" => true,
                                "repeat" => false,
                                _ => return Err("expected once or repeat".into()),
                            };
                        }
                        targets.push((target, settings));
                    }
                    let id = document.add_animation_button_for_targets_with_settings(&targets)?;
                    println!("line {}: animation_button_multi -> object {id}", index + 1);
                }
                "movement_button" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [source, destination, speed, mode] = parts.as_slice() else {
                        return Err(
                            "expected source destination slow|medium|fast|instant follow|initial"
                                .into(),
                        );
                    };
                    let source = source.parse().map_err(|_| "invalid source point")?;
                    let destination = destination
                        .parse()
                        .map_err(|_| "invalid destination point")?;
                    let speed = match *speed {
                        "slow" => MovementSpeed::Slow,
                        "medium" => MovementSpeed::Medium,
                        "fast" => MovementSpeed::Fast,
                        "instant" => MovementSpeed::Instant,
                        _ => return Err("invalid Movement speed".into()),
                    };
                    let follow = match *mode {
                        "follow" => true,
                        "initial" => false,
                        _ => return Err("expected follow or initial".into()),
                    };
                    let id = document.add_movement_button(source, destination, speed, follow)?;
                    println!("line {}: movement_button -> object {id}", index + 1);
                }
                "scroll_button" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [target, placement] = parts.as_slice() else {
                        return Err("expected point_id upper_left|center".into());
                    };
                    let target = target.parse().map_err(|_| "invalid Scroll target")?;
                    let placement = match *placement {
                        "upper_left" => ScrollPlacement::UpperLeft,
                        "center" => ScrollPlacement::Center,
                        _ => return Err("expected upper_left or center".into()),
                    };
                    let id = document.add_scroll_button(target, placement)?;
                    println!("line {}: scroll_button -> object {id}", index + 1);
                }
                "presentation_button" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() < 2 {
                        return Err("expected simultaneous|sequential action_button_id...".into());
                    }
                    let mode = match parts[0] {
                        "simultaneous" => PresentationMode::Simultaneous,
                        "sequential" => PresentationMode::Sequential,
                        _ => return Err("expected simultaneous or sequential".into()),
                    };
                    let targets = parts[1..]
                        .iter()
                        .map(|part| {
                            part.parse::<usize>()
                                .map_err(|_| "invalid action button ID")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let id = document.add_presentation_button_for_targets(&targets, mode)?;
                    println!("line {}: presentation_button -> object {id}", index + 1);
                }
                "link_url_button" => {
                    let id = document.add_link_url_button(rest)?;
                    println!("line {}: link_url_button -> object {id}", index + 1);
                }
                "add_page" if rest.is_empty() => {
                    let page = document.add_blank_page()?;
                    println!("line {}: add_page -> page {page}", index + 1);
                }
                "copy_page" => {
                    let page: usize = rest.parse().map_err(|_| "invalid page number")?;
                    if page == 0 {
                        return Err("page numbers start at 1".into());
                    }
                    let added = document.copy_page(page - 1)?;
                    println!("line {}: copy_page -> page {added}", index + 1);
                }
                "rename_page" => {
                    let (page, name) = rest
                        .split_once(char::is_whitespace)
                        .ok_or("expected page number and name")?;
                    let page: usize = page.parse().map_err(|_| "invalid page number")?;
                    if page == 0 {
                        return Err("page numbers start at 1".into());
                    }
                    document.set_page_name(page - 1, name.trim())?;
                }
                "remove_page" => {
                    let page: usize = rest.parse().map_err(|_| "invalid page number")?;
                    if page == 0 {
                        return Err("page numbers start at 1".into());
                    }
                    document.remove_page(page - 1)?;
                }
                "move_page" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    let [from, to] = parts.as_slice() else {
                        return Err("expected source and destination page numbers".into());
                    };
                    let from: usize = from.parse().map_err(|_| "invalid source page")?;
                    let to: usize = to.parse().map_err(|_| "invalid destination page")?;
                    if from == 0 || to == 0 {
                        return Err("page numbers start at 1".into());
                    }
                    document.move_page(from - 1, to - 1)?;
                }
                "page" => {
                    let page: usize = rest.parse().map_err(|_| "invalid page number")?;
                    if page == 0 || page > document.page_count {
                        return Err("page number is outside this document".into());
                    }
                    document.active_page = page - 1;
                }
                "link_page_button" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if !matches!(parts.len(), 1 | 2) {
                        return Err("expected destination page and optional button ordinal".into());
                    }
                    let page: usize = parts[0].parse().map_err(|_| "invalid destination page")?;
                    if page == 0 {
                        return Err("destination pages start at 1".into());
                    }
                    let ordinal = parts
                        .get(1)
                        .map(|part| {
                            part.parse()
                                .map_err(|_| "invalid destination button ordinal")
                        })
                        .transpose()?;
                    let id = document.add_link_page_button_with_target(page - 1, ordinal)?;
                    println!("line {}: link_page_button -> object {id}", index + 1);
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
                        .ok_or("expected object ID and label")?;
                    let id: usize = id.parse().map_err(|_| "invalid object ID")?;
                    document.set_object_label(id, name, true)?;
                }
                "label_show" => {
                    let (id, state) = rest
                        .split_once(char::is_whitespace)
                        .ok_or("expected object ID and on/off")?;
                    let id: usize = id.parse().map_err(|_| "invalid object ID")?;
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
                    document.set_object_label(id, &name, visible)?;
                }
                "colorize_spectrum" => {
                    let (source, value) = two_ids(rest)?;
                    let id = document.add_colorized(
                        source,
                        ColorBinding::Spectrum { value },
                        0.,
                        1.,
                        ColorRangeBehavior::Clamp,
                    )?;
                    println!("line {}: colorize_spectrum -> object {id}", index + 1);
                }
                "point_on" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected host ID and x y coordinates".into());
                    }
                    let host = parts[0].parse().map_err(|_| "invalid host ID")?;
                    let x = parts[1].parse().map_err(|_| "invalid x coordinate")?;
                    let y = parts[2].parse().map_err(|_| "invalid y coordinate")?;
                    let kind = document.constrained_point(host, Point { x, y }).ok_or(
                        "host must be a nondegenerate straight path, circle, arc, or polygon",
                    )?;
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
                "iterate" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 3 {
                        return Err("expected seed ID, first-image ID, and depth".into());
                    }
                    let seed = parts[0].parse().map_err(|_| "invalid seed ID")?;
                    let image = parts[1].parse().map_err(|_| "invalid first-image ID")?;
                    let depth = parts[2].parse().map_err(|_| "invalid iteration depth")?;
                    let id =
                        document.add_point_iteration(seed, image, IterationDepth::Fixed(depth))?;
                    println!("line {}: iterate -> object {id}", index + 1);
                }
                "iterate_segment" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 4 {
                        return Err(
                            "expected seed ID, first-image ID, segment ID, and depth".into()
                        );
                    }
                    let seed = parts[0].parse().map_err(|_| "invalid seed ID")?;
                    let image = parts[1].parse().map_err(|_| "invalid first-image ID")?;
                    let segment = parts[2].parse().map_err(|_| "invalid segment ID")?;
                    let depth = parts[3].parse().map_err(|_| "invalid iteration depth")?;
                    let id = document.add_segment_iteration(
                        seed,
                        image,
                        segment,
                        IterationDepth::Fixed(depth),
                    )?;
                    println!("line {}: iterate_segment -> object {id}", index + 1);
                }
                "iterate_circle" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if parts.len() != 4 {
                        return Err("expected seed ID, first-image ID, circle ID, and depth".into());
                    }
                    let seed = parts[0].parse().map_err(|_| "invalid seed ID")?;
                    let image = parts[1].parse().map_err(|_| "invalid first-image ID")?;
                    let circle = parts[2].parse().map_err(|_| "invalid circle ID")?;
                    let depth = parts[3].parse().map_err(|_| "invalid iteration depth")?;
                    let id = document.add_circle_iteration(
                        seed,
                        image,
                        circle,
                        IterationDepth::Fixed(depth),
                    )?;
                    println!("line {}: iterate_circle -> object {id}", index + 1);
                }
                "iterate_mapped" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    if !(4..=5).contains(&parts.len()) {
                        return Err("expected SEED_IDS BRANCH_IDS[;BRANCH_IDS...] CARRIED_IDS DEPTH_OR_value:PARAMETER [VISIBLE_BINDING_IDS]".into());
                    }
                    let ids = |part: &str| {
                        part.split(',')
                            .map(|v| {
                                v.parse::<usize>()
                                    .map_err(|_| "invalid mapped iteration ID")
                            })
                            .collect::<Result<Vec<_>, _>>()
                    };
                    let seeds = ids(parts[0])?;
                    let branches = parts[1]
                        .split(';')
                        .map(ids)
                        .collect::<Result<Vec<_>, _>>()?;
                    let carried = ids(parts[2])?;
                    let depth = if let Some(id) = parts[3].strip_prefix("value:") {
                        IterationDepth::Value(id.parse().map_err(|_| "invalid depth parameter")?)
                    } else {
                        IterationDepth::Fixed(
                            parts[3]
                                .parse()
                                .map_err(|_| "invalid mapped iteration depth")?,
                        )
                    };
                    let id = if let Some(bindings) = parts.get(4) {
                        document.add_mapped_point_iteration_with_bindings(
                            &seeds,
                            &branches,
                            &carried,
                            &ids(bindings)?,
                            depth,
                        )?
                    } else {
                        document.add_mapped_point_iteration(&seeds, &branches, &carried, depth)?
                    };
                    println!("line {}: iterate_mapped -> object {id}", index + 1);
                }
                "iterate_polygon_branch" | "iterate_polygon_branch_live" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    if parts.len() < 5 {
                        return Err("expected SEED FIRST_IMAGE SECOND_IMAGE POLYGON DEPTH_OR_PARAMETER [SEGMENT_IDS...]".into());
                    }
                    let ids = parts
                        .iter()
                        .map(|p| {
                            p.parse::<usize>()
                                .map_err(|_| "invalid branch iteration ID or depth")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let depth = if command == "iterate_polygon_branch_live" {
                        sketchpad_rs::document::IterationDepth::Value(ids[4])
                    } else {
                        sketchpad_rs::document::IterationDepth::Fixed(
                            u32::try_from(ids[4]).map_err(|_| "invalid depth")?,
                        )
                    };
                    let id = document.add_branching_polygon_iteration(
                        ids[0],
                        ids[1],
                        ids[2],
                        ids[3],
                        &ids[5..],
                        depth,
                    )?;
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "rotate_value" => {
                    let ids = rest
                        .split_whitespace()
                        .map(|p| p.parse::<usize>().map_err(|_| "invalid rotation ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let [source, center, angle] = ids.as_slice() else {
                        return Err("expected SOURCE CENTER ANGLE_VALUE".into());
                    };
                    let id = document.add_calculated_angle_rotation(*source, *center, *angle)?;
                    println!("line {}: rotate_value -> object {id}", index + 1);
                }
                "iterate_point_live" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [seed, image, depth] = parts.as_slice() else {
                        return Err("expected SEED IMAGE DEPTH_VALUE".into());
                    };
                    let id = document.add_point_iteration(
                        seed.parse().map_err(|_| "invalid seed")?,
                        image.parse().map_err(|_| "invalid image")?,
                        IterationDepth::Value(depth.parse().map_err(|_| "invalid depth")?),
                    )?;
                    println!("line {}: iterate_point_live -> object {id}", index + 1);
                }
                "iterate_segment_live" => {
                    let ids = rest
                        .split_whitespace()
                        .map(|p| p.parse::<usize>().map_err(|_| "invalid iteration ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let [seed, image, segment, depth] = ids.as_slice() else {
                        return Err("expected SEED IMAGE SEGMENT DEPTH_VALUE".into());
                    };
                    let id = document.add_segment_iteration(
                        *seed,
                        *image,
                        *segment,
                        IterationDepth::Value(*depth),
                    )?;
                    println!("line {}: iterate_segment_live -> object {id}", index + 1);
                }
                "iterate_circle_live" => {
                    let ids = rest
                        .split_whitespace()
                        .map(|p| p.parse::<usize>().map_err(|_| "invalid iteration ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let [seed, image, circle, depth] = ids.as_slice() else {
                        return Err("expected SEED IMAGE SEGMENT DEPTH_VALUE".into());
                    };
                    let id = document.add_circle_iteration(
                        *seed,
                        *image,
                        *circle,
                        IterationDepth::Value(*depth),
                    )?;
                    println!("line {}: iterate_circle_live -> object {id}", index + 1);
                }
                "iterate_numeric" | "iterate_numeric_live" => {
                    let parts = rest.split_whitespace().collect::<Vec<_>>();
                    let [seed, parameter, image, calculation, label, depth] = parts.as_slice()
                    else {
                        return Err(
                            "expected iterate_numeric SEED PARAMETER IMAGE CALCULATION LABEL DEPTH"
                                .into(),
                        );
                    };
                    let parse = |text: &str| {
                        text.parse::<usize>()
                            .map_err(|_| "Invalid iteration parent ID")
                    };
                    let depth = if command == "iterate_numeric_live" {
                        IterationDepth::Value(parse(depth)?)
                    } else {
                        IterationDepth::Fixed(depth.parse().map_err(|_| "Invalid iteration depth")?)
                    };
                    let id = document.add_point_parameter_iteration(
                        parse(seed)?,
                        parse(parameter)?,
                        parse(image)?,
                        parse(calculation)?,
                        parse(label)?,
                        depth,
                    )?;
                    println!("line {}: {command} -> object {id}", index + 1);
                }
                "iterate_ray" => {
                    let parts: Vec<_> = rest.split_whitespace().collect();
                    let [seed, image, ray, parameter] = parts.as_slice() else {
                        return Err(
                            "expected seed ID, first-image ID, ray ID, and scalar depth value ID"
                                .into(),
                        );
                    };
                    let seed = seed.parse().map_err(|_| "invalid seed ID")?;
                    let image = image.parse().map_err(|_| "invalid first-image ID")?;
                    let ray = ray.parse().map_err(|_| "invalid ray ID")?;
                    let parameter = parameter
                        .parse()
                        .map_err(|_| "invalid depth parameter ID")?;
                    let id = document.add_ray_iteration(seed, image, ray, parameter)?;
                    println!("line {}: iterate_ray -> object {id}", index + 1);
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
                        Some(kind) if kind.is_polygon()
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
                    let kind = match document
                        .objects
                        .get(ids[0].saturating_sub(1))
                        .map(|o| &o.kind)
                    {
                        Some(Kind::Segment(_, _)) => {
                            Kind::RectangularMarkedTranslatedSegment(ids[0], ids[1], ids[2])
                        }
                        Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)) => {
                            Kind::RectangularMarkedTranslatedCircle(ids[0], ids[1], ids[2])
                        }
                        Some(Kind::Polygon(_)) => {
                            Kind::RectangularMarkedTranslatedPolygon(ids[0], ids[1], ids[2])
                        }
                        _ => Kind::RectangularMarkedTranslatedPoint(ids[0], ids[1], ids[2]),
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
                    let kind = match document
                        .objects
                        .get(source.saturating_sub(1))
                        .map(|o| &o.kind)
                    {
                        Some(Kind::Segment(_, _)) => Kind::MixedRectangularTranslatedSegment(
                            source,
                            distance,
                            fixed_cm,
                            fixed_horizontal,
                        ),
                        Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)) => {
                            Kind::MixedRectangularTranslatedCircle(
                                source,
                                distance,
                                fixed_cm,
                                fixed_horizontal,
                            )
                        }
                        Some(Kind::Polygon(_)) => Kind::MixedRectangularTranslatedPolygon(
                            source,
                            distance,
                            fixed_cm,
                            fixed_horizontal,
                        ),
                        _ => Kind::MixedRectangularTranslatedPoint(
                            source,
                            distance,
                            fixed_cm,
                            fixed_horizontal,
                        ),
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
                    } else if matches!(document.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(kind) if kind.fixed_transform_straight_mode() == Some(sketchpad_rs::document::LineMode::Ray))
                    {
                        Kind::ScaledRay(source, center, factor)
                    } else if matches!(document.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(kind) if kind.fixed_transform_straight_mode() == Some(sketchpad_rs::document::LineMode::Line))
                    {
                        Kind::ScaledLine(source, center, factor)
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
                "dilate_value" => {
                    let ids = rest
                        .split_whitespace()
                        .map(|p| p.parse::<usize>().map_err(|_| "invalid dilation ID"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let [source, center, ratio] = ids.as_slice() else {
                        return Err("expected SOURCE CENTER RATIO_VALUE".into());
                    };
                    let id = document.add_calculated_ratio_dilation(*source, *center, *ratio)?;
                    println!("line {}: dilate_value -> object {id}", index + 1);
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
                        return Err("expected source object ID, center point ID, and angle".into());
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
                        Some(kind)
                            if kind.fixed_transform_straight_mode()
                                == Some(sketchpad_rs::document::LineMode::Ray) =>
                        {
                            Kind::RotatedRay(source, center, angle)
                        }
                        Some(kind)
                            if kind.fixed_transform_straight_mode()
                                == Some(sketchpad_rs::document::LineMode::Line) =>
                        {
                            Kind::RotatedLine(source, center, angle)
                        }
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
                "segment"
                | "circle"
                | "circle_segment"
                | "midpoint"
                | "line"
                | "ray"
                | "reflect"
                | "parallel"
                | "intersection"
                | "boundary_intersection"
                | "perpendicular" => {
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
                        "boundary_intersection" => Kind::BoundaryIntersection(a, b),
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
                "bisector" | "arc3" | "arc_on_circle" | "arc_center" => {
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
                        "arc_on_circle" => Kind::ArcOnCircle(start, vertex, end),
                        _ => Kind::CenterArc(start, vertex, end),
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
                                | Kind::PointOnRay(_, _)
                                | Kind::PointOnLine(_, _)
                                | Kind::PointOnCircle(_, _, _)
                                | Kind::PointOnArc(_, _)
                                | Kind::PointOnPolygon(_, _, _)
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
                "check" if rest.is_empty() => document.verify_roundtrip()?,
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

#[cfg(feature = "desktop")]
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
        if let Some(surfaces) = document
            .mapped_collection_surfaces(object.id, 32)
            .or_else(|| document.locus_surfaces(object.id, 32))
        {
            for point in surfaces.into_iter().flat_map(|surface| surface.points) {
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

#[cfg(not(feature = "desktop"))]
fn svg(_input: &str, _output: &str) -> Result<(), String> {
    Err("SVG export requires the desktop feature; rebuild without --no-default-features".into())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [command, path] if command == "inspect" => inspect(path),
        [command, before, after] if command == "diff" => diff(before, after),
        [command, paths @ ..] if command == "verify" => verify(paths),
        [command, paths @ ..] if command == "coverage" => coverage(paths),
        [command, path] if command == "demo" => demo(path, sketchpad_rs::demo::GEOMETRY_STEPS),
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
