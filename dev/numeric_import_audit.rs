//! Standalone numeric prerequisite audit. Compile against the current library:
//! rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=TARGET_RLIB dev/numeric_import_audit.rs -o /tmp/numeric_import_audit
//! /tmp/numeric_import_audit ../gsp-rs
use sketchpad_rs::document::{Document, Kind, records};
use sketchpad_rs::table_records::{CalculationSourceKind, NumericUnit, ParameterCalculationGroup};
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
fn source_unit(kind: &Kind, role: CalculationSourceKind) -> Option<NumericUnit> {
    match role {
        CalculationSourceKind::Numeric => kind.numeric_unit(),
        CalculationSourceKind::Function if matches!(kind, Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)) => Some(NumericUnit::Scalar),
        _ => None,
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
    let mut missing_parents: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut audited = 0;

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
        let mut tool_depth = 0usize;
        for record in &rs {
            if record.kind == 0x03ea {
                tool_depth += 1;
                continue;
            }
            if record.kind == 0x03eb && tool_depth > 0 {
                tool_depth -= 1;
                continue;
            }
            if tool_depth > 0 {
                continue;
            }
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
            audited += 1;
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
                    for (id, role) in calculation.sources().into_iter().zip(calculation.source_kinds().unwrap()) {
                        if let Some(parent) = document.objects.get(base + id as usize - 1)
                            && source_unit(&parent.kind, role).is_none()
                        {
                            let reason = match &parent.kind {
                                Kind::Unsupported(code) => format!("unsupported code {code}"),
                                _ => "decoded parent has incompatible numeric/function role".to_owned(),
                            };
                            let entry = missing_parents.entry(reason).or_insert_with(|| (
                                0, format!("{} object {} parent {}", path.display(), object.id, parent.id),
                            ));
                            entry.0 += 1;
                        }
                    }
                    let units = calculation
                        .sources()
                        .into_iter().zip(calculation.source_kinds().unwrap())
                        .map(|(id, role)| {
                            document
                                .objects
                                .get(base + id as usize - 1)
                                .and_then(|parent| source_unit(&parent.kind, role))
                        })
                        .collect::<Option<Vec<_>>>();
                    match units {
                        None => "parent numeric/function source type unavailable".to_owned(),
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
    println!("Unsupported drawing calculations audited: {audited}");
    let mut results = counts.into_iter().collect::<Vec<_>>();
    results.sort_by_key(|(_, (count, _))| std::cmp::Reverse(*count));
    for (reason, (count, example)) in results {
        println!("{count}: {reason}\n  {example}");
    }
    let mut parents = missing_parents.into_iter().collect::<Vec<_>>();
    parents.sort_by_key(|(_, (count, _))| std::cmp::Reverse(*count));
    println!("Unavailable numeric parent references (a calculation can contribute multiple):");
    for (reason, (count, example)) in parents {
        println!("{count}: {reason}\n  {example}");
    }
}
