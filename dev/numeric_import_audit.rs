//! Standalone numeric prerequisite audit. Compile against the current library:
//! rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=TARGET_RLIB dev/numeric_import_audit.rs -o /tmp/numeric_import_audit
//! /tmp/numeric_import_audit ../gsp-rs
use sketchpad_rs::document::{Document, Kind, records};
use sketchpad_rs::table_records::ParameterCalculationGroup;
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
    let mut counts: BTreeMap<String, (usize, String)> = BTreeMap::new();
    for path in files {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(document) = Document::from_bytes(&bytes, None) else {
            continue;
        };
        let Ok(rs) = records(&bytes) else {
            continue;
        };
        let mut start = None;
        let mut ordinal = 0;
        for record in &rs {
            if record.kind == 0x07d0 {
                start = Some(record.start);
            }
            if record.kind != 0x07d7 {
                continue;
            }
            let Some(group_start) = start.take() else {
                continue;
            };
            let Some(object) = document.objects.get(ordinal) else {
                continue;
            };
            ordinal += 1;
            if object.kind != Kind::Unsupported(48) {
                continue;
            }
            let group = &bytes[group_start..record.data.end];
            let reason = match ParameterCalculationGroup::parse(group) {
                Err(error) => format!("layout/expression: {error}"),
                Ok(calculation) => {
                    let base = document
                        .objects
                        .iter()
                        .find(|parent| parent.page == object.page)
                        .unwrap()
                        .id
                        - 1;
                    let units = calculation
                        .sources()
                        .into_iter()
                        .map(|id| {
                            document
                                .objects
                                .get(base + id as usize - 1)
                                .and_then(|parent| parent.kind.numeric_unit())
                        })
                        .collect::<Option<Vec<_>>>();
                    match units {
                        None => "parent numeric units unavailable".to_owned(),
                        Some(units) => match calculation.with_source_units(&units) {
                            Err(error) => format!("units: {error}"),
                            Ok(_) => "valid math and source units; inspect decoder safety gate"
                                .to_owned(),
                        },
                    }
                }
            };
            let entry = counts
                .entry(reason)
                .or_insert_with(|| (0, format!("{} object {}", path.display(), object.id)));
            entry.0 += 1;
        }
    }
    let mut results = counts.into_iter().collect::<Vec<_>>();
    results.sort_by_key(|(_, (count, _))| std::cmp::Reverse(*count));
    for (reason, (count, example)) in results {
        println!("{count}: {reason}\n  {example}");
    }
}
