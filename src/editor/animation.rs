use super::{AnimationButtonDraft, AnimationTrack, Editor};
use sketchpad_rs::action_records::{AnimationDirection, AnimationSettings, MovementSpeed};
use sketchpad_rs::document::{Kind, LineMode, Point};
use sketchpad_rs::render::TraceMark;
use sketchpad_rs::style_records::{RgbColor, VisualStyle};
use std::collections::BTreeSet;
use std::time::Instant;

impl Editor {
    pub(super) fn capture_traces(&mut self) {
        for object in self
            .doc
            .objects
            .iter()
            .filter(|o| o.tracing && o.visible && !o.deleted && o.page == self.doc.active_page)
        {
            let color = match object.visual_style {
                Some(
                    VisualStyle::Point { color, .. }
                    | VisualStyle::Stroke { color, .. }
                    | VisualStyle::Polygon { color, .. },
                ) => color,
                _ => RgbColor {
                    red: 23,
                    green: 107,
                    blue: 214,
                },
            };
            if let Some(point) = self.doc.position(object.id) {
                self.trace_marks.push(TraceMark::Point(point, color));
            } else if let Some((a, b, mode)) = self.doc.line_points(object.id) {
                self.trace_marks
                    .push(TraceMark::Straight(a, b, mode, color));
            } else if let Some((center, radius)) = self.doc.circle_points(object.id) {
                self.trace_marks
                    .push(TraceMark::Circle(center, radius, color));
            } else if let Some(points) = self.doc.arc_points(object.id, 128) {
                self.trace_marks.push(TraceMark::Arc(points, color));
            } else if let Some(points) = self.doc.polygon_points(object.id) {
                self.trace_marks.push(TraceMark::Polygon(points, color));
            }
        }
        if self.trace_marks.len() > 8192 {
            self.trace_marks.drain(..self.trace_marks.len() - 8192);
        }
    }
    fn next_animation_random(state: &mut u64) -> f64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        (*state >> 11) as f64 / (1u64 << 53) as f64
    }
    fn straight_animation_range(&self, host: usize, mode: LineMode, t: f64) -> Option<(f64, f64)> {
        let (a, b, actual_mode) = self.doc.line_points(host)?;
        if actual_mode != mode {
            return None;
        }
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        if dx * dx + dy * dy < 1e-12 {
            return None;
        }
        let fallback = match mode {
            LineMode::Ray => (0., t.max(1.)),
            LineMode::Line => (t.min(-1.), t.max(2.)),
            LineMode::Segment => (0., 1.),
        };
        let Some((corner_a, corner_b)) = self.animation_bounds else {
            return Some(fallback);
        };
        let margin = 10. / f64::from(self.scale);
        let mut low = if mode == LineMode::Ray {
            0.
        } else {
            f64::NEG_INFINITY
        };
        let mut high = f64::INFINITY;
        for (origin, delta, edge_a, edge_b) in [
            (a.x, dx, corner_a.x, corner_b.x),
            (a.y, dy, corner_a.y, corner_b.y),
        ] {
            let min = edge_a.min(edge_b) + margin;
            let max = edge_a.max(edge_b) - margin;
            if min >= max {
                return Some(fallback);
            }
            if delta.abs() < 1e-12 {
                if origin < min || origin > max {
                    return Some(fallback);
                }
            } else {
                let start = (min - origin) / delta;
                let end = (max - origin) / delta;
                low = low.max(start.min(end));
                high = high.min(start.max(end));
            }
        }
        if !low.is_finite() || !high.is_finite() || high - low < 1e-9 {
            Some(fallback)
        } else {
            Some((low.min(t), high.max(t)))
        }
    }
    fn animation_track(&self, id: usize, allow_hidden: bool) -> Option<AnimationTrack> {
        let object = self.doc.objects.get(id.checked_sub(1)?)?;
        if object.deleted || !allow_hidden && !object.visible || object.page != self.doc.active_page
        {
            return None;
        }
        if let Kind::Free(point) = object.kind {
            let mut random_state = ((id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                ^ point.x.to_bits().rotate_left(17)
                ^ point.y.to_bits().rotate_left(41))
                | 1;
            let angle = Self::next_animation_random(&mut random_state) * std::f64::consts::TAU;
            return Some(AnimationTrack {
                id,
                allow_hidden,
                phase: 0.,
                straight_range: None,
                parameter_domain: None,
                discrete_remainder: 0.,
                direction: 1.,
                free_velocity: Some(Point {
                    x: angle.cos(),
                    y: angle.sin(),
                }),
                free_turn_remaining: 0.35,
                random_state,
                button_settings: None,
                start_phase: 0.,
                distance_travelled: 0.,
            });
        }
        if let Kind::Parameter(group) = &object.kind {
            let (min, max) = group.animation_domain();
            let phase = (group.value() - min) / (max - min);
            if !min.is_finite() || !max.is_finite() || max <= min || !phase.is_finite() {
                return None;
            }
            return Some(AnimationTrack {
                id,
                allow_hidden,
                phase: phase.clamp(0., 1.),
                straight_range: None,
                parameter_domain: Some((min, max)),
                discrete_remainder: 0.,
                direction: 1.,
                free_velocity: None,
                free_turn_remaining: 0.,
                random_state: ((id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ phase.to_bits())
                    | 1,
                button_settings: None,
                start_phase: phase.clamp(0., 1.),
                distance_travelled: 0.,
            });
        }
        let (phase, straight_range) = match object.kind {
            Kind::PointOnSegment(host, t)
                if matches!(self.doc.line_points(host), Some((_, _, LineMode::Segment))) =>
            {
                (t, None)
            }
            Kind::PointOnRay(host, t) => {
                let range = self.straight_animation_range(host, LineMode::Ray, t)?;
                ((t - range.0) / (range.1 - range.0), Some(range))
            }
            Kind::PointOnLine(host, t) => {
                let range = self.straight_animation_range(host, LineMode::Line, t)?;
                ((t - range.0) / (range.1 - range.0), Some(range))
            }
            Kind::PointOnCircle(host, x, y) if self.doc.circle_points(host).is_some() => (
                y.atan2(x).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU,
                None,
            ),
            Kind::PointOnArc(host, t) if self.doc.point_on_arc(host, t).is_some() => (t, None),
            Kind::PointOnPolygon(host, edge, t) => {
                (self.doc.polygon_boundary_parameter(host, edge, t)?, None)
            }
            _ => return None,
        };
        phase.is_finite().then_some(AnimationTrack {
            id,
            allow_hidden,
            phase: phase.clamp(0., 1.),
            straight_range,
            parameter_domain: None,
            discrete_remainder: 0.,
            direction: 1.,
            free_velocity: None,
            free_turn_remaining: 0.,
            random_state: ((id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ phase.to_bits()) | 1,
            button_settings: None,
            start_phase: phase.clamp(0., 1.),
            distance_travelled: 0.,
        })
    }
    fn animate_targets(&mut self, targets: &[usize], show_controller: bool) {
        let added = targets
            .iter()
            .copied()
            .filter(|id| !self.animations.iter().any(|track| track.id == *id))
            .filter_map(|id| self.animation_track(id, true))
            .collect::<Vec<_>>();
        if added.is_empty() {
            self.status = "Select a parameter or a free or path-constrained point".into();
            return;
        }
        if self.animations.is_empty() {
            self.doc.checkpoint();
        }
        self.status = format!("Animating {} object(s)", added.len());
        self.animations.extend(added);
        self.animation_paused = false;
        self.animation_last_tick = Instant::now();
        self.show_motion_controller |= show_controller;
    }
    fn collect_animation_sources(
        &self,
        id: usize,
        indirect: bool,
        visited: &mut BTreeSet<usize>,
        sources: &mut Vec<usize>,
    ) {
        if !visited.insert(id) {
            return;
        }
        let Some(object) = id
            .checked_sub(1)
            .and_then(|index| self.doc.objects.get(index))
        else {
            return;
        };
        if object.deleted || !indirect && !object.visible || object.page != self.doc.active_page {
            return;
        }
        if self.animation_track(id, indirect).is_some() {
            sources.push(id);
            return;
        }
        if object.kind.is_action_button()
            || object.kind.is_measurement()
            || matches!(
                object.kind,
                Kind::ValueTable(_, _) | Kind::ValueTablePair(_, _, _)
            )
        {
            return;
        }
        for parent in object.kind.dependencies() {
            self.collect_animation_sources(parent, true, visited, sources);
        }
    }
    pub(super) fn animation_sources(&self, selected: &[usize]) -> Vec<usize> {
        let mut visited = BTreeSet::new();
        let mut sources = Vec::new();
        for &id in selected {
            self.collect_animation_sources(id, false, &mut visited, &mut sources);
        }
        sources
    }
    pub(super) fn animation_button_draft(&self) -> Option<AnimationButtonDraft> {
        let source_kind = match self.selection_order.as_slice() {
            [id] => self
                .doc
                .objects
                .get(id.checked_sub(1)?)
                .map(|object| &object.kind),
            _ => None,
        };
        let mut targets = self.animation_sources(&self.selection_order);
        let label = source_kind
            .and_then(Kind::animation_button_source_label)
            .map(str::to_owned);
        if label.is_some() {
            targets.reverse();
        }
        if targets.is_empty() {
            return None;
        }
        let settings = targets
            .iter()
            .map(|&id| self.doc.animation_button_default_settings(id))
            .collect::<Option<Vec<_>>>()?;
        Some(AnimationButtonDraft {
            targets,
            settings,
            selected: 0,
            label,
        })
    }
    pub(super) fn animate_selected(&mut self) {
        let selected = self.doc.selected.iter().copied().collect::<Vec<_>>();
        let sources = self.animation_sources(&selected);
        self.animate_targets(&sources, true);
    }
    pub(super) fn animate_button_targets(
        &mut self,
        targets: &[usize],
        settings: &[AnimationSettings],
    ) {
        if targets
            .iter()
            .any(|target| self.animations.iter().any(|track| track.id == *target))
        {
            self.animations.retain(|track| !targets.contains(&track.id));
            self.status = "Stopped animation".into();
            return;
        }
        self.animate_targets(targets, false);
        for (&target, &setting) in targets.iter().zip(settings) {
            if let Some(track) = self.animations.iter_mut().find(|track| track.id == target) {
                track.button_settings = Some(setting);
                track.direction = if setting.direction == AnimationDirection::Backward {
                    -1.
                } else {
                    1.
                };
                if let Some(parameter) = setting.parameter {
                    let (min, max) = (f64::from(parameter.min), f64::from(parameter.max));
                    if let Kind::Parameter(group) = &self.doc.objects[target - 1].kind {
                        track.parameter_domain = Some((min, max));
                        track.phase = ((group.value() - min) / (max - min)).clamp(0., 1.);
                        track.start_phase = track.phase;
                    }
                }
            }
        }
    }
    pub(super) fn tick_movement(&mut self, elapsed_seconds: f64) {
        if self.movements.is_empty() || !elapsed_seconds.is_finite() {
            return;
        }
        self.capture_traces();
        let mut tracks = std::mem::take(&mut self.movements);
        tracks.retain(|track| {
            let Some(current) = self.doc.position(track.source) else {
                return false;
            };
            let Some(target) = track
                .initial_destination
                .or_else(|| self.doc.position(track.destination))
            else {
                return false;
            };
            let delta_x = target.x - current.x;
            let delta_y = target.y - current.y;
            let distance = delta_x.hypot(delta_y);
            if !distance.is_finite() {
                return false;
            }
            let pixels_per_second = match track.speed {
                MovementSpeed::Slow => 60.,
                MovementSpeed::Medium => 150.,
                MovementSpeed::Fast => 360.,
                MovementSpeed::Instant => f64::INFINITY,
            };
            let step = pixels_per_second * elapsed_seconds.clamp(0., 0.1) / f64::from(self.scale);
            let fraction = if distance <= step || distance < 1e-9 {
                1.
            } else {
                step / distance
            };
            let moved = self.doc.move_point(
                track.source,
                Point {
                    x: current.x + delta_x * fraction,
                    y: current.y + delta_y * fraction,
                },
            );
            moved && fraction < 1.
        });
        self.movements = tracks;
    }
    pub(super) fn stop_animation(&mut self) {
        self.animations.clear();
        self.movements.clear();
        self.animation_paused = false;
        self.status = "Stopped animation".into();
    }
    pub(super) fn tick_animation(&mut self, elapsed_seconds: f64) {
        if self.animations.is_empty() || self.animation_paused || !elapsed_seconds.is_finite() {
            return;
        }
        let base_step = elapsed_seconds.clamp(0., 0.1) * self.animation_speed / 2.;
        if base_step == 0. {
            return;
        }
        self.capture_traces();
        let mut tracks = std::mem::take(&mut self.animations);
        tracks.retain_mut(|track| {
            let Some(object) = self.doc.objects.get(track.id - 1) else {
                return false;
            };
            if object.deleted
                || !track.allow_hidden && !object.visible
                || object.page != self.doc.active_page
            {
                return false;
            }
            let kind = &object.kind;
            let speed_scale = track
                .button_settings
                .map_or(1., |settings| f64::from(settings.speed) / 24.);
            let step = if let Some((min, max)) = track.parameter_domain {
                let amount = elapsed_seconds.clamp(0., 0.1)
                    * self.animation_speed
                    * track
                        .button_settings
                        .map_or(1., |settings| f64::from(settings.speed));
                let amount = if let Some(parameter) = track
                    .button_settings
                    .and_then(|settings| settings.parameter)
                    .filter(|parameter| parameter.discrete)
                {
                    track.discrete_remainder += amount;
                    let count = (track.discrete_remainder + 1e-9).floor();
                    track.discrete_remainder = (track.discrete_remainder - count).max(0.);
                    count * f64::from(parameter.step)
                } else {
                    amount
                };
                amount / (max - min)
            } else {
                base_step * speed_scale
            };
            if let Some(mut velocity) = track.free_velocity {
                let Kind::Free(current) = *kind else {
                    return false;
                };
                let elapsed = elapsed_seconds.clamp(0., 0.1);
                track.free_turn_remaining -= elapsed * self.animation_speed * speed_scale;
                if track.free_turn_remaining <= 0. {
                    let angle = Self::next_animation_random(&mut track.random_state)
                        * std::f64::consts::TAU;
                    velocity = Point {
                        x: angle.cos(),
                        y: angle.sin(),
                    };
                    track.free_turn_remaining +=
                        0.35 + 0.45 * Self::next_animation_random(&mut track.random_state);
                }
                let distance =
                    90. * elapsed * self.animation_speed * speed_scale / f64::from(self.scale);
                let mut destination = Point {
                    x: current.x + velocity.x * distance,
                    y: current.y + velocity.y * distance,
                };
                if let Some((min, max)) = self.animation_bounds {
                    let margin = 10. / f64::from(self.scale);
                    let left = min.x.min(max.x) + margin;
                    let right = min.x.max(max.x) - margin;
                    let top = min.y.min(max.y) + margin;
                    let bottom = min.y.max(max.y) - margin;
                    if left < right && top < bottom {
                        if destination.x < left {
                            destination.x = left;
                            velocity.x = velocity.x.abs();
                        } else if destination.x > right {
                            destination.x = right;
                            velocity.x = -velocity.x.abs();
                        }
                        if destination.y < top {
                            destination.y = top;
                            velocity.y = velocity.y.abs();
                        } else if destination.y > bottom {
                            destination.y = bottom;
                            velocity.y = -velocity.y.abs();
                        }
                    }
                }
                track.free_velocity = Some(velocity);
                let moved = self.doc.move_point(track.id, destination);
                return moved && !track.button_settings.is_some_and(|settings| settings.once);
            }
            if track.parameter_domain.is_some() && step == 0. {
                return true;
            }
            let circular = matches!(
                kind,
                Kind::PointOnCircle(_, _, _) | Kind::PointOnPolygon(_, _, _)
            );
            match track.button_settings.map(|settings| settings.direction) {
                Some(AnimationDirection::Forward) => {
                    track.phase = (track.phase + step).rem_euclid(1.);
                    track.distance_travelled += step;
                }
                Some(AnimationDirection::Backward) => {
                    track.phase = (track.phase - step).rem_euclid(1.);
                    track.distance_travelled += step;
                }
                Some(AnimationDirection::Random) => {
                    track.phase = Self::next_animation_random(&mut track.random_state);
                }
                Some(AnimationDirection::Bidirectional) | None => {
                    track.phase += track.direction * step;
                    if circular {
                        track.phase = track.phase.rem_euclid(1.);
                    } else if track.phase > 1. {
                        track.phase = 2. - track.phase;
                        track.direction = -1.;
                    } else if track.phase < 0. {
                        track.phase = -track.phase;
                        track.direction = 1.;
                    }
                }
            }
            let once_complete = track.button_settings.is_some_and(|settings| {
                settings.once
                    && (settings.direction == AnimationDirection::Random
                        || track.distance_travelled >= 1.)
            });
            if once_complete
                && !matches!(
                    track.button_settings,
                    Some(AnimationSettings {
                        direction: AnimationDirection::Random,
                        ..
                    })
                )
            {
                track.phase = track.start_phase;
            }
            if let Kind::Parameter(_) = kind {
                let Some((min, max)) = track.parameter_domain else {
                    return false;
                };
                let value = min + (max - min) * track.phase;
                let moved = self.doc.animate_parameter_value(track.id, value).is_ok();
                return moved && !once_complete;
            }
            let destination = match *kind {
                Kind::PointOnSegment(host, _) => self
                    .doc
                    .line_points(host)
                    .filter(|(_, _, mode)| *mode == LineMode::Segment)
                    .map(|(a, b, _)| Point {
                        x: a.x + (b.x - a.x) * track.phase,
                        y: a.y + (b.y - a.y) * track.phase,
                    }),
                Kind::PointOnRay(host, _) | Kind::PointOnLine(host, _) => {
                    self.doc.line_points(host).and_then(|(a, b, _)| {
                        track.straight_range.map(|(min, max)| {
                            let t = min + (max - min) * track.phase;
                            Point {
                                x: a.x + (b.x - a.x) * t,
                                y: a.y + (b.y - a.y) * t,
                            }
                        })
                    })
                }
                Kind::PointOnCircle(host, _, _) => self.doc.circle_points(host).map(|(c, r)| {
                    let angle = track.phase * std::f64::consts::TAU;
                    Point {
                        x: c.x + r * angle.cos(),
                        y: c.y - r * angle.sin(),
                    }
                }),
                Kind::PointOnArc(host, _) => self.doc.point_on_arc(host, track.phase),
                Kind::PointOnPolygon(host, _, _) => {
                    self.doc.point_at_polygon_parameter(host, track.phase)
                }
                _ => None,
            };
            let moved = destination.is_some_and(|point| self.doc.move_point(track.id, point));
            moved && !once_complete
        });
        self.animations = tracks;
    }
    pub(super) fn tick_animation_clock(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.animation_last_tick).as_secs_f64();
        self.animation_last_tick = now;
        self.tick_animation(elapsed);
        self.tick_movement(elapsed);
        self.tick_presentations();
    }
}
