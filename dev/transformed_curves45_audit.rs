//! Read-only drawing-only audit of unsupported Native transforms and downstream references.
use sketchpad_rs::document::{Document, Kind, records};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
fn collect(p: &Path, v: &mut Vec<PathBuf>) {
    if p.is_dir() {
        for e in fs::read_dir(p).unwrap().flatten() {
            if !e.file_type().unwrap().is_symlink() {
                collect(&e.path(), v)
            }
        }
    } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("gsp")) {
        v.push(p.into())
    }
}
fn n(b: &[u8], i: usize) -> usize {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap()) as usize
}
fn main() {
    let mut files = vec![];
    let inputs: Vec<_> = std::env::args().skip(1).collect();
    if inputs.is_empty() {
        collect(Path::new("../gsp-rs"), &mut files);
    } else {
        for path in inputs {
            collect(Path::new(&path), &mut files);
        }
    }
    files.sort();
    let mut cats: BTreeMap<
        String,
        (
            usize,
            usize,
            BTreeSet<PathBuf>,
            BTreeSet<(PathBuf, usize)>,
            Vec<String>,
            BTreeSet<(PathBuf, usize)>,
        ),
    > = BTreeMap::new();
    let mut families: BTreeMap<
        String,
        (usize, usize, BTreeSet<PathBuf>, BTreeSet<(PathBuf, usize)>),
    > = BTreeMap::new();
    for path in files {
        let b = fs::read(&path).unwrap();
        let Ok(doc) = Document::from_bytes(&b, None) else {
            continue;
        };
        let Ok(rs) = records(&b) else { continue };
        let starts: BTreeMap<_, _> = rs
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == 0x7d0)
            .map(|(i, r)| (r.data.start, i))
            .collect();
        let mut refs = BTreeMap::new();
        let mut roots = vec![];
        for o in &doc.objects {
            let Some(at) = o.header_offset else { continue };
            let Some(&start) = starts.get(&at) else {
                continue;
            };
            let Some(end) = (start..rs.len()).find(|&i| rs[i].kind == 0x7d7) else {
                continue;
            };
            let h = &b[rs[start].data.clone()];
            if h.len() < 10 {
                continue;
            }
            let group = &rs[start..=end];
            let base = doc.objects.iter().position(|x| x.page == o.page).unwrap();
            let parents: Vec<_> = group
                .iter()
                .find(|r| r.kind == 0x7d2)
                .map(|r| &b[r.data.clone()])
                .filter(|d| d.len() >= 4 && d.len() % 4 == 0 && n(d, 0) == (d.len() - 4) / 4)
                .map(|d| d[4..].chunks_exact(4).map(|d| base + n(d, 0)).collect())
                .unwrap_or_default();
            refs.insert(o.id, parents.clone());
            let code = n(h, 0) & 65535;
            if !matches!(
                code,
                16 | 17 | 18 | 19 | 20 | 21 | 24 | 27 | 28 | 29 | 30 | 32 | 33 | 34
            ) || !matches!(o.kind, Kind::Unsupported(_))
            {
                continue;
            }
            let ready = !parents.is_empty()
                && parents.iter().all(|id| {
                    doc.objects.get(id.saturating_sub(1)).is_some_and(|x| {
                        x.page == o.page && !x.deleted && !matches!(x.kind, Kind::Unsupported(_))
                    })
                });
            let key = format!(
                "code={code} class={} subtype={}",
                u16::from_le_bytes(h[6..8].try_into().unwrap()),
                u16::from_le_bytes(h[8..10].try_into().unwrap())
            );
            let parent_names: Vec<_> = parents
                .iter()
                .map(|id| {
                    doc.objects
                        .get(id.saturating_sub(1))
                        .map(|x| format!("{}:{:?}", id, x.kind))
                        .unwrap_or_else(|| format!("{id}:INVALID"))
                })
                .collect();
            roots.push((
                o.id,
                key,
                ready,
                format!(
                    "{} id={} raw_header={h:02x?} parents={parent_names:?} records={:?}",
                    path.display(),
                    o.id,
                    group
                        .iter()
                        .map(|r| (r.kind, r.data.len()))
                        .collect::<Vec<_>>()
                ),
            ))
        }
        for (id, key, ready, example) in roots {
            let mut descendants = BTreeSet::from([id]);
            loop {
                let old = descendants.len();
                for (child, parents) in &refs {
                    if parents.iter().any(|p| descendants.contains(p)) {
                        descendants.insert(*child);
                    }
                }
                if old == descendants.len() {
                    break;
                }
            }
            let family = if key.contains("class=12 ") {
                "Arc"
            } else if key.contains("class=13 ") {
                "ArcInterior"
            } else if key.contains("class=16 ") {
                "Image"
            } else if key.contains("class=4 ") {
                "Polygon"
            } else if key.contains("class=2 ") {
                "Straight"
            } else {
                "Other"
            };
            let family = families.entry(family.into()).or_default();
            family.0 += 1;
            family.1 += usize::from(ready);
            family.2.insert(path.clone());
            let entry = cats.entry(key).or_default();
            entry.0 += 1;
            entry.1 += usize::from(ready);
            entry.2.insert(path.clone());
            for child in descendants {
                if matches!(doc.objects[child - 1].kind, Kind::Unsupported(_)) {
                    entry.3.insert((path.clone(), child));
                    if ready {
                        entry.5.insert((path.clone(), child));
                        family.3.insert((path.clone(), child));
                    }
                }
            }
            if entry.4.len() < 4 {
                entry.4.push(format!("ready={ready} {example}"))
            }
        }
    }
    for (key, (total, ready, files, downstream, examples, ready_downstream)) in cats {
        println!(
            "{key} unsupported={total} ready={ready} files={} downstream_unsupported={} ready_root_downstream_unsupported={}",
            files.len(),
            downstream.len(),
            ready_downstream.len()
        );
        for example in examples {
            println!("EXAMPLE {example}");
        }
    }
    for (family, (roots, ready, files, descendants)) in families {
        println!(
            "FAMILY {family} roots={roots} ready={ready} files={} ready_root_downstream_unsupported={}",
            files.len(),
            descendants.len()
        );
    }
}
