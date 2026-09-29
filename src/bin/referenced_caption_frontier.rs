use sketchpad_rs::document::{
    Document, Kind, Object, records,
    referenced_text::{ReferenceRole, ReferencedText},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

fn collect(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap().flatten() {
            if !entry.file_type().unwrap().is_symlink() {
                collect(&entry.path(), files);
            }
        }
    } else if path
        .extension()
        .is_some_and(|value| value.eq_ignore_ascii_case("gsp"))
    {
        files.push(path.to_owned());
    }
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn kind_name(kind: &Kind) -> String {
    format!("{kind:?}")
        .split(['(', '{', ' '])
        .next()
        .unwrap_or("?")
        .to_owned()
}

fn supported_source(objects: &[Object], object: &Object, role: ReferenceRole) -> bool {
    if role == ReferenceRole::SignedValue {
        return object.kind.numeric_unit().is_some();
    }
    if role != ReferenceRole::Value {
        return match &object.kind {
            Kind::Parameter(group) => role == ReferenceRole::Expression || !group.name().is_empty(),
            Kind::SegmentLength(segment) if role != ReferenceRole::Name => objects
                .get(segment.saturating_sub(1))
                .is_some_and(|source| {
                    !source.deleted
                        && source.page == object.page
                        && matches!(source.kind, Kind::Segment(..))
                }),
            Kind::ParameterCalculation(..) | Kind::PointRatio(..) | Kind::SegmentLength(..) => true,
            Kind::FunctionDefinition(..)
            | Kind::ReferencedFunction(..)
            | Kind::DerivativeFunction(..)
            | Kind::PlainText { .. }
            | Kind::ReferencedText { .. } => true,
            _ => false,
        };
    }
    object.kind.numeric_unit().is_some()
        || object.kind.is_action_button()
        || object.kind.is_point()
        || object.label.is_some()
        || matches!(
            object.kind,
            Kind::PlainText { .. }
                | Kind::ReferencedText { .. }
                | Kind::FunctionDefinition(..)
                | Kind::ReferencedFunction(..)
                | Kind::DerivativeFunction(..)
        )
}

fn main() {
    let mut files = Vec::new();
    collect(Path::new("../gsp-rs/tests"), &mut files);
    let mut reasons = BTreeMap::<String, (usize, BTreeSet<PathBuf>, Vec<String>)>::new();
    for path in files {
        let bytes = fs::read(&path).unwrap();
        let Ok(document) = Document::from_bytes(&bytes, None) else {
            continue;
        };
        let Ok(all) = records(&bytes) else { continue };
        for object in &document.objects {
            if !matches!(object.kind, Kind::Unsupported(73)) {
                continue;
            }
            let Some(at) = object.header_offset else {
                continue;
            };
            let Some(first) = all
                .iter()
                .position(|record| record.kind == 0x07d0 && record.data.start == at)
            else {
                continue;
            };
            let Some(last) = (first..all.len()).find(|&index| all[index].kind == 0x07d7) else {
                continue;
            };
            let group = &all[first..=last];
            let base = document
                .objects
                .iter()
                .position(|candidate| candidate.page == object.page)
                .unwrap();
            let parents = group
                .iter()
                .find(|record| record.kind == 0x07d2)
                .map(|record| {
                    bytes[record.data.start + 4..record.data.end]
                        .chunks_exact(4)
                        .map(|chunk| base + u32_at(chunk, 0) as usize)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let Some(rich) = group.iter().find(|record| record.kind == 0x08fc) else {
                continue;
            };
            let payload = &bytes[rich.data.clone()];
            let parsed = match ReferencedText::from_rich(payload, parents.len()) {
                Ok(parsed) => parsed,
                Err(error) => {
                    let entry = reasons.entry(format!("PARSE {error}")).or_default();
                    entry.0 += 1;
                    entry.1.insert(path.clone());
                    continue;
                }
            };
            let mut structural = false;
            for parent in &parents {
                let reason = match parent.checked_sub(1).and_then(|index| document.objects.get(index)) {
                    None => Some("MISSING"),
                    Some(_) if *parent >= object.id => Some("FORWARD"),
                    Some(source) if source.page != object.page => Some("CROSS_PAGE"),
                    Some(source) if source.deleted => Some("DELETED"),
                    Some(source) if matches!(source.kind, Kind::Unsupported(_)) => Some("UNSUPPORTED_PARENT"),
                    Some(_) => None,
                };
                if let Some(reason) = reason {
                    structural = true;
                    let source = parent
                        .checked_sub(1)
                        .and_then(|index| document.objects.get(index))
                        .map(|source| kind_name(&source.kind))
                        .unwrap_or_else(|| "none".into());
                    let entry = reasons.entry(format!("{reason} {source}")).or_default();
                    entry.0 += 1;
                    entry.1.insert(path.clone());
                    if entry.2.len() < 3 {
                        entry.2.push(format!("id={} parent={parent}", object.id));
                    }
                }
            }
            if structural {
                continue;
            }
            for (slot, role) in parsed.referenced_roles() {
                let Some(parent) = parents.get(slot - 1) else {
                    continue;
                };
                let source = &document.objects[*parent - 1];
                if supported_source(&document.objects, source, role) {
                    continue;
                }
                let key = format!("ROLE {role:?} SOURCE {}", kind_name(&source.kind));
                let entry = reasons.entry(key).or_default();
                entry.0 += 1;
                entry.1.insert(path.clone());
                if entry.2.len() < 3 {
                    entry.2.push(format!("id={} parent={parent}", object.id));
                }
            }
        }
    }
    for (reason, (occurrences, files, examples)) in reasons {
        println!("{occurrences:4} {:3} {reason} {examples:?}", files.len());
    }
}
