//! Native geometric-measurement mapped-state controls. No production guard bypass.
//! cargo build --lib && rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/measurement_mapped44_generate.rs -o /tmp/measurement_mapped44_generate
use sketchpad_rs::document::{Document, IterationDepth, Kind, Point};
fn main() {
    let prefix = std::env::args().nth(1).expect("prefix");
    for state in ["angle", "length", "angle_edited"] {
        let mut d = Document::new();
        let a = d.add(Kind::Free(Point {
            x: 160.,
            y: if state == "angle_edited" { 160. } else { 200. },
        }));
        let p = d.add(Kind::Free(Point { x: 200., y: 280. }));
        let o = d.add(Kind::Free(Point { x: 80., y: 280. }));
        let e = d.add(Kind::Free(Point { x: 160., y: 280. }));
        let vs = d.add(Kind::Free(Point { x: 40., y: 400. }));
        let vt = d.add(Kind::Free(Point { x: 120., y: 400. }));
        let host = d.add(Kind::Segment(o, a));
        let numeric = d.add(if state == "length" {
            Kind::SegmentLength(host)
        } else {
            Kind::AngleMeasurement(a, o, e)
        });
        let ratio = if state == "length" {
            Some(
                d.add_parameter_calculation(numeric, "v/(4 cm)", (10, 100))
                    .unwrap(),
            )
        } else {
            None
        };
        let n = d.add(Kind::VectorTranslatedPoint(a, vs, vt));
        let s = d.add(Kind::Segment(o, p));
        let transformed = if let Some(r) = ratio {
            d.add_calculated_ratio_dilation(s, o, r).unwrap()
        } else {
            d.add_calculated_angle_rotation(s, o, numeric).unwrap()
        };
        let mut api = d.clone();
        let parent = d.add(Kind::MappedPointIteration {
            seeds: vec![a, p],
            branches: vec![vec![n, p]],
            carried: vec![s, n],
            depth: IterationDepth::Fixed(3),
            final_only: false,
        });
        let binding = d.add(Kind::IterationBinding(s, parent));
        let mut bytes = d.encode().unwrap();
        let record = |ids: &[usize]| {
            let mut r = ((ids.len() * 4 + 4) as u32).to_le_bytes().to_vec();
            r.extend([0xd2, 7, 0, 0]);
            r.extend((ids.len() as u32).to_le_bytes());
            for id in ids {
                r.extend((*id as u32).to_le_bytes());
            }
            r
        };
        let old = record(&[a, p, n, p, s, n]);
        let mut ids = vec![a, p, n, p, host, numeric];
        if let Some(r) = ratio {
            ids.push(r)
        };
        ids.extend([s, transformed, n]);
        let new = record(&ids);
        let at = bytes.windows(old.len()).position(|w| w == old).unwrap();
        bytes.splice(at..at + old.len(), new);
        let old = record(&[s, parent]);
        let at = bytes.windows(old.len()).position(|w| w == old).unwrap() + 12;
        bytes[at..at + 4].copy_from_slice(&(transformed as u32).to_le_bytes());
        let mut carried = vec![host, numeric];
        if let Some(r) = ratio {
            carried.push(r)
        };
        carried.extend([s, transformed, n]);
        let it = api
            .add_mapped_point_iteration_with_bindings(
                &[a, p],
                &[vec![n, p]],
                &carried,
                &[transformed],
                IterationDepth::Fixed(3),
            )
            .unwrap();
        api.verify_roundtrip().unwrap();
        assert_eq!(api.encode().unwrap(), bytes, "API reconstruction {state}");
        println!(
            "{state} API exact segments {:?}",
            api.branch_segment_iteration_images(it + 1)
        );
        std::fs::write(format!("{prefix}_{state}.gsp"), &bytes).unwrap();
        println!(
            "{state} {} bytes IDs A={a} P={p} O={o} E={e} host={host} numeric={numeric} ratio={ratio:?} next={n} s={s} transformed={transformed} iteration={parent} binding={binding}",
            bytes.len()
        );
    }
}
