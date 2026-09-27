//! Read-only checkpoint 44 broad geometry prerequisite cohort audit; no decoder/evaluator changes.
//! Compiles against the current library. Candidate counts describe the proposed
//! compositional point/segment graph guard, not verified native semantics.
//! Drawing-only inventory of broad geometry prerequisites: point/Scalar seeds, broad value DAG, point/segment bindings.
//! Counts decoded as well as unsupported original iteration roots; ignores roots needing no geometry additions.
//! Candidate counts are prerequisites, not Native semantics or full decoder recovery predictions.
use sketchpad_rs::document::transform_model::ElementType;
use sketchpad_rs::document::{Document, Kind, records};
use sketchpad_rs::table_records::NumericUnit;
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
fn n(b: &[u8], a: usize) -> usize {
    u32::from_le_bytes(b[a..a + 4].try_into().unwrap()) as usize
}
fn name(k: &Kind) -> String {
    format!("{k:?}")
        .split(|c: char| !c.is_alphanumeric())
        .next()
        .unwrap()
        .into()
}
fn point(k: &Kind) -> bool {
    k.transform_definition()
        .is_some_and(|d| d.element() == ElementType::Point)
        || matches!(
            k,
            Kind::Free(_)
                | Kind::SegmentMidpoint(_)
                | Kind::PointOnSegment(..)
                | Kind::PointOnRay(..)
                | Kind::PointOnLine(..)
                | Kind::PointOnCircle(..)
                | Kind::PointOnArc(..)
                | Kind::PointOnPolygon(..)
                | Kind::GraphHorizontalUnit(..)
                | Kind::GraphVerticalUnit(..)
                | Kind::DistanceVerticalUnit(..)
                | Kind::PathLiteralPoint(..)
                | Kind::PathValuePoint(..)
                | Kind::PlottedPoint(..)
                | Kind::NumericCoordinatePoint(..)
                | Kind::Midpoint(..)
                | Kind::Intersection(..)
                | Kind::LineCircleIntersection(..)
                | Kind::CircleCircleIntersection(..)
        )
}
fn main() {
    let mut files = vec![];
    collect(Path::new("../gsp-rs"), &mut files);
    files.sort();
    let mut cats: BTreeMap<String, (usize, BTreeSet<PathBuf>, Vec<String>)> = BTreeMap::new();
    for path in files {
        let b = fs::read(&path).unwrap();
        let Ok(doc) = Document::from_bytes(&b, None) else {
            continue;
        };
        let Ok(rs) = records(&b) else { continue };
        let mut eligible = BTreeSet::new();
        let mut root_carried = BTreeMap::new();
        for o in &doc.objects {
            let Some(at) = o.header_offset else { continue };
            let code = (n(&b, at) & 0xffff) as u16;
            if !matches!(code, 76 | 89) {
                continue;
            }
            let Some(start) = rs
                .iter()
                .position(|r| r.kind == 0x7d0 && r.data.start == at)
            else {
                continue;
            };
            let Some(end) = (start..rs.len()).find(|&i| rs[i].kind == 0x7d7) else {
                continue;
            };
            let g = &rs[start..=end];
            if b[at + 6..at + 10] != [14, 0, 64, 0] {
                continue;
            }
            let Some(d) = g
                .iter()
                .find(|r| r.kind == 0x90a)
                .map(|r| &b[r.data.clone()])
            else {
                continue;
            };
            if d.len() != 24
                || n(d, 0) != 0
                || n(d, 20) != 0
                || !matches!(&d[4..6], [0, 0] | [1, 0])
                || (code == 76 && !(1..=128).contains(&n(d, 16)))
            {
                continue;
            };
            let count = u16::from_le_bytes(d[6..8].try_into().unwrap()) as usize;
            let branches = n(d, 8);
            let skip = usize::from(code == 89);
            let Some(raw) = g
                .iter()
                .find(|r| r.kind == 0x7d2)
                .map(|r| &b[r.data.clone()])
            else {
                continue;
            };
            let base = doc.objects.iter().position(|x| x.page == o.page).unwrap();
            let ids: Vec<_> = raw[4..].chunks_exact(4).map(|c| n(c, 0) + base).collect();
            if ids.len() <= skip + count * (branches + 1)
                || count < 2
                || count > 16
                || branches == 0
                || branches > 8
            {
                continue;
            };
            if ids.iter().any(|&id| {
                doc.objects
                    .get(id - 1)
                    .is_none_or(|x| matches!(x.kind, Kind::Unsupported(_)))
            }) {
                continue;
            };
            let seeds = &ids[skip..skip + count];
            let targets = &ids[skip + count..skip + count * (branches + 1)];
            let carried = &ids[skip + count * (branches + 1)..];
            if seeds.iter().copied().collect::<BTreeSet<_>>().len() != seeds.len()
                || carried.iter().copied().collect::<BTreeSet<_>>().len() != carried.len()
            {
                continue;
            }
            if code == 89 && !doc.objects[ids[0] - 1].kind.is_iteration_depth_value() {
                continue;
            }
            let seedtypes = seeds
                .iter()
                .map(|id| name(&doc.objects[id - 1].kind))
                .collect::<Vec<_>>();
            let mut blocked = BTreeSet::new();
            if seeds.iter().any(|&id| {
                !point(&doc.objects[id - 1].kind)
                    && doc.objects[id - 1].kind.numeric_unit() != Some(NumericUnit::Scalar)
            }) {
                blocked.insert("seed_type".to_string());
            }
            if targets.chunks(count).any(|branch| {
                branch.iter().zip(seeds).any(|(image, seed)| {
                    let a = &doc.objects[*image - 1].kind;
                    let b = &doc.objects[*seed - 1].kind;
                    if point(b) {
                        !point(a)
                    } else {
                        a.numeric_unit() != Some(NumericUnit::Scalar)
                    }
                })
            }) {
                blocked.insert("target_type".to_string());
            }
            let mut categories = BTreeSet::new();
            for &seed in seeds {
                if doc.objects[seed - 1].kind.dependencies().iter().any(|&p| {
                    p >= seed
                        || doc
                            .objects
                            .get(p.wrapping_sub(1))
                            .is_none_or(|x| x.page != o.page || x.deleted)
                }) {
                    blocked.insert("invalid_seed_dependencies".into());
                }
            }
            let mut pending = targets.iter().chain(carried).copied().collect::<Vec<_>>();
            let mut seen = BTreeSet::new();
            let mut flavors = BTreeSet::new();
            while let Some(id) = pending.pop() {
                if !seen.insert(id) || seeds.contains(&id) {
                    continue;
                }
                let k = &doc.objects[id - 1].kind;
                if doc.objects[id - 1].page != o.page
                    || k.dependencies().iter().any(|&p| {
                        p >= id
                            || doc
                                .objects
                                .get(p.wrapping_sub(1))
                                .is_none_or(|x| x.page != o.page || x.deleted)
                    })
                {
                    blocked.insert("invalid_DAG".into());
                    continue;
                }

                if let Some(t) = k.transform_definition() {
                    flavors.insert(format!("{:?}/{:?}", t.element(), t.wire_flavor()));
                    if !matches!(t.element(), ElementType::Point | ElementType::Segment) {
                        categories.insert(format!("transform_{:?}", t.element()));
                    }
                    pending.extend(t.dependencies());
                    continue;
                }
                if !matches!(
                    k,
                    Kind::Free(_)
                        | Kind::Segment(..)
                        | Kind::Midpoint(..)
                        | Kind::SegmentMidpoint(..)
                        | Kind::Ray(..)
                        | Kind::Line(..)
                        | Kind::Parameter(_)
                        | Kind::Calculation(_)
                        | Kind::FunctionDefinition(_)
                        | Kind::ReferencedFunction(..)
                        | Kind::DerivativeFunction(..)
                        | Kind::ParameterCalculation(..)
                        | Kind::GraphHorizontalUnit(..)
                        | Kind::GraphVerticalUnit(..)
                        | Kind::GraphAxis(..)
                        | Kind::DistanceAxis(..)
                        | Kind::DistanceVerticalUnit(..)
                        | Kind::CoordinateSystem(..)
                        | Kind::PlottedPoint(..)
                        | Kind::NumericCoordinatePoint(..)
                ) {
                    if point(k)
                        || k.numeric_unit().is_some()
                        || k.is_measurement()
                        || k.is_circle()
                        || k.is_polygon()
                        || k.is_line()
                        || matches!(
                            k,
                            Kind::Parallel(..)
                                | Kind::Perpendicular(..)
                                | Kind::Bisector(..)
                                | Kind::ThreePointArc(..)
                                | Kind::ArcOnCircle(..)
                                | Kind::CenterArc(..)
                                | Kind::CircleAxis(..)
                        )
                    {
                        categories.insert(name(k));
                    } else {
                        blocked.insert(name(k));
                    }
                }
                pending.extend(k.dependencies());
            }
            if blocked.is_empty() && categories.is_empty() {
                continue;
            }
            let key = if blocked.is_empty() {
                eligible.insert(o.id);
                root_carried.insert(o.id, carried.to_vec());
                format!(
                    "candidate code{code} seeds{count} branches{branches} seedtypes{seedtypes:?} additions{categories:?}"
                )
            } else {
                format!("blocked code{code} {blocked:?}")
            };
            let cat = cats.entry(key).or_default();
            if blocked.is_empty() {
                println!(
                    "ROOT {} id{} code{} additions{:?} seeds{:?} targets{:?} carried{:?} decoded{}",
                    path.display(),
                    o.id,
                    code,
                    categories,
                    seeds,
                    targets,
                    carried,
                    !matches!(o.kind, Kind::Unsupported(_))
                );
            }
            cat.0 += 1;
            cat.1.insert(path.clone());
            if cat.2.len() < 4 {
                cat.2.push(format!(
                    "{} global{} page{} refs{ids:?} transforms{flavors:?}",
                    path.display(),
                    o.id,
                    o.page
                ));
            }
        }
        let mut bindings = 0;
        for o in &doc.objects {
            if !matches!(o.kind, Kind::Unsupported(77) | Kind::IterationBinding(..)) {
                continue;
            }
            let Some(at) = o.header_offset else { continue };
            let Some(start) = rs
                .iter()
                .position(|r| r.kind == 0x7d0 && r.data.start == at)
            else {
                continue;
            };
            let Some(end) = (start..rs.len()).find(|&i| rs[i].kind == 0x7d7) else {
                continue;
            };
            let Some(raw) = rs[start..=end]
                .iter()
                .find(|r| r.kind == 0x7d2)
                .map(|r| &b[r.data.clone()])
            else {
                continue;
            };
            let base = doc.objects.iter().position(|x| x.page == o.page).unwrap();
            if raw.len() == 12
                && b[at + 6..at + 10] == [15, 0, 65, 0]
                && eligible.contains(&(n(raw, 8) + base))
                && root_carried[&(n(raw, 8) + base)].contains(&(n(raw, 4) + base))
            {
                let bound = n(raw, 4) + base;
                if point(&doc.objects[bound - 1].kind) || doc.objects[bound - 1].kind.is_segment() {
                    println!(
                        "BIND {} id{} root{} source{} kind{}",
                        path.display(),
                        o.id,
                        n(raw, 8) + base,
                        bound,
                        name(&doc.objects[bound - 1].kind)
                    );
                    bindings += 1;
                }
            }
        }
        if !eligible.is_empty() {
            println!(
                "ELIGIBLE {} roots{} bindings{bindings}",
                path.display(),
                eligible.len()
            );
        }
    }
    for (k, (n, f, ex)) in cats {
        println!("COUNT {n} files{} {k}", f.len());
        for e in ex {
            println!("EX {e}")
        }
    }
}
