use super::{Document, Kind, Object, Point};
use crate::function_records::{FunctionPlotMode, SimpleFunction};
use crate::graph::GridForm;
use crate::plot_points_records::PlotPointsMode;
use std::collections::BTreeMap;

pub(super) fn function_sources_valid(
    expression: &SimpleFunction,
    sources: &[usize],
    objects: &[Object],
    page: usize,
) -> bool {
    use crate::table_records::CalculationSourceKind::{Function, Numeric};
    let Ok(kinds) = expression.source_kinds(sources.len()) else {
        return false;
    };
    sources.iter().zip(kinds).all(|(&id, role)| {
        let Some(object) = id.checked_sub(1).and_then(|i| objects.get(i)) else {
            return false;
        };
        !object.deleted
            && object.page == page
            && match role {
                Some(Numeric) => object.kind.numeric_unit().is_some(),
                Some(Function) => object.kind.is_function(),
                None => object.kind.numeric_unit().is_some() || object.kind.is_function(),
            }
    })
}

impl Document {
    pub fn add_function_definition(&mut self, expression: SimpleFunction) -> Result<usize, String> {
        self.add_function_definition_with_angle_unit(expression, true)
    }
    pub(super) fn function_expression(&self, id: usize) -> Option<&SimpleFunction> {
        match &self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::FunctionDefinition(expression)
            | Kind::ReferencedFunction(_, expression)
            | Kind::DerivativeFunction(_, expression) => Some(expression),
            _ => None,
        }
    }
    pub(super) fn function_symbol(&self, id: usize) -> Option<String> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        match object.kind {
            Kind::FunctionDefinition(_)
            | Kind::ReferencedFunction(_, _)
            | Kind::DataFunction { .. } => Some(
                object
                    .function_name
                    .as_deref()
                    .filter(|name| !name.is_empty())
                    .unwrap_or("f")
                    .to_owned(),
            ),
            Kind::DerivativeFunction(source, _) => {
                Some(format!("{}′", self.function_symbol(source)?))
            }
            _ => None,
        }
    }
    pub fn function_sources(&self, id: usize) -> Option<&[usize]> {
        match &self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::ReferencedFunction(sources, _) => Some(sources),
            Kind::FunctionDefinition(_)
            | Kind::DerivativeFunction(_, _)
            | Kind::DataFunction { .. } => Some(&[]),
            _ => None,
        }
    }
    pub fn function_value(&self, id: usize, argument: f64) -> Option<f64> {
        if let Some(original) = self.frozen_evaluation_document(id) {
            return original.function_value(id, argument);
        }
        let object = self.objects.get(id.checked_sub(1)?)?;
        if !argument.is_finite() || object.deleted {
            return None;
        }
        if matches!(object.kind, Kind::DataFunction { .. }) {
            return self.data_function_value(id, argument);
        }
        let sources = self.function_sources(id)?;
        let parent = |index: usize| {
            let source = *sources.get(index)?;
            let parent = self.objects.get(source.checked_sub(1)?)?;
            (source < id && !parent.deleted && parent.page == object.page).then_some(source)
        };
        let value = self.function_expression(id)?.evaluate_with_sources(
            argument,
            self.angle_unit_radians,
            &|index| {
                let source = parent(index)?;
                let value = self.numeric_value(source)?;
                Some(
                    if self.angle_unit_radians
                        && self.objects[source - 1].kind.numeric_unit()
                            == Some(crate::table_records::NumericUnit::Degrees)
                    {
                        value.to_radians()
                    } else {
                        value
                    },
                )
            },
            &|index, x| self.function_value(parent(index)?, x),
        );
        value.is_finite().then_some(value)
    }
    fn function_may_have_poles(&self, id: usize) -> bool {
        self.function_expression(id)
            .is_some_and(|expression| expression.may_have_poles())
            || self.function_sources(id).is_some_and(|sources| {
                sources.iter().any(|&source| {
                    source < id
                        && self
                            .objects
                            .get(source.saturating_sub(1))
                            .is_some_and(|o| o.kind.is_function())
                        && self.function_may_have_poles(source)
                })
            })
    }
    pub(super) fn function_value_with_unit(
        &self,
        id: usize,
        argument: f64,
        unit: crate::table_records::NumericUnit,
    ) -> Option<f64> {
        let argument =
            if unit == crate::table_records::NumericUnit::Degrees && self.angle_unit_radians {
                argument.to_radians()
            } else {
                argument
            };
        self.function_value(id, argument)
    }
    pub fn function_display_text(&self, id: usize) -> Option<String> {
        if let Kind::DataFunction { source, .. } = self.objects.get(id.checked_sub(1)?)?.kind {
            let symbol = self.function_symbol(id)?;
            return Some(format!("{symbol}(x): picture {source}"));
        }
        let expression = self.function_expression(id)?;
        let symbol = self.function_symbol(id)?;
        let text = if let Kind::ReferencedFunction(sources, _) = &self.objects[id - 1].kind {
            let names = sources
                .iter()
                .map(|&source| self.calculation_source_name(source))
                .collect::<Option<Vec<_>>>()?;
            expression.display_for_sources(&names.iter().map(String::as_str).collect::<Vec<_>>())
        } else {
            expression.display()
        };
        Some(text.replacen("f(x)", &format!("{symbol}(x)"), 1))
    }

    pub fn add_derivative_function(&mut self, source: usize) -> Result<usize, String> {
        self.add_derivative_function_with_angle_unit(source, self.angle_unit_radians)
    }
    pub fn add_derivative_function_with_angle_unit(
        &mut self,
        source: usize,
        radians: bool,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot define a derivative in this sketch".into());
        }
        let object = self
            .objects
            .get(source.saturating_sub(1))
            .ok_or("Select one function")?;
        if object.id != source || object.deleted || object.page != self.active_page {
            return Err("Select one function".into());
        }
        let expression = self
            .function_expression(source)
            .ok_or("Select one function")?;
        let derivative = expression.symbolic_derivative()?;
        let uses_trigonometry = expression.uses_trigonometry();
        self.checkpoint();
        if uses_trigonometry {
            self.angle_unit_radians = radians;
        }
        Ok(self.append_object(Kind::DerivativeFunction(source, derivative)))
    }
    pub fn add_function_definition_with_angle_unit(
        &mut self,
        expression: SimpleFunction,
        radians: bool,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot add a function to this sketch".into());
        }
        expression.source_kinds(0)?;
        self.checkpoint();
        if expression.uses_trigonometry() {
            self.angle_unit_radians = radians;
        }
        Ok(self.append_object(Kind::FunctionDefinition(expression)))
    }
    pub fn set_function_expression(
        &mut self,
        definition: usize,
        expression: SimpleFunction,
    ) -> Result<(), String> {
        self.set_function_expression_with_angle_unit(
            definition,
            expression,
            self.angle_unit_radians,
        )
    }
    pub fn set_function_expression_with_angle_unit(
        &mut self,
        definition: usize,
        expression: SimpleFunction,
        radians: bool,
    ) -> Result<(), String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot edit a function in this sketch".into());
        }
        let object = self
            .objects
            .get(definition.saturating_sub(1))
            .ok_or("Select one function definition")?;
        if object.id != definition
            || object.page != self.active_page
            || object.deleted
            || !matches!(
                object.kind,
                Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _)
            )
        {
            return Err("Select one function definition".into());
        }
        let sources = self.function_sources(definition).unwrap().to_vec();
        if !function_sources_valid(&expression, &sources, &self.objects, object.page) {
            return Err("Invalid function sources".into());
        }
        if self.function_expression(definition) == Some(&expression)
            && (!expression.uses_trigonometry() || self.angle_unit_radians == radians)
        {
            return Ok(());
        }
        let uses_trigonometry = expression.uses_trigonometry();
        let mut updates = BTreeMap::new();
        updates.insert(definition, expression.clone());
        for object in &self.objects {
            if let Kind::DerivativeFunction(source, _) = object.kind
                && let Some(source_expression) = updates.get(&source)
            {
                updates.insert(object.id, source_expression.symbolic_derivative()?);
            }
        }
        self.checkpoint();
        self.objects[definition - 1].kind = if sources.is_empty() {
            Kind::FunctionDefinition(expression)
        } else {
            Kind::ReferencedFunction(sources, expression)
        };
        for (id, expression) in updates {
            if id != definition
                && let Kind::DerivativeFunction(source, _) = self.objects[id - 1].kind
            {
                self.objects[id - 1].kind = Kind::DerivativeFunction(source, expression);
            }
        }
        if uses_trigonometry {
            self.angle_unit_radians = radians;
        }
        Ok(())
    }
    pub fn plot_new_function(
        &mut self,
        expression: SimpleFunction,
        origin: Point,
    ) -> Result<usize, String> {
        self.plot_new_function_with_angle_unit(expression, origin, true)
    }
    pub fn plot_new_function_with_angle_unit(
        &mut self,
        expression: SimpleFunction,
        origin: Point,
        radians: bool,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot plot a function in this sketch".into());
        }
        expression.source_kinds(0)?;
        let frame = match self.active_coordinate_system() {
            Some(id) => id,
            None => self.add_default_coordinate_system(origin)?,
        };
        self.checkpoint();
        if expression.uses_trigonometry() {
            self.angle_unit_radians = radians;
        }
        let definition = self.append_object(Kind::FunctionDefinition(expression));
        Ok(self.append_object(Kind::FunctionPlot(
            definition,
            frame,
            -5.265208333333335,
            5.265208333333335,
            750,
            FunctionPlotMode::YFromX,
        )))
    }
    pub fn plot_function(&mut self, definition: usize, origin: Point) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot plot a function in this sketch".into());
        }
        if !self
            .objects
            .get(definition.saturating_sub(1))
            .is_some_and(|object| {
                object.id == definition
                    && object.page == self.active_page
                    && !object.deleted
                    && self.function_expression(definition).is_some()
            })
        {
            return Err("Select one function definition".into());
        }
        let frame = match self.active_coordinate_system() {
            Some(id) => id,
            None => self.add_default_coordinate_system(origin)?,
        };
        self.checkpoint();
        Ok(self.append_object(Kind::FunctionPlot(
            definition,
            frame,
            -5.265208333333335,
            5.265208333333335,
            750,
            FunctionPlotMode::YFromX,
        )))
    }
    /// Plot ranges use public numeric values. Angle parameter curves expose
    /// degrees here while retaining native radians in their stored descriptor.
    pub fn plot_range(&self, plot: usize) -> Option<(f64, f64, u32)> {
        let object = self.objects.get(plot.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        match &object.kind {
            Kind::FunctionPlot(_, _, min, max, samples, _)
            | Kind::ParametricPlot(_, _, _, min, max, samples, _) => Some((*min, *max, *samples)),
            Kind::ParameterLocus(refs, min, max, samples) => {
                let driver = *refs.get(1)?;
                let scale =
                    super::kind::parameter_locus_driver_profile(&self.objects, driver)?.scale();
                Some((*min / scale, *max / scale, *samples))
            }
            _ => None,
        }
    }
    pub fn set_plot_range(
        &mut self,
        plot: usize,
        min: f64,
        max: f64,
        samples: u32,
    ) -> Result<(), String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot edit a plot in this sketch".into());
        }
        if !min.is_finite() || !max.is_finite() || min >= max || !(2..=4096).contains(&samples) {
            return Err("Invalid plot range or sample count".into());
        }
        let object = self
            .objects
            .get(plot.saturating_sub(1))
            .ok_or("Select one plot")?;
        if object.id != plot || object.page != self.active_page || object.deleted {
            return Err("Select one plot".into());
        }
        // Avoid a radians/degrees round trip rewriting an untouched native
        // endpoint when the user confirms the existing range.
        if self.plot_range(plot) == Some((min, max, samples)) {
            return Ok(());
        }
        let updated = match object.kind {
            Kind::FunctionPlot(definition, frame, _, _, _, mode) => {
                Kind::FunctionPlot(definition, frame, min, max, samples, mode)
            }
            Kind::ParametricPlot(x, y, frame, _, _, _, mode) => {
                Kind::ParametricPlot(x, y, frame, min, max, samples, mode)
            }
            Kind::ParameterLocus(ref refs, _, _, _) => {
                let scale = super::kind::parameter_locus_driver_profile(&self.objects, refs[1])
                    .ok_or("Parameter locus has no supported numeric driver")?
                    .scale();
                Kind::ParameterLocus(refs.clone(), min * scale, max * scale, samples)
            }
            _ => return Err("Select one function, parametric plot, or parameter locus".into()),
        };
        if object.kind == updated {
            return Ok(());
        }
        self.checkpoint();
        self.objects[plot - 1].kind = updated;
        Ok(())
    }
    pub fn function_plot_points(&self, id: usize, count: usize) -> Option<Vec<Point>> {
        let Kind::FunctionPlot(definition, frame, min, max, samples, mode) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        self.function_expression(definition)?;
        let frame = self.graph_frame(frame)?;
        let count = count.min(samples as usize).clamp(2, 4096);
        (0..count)
            .map(|i| {
                let parameter = min + (max - min) * (i as f64 / (count - 1) as f64);
                let value = self
                    .function_value(definition, parameter)
                    .unwrap_or(f64::NAN);
                let (x, y) = function_plot_coordinates(mode, parameter, value);
                let p = frame.world(x, y)?;
                Some(Point { x: p.x, y: p.y })
            })
            .collect()
    }
    pub fn function_plot_segments(&self, id: usize, count: usize) -> Option<Vec<(Point, Point)>> {
        let Kind::FunctionPlot(definition, frame, min, max, samples, mode) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        self.function_expression(definition)?;
        let frame = self.graph_frame(frame)?;
        let count = count.min(samples as usize).clamp(2, 4096);
        let sample = |index: usize| {
            let parameter = min + (max - min) * (index as f64 / (count - 1) as f64);
            let value = self
                .function_value(definition, parameter)
                .unwrap_or(f64::NAN);
            (parameter, value)
        };
        let mut segments = Vec::with_capacity(count - 1);
        let mut left = sample(0);
        for index in 1..count {
            let right = sample(index);
            let (parameter0, value0) = left;
            let (parameter1, value1) = right;
            left = right;
            if !value0.is_finite() || !value1.is_finite() {
                continue;
            }
            let low = value0.min(value1);
            let high = value0.max(value1);
            let tolerance = 1e-3 * high.abs().max(low.abs()).max(1.0);
            let crosses_gap = self.function_may_have_poles(definition)
                && [0.25, 0.5, 0.75].into_iter().any(|fraction| {
                    let parameter = parameter0 + (parameter1 - parameter0) * fraction;
                    let value = self
                        .function_value(definition, parameter)
                        .unwrap_or(f64::NAN);
                    !value.is_finite() || value < low - tolerance || value > high + tolerance
                });
            if crosses_gap {
                continue;
            }
            let (x0, y0) = function_plot_coordinates(mode, parameter0, value0);
            let (x1, y1) = function_plot_coordinates(mode, parameter1, value1);
            if let (Some(a), Some(b)) = (frame.world(x0, y0), frame.world(x1, y1)) {
                segments.push((Point { x: a.x, y: a.y }, Point { x: b.x, y: b.y }));
            }
        }
        Some(segments)
    }
    pub fn plot_parametric_curve(
        &mut self,
        x_definition: usize,
        y_definition: usize,
        mode: PlotPointsMode,
        min: f64,
        max: f64,
        origin: Point,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || x_definition == y_definition {
            return Err("Select two distinct functions".into());
        }
        for id in [x_definition, y_definition] {
            if !self
                .objects
                .get(id.saturating_sub(1))
                .is_some_and(|object| {
                    object.id == id
                        && object.page == self.active_page
                        && !object.deleted
                        && self.function_expression(id).is_some()
                })
            {
                return Err("Select two distinct functions".into());
            }
        }
        if !min.is_finite() || !max.is_finite() || min >= max {
            return Err("Invalid parametric plot domain".into());
        }
        let existing_frame = self.active_coordinate_system();
        let frame = match existing_frame {
            Some(id) => id,
            None => self.add_default_coordinate_system(origin)?,
        };
        if existing_frame.is_none() && mode == PlotPointsMode::Polar {
            self.frame_styles.insert(frame, (GridForm::Polar, false));
            self.grid_form[self.active_page] = GridForm::Polar;
        }
        self.checkpoint();
        Ok(self.append_object(Kind::ParametricPlot(
            x_definition,
            y_definition,
            frame,
            min,
            max,
            750,
            mode,
        )))
    }
    pub fn parametric_plot_segments(&self, id: usize, count: usize) -> Option<Vec<(Point, Point)>> {
        let Kind::ParametricPlot(x_id, y_id, frame_id, min, max, samples, mode) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        self.function_expression(x_id)?;
        self.function_expression(y_id)?;
        let frame = self.graph_frame(frame_id)?;
        let sample = |t: f64| {
            let x = self.function_value(x_id, t).unwrap_or(f64::NAN);
            let y = self.function_value(y_id, t).unwrap_or(f64::NAN);
            match mode {
                PlotPointsMode::Cartesian => (x, y),
                PlotPointsMode::Polar => (x * y.cos(), x * y.sin()),
            }
        };
        let count = count.min(samples as usize).clamp(2, 4096);
        let mut segments = Vec::with_capacity(count - 1);
        let mut left = (min, sample(min));
        for index in 1..count {
            let t = min + (max - min) * (index as f64 / (count - 1) as f64);
            let right = (t, sample(t));
            let ((t0, (x0, y0)), (t1, (x1, y1))) = (left, right);
            left = right;
            if ![x0, y0, x1, y1].into_iter().all(f64::is_finite) {
                continue;
            }
            let tolerance = 1e-3
                * [x0, y0, x1, y1]
                    .into_iter()
                    .map(f64::abs)
                    .fold(1.0, f64::max);
            let crosses_gap = (self.function_may_have_poles(x_id)
                || self.function_may_have_poles(y_id))
                && [0.25, 0.5, 0.75].into_iter().any(|fraction| {
                    let (x, y) = sample(t0 + (t1 - t0) * fraction);
                    !x.is_finite()
                        || !y.is_finite()
                        || x < x0.min(x1) - tolerance
                        || x > x0.max(x1) + tolerance
                        || y < y0.min(y1) - tolerance
                        || y > y0.max(y1) + tolerance
                });
            if crosses_gap {
                continue;
            }
            if let (Some(a), Some(b)) = (frame.world(x0, y0), frame.world(x1, y1)) {
                segments.push((Point { x: a.x, y: a.y }, Point { x: b.x, y: b.y }));
            }
        }
        Some(segments)
    }
}

impl Document {
    pub fn add_referenced_function_definition(
        &mut self,
        sources: &[usize],
        expression: SimpleFunction,
    ) -> Result<usize, String> {
        self.add_referenced_function_definition_with_angle_unit(sources, expression, true)
    }
    fn validate_new_function_sources(
        &self,
        sources: &[usize],
        expression: &SimpleFunction,
    ) -> Result<(), String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot add a function in this sketch safely".into());
        }
        if sources.is_empty()
            || !function_sources_valid(expression, sources, &self.objects, self.active_page)
        {
            return Err("Select valid numeric or function sources".into());
        }
        crate::function_records::encode_referenced_definition_group(sources, expression)?;
        Ok(())
    }
    pub fn add_referenced_function_definition_with_angle_unit(
        &mut self,
        sources: &[usize],
        expression: SimpleFunction,
        radians: bool,
    ) -> Result<usize, String> {
        self.validate_new_function_sources(sources, &expression)?;
        self.checkpoint();
        if expression.uses_trigonometry() {
            self.angle_unit_radians = radians;
        }
        Ok(self.append_object(Kind::ReferencedFunction(sources.to_vec(), expression)))
    }
    pub fn plot_new_referenced_function(
        &mut self,
        sources: &[usize],
        expression: SimpleFunction,
        origin: Point,
    ) -> Result<usize, String> {
        self.plot_new_referenced_function_with_angle_unit(sources, expression, origin, true)
    }
    pub fn plot_new_referenced_function_with_angle_unit(
        &mut self,
        sources: &[usize],
        expression: SimpleFunction,
        origin: Point,
        radians: bool,
    ) -> Result<usize, String> {
        self.validate_new_function_sources(sources, &expression)?;
        if !origin.x.is_finite() || !origin.y.is_finite() {
            return Err("Plot origin must be finite".into());
        }
        let frame = match self.active_coordinate_system() {
            Some(frame) => {
                self.checkpoint();
                frame
            }
            // The frame constructor checkpoints the state before the entire command.
            None => self.add_default_coordinate_system(origin)?,
        };
        if expression.uses_trigonometry() {
            self.angle_unit_radians = radians;
        }
        let definition = self.append_object(Kind::ReferencedFunction(sources.to_vec(), expression));
        Ok(self.append_object(Kind::FunctionPlot(
            definition,
            frame,
            -5.265208333333335,
            5.265208333333335,
            750,
            FunctionPlotMode::YFromX,
        )))
    }
}

pub(super) fn function_plot_coordinates(
    mode: FunctionPlotMode,
    parameter: f64,
    value: f64,
) -> (f64, f64) {
    match mode {
        FunctionPlotMode::YFromX => (parameter, value),
        FunctionPlotMode::XFromY => (value, parameter),
        FunctionPlotMode::Polar => (value * parameter.cos(), value * parameter.sin()),
    }
}
