//! One executor for interactive changes and temporary sampling inputs.
use super::{Document, Kind, Point};
use sketchpad_ir::UpdatePlan;

pub(super) enum EvaluationInput {
    Point(usize, Point),
    Number(usize, f64),
    Kind(usize, Kind),
}

impl Document {
    pub(super) fn execute_evaluation_plan(
        &mut self,
        plan: &UpdatePlan,
        inputs: impl IntoIterator<Item = EvaluationInput>,
    ) -> Option<()> {
        self.invalidate_evaluation_ids(0, plan.order().iter().copied());
        self.clear_evaluation_seed_ids(plan.inputs());
        for input in inputs {
            match input {
                EvaluationInput::Point(id, point) => self.set_evaluation_point(id, point)?,
                EvaluationInput::Number(id, value) => self.seed_evaluation_number(id, value)?,
                EvaluationInput::Kind(id, kind) => {
                    self.objects.get_mut(id.checked_sub(1)?)?.kind = kind;
                }
            }
        }
        // SAC queries repair only the values actually demanded by consumers.
        Some(())
    }
}
