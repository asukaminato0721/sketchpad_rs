//! Read-only structural recognition. A match does not establish edit history.
use super::{Document, EmbeddedToolInputKind, Kind, Object, kind};
use crate::table_records::NumericUnit;
use std::collections::{BTreeSet, HashMap};
use std::mem::{Discriminant, discriminant};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolMatch {
    pub tool_index: usize,
    pub name: String,
    /// Drawing IDs in the tool's declared input order.
    pub inputs: Vec<usize>,
    /// All constructed objects, including hidden intermediate objects.
    pub objects: Vec<usize>,
}

#[derive(Default)]
pub struct ToolMatches {
    pub groups: Vec<ToolMatch>,
    /// A bounded search can leave otherwise supported constructions ungrouped.
    pub limited: bool,
}

struct Pattern {
    index: usize,
    name: String,
    kinds: Vec<Kind>,
    inputs: Vec<(usize, EmbeddedToolInputKind)>,
    outputs: Vec<usize>,
}

// Use the native segment relation for midpoints in both namespaces.
fn construction_kind(object: &Object) -> Kind {
    if matches!(object.kind, Kind::Midpoint(..))
        && let Some(host) = object.midpoint_host
    {
        Kind::SegmentMidpoint(host)
    } else {
        object.kind.clone()
    }
}

impl Document {
    /// Find non-overlapping constructions on the active page that match embedded
    /// tools. Prefer larger definitions, then file order. Inputs can be shared.
    /// This method does not change objects, selection, history, or file bytes.
    pub fn matching_tool_constructions(&self) -> ToolMatches {
        let mut result = ToolMatches::default();
        let Ok(tools) = self.embedded_tools() else {
            return result;
        };
        if tools.is_empty() {
            return result;
        }
        let mut patterns = Vec::new();
        for (index, summary) in tools.iter().enumerate().take(128) {
            // Limit recursion and allocation before decoding a prototype.
            if summary.object_codes.len() > 256 {
                result.limited = true;
                continue;
            }
            let (Ok(prototype), Ok(inputs)) = (
                self.embedded_tool_document(index),
                self.embedded_tool_inputs(index),
            ) else {
                continue;
            };
            if prototype.partial_view || inputs.is_empty() {
                continue;
            }
            let Some(inputs) = inputs
                .iter()
                .map(|input| Some((input.local_id, input.kind()?)))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let kinds: Vec<_> = prototype.objects.iter().map(construction_kind).collect();
            let outputs: Vec<_> = (1..=kinds.len())
                .filter(|id| !inputs.iter().any(|(input, _)| input == id))
                .collect();
            // Require a dependency DAG rooted in declared inputs. Independent
            // constants and unknown operations cannot identify an invocation.
            if outputs.is_empty()
                || outputs.iter().any(|&id| {
                    let parents = kinds[id - 1].dependencies();
                    parents.is_empty()
                        || parents.iter().any(|&parent| parent == 0 || parent >= id)
                        || matches!(kinds[id - 1], Kind::Unsupported(_) | Kind::ToolInput(_))
                })
            {
                continue;
            }
            patterns.push(Pattern {
                index,
                name: summary
                    .name
                    .clone()
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| format!("Tool {}", index + 1)),
                kinds,
                inputs,
                outputs,
            });
        }
        result.limited |= tools.len() > 128;
        patterns.sort_by_key(|pattern| (std::cmp::Reverse(pattern.outputs.len()), pattern.index));
        let kinds: Vec<_> = self.objects.iter().map(construction_kind).collect();
        let mut candidates: HashMap<Discriminant<Kind>, Vec<usize>> = HashMap::new();
        for object in self
            .objects
            .iter()
            .filter(|object| !object.deleted && object.page == self.active_page)
        {
            candidates
                .entry(discriminant(&kinds[object.id - 1]))
                .or_default()
                .push(object.id);
        }
        let mut claimed = BTreeSet::new();
        let mut budget = 200_000usize;
        for pattern in &patterns {
            let mut matcher = Matcher {
                document: self,
                kinds: &kinds,
                candidates: &candidates,
                pattern,
                claimed: &mut claimed,
                budget: &mut budget,
            };
            // Start with the rarest output kind. Its ancestors bind the inputs.
            let anchor = *pattern
                .outputs
                .iter()
                .min_by_key(|&&id| (matcher.candidates_for(id).len(), std::cmp::Reverse(id)))
                .unwrap();
            for target in matcher.candidates_for(anchor).to_vec() {
                if *matcher.budget == 0 {
                    break;
                }
                let mut mapping = vec![0; pattern.kinds.len() + 1];
                if !matcher.bind(anchor, target, &mut mapping, 0) {
                    continue;
                }
                if let Some(mapping) = matcher.complete(mapping, 0) {
                    let mut objects: Vec<_> =
                        pattern.outputs.iter().map(|&id| mapping[id]).collect();
                    objects.sort_unstable();
                    matcher.claimed.extend(&objects);
                    result.groups.push(ToolMatch {
                        tool_index: pattern.index,
                        name: pattern.name.clone(),
                        inputs: pattern.inputs.iter().map(|&(id, _)| mapping[id]).collect(),
                        objects,
                    });
                }
            }
        }
        result.limited |= budget == 0;
        result.groups.sort_by_key(|group| group.objects[0]);
        result
    }
}

struct Matcher<'a> {
    document: &'a Document,
    kinds: &'a [Kind],
    candidates: &'a HashMap<Discriminant<Kind>, Vec<usize>>,
    pattern: &'a Pattern,
    claimed: &'a mut BTreeSet<usize>,
    budget: &'a mut usize,
}

impl Matcher<'_> {
    fn candidates_for(&self, local: usize) -> &[usize] {
        self.candidates
            .get(&discriminant(&self.pattern.kinds[local - 1]))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    fn bind(&mut self, local: usize, target: usize, mapping: &mut [usize], depth: usize) -> bool {
        if *self.budget == 0 || depth >= 256 {
            return false;
        }
        *self.budget -= 1;
        if mapping[local] != 0 {
            return mapping[local] == target;
        }
        let Some(object) = target
            .checked_sub(1)
            .and_then(|id| self.document.objects.get(id))
        else {
            return false;
        };
        if object.deleted || object.page != self.document.active_page {
            return false;
        }
        if let Some(&(_, category)) = self.pattern.inputs.iter().find(|&&(id, _)| id == local) {
            if !accepts_input(category, &object.kind)
                || self.pattern.outputs.iter().any(|&id| mapping[id] == target)
            {
                return false;
            }
            mapping[local] = target;
            return true;
        }
        if self.claimed.contains(&target) || mapping.contains(&target) {
            return false;
        }
        let expected = &self.pattern.kinds[local - 1];
        let actual = &self.kinds[target - 1];
        if discriminant(expected) != discriminant(actual) {
            return false;
        }
        let parents = expected.dependencies();
        let actual_parents = actual.dependencies();
        if parents.len() != actual_parents.len() {
            return false;
        }
        mapping[local] = target;
        for (parent, actual_parent) in parents.into_iter().zip(actual_parents) {
            if !self.bind(parent, actual_parent, mapping, depth + 1) {
                return false;
            }
        }
        // Compare the entire construction, including fixed angles, ratios,
        // intersection branches, and other parameters. Coordinates alone are
        // not evidence that two constructions have the same definition.
        expected
            .remap_checked(mapping)
            .is_ok_and(|kind| kind == *actual)
    }

    fn complete(&mut self, mapping: Vec<usize>, depth: usize) -> Option<Vec<usize>> {
        if *self.budget == 0 || depth >= 256 {
            return None;
        }
        let next = self
            .pattern
            .outputs
            .iter()
            .copied()
            .filter(|&id| mapping[id] == 0)
            .min_by_key(|&id| {
                (
                    std::cmp::Reverse(
                        self.pattern.kinds[id - 1]
                            .dependencies()
                            .iter()
                            .filter(|&&parent| mapping[parent] != 0)
                            .count(),
                    ),
                    self.candidates_for(id).len(),
                )
            });
        let Some(local) = next else {
            return self
                .pattern
                .inputs
                .iter()
                .all(|&(id, _)| mapping[id] != 0)
                .then_some(mapping);
        };
        // A stable candidate list and backtracking handle separate output
        // branches without accepting a partial prototype.
        for target in self.candidates_for(local).to_vec() {
            if *self.budget == 0 {
                break;
            }
            let mut next_mapping = mapping.clone();
            if self.bind(local, target, &mut next_mapping, 0)
                && let Some(complete) = self.complete(next_mapping, depth + 1)
            {
                return Some(complete);
            }
        }
        None
    }
}

fn accepts_input(category: EmbeddedToolInputKind, actual: &Kind) -> bool {
    match category {
        EmbeddedToolInputKind::Point => kind::is_point_kind(actual),
        EmbeddedToolInputKind::Path => {
            kind::is_straight_kind(actual)
                || actual.is_circle()
                || actual.is_arc()
                || actual.is_polygon()
        }
        EmbeddedToolInputKind::Segment | EmbeddedToolInputKind::SegmentDistance => {
            actual.is_segment()
        }
        EmbeddedToolInputKind::Polygon => actual.is_polygon(),
        EmbeddedToolInputKind::CoordinateSystem => matches!(actual, Kind::CoordinateSystem(..)),
        EmbeddedToolInputKind::Numeric => actual.numeric_unit().is_some(),
        EmbeddedToolInputKind::Distance => actual.is_distance_value(),
        EmbeddedToolInputKind::Function => actual.is_function(),
        EmbeddedToolInputKind::ScalarParameter => {
            matches!(actual, Kind::Parameter(group) if group.unit() == NumericUnit::Scalar)
        }
        EmbeddedToolInputKind::DistanceParameter => {
            matches!(actual, Kind::Parameter(group) if group.unit() == NumericUnit::Length)
        }
        EmbeddedToolInputKind::AngleParameter => {
            matches!(actual, Kind::Parameter(group) if group.unit() == NumericUnit::Degrees)
        }
        EmbeddedToolInputKind::Preimage => actual.is_preimage_geometry(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Point;

    fn construction() -> Document {
        let mut doc = Document::new();
        doc.add(Kind::Free(Point { x: 0., y: 0. }));
        doc.add(Kind::Free(Point { x: 80., y: 0. }));
        doc.add(Kind::Segment(1, 2));
        doc.add(Kind::Midpoint(1, 2));
        doc.objects.edit(2).visible = false;
        doc.selected = [1, 2, 4].into();
        doc.create_embedded_tool("Midpoint construction").unwrap();
        doc
    }

    #[test]
    fn matches_all_outputs_and_binding_order_without_changing_document() {
        let mut doc = construction();
        let left = doc.add(Kind::Free(Point { x: 200., y: 60. }));
        let right = doc.add(Kind::Free(Point { x: 400., y: 160. }));
        let outputs = doc.invoke_embedded_tool(0, &[right, left]).unwrap();
        let before = doc.encode().unwrap();
        let selection = doc.selected.clone();
        let history = doc.undo.len();
        let groups = doc.matching_tool_constructions();
        assert!(!groups.limited);
        assert_eq!(groups.groups.len(), 2, "{:?}", groups.groups);
        assert_eq!(groups.groups[0].objects, [3, 4]);
        assert_eq!(groups.groups[1].inputs, [right, left]);
        assert_eq!(groups.groups[1].objects, outputs);
        assert_eq!(doc.encode().unwrap(), before);
        assert_eq!(doc.selected, selection);
        assert_eq!(doc.undo.len(), history);
        let reopened = Document::from_bytes(&before, None).unwrap();
        assert_eq!(reopened.matching_tool_constructions().groups, groups.groups);
    }

    #[test]
    fn shared_inputs_form_distinct_groups_and_larger_tools_take_priority() {
        let mut doc = construction();
        doc.selected = [1, 2, 3].into();
        doc.create_embedded_tool("Segment only").unwrap();
        let outputs = doc.invoke_embedded_tool(0, &[1, 2]).unwrap();
        let groups = doc.matching_tool_constructions().groups;
        assert_eq!(groups.len(), 2, "{groups:?}");
        assert!(groups.iter().all(|group| group.tool_index == 0));
        assert_eq!(groups[1].objects, outputs);
        assert_eq!(groups[0].inputs, groups[1].inputs);
    }

    #[test]
    fn incomplete_deleted_and_cross_page_constructions_remain_separate() {
        let mut doc = construction();
        doc.objects.edit(3).deleted = true;
        assert!(doc.matching_tool_constructions().groups.is_empty());
        doc.objects.edit(3).deleted = false;
        doc.objects.edit(2).page = 1;
        assert!(doc.matching_tool_constructions().groups.is_empty());
        doc.objects.edit(2).page = 0;
        doc.objects.edit(0).deleted = true;
        assert!(doc.matching_tool_constructions().groups.is_empty());
    }

    #[test]
    fn fixed_parameters_and_dependency_identity_prevent_false_matches() {
        let mut doc = Document::new();
        doc.add(Kind::Free(Point { x: 0., y: 0. }));
        doc.add(Kind::Free(Point { x: 80., y: 0. }));
        doc.add(Kind::ScaledPoint(2, 1, 2.));
        doc.selected = [1, 2, 3].into();
        doc.create_embedded_tool("Double distance").unwrap();
        assert_eq!(doc.matching_tool_constructions().groups.len(), 1);
        doc.objects.edit(2).kind = Kind::ScaledPoint(2, 1, 3.);
        assert!(doc.matching_tool_constructions().groups.is_empty());

        let mut doc = construction();
        // Same screen position does not make a free point a constructed midpoint.
        doc.objects.edit(3).kind = Kind::Free(Point { x: 40., y: 0. });
        assert!(doc.matching_tool_constructions().groups.is_empty());
    }

    #[test]
    fn undo_redo_and_point_motion_recompute_matches() {
        let mut doc = construction();
        doc.invoke_embedded_tool(0, &[1, 2]).unwrap();
        assert_eq!(doc.matching_tool_constructions().groups.len(), 2);
        doc.undo();
        assert_eq!(doc.matching_tool_constructions().groups.len(), 1);
        doc.redo();
        assert_eq!(doc.matching_tool_constructions().groups.len(), 2);
        doc.move_point(1, Point { x: -300., y: 50. });
        assert_eq!(doc.matching_tool_constructions().groups.len(), 2);
    }

    #[test]
    fn segment_given_matches_native_midpoint_relation() {
        let mut doc = construction();
        doc.selected = [3, 4].into();
        let index = doc.create_embedded_tool("Segment midpoint").unwrap();
        // The larger point-based tool wins for the original construction.
        let host = doc.add(Kind::VectorTranslatedSegment(3, 1, 2));
        let outputs = doc.invoke_embedded_tool(index, &[host]).unwrap();
        let groups = doc.matching_tool_constructions().groups;
        assert!(
            groups.iter().any(|group| group.tool_index == index
                && group.inputs == [host]
                && group.objects == outputs),
            "{groups:?}"
        );
    }

    #[test]
    fn recognizes_imported_tool_constructions_in_the_native_example() {
        let bytes = include_bytes!("../../../../tests/fixtures/example_tools.gsp");
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        let original_page = doc.active_page;
        let mut groups = Vec::new();
        for page in 0..doc.page_count {
            doc.active_page = page;
            let found = doc.matching_tool_constructions();
            assert!(!found.limited);
            groups.extend(found.groups);
        }
        assert!(
            groups
                .iter()
                .any(|group| group.name == "Square Given Two Vertices"),
            "{groups:?}"
        );
        doc.active_page = original_page;
        assert!(doc.encode().unwrap() == bytes);
    }
}
