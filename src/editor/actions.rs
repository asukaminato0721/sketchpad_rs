use super::{Editor, MovementTrack, PendingAction, PresentationTrack};
use sketchpad_rs::action_records::{MovementSpeed, PresentationMode, ScrollPlacement};
use sketchpad_rs::document::{Kind, Point};
use sketchpad_rs::render::Viewport;
use std::process::Command;
use std::time::Instant;

impl Editor {
    pub(super) fn activate_movement_button(&mut self, id: usize) {
        if self.movements.iter().any(|track| track.button == id) {
            self.movements.retain(|track| track.button != id);
            self.status = "Stopped movement".into();
            return;
        }
        let Kind::MovementButton {
            pairs,
            speed,
            follow,
            ..
        } = self.doc.objects[id - 1].kind.clone()
        else {
            return;
        };
        if pairs.is_empty()
            || pairs
                .iter()
                .any(|&(s, d)| !self.doc.movement_pair_supported(s, d))
        {
            self.status = "Movement source or destination is unavailable".into();
            return;
        }
        let targets = pairs
            .iter()
            .map(|&(_, d)| self.doc.movement_target_value(d).unwrap())
            .collect::<Vec<_>>();
        self.doc.checkpoint();
        if speed == MovementSpeed::Instant {
            // Native applies pairs from last to first. Initial mode freezes
            // all destinations before any source changes. Numeric destinations
            // are sampled together even in Follow mode.
            for (index, &(source, destination)) in pairs.iter().enumerate().rev() {
                let target = if follow
                    && matches!(
                        targets[index],
                        sketchpad_rs::document::MovementValue::Point(_)
                    ) {
                    self.doc.movement_target_value(destination)
                } else {
                    Some(targets[index])
                };
                if let Some(target) = target {
                    self.doc.set_movement_value(source, target);
                }
            }
            self.status = "Moved objects to destinations".into();
        } else {
            for (index, &(source, destination)) in pairs.iter().enumerate() {
                self.movements.push(MovementTrack {
                    button: id,
                    source,
                    destination,
                    initial_destination: (!follow).then_some(targets[index]),
                    speed,
                });
            }
            self.animation_last_tick = Instant::now();
            self.status = "Moving objects to destinations".into();
        }
    }
    pub(super) fn scroll_to_button_target(
        &mut self,
        target: usize,
        placement: ScrollPlacement,
        viewport: &Viewport,
    ) -> Result<(), String> {
        let point = self
            .doc
            .position(target)
            .ok_or("Scroll target is unavailable")?;
        self.center = match placement {
            ScrollPlacement::Center => point,
            ScrollPlacement::UpperLeft => Point {
                x: point.x + f64::from(viewport.rect.width()) / (2. * f64::from(self.scale)),
                y: point.y + f64::from(viewport.rect.height()) / (2. * f64::from(self.scale)),
            },
        };
        self.status = format!("Scrolled to point {target}");
        Ok(())
    }

    pub(super) fn activate_link_url(&mut self, url: &str) -> Result<(), String> {
        if let Some(relative) = url.strip_prefix("sketchdoc://") {
            if relative.is_empty() || std::path::Path::new(relative).is_absolute() {
                return Err("Invalid sketchdoc link path".into());
            }
            let base = self
                .doc
                .path
                .as_ref()
                .and_then(|path| path.parent())
                .ok_or("Save this sketch before following its document link")?;
            let path = base.join(relative);
            if self.doc.dirty {
                self.pending_action = Some(PendingAction::Open(path));
            } else {
                self.open(path);
            }
            return Ok(());
        }
        if !["http://", "https://", "file://"]
            .iter()
            .any(|prefix| url.starts_with(prefix))
        {
            return Err("This Link URL scheme is not yet supported".into());
        }
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = Command::new("rundll32");
            command.arg("url.dll,FileProtocolHandler");
            command
        };
        #[cfg(target_os = "macos")]
        let mut command = Command::new("open");
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let mut command = Command::new("xdg-open");
        command
            .arg(url)
            .spawn()
            .map_err(|error| format!("Could not open link: {error}"))?;
        self.status = format!("Opened link {url}");
        Ok(())
    }

    pub(super) fn activate_link_page(
        &mut self,
        target_page: usize,
        target_button_ordinal: Option<usize>,
        viewport: &Viewport,
        depth: usize,
    ) -> Result<(), String> {
        if target_page >= self.doc.page_count {
            return Err("Link destination page is unavailable".into());
        }
        let destination_button = target_button_ordinal
            .map(|ordinal| {
                self.doc
                    .link_destination_buttons(target_page)
                    .get(
                        ordinal
                            .checked_sub(1)
                            .ok_or("Invalid destination button ordinal")?,
                    )
                    .copied()
                    .ok_or("Link destination button is unavailable")
            })
            .transpose()?;
        self.animations.clear();
        self.movements.clear();
        self.presentations.clear();
        self.doc.active_page = target_page;
        self.doc.selected.clear();
        self.selection_order.clear();
        self.fit();
        self.status = format!("Opened page {}", target_page + 1);
        if let Some(id) = destination_button {
            let destination_viewport = Viewport {
                rect: viewport.rect,
                scale: self.scale,
                center: self.center,
            };
            self.activate_child_action(id, &destination_viewport, depth + 1)?;
        }
        Ok(())
    }

    fn action_is_running(&self, id: usize) -> bool {
        match self
            .doc
            .objects
            .get(id.saturating_sub(1))
            .map(|object| &object.kind)
        {
            Some(Kind::AnimationButton { targets, .. }) => self
                .animations
                .iter()
                .any(|track| targets.contains(&track.id)),
            Some(Kind::MovementButton { .. }) => {
                self.movements.iter().any(|track| track.button == id)
            }
            Some(Kind::PresentationButton { .. }) => {
                self.presentations.iter().any(|track| track.id == id)
            }
            _ => false,
        }
    }

    fn stop_child_action(&mut self, id: usize, depth: usize) {
        if depth > 32 {
            return;
        }
        match self
            .doc
            .objects
            .get(id.saturating_sub(1))
            .map(|object| &object.kind)
        {
            Some(Kind::AnimationButton { targets, .. }) => {
                let targets = targets.clone();
                self.animations.retain(|track| !targets.contains(&track.id));
            }
            Some(Kind::MovementButton { .. }) => {
                self.movements.retain(|track| track.button != id);
            }
            Some(Kind::PresentationButton { .. }) => {
                if let Some(index) = self.presentations.iter().position(|track| track.id == id) {
                    let track = self.presentations.remove(index);
                    for child in track.active {
                        self.stop_child_action(child, depth + 1);
                    }
                }
            }
            _ => {}
        }
    }

    fn activate_child_action(
        &mut self,
        id: usize,
        viewport: &Viewport,
        depth: usize,
    ) -> Result<(), String> {
        self.activate_action(id, viewport, depth, false)
    }

    pub(super) fn activate_referenced_action(&mut self, id: usize, viewport: &Viewport) -> Result<(), String> {
        self.activate_action(id, viewport, 0, true)
    }

    fn activate_action(&mut self, id: usize, viewport: &Viewport, depth: usize, indirect: bool) -> Result<(), String> {
        if depth > 32 {
            return Err("Presentation buttons are nested too deeply".into());
        }
        let object = self
            .doc
            .objects
            .get(id.checked_sub(1).ok_or("Invalid action button ID")?)
            .ok_or("Invalid action button ID")?;
        if object.deleted
            || (!indirect && !object.visible && !matches!(object.kind, Kind::HideShowButton { .. }))
            || object.page != self.doc.active_page
        {
            return Err("A Presentation parent button is unavailable".into());
        }
        match object.kind.clone() {
            Kind::HideShowButton { .. } => self.doc.activate_hide_show_button_as_child(id),
            Kind::AnimationButton {
                targets, settings, ..
            } => {
                self.animate_button_targets(&targets, &settings);
                Ok(())
            }
            Kind::MovementButton { .. } => {
                self.activate_movement_button(id);
                Ok(())
            }
            Kind::ScrollButton {
                target, placement, ..
            } => self.scroll_to_button_target(target, placement, viewport),
            Kind::LinkUrlButton { url, .. } => self.activate_link_url(&url),
            Kind::LinkPageButton {
                target_page,
                target_button_ordinal,
                ..
            } => self.activate_link_page(target_page, target_button_ordinal, viewport, depth + 1),
            Kind::PresentationButton { .. } => {
                self.activate_presentation_button(id, viewport, depth + 1)
            }
            _ => Err("Object is not an action button".into()),
        }
    }

    fn run_sequential_presentation(
        &mut self,
        mut track: PresentationTrack,
        viewport: &Viewport,
        depth: usize,
    ) -> Result<(), String> {
        while track.next < track.targets.len() {
            let child = track.targets[track.next];
            track.next += 1;
            self.activate_child_action(child, viewport, depth + 1)?;
            if self.action_is_running(child) {
                track.active = vec![child];
                self.presentations.push(track);
                return Ok(());
            }
        }
        Ok(())
    }

    pub(super) fn activate_presentation_button(
        &mut self,
        id: usize,
        viewport: &Viewport,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 32 {
            return Err("Presentation buttons are nested too deeply".into());
        }
        if let Some(index) = self.presentations.iter().position(|track| track.id == id) {
            let mut track = self.presentations.remove(index);
            for child in track.active.drain(..) {
                self.stop_child_action(child, depth + 1);
            }
            if track.mode == PresentationMode::Sequential {
                self.run_sequential_presentation(track, viewport, depth + 1)?;
            }
            return Ok(());
        }
        let Some(object) = self
            .doc
            .objects
            .get(id.checked_sub(1).ok_or("Invalid button ID")?)
        else {
            return Err("Invalid button ID".into());
        };
        if object.deleted || !object.visible || object.page != self.doc.active_page {
            return Err("Presentation button is unavailable".into());
        }
        let Kind::PresentationButton { targets, mode, .. } = &object.kind else {
            return Err("Object is not a Presentation button".into());
        };
        let targets = targets.clone();
        let mode = *mode;
        let mut track = PresentationTrack {
            id,
            targets,
            mode,
            next: 0,
            active: Vec::new(),
        };
        match mode {
            PresentationMode::Simultaneous => {
                for &child in &track.targets {
                    self.activate_child_action(child, viewport, depth + 1)?;
                    if self.action_is_running(child) {
                        track.active.push(child);
                    }
                }
                if !track.active.is_empty() {
                    self.presentations.push(track);
                }
            }
            PresentationMode::Sequential => {
                self.run_sequential_presentation(track, viewport, depth + 1)?
            }
        }
        self.status = format!("Activated Presentation button {id}");
        Ok(())
    }

    pub(super) fn tick_presentations(&mut self) {
        let Some(rect) = self.viewport_rect else {
            return;
        };
        let tracks = std::mem::take(&mut self.presentations);
        for mut track in tracks {
            match track.mode {
                PresentationMode::Simultaneous => {
                    track.active.retain(|&id| self.action_is_running(id));
                    if !track.active.is_empty() {
                        self.presentations.push(track);
                    }
                }
                PresentationMode::Sequential => {
                    if track.active.iter().all(|&id| !self.action_is_running(id)) {
                        track.active.clear();
                        let viewport = Viewport {
                            rect,
                            scale: self.scale,
                            center: self.center,
                        };
                        if let Err(error) = self.run_sequential_presentation(track, &viewport, 0) {
                            self.status = error;
                        }
                    } else {
                        self.presentations.push(track);
                    }
                }
            }
        }
    }
}
