//! Incremental, read-only evaluation for a long-lived consumer such as a view.
use super::{Document, EvaluatedDocument};
use lru::LruCache;
use sketchpad_ir::{DependencyProgram, UpdatePlan};
use std::num::NonZeroUsize;
use std::sync::Arc;

pub struct EvaluationCache {
    previous: Option<Arc<EvaluatedDocument>>,
    program: Option<Arc<DependencyProgram>>,
    updates: LruCache<Vec<usize>, Arc<UpdatePlan>>,
}

impl Default for EvaluationCache {
    fn default() -> Self {
        Self {
            previous: None,
            program: None,
            updates: LruCache::new(NonZeroUsize::new(32).unwrap()),
        }
    }
}

impl EvaluationCache {
    pub fn evaluate(&mut self, doc: &Document) -> Arc<EvaluatedDocument> {
        // Scoped substitutions have their own invalidation rules. Do not carry
        // values between distinct transient or mapped evaluation contexts.
        if doc.evaluation_context.is_some() {
            self.previous = None;
            self.program = None;
            self.updates.clear();
            return Arc::new(doc.evaluated());
        }
        let previous = self
            .previous
            .as_ref()
            .filter(|old| same_environment(doc, old));
        let program = doc.objects.dependency_program().ok();
        let topology_matches = self
            .program
            .as_ref()
            .zip(program.as_ref())
            .is_some_and(|(old, new)| Arc::ptr_eq(old, new));
        let changed = previous
            .filter(|_| topology_matches)
            .and_then(|old| doc.objects.changes_since(&old.objects));
        let Some(changed) = changed else {
            self.program = program;
            self.updates.clear();
            let result = Arc::new(doc.evaluated());
            self.previous = Some(result.clone());
            return result;
        };
        let old = previous.unwrap();
        if changed.is_empty()
            && doc.selected == old.selected
            && doc.path == old.path
            && doc.dirty == old.dirty
            && doc.last_moved_object == old.last_moved_object
        {
            return old.clone();
        }
        let update = if let Some(plan) = self.updates.get(&changed) {
            plan.clone()
        } else {
            let plan = Arc::new(self.program.as_ref().unwrap().update(&changed));
            self.updates.put(changed.clone(), plan.clone());
            plan
        };
        let mut snapshot = doc.clone_for_evaluation();
        snapshot.evaluation_context = old.evaluation_context.clone();
        snapshot.invalidate_text_fonts(&changed);
        if let Some(context) = snapshot.evaluation_context_mut() {
            context.original = Arc::new(doc.clone_for_evaluation());
        }
        let _ = snapshot.execute_evaluation_plan(&update, []);
        let result = Arc::new(EvaluatedDocument(snapshot));
        self.previous = Some(result.clone());
        result
    }
}

// Object edits are handled by the dependency graph. Changes outside that graph
// conservatively rebuild values, including file replacement and page settings.
fn same_environment(a: &Document, b: &Document) -> bool {
    a.original == b.original
        && a.active_page == b.active_page
        && a.page_count == b.page_count
        && a.partial_view == b.partial_view
        && a.safe_structure == b.safe_structure
        && a.image_read_only == b.image_read_only
        && a.source_count == b.source_count
        && a.snap_points == b.snap_points
        && a.grid_dotted == b.grid_dotted
        && a.grid_form == b.grid_form
        && a.page_names == b.page_names
        && a.angle_unit_radians == b.angle_unit_radians
        && a.marked_coordinate_system == b.marked_coordinate_system
        && a.frame_styles == b.frame_styles
        && a.merge_aliases == b.merge_aliases
        && a.measurement_names == b.measurement_names
        && a.run_font_families == b.run_font_families
        && a.object_color_defaults == b.object_color_defaults
        && a.object_style_defaults == b.object_style_defaults
        && a.measurement_precision_defaults == b.measurement_precision_defaults
        && a.new_polygon_opacity == b.new_polygon_opacity
        && a.randomize_interior_colors == b.randomize_interior_colors
        && a.interior_color_cursor == b.interior_color_cursor
        && a.custom_transform_definitions == b.custom_transform_definitions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Kind, Point};

    fn compare(cache: &mut EvaluationCache, doc: &Document) -> Arc<EvaluatedDocument> {
        let view = cache.evaluate(doc);
        view.prepare_locus_strokes(16);
        let fresh = doc.evaluated();
        for object in &doc.objects {
            let id = object.id;
            assert_eq!(view.position(id), fresh.position(id), "point {id}");
            assert_eq!(
                view.numeric_value(id),
                fresh.numeric_value(id),
                "number {id}"
            );
            assert_eq!(view.line_points(id), fresh.line_points(id), "line {id}");
            assert_eq!(
                view.circle_points(id),
                fresh.circle_points(id),
                "circle {id}"
            );
            if object.kind.transform_definition().is_some() {
                assert_eq!(
                    view.evaluate_transform(id),
                    doc.evaluate_transform(id),
                    "transform {id}"
                );
            }
            assert_eq!(
                view.text_font_state(id),
                fresh.text_font_state(id),
                "font {id}"
            );
            assert_eq!(
                view.locus_styled_strokes(id, 16),
                fresh.locus_styled_strokes(id, 16),
                "curve {id}"
            );
        }
        assert_eq!(view.selected, doc.selected);
        assert_eq!(view.encode(), doc.encode());
        view
    }

    #[test]
    fn incremental_views_match_full_evaluation_after_edits_and_undo() {
        let mut cache = EvaluationCache::default();
        for name in [
            "locus_midpoint_original.gsp",
            "curve_style_parameter_locus_native_verified.gsp",
            "locus_undefined_sample_verified.gsp",
            "numeric_cood_trace_native.gsp",
            "mapped_calculated_angle_iteration_full_native_verified.gsp",
            "source_exponents_numeric_iteration_port.gsp",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures")
                .join(name);
            let mut doc = Document::open(&path).unwrap();
            let first = compare(&mut cache, &doc);
            assert!(Arc::ptr_eq(&first, &cache.evaluate(&doc)));
            let free: Vec<_> = doc
                .objects
                .iter()
                .filter_map(|o| match o.kind {
                    Kind::Free(at) => Some((o.id, at)),
                    _ => None,
                })
                .take(3)
                .collect();
            for (id, at) in free {
                doc.checkpoint();
                doc.selected.insert(id);
                assert!(doc.move_point(
                    id,
                    Point {
                        x: at.x + 13.,
                        y: at.y - 7.
                    }
                ));
                compare(&mut cache, &doc);
                assert_eq!(first.position(id), Some(at));
                doc.undo();
                compare(&mut cache, &doc);
            }
            doc.angle_unit_radians = !doc.angle_unit_radians;
            compare(&mut cache, &doc);
        }
    }

    #[test]
    fn rewiring_undefined_values_and_transient_inputs_invalidate_cached_results() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 10., y: 10. }));
        let middle = doc.add(Kind::Midpoint(a, b));
        let parameter = doc.add_named_parameter("n", 2., (0, 0)).unwrap();
        let reciprocal = doc
            .add_parameter_calculation(parameter, "1/n", (0, 20))
            .unwrap();
        let mut cache = EvaluationCache::default();
        let original = compare(&mut cache, &doc);
        for value in [0., 4., -2.] {
            doc.set_parameter_value(parameter, value).unwrap();
            compare(&mut cache, &doc);
        }
        assert_eq!(original.numeric_value(reciprocal), Some(0.5));
        let later = doc.add(Kind::Free(Point { x: 30., y: 40. }));
        doc.objects.edit(middle - 1).kind = Kind::Midpoint(a, later);
        compare(&mut cache, &doc);
        doc.objects.edit(a - 1).deleted = true;
        assert_eq!(compare(&mut cache, &doc).position(middle), None);
        doc.objects.edit(a - 1).deleted = false;
        doc.set_transient_point_position(a, Point { x: 100., y: 200. });
        compare(&mut cache, &doc);
        doc.clear_transient_point_positions(&[a]);
        compare(&mut cache, &doc);
    }

    #[test]
    fn font_edits_refresh_metadata_without_mutating_an_older_view() {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../../tests/fixtures/text_object_generated_Hello.gsp"),
            None,
        )
        .unwrap();
        let mut cache = EvaluationCache::default();
        let old = compare(&mut cache, &doc);
        let old_font = old.text_font_state(1);
        doc.selected.insert(1);
        doc.set_selected_text_family("Test Changed Font").unwrap();
        assert_eq!(
            compare(&mut cache, &doc).text_font_state(1).unwrap().family,
            "Test Changed Font"
        );
        assert_eq!(old.text_font_state(1), old_font);
        doc.undo();
        assert_eq!(compare(&mut cache, &doc).text_font_state(1), old_font);
    }

    #[test]
    fn cached_transform_validation_rejects_changed_ancestors() {
        let mut doc = Document::new();
        let point = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let moved = doc.add(Kind::PolarTranslatedPoint(point, 0., 1., 3.));
        let descendant = doc.add(Kind::PolarTranslatedPoint(moved, 0., 1., 5.));
        let mut cache = EvaluationCache::default();
        let original = compare(&mut cache, &doc);
        assert!(original.evaluate_transform(descendant).is_ok());
        doc.objects.edit(point - 1).deleted = true;
        assert!(
            compare(&mut cache, &doc)
                .evaluate_transform(descendant)
                .is_err()
        );
        doc.objects.edit(point - 1).deleted = false;
        assert!(
            compare(&mut cache, &doc)
                .evaluate_transform(descendant)
                .is_ok()
        );
        doc.objects.edit(point - 1).page = 1;
        assert!(cache.evaluate(&doc).evaluate_transform(descendant).is_err());
        assert!(original.evaluate_transform(descendant).is_ok());
    }

    #[test]
    fn parallel_traces_keep_sample_substitutions_and_older_views_isolated() {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../../tests/fixtures/locus_midpoint_original.gsp"),
            None,
        )
        .unwrap();
        let locus = doc.objects[7].kind.clone();
        let mut ids = vec![8];
        for _ in 0..7 {
            ids.push(doc.add(locus.clone()));
        }
        let mut cache = EvaluationCache::default();
        let original = cache.evaluate(&doc);
        original.prepare_locus_strokes(32);
        let before: Vec<_> = ids
            .iter()
            .map(|&id| original.locus_styled_strokes(id, 32))
            .collect();
        for &id in &ids {
            assert_eq!(
                original.locus_styled_strokes(id, 32),
                doc.locus_styled_strokes(id, 32)
            );
        }
        assert!(doc.move_point(1, Point { x: 100., y: 220. }));
        let moved = cache.evaluate(&doc);
        moved.prepare_locus_strokes(32);
        for (&id, before) in ids.iter().zip(before) {
            assert_eq!(original.locus_styled_strokes(id, 32), before);
            assert_eq!(
                moved.locus_styled_strokes(id, 32),
                doc.locus_styled_strokes(id, 32)
            );
            assert_ne!(moved.locus_styled_strokes(id, 32), before);
        }
    }
}
