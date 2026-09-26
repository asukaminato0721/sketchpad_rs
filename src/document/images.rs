use super::{Document, Kind, Point};
use crate::image_records::ImageFile;

impl Document {
    pub fn import_png(&mut self, png: &[u8], at: Point) -> Result<usize, String> {
        if self.partial_view
            || self.page_count != 1
            || self.image_read_only
            || self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Image { .. }))
        {
            return Err(
                "Only one independent picture can be edited in a single-page sketch".into(),
            );
        }
        if !at.x.is_finite() || !at.y.is_finite() {
            return Err("Picture position must be finite".into());
        }
        image::load_from_memory_with_format(png, image::ImageFormat::Png)
            .map_err(|error| format!("PNG pixels cannot be decoded: {error}"))?;
        ImageFile::independent_from_png(png, [1., 0., at.x, 0., 1., at.y])?;
        Ok(self.add(Kind::Image {
            png: png.to_vec(),
            transform: [1., 0., at.x, 0., 1., at.y],
        }))
    }
    pub fn move_image(&mut self, id: usize, at: Point) -> Result<(), String> {
        self.move_image_inner(id, at, true)
    }
    pub fn drag_image(&mut self, id: usize, at: Point) -> Result<(), String> {
        self.move_image_inner(id, at, false)
    }
    fn move_image_inner(&mut self, id: usize, at: Point, checkpoint: bool) -> Result<(), String> {
        if self.image_read_only || !at.x.is_finite() || !at.y.is_finite() {
            return Err("Picture cannot be moved safely".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid picture ID")?)
            .ok_or("Invalid picture ID")?;
        if !matches!(object.kind, Kind::Image { .. }) {
            return Err("Object is not a picture".into());
        }
        if checkpoint {
            self.checkpoint();
        }
        if let Kind::Image { transform, .. } = &mut self.objects[id - 1].kind {
            transform[2] = at.x;
            transform[5] = at.y;
        }
        self.dirty = true;
        Ok(())
    }
}
