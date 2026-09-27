//! Native proof controls for scalar simultaneous mappings, including calculated seeds.
//! Uses a validated public-API drawing and the existing native mapped iteration groups.
//! cargo build --lib
//! rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/numeric_mapped43_generate.rs -o /tmp/numeric_mapped43_generate
//! /tmp/numeric_mapped43_generate fibonacci|calculated|single
//! Outputs /tmp/numeric_mapped43_STATE.gsp. Numeric mapping decoding was unsupported
//! at the evidence baseline; byte-identical reader preservation is checked here.
use sketchpad_rs::document::{Document, Kind, Point};
fn main() {
    let state = std::env::args().nth(1).unwrap_or("fibonacci".into());
    let mut d = Document::new();
    let a = if state == "calculated" {
        d.add_calculation_expression("1", (10, 30)).unwrap()
    } else {
        d.add_named_parameter("a", 1., (10, 30)).unwrap()
    };
    let b = if state == "calculated" {
        d.add_calculation_expression("1", (10, 60)).unwrap()
    } else {
        d.add_named_parameter("b", 1., (10, 60)).unwrap()
    };
    let c = d.add_multi_calculation(&[a, b], "v1+v2", (10, 90)).unwrap();
    let f = d
        .add_default_coordinate_system(Point { x: 160., y: 360. })
        .unwrap();
    let p = d.add(Kind::NumericCoordinatePoint(a, b, f));
    let q = d.add(Kind::NumericCoordinatePoint(b, c, f));
    let s = d.add(Kind::Segment(p, q));
    let mut bytes = d.encode().unwrap();
    let rs = sketchpad_rs::document::records(&bytes).unwrap();
    let pre = rs.iter().find(|r| r.kind == 0x3e8).unwrap();
    let trailer = rs.iter().find(|r| r.kind == 0x3e9).unwrap().start;
    let last_end = rs
        .iter()
        .filter(|r| r.kind == 0x7d7 && r.start < trailer)
        .map(|r| r.data.end)
        .max()
        .unwrap_or(0);
    let insert_at = rs
        .iter()
        .find(|r| matches!(r.kind, 0x2328 | 0x232d) && r.start >= last_end && r.start < trailer)
        .map(|r| r.start)
        .unwrap_or(trailer);
    bytes[pre.data.start + 24..pre.data.start + 28].copy_from_slice(&15u32.to_le_bytes());
    fn rec(b: &mut Vec<u8>, k: u32, p: &[u8]) {
        b.extend((p.len() as u32).to_le_bytes());
        b.extend(k.to_le_bytes());
        b.extend(p);
        if p.len() % 2 != 0 {
            b.push(0)
        }
    }
    let mut added = Vec::new();
    let mut header = *include_bytes!("../assets/mapped_point_iteration_header.bin");
    header[..2].copy_from_slice(&76u16.to_le_bytes());
    rec(&mut added, 0x7d0, &header);
    let ids = if state == "single" {
        vec![a, c, c, p, q, s]
    } else {
        vec![a, b, b, c, c, p, q, s]
    };
    let mut refs = (ids.len() as u32).to_le_bytes().to_vec();
    for id in ids {
        refs.extend((id as u32).to_le_bytes())
    }
    rec(&mut added, 0x7d2, &refs);
    let mut def = include_bytes!("../assets/mapped_point_iteration_definition.bin").to_vec();
    def[6..8].copy_from_slice(&(if state == "single" { 1u16 } else { 2u16 }).to_le_bytes());
    def[8..12].copy_from_slice(&1u32.to_le_bytes());
    def[16..20].copy_from_slice(&3u32.to_le_bytes());
    rec(&mut added, 0x90a, &def);
    rec(&mut added, 0x7d7, &[]);
    for (id, template) in [
        (
            p,
            include_bytes!("../assets/point_iteration_binding_group.bin").as_slice(),
        ),
        (
            s,
            include_bytes!("../assets/branch_polygon_iteration_segment_binding.bin").as_slice(),
        ),
    ] {
        let mut g = template.to_vec();
        g[48..52].copy_from_slice(&(id as u32).to_le_bytes());
        g[52..56].copy_from_slice(&13u32.to_le_bytes());
        added.extend(g)
    }
    bytes.splice(insert_at..insert_at, added);
    let parsed = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(parsed.objects.len(), 15);
    assert_eq!(parsed.encode().unwrap(), bytes);
    std::fs::write(format!("/tmp/numeric_mapped43_{state}.gsp"), bytes).unwrap();
    println!("a {a} b {b} c {c} f {f} p {p} q {q} s {s}");
}
