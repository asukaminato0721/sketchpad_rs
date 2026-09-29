use super::{Document, DocumentSnapshot};

impl Document {
    fn snapshot(&self) -> DocumentSnapshot {
        DocumentSnapshot {
            objects: self.objects.clone(),
            merge_aliases: self.merge_aliases.clone(),
            measurement_names: self.measurement_names.clone(),
            snap_points: self.snap_points.clone(),
            grid_dotted: self.grid_dotted.clone(),
            grid_form: self.grid_form.clone(),
            page_names: self.page_names.clone(),
            angle_unit_radians: self.angle_unit_radians,
            marked_coordinate_system: self.marked_coordinate_system.clone(),
            frame_styles: self.frame_styles.clone(),
            run_font_families: self.run_font_families.clone(),
            last_moved_object: self.last_moved_object,
            page_count: self.page_count,
            active_page: self.active_page,
            safe_structure: self.safe_structure,
            partial_view: self.partial_view,
            source: None,
        }
    }
    fn restore_snapshot(&mut self, snapshot: DocumentSnapshot) -> DocumentSnapshot {
        let mut previous = self.snapshot();
        let page_count_changed = self.page_count != snapshot.page_count;
        self.objects = snapshot.objects;
        self.merge_aliases = snapshot.merge_aliases;
        self.measurement_names = snapshot.measurement_names;
        self.snap_points = snapshot.snap_points;
        self.grid_dotted = snapshot.grid_dotted;
        self.grid_form = snapshot.grid_form;
        self.page_names = snapshot.page_names;
        self.angle_unit_radians = snapshot.angle_unit_radians;
        self.marked_coordinate_system = snapshot.marked_coordinate_system;
        self.frame_styles = snapshot.frame_styles;
        self.run_font_families = snapshot.run_font_families;
        self.last_moved_object = snapshot.last_moved_object;
        self.page_count = snapshot.page_count;
        self.active_page = snapshot.active_page;
        self.safe_structure = snapshot.safe_structure;
        self.partial_view = snapshot.partial_view;
        if let Some((source, count, read_only)) = snapshot.source {
            previous.source = Some((
                std::mem::replace(&mut self.original, source),
                std::mem::replace(&mut self.source_count, count),
                std::mem::replace(&mut self.image_read_only, read_only),
            ));
        }
        if page_count_changed {
            self.selected.clear();
        }
        self.dirty = true;
        previous
    }
    pub fn checkpoint(&mut self) {
        self.refresh_measurement_names();
        self.undo.push(self.snapshot());
        self.redo.clear();
        self.dirty = true;
    }
    pub(super) fn checkpoint_structure(&mut self) {
        self.refresh_measurement_names();
        let mut snapshot = self.snapshot();
        snapshot.source = Some((
            self.original.clone(),
            self.source_count,
            self.image_read_only,
        ));
        self.undo.push(snapshot);
        self.redo.clear();
        self.dirty = true;
    }
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo.pop() {
            let previous = self.restore_snapshot(snapshot);
            self.redo.push(previous);
        }
    }
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo.pop() {
            let previous = self.restore_snapshot(snapshot);
            self.undo.push(previous);
        }
    }
}
