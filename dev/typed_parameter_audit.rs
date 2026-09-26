//! Audit every native subtype 80/81 group, including groups rejected by the decoder.
//! rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=TARGET_RLIB dev/typed_parameter_audit.rs -o /tmp/typed_parameter_audit
//! /tmp/typed_parameter_audit ../gsp-rs
use sketchpad_rs::{
    document::{Document, Kind, records},
    table_records::ParameterGroup,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn collect(path: &Path, output: &mut Vec<PathBuf>) {
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|kind| !kind.is_symlink()) {
                    collect(&entry.path(), output);
                }
            }
        }
    } else if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gsp"))
    {
        output.push(path.to_path_buf());
    }
}

fn main() {
    let mut files = Vec::new();
    collect(
        Path::new(
            &std::env::args()
                .nth(1)
                .unwrap_or_else(|| "../gsp-rs".into()),
        ),
        &mut files,
    );
    files.sort();
    let total_files = files.len();
    let mut unreadable = 0;
    let mut layouts = BTreeMap::<String, usize>::new();
    let mut results = BTreeMap::<String, usize>::new();
    for path in files {
        let Ok(bytes) = fs::read(&path) else {
            unreadable += 1;
            continue;
        };
        let Ok(rs) = records(&bytes) else {
            unreadable += 1;
            continue;
        };
        let mut ordinal = 0;
        let mut start = None;
        let mut document = None;
        for (index, record) in rs.iter().enumerate() {
            if record.kind == 0x07d0 {
                start = Some(index);
            }
            if record.kind != 0x07d7 {
                continue;
            }
            let Some(group_start) = start.take() else {
                continue;
            };
            ordinal += 1;
            let header = &bytes[rs[group_start].data.clone()];
            if header.len() != 28 {
                continue;
            }
            let subtype = u16::from_le_bytes(header[8..10].try_into().unwrap());
            if !matches!(subtype, 80 | 81)
                || u16::from_le_bytes(header[..2].try_into().unwrap()) != 0
            {
                continue;
            }
            let group = &bytes[rs[group_start].start..record.data.end];
            let words = rs[group_start..=index]
                .iter()
                .find(|r| r.kind == 0x0907)
                .map(|r| &bytes[r.data.clone()])
                .filter(|payload| payload.len() >= 94 && payload.len() % 2 == 0)
                .map(|payload| {
                    payload[92..]
                        .chunks_exact(2)
                        .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
                        .collect::<Vec<_>>()
                });
            let layout = match words {
                Some(words) => {
                    let suffix = words
                        .iter()
                        .position(|word| *word > 10)
                        .unwrap_or(words.len());
                    format!("subtype {subtype} decimal + {:04x?}", &words[suffix..])
                }
                None => format!("subtype {subtype} missing/short value record"),
            };
            *layouts.entry(layout).or_default() += 1;
            let result = match ParameterGroup::parse(group) {
                Err(error) => {
                    println!("REJECTED {} object {ordinal}: {error}", path.display());
                    format!("subtype {subtype} rejected: {error}")
                }
                Ok(_) => {
                    let decoded =
                        document.get_or_insert_with(|| Document::from_bytes(&bytes, None));
                    match decoded {
                        Ok(doc)
                            if doc.objects.get(ordinal - 1).is_some_and(|object| {
                                matches!(object.kind, Kind::Parameter(_))
                            }) =>
                        {
                            format!("subtype {subtype} parsed and imported")
                        }
                        _ => {
                            println!("DECODE GAP {} object {ordinal}", path.display());
                            format!("subtype {subtype} parsed but not imported")
                        }
                    }
                }
            };
            *results.entry(result).or_default() += 1;
        }
    }
    println!("Files: {total_files}; unreadable/non-GSP4: {unreadable}");
    for (layout, count) in layouts {
        println!("{count}: {layout}");
    }
    for (result, count) in results {
        println!("{count}: {result}");
    }
}
