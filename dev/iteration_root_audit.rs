//! Drawing-only audit of unsupported iteration roots and related transforms/actions.
//! Compile with the current library (no parser or production mutations):
//! rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=TARGET_RLIB dev/iteration_root_audit.rs -o /tmp/iteration_root_audit
//! /tmp/iteration_root_audit ../gsp-rs
//! `ready` means decoded direct predecessors, not proven iteration semantics.
//! Paired HTM exports omit native iteration objects and may renumber predecessors.
use sketchpad_rs::document::{Document, Kind, Object, records};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

fn collect(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|kind| !kind.is_symlink()) {
                    collect(&entry.path(), files);
                }
            }
        }
    } else if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gsp"))
    {
        files.push(path.to_owned());
    }
}
fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn kind_name(object: Option<&Object>) -> String {
    object
        .map(|object| {
            format!("{:?}", object.kind)
                .split(|c: char| !c.is_alphanumeric())
                .next()
                .unwrap_or("?")
                .to_owned()
        })
        .unwrap_or_else(|| "MISSING".into())
}
#[derive(Default)]
struct Category {
    count: usize,
    files: BTreeSet<PathBuf>,
    examples: Vec<String>,
}
fn main() {
    let root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../gsp-rs".into());
    let mut files = Vec::new();
    collect(Path::new(&root), &mut files);
    files.sort();
    let mut categories = BTreeMap::<String, Category>::new();
    let mut bindings = BTreeMap::<String, usize>::new();
    for path in files {
        let Ok(bytes) = fs::read(&path) else { continue };
        let Ok(document) = Document::from_bytes(&bytes, None) else {
            continue;
        };
        let Ok(records) = records(&bytes) else {
            continue;
        };
        let mut ready_roots = BTreeSet::new();
        // Document.objects contains drawing objects; isolated custom-tool prototypes
        // are never traversed here. Header offsets identify their actual raw group.
        for object in &document.objects {
            let Kind::Unsupported(code @ (16 | 62 | 76 | 77 | 89)) = object.kind else {
                continue;
            };
            let Some(offset) = object.header_offset else {
                continue;
            };
            let Some(start) = records
                .iter()
                .position(|r| r.kind == 0x07d0 && r.data.start == offset)
            else {
                continue;
            };
            let Some(end) = (start..records.len()).find(|&i| records[i].kind == 0x07d7) else {
                continue;
            };
            let group = &records[start..=end];
            let header = &bytes[records[start].data.clone()];
            if header.len() != 28 {
                continue;
            }
            let base = document
                .objects
                .iter()
                .position(|o| o.page == object.page)
                .unwrap();
            let references = group
                .iter()
                .find(|r| r.kind == 0x07d2)
                .map(|r| &bytes[r.data.clone()])
                .filter(|data| data.len() >= 4 && data.len() % 4 == 0)
                .filter(|data| u32_at(data, 0) as usize == (data.len() - 4) / 4)
                .map(|data| {
                    data[4..]
                        .chunks_exact(4)
                        .map(|data| u32_at(data, 0) as usize)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let parents = references
                .iter()
                .map(|id| {
                    id.checked_sub(1)
                        .and_then(|id| document.objects.get(base + id))
                        .filter(|o| o.page == object.page)
                })
                .collect::<Vec<_>>();
            let ready = !parents.is_empty()
                && parents.iter().all(|parent| {
                    parent.is_some_and(|o| !o.deleted && !matches!(o.kind, Kind::Unsupported(_)))
                });
            let definition = group
                .iter()
                .find(|r| r.kind == 0x090a)
                .map(|r| &bytes[r.data.clone()]);
            if matches!(code, 76 | 89) && ready {
                ready_roots.insert(object.id);
            }
            if code == 77 {
                if parents
                    .get(1)
                    .and_then(|o| *o)
                    .is_some_and(|o| ready_roots.contains(&o.id))
                {
                    *bindings
                        .entry("direct-ready iteration root".into())
                        .or_default() += 1;
                }
                for parent in parents.iter().flatten() {
                    if let Kind::Unsupported(code) = parent.kind {
                        *bindings
                            .entry(format!("unsupported parent code{code}"))
                            .or_default() += 1;
                    }
                }
            }
            let key = match code {
                76 | 89 => {
                    let shape = definition
                        .filter(|data| data.len() == 24)
                        .map(|data| {
                            format!(
                                "prefix{} final{} seeds{} branches{}",
                                u32_at(data, 0),
                                u16_at(data, 4),
                                u16_at(data, 6),
                                u32_at(data, 8)
                            )
                        })
                        .unwrap_or_else(|| {
                            format!("other definition {:?}", definition.map(<[u8]>::len))
                        });
                    format!("code{code} ready{ready} {shape}")
                }
                62 => {
                    let action = group
                        .iter()
                        .find(|r| r.kind == 0x0906)
                        .map(|r| &bytes[r.data.clone()]);
                    let mode = action
                        .filter(|data| data.len() >= 16)
                        .map(|data| format!("mode{} option{}", u16_at(data, 12), u16_at(data, 14)))
                        .unwrap_or_else(|| "missing action".into());
                    format!(
                        "code62 ready{ready} {mode} layout{:x?}",
                        group.iter().map(|r| r.kind).collect::<Vec<_>>()
                    )
                }
                _ => format!(
                    "code{code} ready{ready} class{} subtype{}",
                    u16_at(header, 6),
                    u16_at(header, 8)
                ),
            };
            let category = categories.entry(key).or_default();
            category.count += 1;
            category.files.insert(path.clone());
            if category.examples.len() < 3 {
                let parent_names = parents
                    .iter()
                    .map(|parent| kind_name(*parent))
                    .collect::<Vec<_>>();
                category.examples.push(format!("{} global{} page{} local{} refs{references:?} kinds{parent_names:?} definition{:02x?}",
                    path.display(), object.id, object.page, object.id - base, definition));
            }
        }
    }
    for (key, category) in categories {
        println!(
            "COUNT {} files{} {key}",
            category.count,
            category.files.len()
        );
        for example in category.examples {
            println!("EX {example}");
        }
    }
    for (key, count) in bindings {
        println!("BIND {count} {key}");
    }
}
