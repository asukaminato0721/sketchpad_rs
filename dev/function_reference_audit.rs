use sketchpad_rs::document::{records, Document, Kind};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
fn collect(p: &Path, v: &mut Vec<PathBuf>) {
    if p.is_dir() {
        if let Ok(es) = fs::read_dir(p) {
            for e in es.flatten() {
                if e.file_type().is_ok_and(|t| !t.is_symlink()) {
                    collect(&e.path(), v)
                }
            }
        }
    } else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("gsp")) {
        v.push(p.to_owned())
    }
}
fn u32at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
fn main() {
    let mut files = vec![];
    collect(
        Path::new(&std::env::args().nth(1).unwrap_or("../gsp-rs".into())),
        &mut files,
    );
    files.sort();
    let mut counts = BTreeMap::<String, (usize, BTreeSet<PathBuf>)>::new();
    let mut examples = BTreeMap::<String, Vec<String>>::new();
    for p in files {
        let Ok(b) = fs::read(&p) else { continue };
        let Ok(d) = Document::from_bytes(&b, None) else {
            continue;
        };
        let Ok(rs) = records(&b) else { continue };
        for o in &d.objects {
            if !matches!(o.kind, Kind::Unsupported(_)) {
                continue;
            }
            let Some(off) = o.header_offset else { continue };
            let Some(a) = rs
                .iter()
                .position(|r| r.kind == 0x7d0 && r.data.start == off)
            else {
                continue;
            };
            let h = &b[rs[a].data.clone()];
            if h.len() != 28 || h[6..10] != [11, 0, 57, 0] {
                continue;
            }
            let e = (a..rs.len()).find(|&i| rs[i].kind == 0x7d7).unwrap();
            let Some(r) = rs[a..e].iter().find(|r| r.kind == 0x907) else {
                continue;
            };
            let q = &b[r.data.clone()];
            if q.len() < 92 {
                continue;
            }
            let words = q[92..]
                .chunks_exact(2)
                .map(|w| u16::from_le_bytes(w.try_into().unwrap()))
                .collect::<Vec<_>>();
            let refs = rs[a..e]
                .iter()
                .find(|r| matches!(r.kind, 0x7d2 | 0x7d3))
                .map(|r| {
                    b[r.data.clone()]
                        .chunks_exact(4)
                        .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
                        .skip(1)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let roles = words
                .iter()
                .filter(|&&w| w >= 0x6000)
                .map(|w| format!("{:x}", w >> 12))
                .collect::<BTreeSet<_>>();
            let code = u32at(h, 0) as u16;
            let key = format!(
                "code{code} refs{} roles{:?} tokens{} wrapper{}",
                refs.len(),
                roles,
                if words.len() > 64 { "over64" } else { "<=64" },
                q[16]
            );
            let en = counts.entry(key.clone()).or_default();
            en.0 += 1;
            en.1.insert(p.clone());
            let ex = examples.entry(key).or_default();
            if ex.len() < 8 {
                let pagebase = d.objects.iter().position(|x| x.page == o.page).unwrap_or(0);
                let parents = refs
                    .iter()
                    .map(|r| {
                        d.objects
                            .get(pagebase + *r as usize - 1)
                            .map(|x| format!("{r}:{:?}", x.kind))
                            .unwrap_or_else(|| format!("{r}:MISSING"))
                    })
                    .collect::<Vec<_>>();
                let htm = p.with_extension("htm");
                let ht = fs::read_to_string(&htm).ok().and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with(&format!("{{{}}} Function(", o.id - pagebase)))
                        .map(str::to_owned)
                });
                ex.push(format!("{} obj{} page{} header{:x?}\nrefs{refs:?} parents{parents:?}\nwrapper{:02x?}\nwords{words:04x?}\nHTM {}",p.display(),o.id,o.page,h,&q[..92],ht.unwrap_or("MISSING".into())));
            }
        }
    }
    for (k, (n, fs)) in &counts {
        println!("COUNT {n} files{} {k}", fs.len())
    }
    for (k, ex) in examples {
        println!("\nCATEGORY {k}");
        for e in ex {
            println!("{e}\n")
        }
    }
}
