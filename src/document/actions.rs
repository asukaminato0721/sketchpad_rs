use super::kind::{is_multi_page_addition_kind, is_point_kind, straight_kind_mode};
use super::{Document, Kind, LineMode};
use crate::action_records::{
    AnimationSettings, MovementSpeed, PresentationMode, ScrollPlacement, VisibilityAction, VisibilityOptions,
};

impl Document {
    pub fn add_hide_show_button(&mut self, target: usize) -> Result<usize, String> {
        self.add_hide_show_button_for_targets(&[target])
    }
    pub fn can_add_hide_show_button_for_targets(&self, targets: &[usize]) -> Result<(), String> {
        let multi_page_append = self.page_count > 1
            && !self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Unsupported(_)))
            && self
                .objects
                .iter()
                .skip(self.source_count)
                .all(|object| is_multi_page_addition_kind(&object.kind));
        if targets.is_empty() || !multi_page_append && (self.partial_view || self.page_count != 1) {
            return Err("A Hide/Show button needs visible supported objects on this page".into());
        }
        for (index, &target) in targets.iter().enumerate() {
            let object = self
                .objects
                .get(target.checked_sub(1).ok_or("Invalid button target")?)
                .ok_or("Invalid button target")?;
            if targets[..index].contains(&target)
                || object.deleted
                || !object.visible
                || object.page != self.active_page
                || matches!(
                    object.kind,
                    Kind::HideShowButton { .. }
                        | Kind::AnimationButton { .. }
                        | Kind::MovementButton { .. }
                        | Kind::ScrollButton { .. }
                        | Kind::PresentationButton { .. }
                        | Kind::Unsupported(_)
                )
            {
                return Err(
                    "A Hide/Show button needs distinct visible supported objects on this page"
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub fn add_hide_show_button_for_targets(&mut self, targets: &[usize]) -> Result<usize, String> {
        self.can_add_hide_show_button_for_targets(targets)?;
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page
                    && !object.deleted
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        let label = if targets.len() == 1
            && matches!(self.objects[targets[0] - 1].kind, Kind::Segment(_, _))
        {
            "Hide Segment"
        } else {
            "Hide Objects"
        };
        Ok(self.add(Kind::HideShowButton {
            targets: targets.to_vec(),
            action: VisibilityAction::Hide,
            options: VisibilityOptions::default(),
            caption_flags: Some(4),
            label: label.into(),
            anchor: (10, y),
        }))
    }
    /// Create a fixed or alternating visibility action, including an explicit
    /// Show action whose targets are currently hidden.
    pub fn add_visibility_button_for_targets(
        &mut self, targets: &[usize], action: VisibilityAction, options: VisibilityOptions,
    ) -> Result<usize,String> {
        let multi_page_append = self.page_count > 1 && !self.objects.iter()
            .any(|object| matches!(object.kind,Kind::Unsupported(_))) && self.objects.iter()
            .skip(self.source_count).all(|object| is_multi_page_addition_kind(&object.kind));
        if targets.is_empty() || !multi_page_append && (self.partial_view || self.page_count != 1) {
            return Err("A visibility button needs supported objects on this page".into());
        }
        for (index,&target) in targets.iter().enumerate() {
            let object = self.objects.get(target.checked_sub(1).ok_or("Invalid button target")?)
                .ok_or("Invalid button target")?;
            if targets[..index].contains(&target) || object.deleted || object.page != self.active_page
                || matches!(object.kind,Kind::Unsupported(_)) {
                return Err("A visibility button needs distinct supported targets on this page".into());
            }
        }
        let row = self.objects.iter().filter(|object| object.page == self.active_page && !object.deleted
            && matches!(object.kind,Kind::HideShowButton{..}|Kind::AnimationButton{..}
                |Kind::MovementButton{..}|Kind::ScrollButton{..}|Kind::PresentationButton{..})).count();
        let label = self.generated_visibility_caption(targets,action);
        Ok(self.add(Kind::HideShowButton {
            targets:targets.to_vec(),action,options,caption_flags:Some(4),label,
            anchor:(10,(5usize+row*30).min(i16::MAX as usize) as i16),
        }))
    }

    pub fn set_hide_show_button_options(&mut self, id: usize, options: VisibilityOptions) -> Result<(),String> {
        let object = self.objects.get(id.checked_sub(1).ok_or("Invalid button ID")?)
            .ok_or("Invalid button ID")?;
        if object.deleted || object.page != self.active_page {
            return Err("Button is unavailable on this page".into());
        }
        let Kind::HideShowButton { options: current,.. } = &object.kind else {
            return Err("Object is not a Hide/Show button".into());
        };
        if *current == options { return Ok(()); }
        self.checkpoint();
        if let Kind::HideShowButton { options:current,.. } = &mut self.objects[id-1].kind {
            *current = options;
        }
        Ok(())
    }

    /// Native format7 is a custom name; an explicitly empty caption omits the
    /// label record. Changing the policy later still preserves these raw flags.
    pub fn set_hide_show_button_caption(&mut self, id: usize, caption: &str) -> Result<(),String> {
        if caption.len() > u16::MAX as usize {
            return Err("Button caption exceeds65535 bytes".into());
        }
        let object = self.objects.get(id.checked_sub(1).ok_or("Invalid button ID")?)
            .ok_or("Invalid button ID")?;
        if object.deleted || object.page != self.active_page {
            return Err("Button is unavailable on this page".into());
        }
        let Kind::HideShowButton { caption_flags,label,.. } = &object.kind else {
            return Err("Object is not a Hide/Show button".into());
        };
        let flags = if caption.is_empty() { None } else { Some(caption_flags.unwrap_or(0)&0xffff0000 |7) };
        if label == caption && *caption_flags == flags { return Ok(()); }
        self.checkpoint();
        if let Kind::HideShowButton { caption_flags,label,.. } = &mut self.objects[id-1].kind {
            *caption_flags = flags;
            *label = caption.into();
        }
        Ok(())
    }

    pub fn reset_hide_show_button_caption(&mut self,id: usize) -> Result<(),String> {
        let object = self.objects.get(id.checked_sub(1).ok_or("Invalid button ID")?)
            .ok_or("Invalid button ID")?;
        if object.deleted || object.page != self.active_page {
            return Err("Button is unavailable on this page".into());
        }
        let Kind::HideShowButton { targets,action,caption_flags,label,.. } = &object.kind else {
            return Err("Object is not a Hide/Show button".into());
        };
        if targets.is_empty() || targets.iter().any(|&target| target == 0 || self.objects.get(target-1)
            .is_none_or(|target| target.deleted || target.page != self.active_page)) {
            return Err("Button target is unavailable".into());
        }
        let next_label = self.generated_visibility_caption(targets,*action);
        let flags = Some(caption_flags.unwrap_or(0)&0xffff0000 |4);
        if *label == next_label && *caption_flags == flags { return Ok(()); }
        self.checkpoint();
        if let Kind::HideShowButton { caption_flags,label,.. } = &mut self.objects[id-1].kind {
            *caption_flags = flags;
            *label = next_label;
        }
        Ok(())
    }

    pub fn add_animation_button(&mut self, target: usize) -> Result<usize, String> {
        let settings = self
            .animation_button_default_settings(target)
            .ok_or("An Animation button needs an animatable point on this page")?;
        self.add_animation_button_with_settings(target, settings)
    }
    pub fn add_movement_button(
        &mut self,
        source: usize,
        destination: usize,
        speed: MovementSpeed,
        follow: bool,
    ) -> Result<usize, String> {
        self.add_movement_button_for_pairs(&[(source, destination)], speed, follow)
    }
    pub fn add_movement_button_for_pairs(
        &mut self,
        pairs: &[(usize, usize)],
        speed: MovementSpeed,
        follow: bool,
    ) -> Result<usize, String> {
        self.can_add_movement_button_for_pairs(pairs)?;
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page
                    && !object.deleted
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::MovementButton {
            pairs: pairs.to_vec(),
            speed,
            follow,
            label: "Move to Point".into(),
            anchor: (10, y),
        }))
    }
    pub fn can_add_movement_button(&self, source: usize, destination: usize) -> Result<(), String> {
        self.can_add_movement_button_for_pairs(&[(source, destination)])
    }
    pub fn can_add_movement_button_for_pairs(
        &self,
        pairs: &[(usize, usize)],
    ) -> Result<(), String> {
        let multi_page_append = self.page_count > 1
            && !self
                .objects
                .iter()
                .any(|o| matches!(o.kind, Kind::Unsupported(_)))
            && self
                .objects
                .iter()
                .skip(self.source_count)
                .all(|o| is_multi_page_addition_kind(&o.kind));
        if (!multi_page_append && (self.partial_view || self.page_count != 1)) || pairs.is_empty() {
            return Err(
                "A Movement button needs compatible source and destination pairs on this page"
                    .into(),
            );
        }
        let mut sources = std::collections::HashSet::new();
        for &(source, destination) in pairs {
            if !self.movement_pair_supported(source, destination)
                || !sources.insert(source)
                || [source, destination].iter().any(|&id| {
                    let object = &self.objects[id - 1];
                    !object.visible || object.page != self.active_page
                })
            {
                return Err("A Movement button needs distinct editable sources and compatible destinations on this page".into());
            }
        }
        Ok(())
    }
    pub fn add_scroll_button(
        &mut self,
        target: usize,
        placement: ScrollPlacement,
    ) -> Result<usize, String> {
        self.can_add_scroll_button(target)?;
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page
                    && !object.deleted
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::ScrollButton {
            target,
            placement,
            label: "Scroll".into(),
            anchor: (10, y),
        }))
    }
    pub fn can_add_scroll_button(&self, target: usize) -> Result<(), String> {
        let multi_page_append = self.page_count > 1
            && !self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Unsupported(_)))
            && self
                .objects
                .iter()
                .skip(self.source_count)
                .all(|object| is_multi_page_addition_kind(&object.kind));
        if !multi_page_append && (self.partial_view || self.page_count != 1) {
            return Err("A Scroll button needs a point on this page".into());
        }
        let object = self
            .objects
            .get(target.checked_sub(1).ok_or("Invalid Scroll target")?)
            .ok_or("Invalid Scroll target")?;
        if object.deleted
            || !object.visible
            || object.page != self.active_page
            || !is_point_kind(&object.kind)
            || self.position(target).is_none()
        {
            return Err("A Scroll button needs a visible point on this page".into());
        }
        Ok(())
    }
    pub fn add_presentation_button_for_targets(
        &mut self,
        targets: &[usize],
        mode: PresentationMode,
    ) -> Result<usize, String> {
        self.can_add_presentation_button_for_targets(targets)?;
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page && !object.deleted && object.kind.is_action_button()
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::PresentationButton {
            targets: targets.to_vec(),
            mode,
            label: "Run Presentation".into(),
            anchor: (10, y),
        }))
    }
    pub fn can_add_presentation_button_for_targets(&self, targets: &[usize]) -> Result<(), String> {
        let multi_page_append = self.page_count > 1
            && !self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Unsupported(_)))
            && self
                .objects
                .iter()
                .skip(self.source_count)
                .all(|object| is_multi_page_addition_kind(&object.kind));
        if targets.is_empty() || (!multi_page_append && (self.partial_view || self.page_count != 1))
        {
            return Err("A Presentation button needs action buttons on this page".into());
        }
        for (index, &target) in targets.iter().enumerate() {
            let object = self
                .objects
                .get(target.checked_sub(1).ok_or("Invalid button target")?)
                .ok_or("Invalid button target")?;
            if targets[..index].contains(&target)
                || object.deleted
                || !object.visible
                || object.page != self.active_page
                || !object.kind.is_action_button()
            {
                return Err("A Presentation button needs distinct visible action buttons".into());
            }
        }
        Ok(())
    }
    pub fn add_link_url_button(&mut self, url: &str) -> Result<usize, String> {
        let multi_page_append = self.page_count > 1
            && !self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Unsupported(_)))
            && self
                .objects
                .iter()
                .skip(self.source_count)
                .all(|object| is_multi_page_addition_kind(&object.kind));
        if !multi_page_append && (self.partial_view || self.page_count != 1) {
            return Err("A Link URL button needs an editable page".into());
        }
        if url.is_empty() || url.len() > u16::MAX as usize || url.as_bytes().contains(&0) {
            return Err("Link URL must be nonempty text of at most 65535 bytes".into());
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page && !object.deleted && object.kind.is_action_button()
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::LinkUrlButton {
            url: url.into(),
            label: url.into(),
            anchor: (10, y),
        }))
    }
    pub fn has_pending_multi_page_additions(&self) -> bool {
        self.page_count > 1
            && self.objects.len() > self.source_count
            && self.objects[self.source_count..]
                .iter()
                .all(|object| is_multi_page_addition_kind(&object.kind))
    }
    pub fn add_link_page_button(&mut self, target_page: usize) -> Result<usize, String> {
        self.add_link_page_button_with_target(target_page, None)
    }
    pub fn link_destination_buttons(&self, page: usize) -> Vec<usize> {
        self.objects
            .iter()
            .filter(|object| {
                object.page == page
                    && !object.deleted
                    && object.visible
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .map(|object| object.id)
            .collect()
    }
    pub fn add_link_page_button_with_target(
        &mut self,
        target_page: usize,
        target_button_ordinal: Option<usize>,
    ) -> Result<usize, String> {
        if self.page_count < 2
            || target_page >= self.page_count
            || target_page == self.active_page
            || self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Unsupported(_)))
            || self
                .objects
                .iter()
                .skip(self.source_count)
                .any(|object| !is_multi_page_addition_kind(&object.kind))
        {
            return Err("A Page Link needs a supported page and a different destination".into());
        }
        let destination_buttons = self.link_destination_buttons(target_page);
        if target_button_ordinal.is_some_and(|ordinal| {
            ordinal == 0
                || destination_buttons.get(ordinal - 1).is_none_or(|id| {
                    !matches!(self.objects[id - 1].kind, Kind::HideShowButton { .. })
                })
        }) {
            return Err("Page Link destination button is unavailable".into());
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page && !object.deleted && object.kind.is_action_button()
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::LinkPageButton {
            target_page,
            target_button_ordinal,
            label: format!("Go to Page {}", target_page + 1),
            anchor: (10, y),
        }))
    }
    pub fn animation_button_default_settings(&self, target: usize) -> Option<AnimationSettings> {
        let object = self.objects.get(target.checked_sub(1)?)?;
        let direction = match object.kind {
            Kind::Free(_) | Kind::PointOnCircle(_, _, _) | Kind::PointOnPolygon(_, _, _) => {
                crate::action_records::AnimationDirection::Forward
            }
            Kind::Parameter(_)
            | Kind::PointOnSegment(_, _)
            | Kind::PointOnRay(_, _)
            | Kind::PointOnLine(_, _)
            | Kind::PointOnArc(_, _) => crate::action_records::AnimationDirection::Bidirectional,
            _ => return None,
        };
        Some(AnimationSettings {
            direction,
            speed: if matches!(object.kind, Kind::Parameter(_)) {
                1.
            } else {
                24.
            },
            parameter: matches!(object.kind, Kind::Parameter(_))
                .then_some(crate::action_records::ParameterAnimationSettings::default()),
            ..AnimationSettings::default()
        })
    }
    pub fn add_animation_button_with_settings(
        &mut self,
        target: usize,
        settings: AnimationSettings,
    ) -> Result<usize, String> {
        self.add_animation_button_for_targets_with_settings(&[(target, settings)])
    }
    pub fn add_animation_button_for_targets_with_settings(
        &mut self,
        targets_and_settings: &[(usize, AnimationSettings)],
    ) -> Result<usize, String> {
        self.add_animation_button_for_targets_with_settings_and_label(targets_and_settings, None)
    }
    pub fn add_animation_button_for_targets_with_settings_and_label(
        &mut self,
        targets_and_settings: &[(usize, AnimationSettings)],
        label: Option<&str>,
    ) -> Result<usize, String> {
        self.can_add_animation_button_for_targets_with_settings_and_label(
            targets_and_settings,
            label,
        )?;
        let has_parameter = targets_and_settings
            .iter()
            .any(|(target, _)| matches!(self.objects[target - 1].kind, Kind::Parameter(_)));
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page
                    && !object.deleted
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .count();
        let mut y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        for object in &self.objects {
            if object.page == self.active_page && !object.deleted {
                if let Kind::Parameter(group) = &object.kind {
                    let (_, (_, bottom)) = group.bbox();
                    y = y.max(bottom.saturating_add(3));
                }
            }
        }
        if has_parameter {
            y = y.max(47);
        }
        Ok(self.add(Kind::AnimationButton {
            targets: targets_and_settings
                .iter()
                .map(|(target, _)| *target)
                .collect(),
            label: label.map(str::to_owned).unwrap_or_else(|| {
                if has_parameter {
                    "Animate Parameter".into()
                } else {
                    "Animate Point".into()
                }
            }),
            anchor: (10, y),
            settings: targets_and_settings
                .iter()
                .map(|(_, settings)| *settings)
                .collect(),
        }))
    }
    pub fn can_add_animation_button_for_targets_with_settings_and_label(
        &self,
        targets_and_settings: &[(usize, AnimationSettings)],
        label: Option<&str>,
    ) -> Result<(), String> {
        let multi_page_append = self.page_count > 1
            && !self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Unsupported(_)))
            && self
                .objects
                .iter()
                .skip(self.source_count)
                .all(|object| is_multi_page_addition_kind(&object.kind));
        if targets_and_settings.is_empty()
            || !multi_page_append && (self.partial_view || self.page_count != 1)
        {
            return Err("An Animation button needs animatable objects on this page".into());
        }
        if label.is_some_and(|value| value.is_empty() || value.len() > 64 || !value.is_ascii()) {
            return Err("Animation button label must contain 1–64 ASCII characters".into());
        }
        for (index, &(target, settings)) in targets_and_settings.iter().enumerate() {
            settings.validate()?;
            let object = self
                .objects
                .get(target.checked_sub(1).ok_or("Invalid button target")?)
                .ok_or("Invalid button target")?;
            if targets_and_settings[..index]
                .iter()
                .any(|(id, _)| *id == target)
                || object.deleted
                || object.page != self.active_page
                || !matches!(
                    object.kind,
                    Kind::Free(_)
                        | Kind::Parameter(_)
                        | Kind::PointOnSegment(_, _)
                        | Kind::PointOnRay(_, _)
                        | Kind::PointOnLine(_, _)
                        | Kind::PointOnCircle(_, _, _)
                        | Kind::PointOnArc(_, _)
                        | Kind::PointOnPolygon(_, _, _)
                )
            {
                return Err(
                    "An Animation button needs distinct animatable objects on this page".into(),
                );
            }
            if matches!(object.kind, Kind::Free(_))
                && settings.direction != crate::action_records::AnimationDirection::Forward
            {
                return Err("A free-point Animation button has no direction setting".into());
            }
            if matches!(object.kind, Kind::Parameter(_)) != settings.parameter.is_some() {
                return Err("Animation settings do not match the target kind".into());
            }
        }
        let has_parameter = targets_and_settings
            .iter()
            .any(|(target, _)| matches!(self.objects[target - 1].kind, Kind::Parameter(_)));
        if has_parameter && targets_and_settings.len() != 1 {
            return Err("Multi-target parameter Animation buttons are not supported".into());
        }
        Ok(())
    }
    pub fn activate_hide_show_button(&mut self, id: usize) -> Result<(), String> {
        self.activate_hide_show_button_impl(id, true)
    }

    /// Dispatch a visibility action referenced by another action button.
    /// Hidden children remain executable; deletion and page guards still apply.
    pub fn activate_hide_show_button_as_child(&mut self, id: usize) -> Result<(), String> {
        self.activate_hide_show_button_impl(id, false)
    }

    fn activate_hide_show_button_impl(&mut self, id: usize, require_visible: bool) -> Result<(), String> {
        let button = self.objects.get(id.checked_sub(1).ok_or("Invalid button ID")?)
            .ok_or("Invalid button ID")?;
        if button.deleted || require_visible && !button.visible || button.page != self.active_page {
            return Err("Button is not available on this page".into());
        }
        let Kind::HideShowButton { targets, action, options, caption_flags, label, .. } = &button.kind else {
            return Err("Object is not a Hide/Show button".into());
        };
        let (targets, action, options, old_caption_flags, old_label) =
            (targets.clone(), *action, *options, *caption_flags, label.clone());
        if targets.is_empty() || targets.iter().any(|&target| {
            target == 0 || self.objects.get(target - 1)
                .is_none_or(|object| object.deleted || object.page != self.active_page)
        }) {
            return Err("Button target is unavailable".into());
        }
        let next_action = if options.alternates() {
            match action {
                VisibilityAction::Hide => VisibilityAction::Show,
                VisibilityAction::Show => VisibilityAction::Hide,
            }
        } else { action };
        let (next_caption_flags, next_label) = if options.alternates() {
            if old_caption_flags.is_some_and(|flags| flags & 0xffff == 7) {
                // Native custom names keep their noun. Its explicit localized
                // Hide/Show prefix remains state-sensitive; other names stay.
                let suffix = old_label.strip_prefix("隐藏").or_else(|| old_label.strip_prefix("显示"));
                (old_caption_flags, suffix.map(|suffix| format!("{}{suffix}",visibility_action_prefix(next_action)))
                    .unwrap_or_else(|| old_label.clone()))
            } else {
                (Some(old_caption_flags.unwrap_or(0) & 0xffff0000 | 4),
                 self.generated_visibility_caption(&targets,next_action))
            }
        } else { (old_caption_flags,old_label.clone()) };
        let changed = next_action != action || next_caption_flags != old_caption_flags
            || next_label != old_label || targets.iter().any(|&target| {
                self.objects[target-1].visible != (action == VisibilityAction::Show)
            });
        if !changed {
            self.selected.clear();
            return Ok(());
        }
        self.checkpoint();
        for target in targets {
            self.objects[target-1].visible = action == VisibilityAction::Show;
        }
        if let Kind::HideShowButton { action, caption_flags, label, .. } = &mut self.objects[id-1].kind {
            *action = next_action;
            *caption_flags = next_caption_flags;
            *label = next_label;
        }
        self.selected.clear();
        Ok(())
    }

    fn generated_visibility_caption(&self, targets: &[usize], action: VisibilityAction) -> String {
        let category = |target: usize| {
            let kind = &self.objects[target-1].kind;
            if is_point_kind(kind) { "点" }
            else if let Some(mode) = straight_kind_mode(kind) { match mode {
                LineMode::Segment => "线段", LineMode::Ray => "射线", LineMode::Line => "直线",
            }}
            else if kind.is_circle() { "圆" }
            else if kind.is_arc() { "弧" }
            else if kind.is_polygon() && self.polygon_points(target).is_some_and(|points| points.len()==3) { "三角形" }
            else { "对象" }
        };
        let first = category(targets[0]);
        let noun = if targets.iter().all(|&target| category(target) == first) {
            first
        } else { "对象" };
        let name = if targets.len() == 1 {
            self.objects[targets[0]-1].label.as_ref()
                .and_then(|label| label.name_utf8().ok()).filter(|name| !name.is_empty())
        } else { None };
        match name {
            Some(name) => format!("{}{noun} {name}",visibility_action_prefix(action)),
            None => format!("{}{noun}",visibility_action_prefix(action)),
        }
    }

}

fn visibility_action_prefix(action: VisibilityAction) -> &'static str {
    match action { VisibilityAction::Hide => "隐藏", VisibilityAction::Show => "显示" }
}
