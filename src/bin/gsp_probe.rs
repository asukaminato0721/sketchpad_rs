//! Headless probes for comparing original Sketchpad files with port output.

use sketchpad_rs::document::{Document, Kind, Point, Record, records};
use std::{env, fs, path::Path};

fn usage() -> String {
    "Usage: gsp_probe inspect FILE | diff BEFORE AFTER | verify FILE... | demo OUTPUT.gsp [1..9]"
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
        _ => Err(usage()),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
