use super::kind::{is_arc_kind, is_transformed_plot_path_kind, is_transformed_plot_ray_kind, plot_path_accepts, value_plot_path_accepts, literal_plot_path_accepts};
use super::{Document, Kind, Point};
use crate::plot_points_records::PlotPointsMode;
use crate::{plot_axis_records, plot_points_records, style_records};

impl Document {
    pub fn plot_points_at(
        &mut self,
        mode: PlotPointsMode,
        coordinates: [f64; 2],
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Plot Points requires a supported coordinate frame".into());
        }
        let frame = self
            .active_coordinate_system()
            .ok_or("Define a coordinate system before plotting points")?;
        let source = self
            .objects
            .first()
            .filter(|object| {
                object.page == self.active_page && !object.deleted && object.id != frame
            })
            .ok_or("Plot Points requires a source object on this page")?
            .id;
        if coordinates.iter().any(|value| !value.is_finite()) {
            return Err("Plot coordinates must be finite".into());
        }
        let stored = match mode {
            PlotPointsMode::Cartesian => coordinates,
            PlotPointsMode::Polar => [
                coordinates[0],
                coordinates[1] * plot_points_records::DEGREES_TO_RADIANS,
            ],
        };
        let [x, y] = match mode {
            PlotPointsMode::Cartesian => stored,
            PlotPointsMode::Polar => [stored[0] * stored[1].cos(), stored[0] * stored[1].sin()],
        };
        self.graph_frame(frame)
            .and_then(|frame| frame.world(x, y))
            .ok_or("Plot coordinates are outside the supported range")?;
        let group = plot_points_records::encode_group(mode, [source as u32, frame as u32], stored)?;
        let plot = plot_points_records::decode_group(&group)?;
        let style = style_records::visual_style(&group[8..36]);
        self.checkpoint();
        let id = self.append_object(Kind::PlottedPoint(source, frame, mode, plot.stored));
        self.objects[id - 1].visual_style = style;
        Ok(id)
    }
    pub fn plot_pair_table_data(
        &mut self,
        table: usize,
        mode: PlotPointsMode,
    ) -> Result<Vec<usize>, String> {
        if self.partial_view || self.page_count != 1 {
            return Err(
                "Plot Table Data requires the verified t[2], t[1] table and coordinate system"
                    .into(),
            );
        }
        let frame = self
            .active_coordinate_system()
            .ok_or("Define a coordinate system before plotting table data")?;
        let source = self
            .objects
            .first()
            .filter(|object| !object.deleted && object.page == self.active_page)
            .ok_or("Plot Table Data requires a source object")?
            .id;
        let Some(object) = table
            .checked_sub(1)
            .and_then(|index| self.objects.get(index))
        else {
            return Err("Select the verified pair table".into());
        };
        let Kind::ValueTablePair(first, second, table_group) = &object.kind else {
            return Err("Select the verified pair table".into());
        };
        if object.deleted
            || object.page != self.active_page
            || !(1..=256).contains(&table_group.rows())
            || !matches!(self.objects.get(first - 1).map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[2]" && group.unit() == crate::table_records::NumericUnit::Scalar)
            || !matches!(self.objects.get(second - 1).map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[1]" && group.unit() == crate::table_records::NumericUnit::Scalar)
        {
            return Err("Select the supported t[2], t[1] table".into());
        }
        let rows = table_group
            .pair_values()
            .ok_or("Invalid pair table values")?;
        let mut unique = Vec::new();
        for value in rows {
            if !unique.contains(&value) {
                unique.push(value);
            }
        }
        self.checkpoint();
        let mut ids = Vec::new();
        for stored in unique {
            let stored = match mode {
                PlotPointsMode::Cartesian => stored,
                PlotPointsMode::Polar => [
                    stored[0],
                    stored[1] * plot_points_records::DEGREES_TO_RADIANS,
                ],
            };
            let group =
                plot_points_records::encode_group(mode, [source as u32, frame as u32], stored)?;
            let style = style_records::visual_style(&group[8..36]);
            let id = self.append_object(Kind::PlottedPoint(source, frame, mode, stored));
            self.objects[id - 1].visual_style = style;
            ids.push(id);
        }
        Ok(ids)
    }
    /// The numeric category accepted by Plot Value; undefined values report an
    /// error when plotting rather than opening the literal-value dialog.
    pub fn is_scalar_plot_source(&self, id: usize) -> bool {
        id.checked_sub(1).and_then(|index| self.objects.get(index)).is_some_and(|object| {
            !object.deleted && object.page == self.active_page
                && object.kind.numeric_unit() == Some(crate::table_records::NumericUnit::Scalar)
        })
    }

    /// Uses the same geometry eligibility as the plotting API at an in-range
    /// sample, including native-supported undefined collapsed arcs.
    pub fn can_plot_value_on_path(&self, id: usize) -> bool {
        let Some(object) = id.checked_sub(1).and_then(|index| self.objects.get(index)) else {
            return false;
        };
        if object.deleted || object.page != self.active_page { return false; }
        if object.kind.is_polygon() {
            return self.point_at_polygon_parameter(id, 0.5).is_some();
        }
        self.plotted_path_position(id, 0.5).is_ok()
    }

    fn plotted_path_position(&self, path: usize, value: f64) -> Result<Option<Point>, String> {
        let object = self
            .objects
            .get(path.checked_sub(1).ok_or("Invalid path")?)
            .ok_or("Invalid path")?;
        if object.page != self.active_page || object.deleted {
            return Err("Select a path on this page".into());
        }
        if is_arc_kind(&object.kind) {
            if !value.is_finite() || !self.plot_arc_host_available(path) {
                return Err("Select a valid arc and a finite value".into());
            }
            if self.undefined_arc_plot(path, value) { return Ok(None); }
            return self.plotted_arc_position(path, value).map(Some)
                .ok_or("Arc geometry is outside the supported range".into());
        }
        if is_transformed_plot_path_kind(&object.kind) && value.is_finite() {
            let (start, end, _) = self.line_points(path).ok_or("Invalid transformed path")?;
            if is_transformed_plot_ray_kind(&object.kind) && value < 0.0 {
                if start == end { return Err("Invalid transformed path".into()); }
                return Ok(None);
            }
            let point = Point { x: start.x + (end.x - start.x) * value, y: start.y + (end.y - start.y) * value };
            return (point.x.is_finite() && point.y.is_finite()).then_some(Some(point)).ok_or("Plotted value is outside the supported range".into());
        }
        if !plot_path_accepts(&object.kind, value) {
            return Err(match object.kind {
                Kind::Segment(_, _) => "Segment values must be between 0 and 1",
                Kind::Ray(_, _) => "Ray values must be zero or greater",
                _ => "Select a supported path and finite value",
            }
            .into());
        }
        if object.kind.is_circle() {
            let (center, radius) = self.circle_points(path).ok_or("Invalid circle")?;
            let angle = value * std::f64::consts::TAU;
            let point = Point {
                x: center.x + radius * angle.cos(),
                y: center.y - radius * angle.sin(),
            };
            return (point.x.is_finite() && point.y.is_finite())
                .then_some(Some(point))
                .ok_or("Plotted value is outside the supported range".into());
        }
        let (start, end) = match object.kind {
            Kind::GraphAxis(start, end, 0 | 1 | 0x1_0001)
            | Kind::Segment(start, end)
            | Kind::Line(start, end)
            | Kind::Ray(start, end) => (start, end),
            _ => return Err("Select a supported path".into()),
        };
        let start = self.position(start).ok_or("Invalid path start")?;
        let end = self.position(end).ok_or("Invalid path end")?;
        let point = Point {
            x: start.x + (end.x - start.x) * value,
            y: start.y + (end.y - start.y) * value,
        };
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err("Plotted value is outside the supported range".into());
        }
        Ok(Some(point))
    }
    pub fn plot_parameter_on_path(
        &mut self,
        source: usize,
        path: Option<usize>,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot plot a value in this sketch".into());
        }
        let path = path
            .or_else(|| self.active_x_axis(self.active_page))
            .ok_or("No coordinate axis or selected path")?;
        if !self.is_scalar_plot_source(source) {
            return Err("Select a scalar numeric value".into());
        }
        let value = self.numeric_value(source).ok_or("Numeric value is undefined")?;
        let host = self
            .objects
            .get(path.checked_sub(1).ok_or("Invalid path")?)
            .ok_or("Invalid path")?;
        if host.kind.is_polygon() {
            if host.page != self.active_page
                || host.deleted
                || !value_plot_path_accepts(&host.kind, value)
            {
                return Err("Select a polygon on this page and a finite value".into());
            }
            self.point_at_polygon_parameter(path, value)
                .ok_or("Polygon perimeter is undefined")?;
        } else {
            self.plotted_path_position(path, value)?;
        }
        let group = plot_axis_records::encode_group([source as u32, path as u32])?;
        let style = style_records::visual_style(&group[8..36]);
        self.checkpoint();
        let id = self.append_object(Kind::PathValuePoint(source, path));
        self.objects[id - 1].visual_style = style;
        Ok(id)
    }
    pub fn plot_literal_on_path(
        &mut self,
        value: f64,
        path: Option<usize>,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !value.is_finite() {
            return Err("Cannot plot that value in this sketch".into());
        }
        let path = path
            .or_else(|| self.active_x_axis(self.active_page))
            .ok_or("No coordinate axis or selected path")?;
        if self.objects.get(path.saturating_sub(1)).is_some_and(|host| host.kind.is_polygon()) {
            let host = &self.objects[path - 1];
            if host.deleted || host.page != self.active_page
                || !literal_plot_path_accepts(&host.kind, value)
                || self.point_at_polygon_parameter(path, value).is_none()
            {
                return Err("Select a defined polygon on this page".into());
            }
        } else {
            self.plotted_path_position(path, value)?;
        }
        let group = plot_axis_records::encode_literal_group(path as u32, value)?;
        let style = style_records::visual_style(&group[8..36]);
        self.checkpoint();
        let id = self.append_object(Kind::PathLiteralPoint(path, value));
        self.objects[id - 1].visual_style = style;
        Ok(id)
    }
}
