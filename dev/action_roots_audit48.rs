//! Checkpoint48 drawing-only code62 audit. Counts raw subtypes, strict parser failures and ready parents.
use sketchpad_rs::{
    action_records::*,
    document::{Document, Kind, records},
};
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
    } else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("gsp")) {
        v.push(p.into())
    }
}
fn n(b: &[u8], a: usize) -> usize {
    u32::from_le_bytes(b[a..a + 4].try_into().unwrap()) as usize
}
fn s(b: &[u8], a: usize) -> u16 {
    u16::from_le_bytes(b[a..a + 2].try_into().unwrap())
}
fn main() {
    let mut files = vec![];
    collect(Path::new("../gsp-rs"), &mut files);
    files.sort();
    let mut cats: BTreeMap<String, (usize, BTreeSet<PathBuf>, Vec<String>)> = BTreeMap::new();
    let mut total = 0;
    let mut visibility_schema: BTreeMap<(u16, u16, bool, bool, bool), usize> = BTreeMap::new();
    let mut movement: BTreeMap<String, usize> = BTreeMap::new();
    let mut summary: BTreeMap<(u16, u16, u16, u16, bool), (usize, BTreeSet<PathBuf>)> =
        BTreeMap::new();
    for p in files {
        let b = fs::read(&p).unwrap();
        let Ok(doc) = Document::from_bytes(&b, None) else {
            continue;
        };
        let rs = records(&b).unwrap();
        for o in &doc.objects {
            if !matches!(o.kind, Kind::Unsupported(62)) {
                continue;
            }
            total += 1;
            let at = o.header_offset.unwrap();
            let start = rs
                .iter()
                .position(|r| r.kind == 0x7d0 && r.data.start == at)
                .unwrap();
            let end = (start..rs.len()).find(|&i| rs[i].kind == 0x7d7).unwrap();
            let g = &rs[start..=end];
            let bytes = &b[g[0].start..g.last().unwrap().data.end];
            let action = g
                .iter()
                .find(|r| r.kind == 0x906)
                .map(|r| &b[r.data.clone()]);
            let raw = action
                .map(|a| {
                    if a.len() >= 20 {
                        format!("{}:{}:{} len{}", s(a, 12), s(a, 14), n(a, 16), a.len())
                    } else {
                        format!("short{}", a.len())
                    }
                })
                .unwrap_or("missing".into());
            let refs = g
                .iter()
                .find(|r| r.kind == 0x7d2)
                .map(|r| &b[r.data.clone()]);
            let base = doc.objects.iter().find(|x| x.page == o.page).unwrap().id - 1;
            let ids: Vec<_> = refs
                .filter(|r| r.len() >= 4)
                .map(|r| {
                    r[4..]
                        .chunks_exact(4)
                        .map(|q| {
                            let raw = n(q, 0);
                            if raw == 0 { 0 } else { raw + base }
                        })
                        .collect()
                })
                .unwrap_or_default();
            let writable = !ids.is_empty()
                && ids.iter().all(|&id| {
                    id > 0
                        && id < o.id
                        && doc
                            .objects
                            .get(id - 1)
                            .is_some_and(|parent| parent.page == o.page && !parent.deleted)
                });
            let ready = writable
                && ids
                    .iter()
                    .all(|&id| !matches!(doc.objects[id - 1].kind, Kind::Unsupported(_)));
            if let Some(action) = action.filter(|a| a.len() >= 20) {
                let entry = summary
                    .entry((
                        s(action, 12),
                        s(action, 14),
                        s(action, 16),
                        s(action, 18),
                        ready,
                    ))
                    .or_default();
                entry.0 += 1;
                entry.1.insert(p.clone());
            }
            if let Some(action_record) = g.iter().find(|r| r.kind == 0x906) {
                let action = &b[action_record.data.clone()];
                if action.len() == 24 && matches!(s(action, 12), 0 | 1) {
                    // Audit only: substitute the known option7 solely to isolate
                    // optional-layout compatibility from unresolved behavior.
                    let mut structural = bytes.to_vec();
                    let offset = action_record.data.start - g[0].start + 14;
                    structural[offset..offset + 2].copy_from_slice(&7_u16.to_le_bytes());
                    let accepts = HideShowButtonGroup::parse(&structural).is_ok();
                    *visibility_schema
                        .entry((s(action, 12), s(action, 14), ready, writable, accepts))
                        .or_default() += 1;
                }
            }
            if action.is_some_and(|a| a.len() >= 16 && s(a, 12) == 3) && ready {
                let mut names: BTreeSet<String> = BTreeSet::new();
                for pair in ids.chunks_exact(2) {
                    let kind = &doc.objects[pair[0] - 1].kind;
                    names.insert(
                        format!("{kind:?}")
                            .split(|c: char| !c.is_alphanumeric())
                            .next()
                            .unwrap()
                            .into(),
                    );
                }
                *movement
                    .entry(format!("pairs={} source-kinds={:?}", ids.len() / 2, names))
                    .or_default() += 1;
            }
            let parsers = [
                ("hide", HideShowButtonGroup::parse(bytes).map(|_| ())),
                ("animate", AnimationButtonGroup::parse(bytes).map(|_| ())),
                ("move", MovementButtonGroup::parse(bytes).map(|_| ())),
                ("scroll", ScrollButtonGroup::parse(bytes).map(|_| ())),
                ("present", PresentationButtonGroup::parse(bytes).map(|_| ())),
            ];
            let family = parsers
                .iter()
                .find(|(_, r)| r.is_ok())
                .map(|(n, _)| *n)
                .unwrap_or("NONE");
            let err = match action.filter(|a| a.len() >= 14).map(|a| s(a, 12)) {
                Some(0 | 1) => &parsers[0].1,
                Some(2) => &parsers[1].1,
                Some(3) => &parsers[2].1,
                Some(4) => &parsers[3].1,
                Some(7) => &parsers[4].1,
                _ => &parsers[0].1,
            };
            let key = format!(
                "{} ready={} parser={} {:?}",
                raw,
                ready,
                family,
                err.as_ref().err()
            );
            let c = cats.entry(key).or_default();
            c.0 += 1;
            c.1.insert(p.clone());
            if c.2.len() < 3 {
                let kinds: Vec<_> = ids
                    .iter()
                    .map(|id| {
                        doc.objects
                            .get(id - 1)
                            .map(|x| format!("{:?}", x.kind))
                            .unwrap_or("MISSING".into())
                    })
                    .collect();
                c.2.push(format!(
                    "{} object={} page={} refs={:?} kinds={:?} records={:?}",
                    p.display(),
                    o.id,
                    o.page,
                    ids,
                    kinds,
                    g.iter()
                        .map(|r| (format!("{:04x}", r.kind), r.data.len()))
                        .collect::<Vec<_>>()
                ));
            }
        }
    }
    println!("TOTAL {}", total);
    for (key, (count, files)) in &summary {
        println!(
            "SUMMARY action={} option={} word16={} word18={} ready={} count={} files={}",
            key.0,
            key.1,
            key.2,
            key.3,
            key.4,
            count,
            files.len()
        );
    }
    for (key, count) in &visibility_schema {
        println!(
            "VISIBILITY_SCHEMA action={} option={} ready={} writable={} knownlayout={} count={}",
            key.0, key.1, key.2, key.3, key.4, count
        );
    }
    println!("READY MOVEMENT SOURCE SUMMARY {:?}", movement);
    let mut rows: Vec<_> = cats.into_iter().collect();
    rows.sort_by_key(|(_, v)| std::cmp::Reverse(v.0));
    for (k, (n, f, e)) in rows {
        println!("{} files={} {}", n, f.len(), k);
        for line in e {
            println!("  {}", line)
        }
    }
}
