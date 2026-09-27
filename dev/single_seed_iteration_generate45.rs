//! Native single-seed controls use the existing schema and public-API geometry.
//! The raw parent insertion bypasses only the old API minimum seed count.
use sketchpad_rs::document::{Document, IterationDepth, Kind, Point, records};
fn rec(b: &mut Vec<u8>, k: u32, p: &[u8]) {
    b.extend((p.len() as u32).to_le_bytes());
    b.extend(k.to_le_bytes());
    b.extend(p);
    if p.len() % 2 != 0 {
        b.push(0)
    }
}
fn main() {
    for state in [
        "derived",
        "geometric",
        "branches",
        "frozen",
        "live",
        "host_frozen",
    ] {
        if std::env::var("ONLY_STATE").is_ok_and(|x| x != state) {
            continue;
        }
        let mut d = Document::new();
        let a = d.add(Kind::Free(Point { x: 140., y: 250. }));
        let b = d.add(Kind::Free(Point { x: 240., y: 250. }));
        let u = d.add(Kind::Free(Point { x: 80., y: 80. }));
        let v = d.add(Kind::Free(Point { x: 120., y: 80. }));
        let host = d.add(Kind::Segment(a, b));
        let seed = if std::env::var_os("FREE_SEED").is_some() {
            d.add(Kind::Free(Point { x: 165., y: 250. }))
        } else if state == "derived" {
            d.add(Kind::Midpoint(a, b))
        } else {
            d.add(Kind::PointOnSegment(host, 0.25))
        };
        let early_segment = if matches!(state, "frozen" | "live" | "host_frozen") {
            d.add(Kind::Segment(seed, b))
        } else {
            0
        };
        let m = if early_segment != 0 {
            d.add(Kind::SegmentMidpoint(early_segment))
        } else {
            seed
        };
        let n = d.add(Kind::VectorTranslatedPoint(m, u, v));
        let q = if state == "branches" {
            d.add(Kind::VectorTranslatedPoint(seed, v, u))
        } else {
            n
        };
        let s = if early_segment != 0 {
            early_segment
        } else {
            d.add(Kind::Segment(seed, b))
        };
        let last_object = s.max(n).max(q);
        d.selected
            .extend((1..=last_object).filter(|id| ![seed, n, s, b].contains(id)));
        d.set_selected_visibility(false);
        d.selected.clear();
        let mut bytes = d.encode().unwrap();
        let rs = records(&bytes).unwrap();
        let pre = rs.iter().find(|r| r.kind == 0x3e8).unwrap();
        let trailer = rs.iter().find(|r| r.kind == 0x3e9).unwrap().start;
        let last = rs
            .iter()
            .filter(|r| r.kind == 0x7d7 && r.start < trailer)
            .map(|r| r.data.end)
            .max()
            .unwrap();
        let insert = rs
            .iter()
            .find(|r| matches!(r.kind, 0x2328 | 0x232d) && r.start >= last && r.start < trailer)
            .map(|r| r.start)
            .unwrap_or(trailer);
        let parent = last_object + 1;
        bytes[pre.data.start + 24..pre.data.start + 28].copy_from_slice(
            &((last_object + if state == "host_frozen" { 2 } else { 3 }) as u32).to_le_bytes(),
        );
        let branches = if state == "branches" {
            vec![n, q]
        } else {
            vec![n]
        };
        let mut carried = vec![n, s];
        if state == "branches" {
            carried.push(q);
        }
        if state == "live" {
            carried.insert(0, m)
        };
        if state == "host_frozen" {
            carried = vec![m, n];
        }
        if std::env::var_os("SORT_CARRIED").is_some() {
            carried.sort_unstable();
        }
        if std::env::var_os("VERIFY_API").is_some() {
            let mut api = d.clone();
            api.add_mapped_point_iteration_with_bindings(
                &[seed],
                &branches.iter().map(|id| vec![*id]).collect::<Vec<_>>(),
                &carried,
                &if state == "host_frozen" {
                    vec![n]
                } else {
                    vec![s, n]
                },
                IterationDepth::Fixed(3),
            )
            .unwrap();
            api.verify_roundtrip().unwrap();
            let encoded = api.encode().unwrap();
            assert_eq!(
                Document::from_bytes(&encoded, None)
                    .unwrap()
                    .encode()
                    .unwrap(),
                encoded
            );
            std::fs::write(format!("/tmp/single_seed45_api_{state}.gsp"), encoded).unwrap();
        }
        let ids = std::iter::once(seed)
            .chain(branches.iter().copied())
            .chain(carried.iter().copied())
            .collect::<Vec<_>>();
        let mut added = vec![];
        let mut h = *include_bytes!("../assets/mapped_point_iteration_header.bin");
        h[..2].copy_from_slice(&76u16.to_le_bytes());
        rec(&mut added, 0x7d0, &h);
        let mut refs = (ids.len() as u32).to_le_bytes().to_vec();
        for id in &ids {
            refs.extend((*id as u32).to_le_bytes())
        }
        rec(&mut added, 0x7d2, &refs);
        let mut def = include_bytes!("../assets/mapped_point_iteration_definition.bin").to_vec();
        def[6..8].copy_from_slice(&1u16.to_le_bytes());
        def[8..12].copy_from_slice(&(branches.len() as u32).to_le_bytes());
        def[16..20].copy_from_slice(&3u32.to_le_bytes());
        rec(&mut added, 0x90a, &def);
        rec(&mut added, 0x7d7, &[]);
        for (id, t) in [
            (
                s,
                include_bytes!("../assets/branch_polygon_iteration_segment_binding.bin").as_slice(),
            ),
            (
                n,
                include_bytes!("../assets/point_iteration_binding_group.bin").as_slice(),
            ),
        ] {
            if state == "host_frozen" && id == s {
                continue;
            }
            let mut g = t.to_vec();
            g[48..52].copy_from_slice(&(id as u32).to_le_bytes());
            g[52..56].copy_from_slice(&(parent as u32).to_le_bytes());
            added.extend(g)
        }
        bytes.splice(insert..insert, added);
        assert_eq!(
            Document::from_bytes(&bytes, None)
                .unwrap()
                .encode()
                .unwrap(),
            bytes
        );
        std::fs::write(
            format!(
                "{}_{state}.gsp",
                std::env::args()
                    .nth(1)
                    .unwrap_or("/tmp/single_seed45".into())
            ),
            bytes,
        )
        .unwrap();
        println!(
            "{state}: seed{seed} parent{parent} branches{branches:?} carried{carried:?} segment{s} image{n}"
        );
    }
}
