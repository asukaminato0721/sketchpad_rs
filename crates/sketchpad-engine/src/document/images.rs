use super::{Document, ImagePlacement, Kind, Point};
use crate::image_records::ImageFile;

pub type ImageInstances<'a> = (usize, &'a [u8], Vec<[f64; 6]>);
pub type CroppedImage<'a> = (usize, &'a [u8], [f64; 6], Vec<Point>);

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
            placement: ImagePlacement::Independent([1., 0., at.x, 0., 1., at.y]),
        }))
    }
    /// Resolve an imported picture through its live point attachments.
    pub fn image_transform(&self, id: usize) -> Option<[f64; 6]> {
        if let Some(original) = self.frozen_evaluation_document(id) {
            return original.image_transform(id);
        }
        let Kind::Image { png, placement } = &self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        match placement {
            ImagePlacement::Independent(transform) => Some(*transform),
            ImagePlacement::Point { anchor, relative } => {
                let point = self.position(*anchor)?;
                let mut relative = *relative;
                relative[2] += point.x;
                relative[5] += point.y;
                Some(relative)
            }
            ImagePlacement::Rectangle {
                bottom_right,
                top_left,
            } => {
                let bottom_right = self.position(*bottom_right)?;
                let top_left = self.position(*top_left)?;
                let (width, height) = ImageFile::png_size(png).ok()?;
                Some([
                    (bottom_right.x - top_left.x) / f64::from(width),
                    0.,
                    top_left.x,
                    0.,
                    (bottom_right.y - top_left.y) / f64::from(height),
                    top_left.y,
                ])
            }
            ImagePlacement::Parallelogram {
                top_left,
                bottom_right,
                bottom_left,
            } => {
                let top_left = self.position(*top_left)?;
                let bottom_right = self.position(*bottom_right)?;
                let bottom_left = self.position(*bottom_left)?;
                let (width, height) = ImageFile::png_size(png).ok()?;
                Some([
                    (bottom_right.x - bottom_left.x) / f64::from(width),
                    (bottom_left.x - top_left.x) / f64::from(height),
                    top_left.x,
                    (bottom_right.y - bottom_left.y) / f64::from(width),
                    (bottom_left.y - top_left.y) / f64::from(height),
                    top_left.y,
                ])
            }
            ImagePlacement::Transformed(definition) => {
                let source = self.image_transform(definition.source())?;
                let operation = self.transform_operation_matrix(definition).ok()?.0;
                let [[a, b, x], [c, d, y]] = sketchpad_ir::affine::AffineMatrix::from_rows([
                    [source[0], source[1], source[2]],
                    [source[3], source[4], source[5]],
                ])
                .then(operation)
                .rows();
                Some([a, b, x, c, d, y])
            }
        }
    }
    /// Shared pixel source and live placement of a picture or its iterated images.
    pub fn image_instances(&self, id: usize) -> Option<ImageInstances<'_>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let (source, iteration) = match object.kind {
            Kind::Image { .. } => (id, None),
            Kind::IterationBinding(source, iteration) => (source, Some(iteration)),
            _ => return None,
        };
        let source_object = self.objects.get(source.checked_sub(1)?)?;
        let Kind::Image { png, .. } = &source_object.kind else {
            return None;
        };
        if source_object.deleted {
            return None;
        }
        let transforms = if let Some(iteration) = iteration {
            self.mapped_scoped_outputs(source, iteration, false, |sample, id| {
                sample.image_transform(id)
            })?
            .into_iter()
            .flatten()
            .collect()
        } else {
            vec![self.image_transform(source)?]
        };
        Some((source, png, transforms))
    }

    /// Resolve the shared pixels, placement, and live polygon of a cropped picture.
    pub fn cropped_image(&self, id: usize) -> Option<CroppedImage<'_>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::CroppedImage { source, polygon } = object.kind else {
            return None;
        };
        let source_object = self.objects.get(source.checked_sub(1)?)?;
        let Kind::Image { png, .. } = &source_object.kind else {
            return None;
        };
        if source_object.deleted {
            return None;
        }
        Some((
            source,
            png,
            self.image_transform(source)?,
            self.polygon_points(polygon)?,
        ))
    }

    pub fn can_crop_picture_to_polygon(&self, source: usize, polygon: usize) -> Result<(), String> {
        if self.partial_view || self.page_count != 1 || self.image_read_only {
            return Err("Picture cropping requires a supported single-page sketch".into());
        }
        let source_object = self
            .objects
            .get(source.checked_sub(1).ok_or("Invalid picture ID")?)
            .ok_or("Invalid picture ID")?;
        let polygon_object = self
            .objects
            .get(polygon.checked_sub(1).ok_or("Invalid polygon ID")?)
            .ok_or("Invalid polygon ID")?;
        if source_object.deleted
            || !source_object.visible
            || source_object.page != self.active_page
            || !matches!(
                source_object.kind,
                Kind::Image {
                    placement: ImagePlacement::Independent(_),
                    ..
                }
            )
        {
            return Err("Select one visible independent picture".into());
        }
        if polygon_object.deleted
            || polygon_object.page != self.active_page
            || !polygon_object.kind.is_polygon()
            || self
                .polygon_points(polygon)
                .is_none_or(|points| points.len() < 3)
        {
            return Err("Select one polygon on the same page".into());
        }
        Ok(())
    }

    pub fn crop_picture_to_polygon(
        &mut self,
        source: usize,
        polygon: usize,
    ) -> Result<usize, String> {
        self.can_crop_picture_to_polygon(source, polygon)?;
        self.checkpoint();
        self.objects[source - 1].visible = false;
        Ok(self.append_object(Kind::CroppedImage { source, polygon }))
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
        if !matches!(
            object.kind,
            Kind::Image {
                placement: ImagePlacement::Independent(_),
                ..
            }
        ) {
            return Err("Object is not a picture".into());
        }
        if checkpoint {
            self.checkpoint();
        }
        if let Kind::Image {
            placement: ImagePlacement::Independent(transform),
            ..
        } = &mut self.objects[id - 1].kind
        {
            transform[2] = at.x;
            transform[5] = at.y;
        }
        self.dirty = true;
        Ok(())
    }
}
