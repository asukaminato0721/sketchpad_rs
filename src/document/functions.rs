use super::{Document, Kind, Point};
use crate::function_records::SimpleFunction;
use crate::graph::GridForm;
use crate::plot_points_records::PlotPointsMode;
use std::collections::BTreeMap;

impl Document {
    pub fn add_function_definition(&mut self, expression: SimpleFunction) -> Result<usize, String> {
        self.add_function_definition_with_angle_unit(expression, true)
    }
    pub(super) fn function_expression(&self, id: usize) -> Option<SimpleFunction> {
        match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::FunctionDefinition(expression) | Kind::DerivativeFunction(_, expression) => {
                Some(expression)
            }
            _ => None,
        }
    }
    fn function_symbol(&self, id: usize) -> Option<String> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        match object.kind {
            Kind::FunctionDefinition(_) => Some(
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
    pub fn function_display_text(&self, id: usize) -> Option<String> {
        let expression = self.function_expression(id)?;
        let symbol = self.function_symbol(id)?;
        Some(
            expression
                .display()
                .replacen("f(x)", &format!("{symbol}(x)"), 1),
        )
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
        self.checkpoint();
        if expression.uses_trigonometry() {
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
            || !matches!(object.kind, Kind::FunctionDefinition(_))
        {
            return Err("Select one function definition".into());
        }
        if object.kind == Kind::FunctionDefinition(expression)
            && (!expression.uses_trigonometry() || self.angle_unit_radians == radians)
        {
            return Ok(());
        }
        let mut updates = BTreeMap::new();
        updates.insert(definition, expression);
        for object in &self.objects {
            if let Kind::DerivativeFunction(source, _) = object.kind
                && let Some(source_expression) = updates.get(&source).copied()
            {
                updates.insert(object.id, source_expression.symbolic_derivative()?);
            }
        }
        self.checkpoint();
        self.objects[definition - 1].kind = Kind::FunctionDefinition(expression);
        for (id, expression) in updates {
            if id != definition
                && let Kind::DerivativeFunction(source, _) = self.objects[id - 1].kind
            {
                self.objects[id - 1].kind = Kind::DerivativeFunction(source, expression);
            }
        }
        if expression.uses_trigonometry() {
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
                    && matches!(
                        object.kind,
                        Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)
                    )
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
        )))
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
        let updated = match object.kind {
            Kind::FunctionPlot(definition, frame, _, _, _) => {
                Kind::FunctionPlot(definition, frame, min, max, samples)
            }
            Kind::ParametricPlot(x, y, frame, _, _, _, mode) => {
                Kind::ParametricPlot(x, y, frame, min, max, samples, mode)
            }
            Kind::ParameterLocus(ref refs, _, _, _) => Kind::ParameterLocus(refs.clone(), min, max, samples),
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
        let Kind::FunctionPlot(definition, frame, min, max, samples) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let expression = self.function_expression(definition)?;
        let frame = self.graph_frame(frame)?;
        let count = count.min(samples as usize).clamp(2, 4096);
        (0..count)
            .map(|i| {
                let x = min + (max - min) * (i as f64 / (count - 1) as f64);
                let p = frame.world(
                    x,
                    expression.evaluate_with_angle_unit(x, self.angle_unit_radians),
                )?;
                Some(Point { x: p.x, y: p.y })
            })
            .collect()
    }
    pub fn function_plot_segments(&self, id: usize, count: usize) -> Option<Vec<(Point, Point)>> {
        let Kind::FunctionPlot(definition, frame, min, max, samples) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let expression = self.function_expression(definition)?;
        let frame = self.graph_frame(frame)?;
        let count = count.min(samples as usize).clamp(2, 4096);
        let sample = |index: usize| {
            let x = min + (max - min) * (index as f64 / (count - 1) as f64);
            (
                x,
                expression.evaluate_with_angle_unit(x, self.angle_unit_radians),
            )
        };
        let mut segments = Vec::with_capacity(count - 1);
        let mut left = sample(0);
        for index in 1..count {
            let right = sample(index);
            let (x0, y0) = left;
            let (x1, y1) = right;
            left = right;
            if !y0.is_finite() || !y1.is_finite() {
                continue;
            }
            let low = y0.min(y1);
            let high = y0.max(y1);
            let tolerance = 1e-3 * high.abs().max(low.abs()).max(1.0);
            let crosses_gap = expression.may_have_poles()
                && [0.25, 0.5, 0.75].into_iter().any(|fraction| {
                    let x = x0 + (x1 - x0) * fraction;
                    let y = expression.evaluate_with_angle_unit(x, self.angle_unit_radians);
                    !y.is_finite() || y < low - tolerance || y > high + tolerance
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
                        && matches!(
                            object.kind,
                            Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)
                        )
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
        let x_function = self.function_expression(x_id)?;
        let y_function = self.function_expression(y_id)?;
        let frame = self.graph_frame(frame_id)?;
        let sample = |t: f64| {
            let x = x_function.evaluate_with_angle_unit(t, self.angle_unit_radians);
            let y = y_function.evaluate_with_angle_unit(t, self.angle_unit_radians);
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
            let crosses_gap = (x_function.may_have_poles() || y_function.may_have_poles())
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
