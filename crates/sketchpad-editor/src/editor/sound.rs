use super::Editor;
use rodio::{ChannelCount, DeviceSinkBuilder, Player, SampleRate, Source};
use sketchpad_engine::document::{Document, Kind};
use std::num::{NonZeroU16, NonZeroU32};
use std::time::Duration;

const SOUND_SAMPLE_RATE: u32 = 44_100;

struct FunctionSound {
    document: Document,
    functions: Vec<usize>,
    frame: u64,
    channel: usize,
}

impl FunctionSound {
    fn new(document: Document, functions: Vec<usize>) -> Self {
        Self {
            document,
            functions,
            frame: 0,
            channel: 0,
        }
    }
}

impl Iterator for FunctionSound {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let time = self.frame as f64 / f64::from(SOUND_SAMPLE_RATE);
        let function = self.functions[self.channel];
        let sample = self
            .document
            .function_value(function, time)
            .filter(|value| value.is_finite())
            .unwrap_or(0.)
            .clamp(-1., 1.) as f32;
        self.channel += 1;
        if self.channel == self.functions.len() {
            self.channel = 0;
            self.frame = self.frame.saturating_add(1);
        }
        Some(sample)
    }
}

impl Source for FunctionSound {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        NonZeroU16::new(self.functions.len() as u16).expect("validated Sound channels")
    }

    fn sample_rate(&self) -> SampleRate {
        NonZeroU32::new(SOUND_SAMPLE_RATE).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

struct SoundPlayback {
    button: usize,
    player: Player,
}

#[derive(Default)]
pub(super) struct SoundState {
    output: Option<rodio::MixerDeviceSink>,
    playbacks: Vec<SoundPlayback>,
}

impl Editor {
    pub(super) fn sound_is_running(&self, button: usize) -> bool {
        self.sounds
            .playbacks
            .iter()
            .any(|playback| playback.button == button)
    }

    pub(super) fn stop_sound_button(&mut self, button: usize) {
        if let Some(index) = self
            .sounds
            .playbacks
            .iter()
            .position(|playback| playback.button == button)
        {
            self.sounds.playbacks.remove(index).player.stop();
        }
    }

    pub(super) fn stop_all_sounds(&mut self) {
        for playback in self.sounds.playbacks.drain(..) {
            playback.player.stop();
        }
    }

    pub(super) fn activate_sound_button(&mut self, button: usize) -> Result<(), String> {
        if self.sound_is_running(button) {
            self.stop_sound_button(button);
            self.status = format!("Stopped Sound button {button}");
            return Ok(());
        }
        let object = self
            .doc
            .objects
            .get(button.checked_sub(1).ok_or("Invalid Sound button ID")?)
            .ok_or("Invalid Sound button ID")?;
        if object.deleted || !object.visible || object.page != self.doc.active_page {
            return Err("Sound button is unavailable".into());
        }
        let Kind::SoundButton { functions, .. } = &object.kind else {
            return Err("Object is not a Sound button".into());
        };
        if !(1..=2).contains(&functions.len())
            || functions.iter().any(|&function| {
                self.doc
                    .objects
                    .get(function.saturating_sub(1))
                    .is_none_or(|source| source.deleted || !source.kind.is_function())
            })
        {
            return Err("Sound button has unavailable function targets".into());
        }
        let functions = functions.clone();
        if self.sounds.output.is_none() {
            self.sounds.output = Some(
                DeviceSinkBuilder::open_default_sink()
                    .map_err(|error| format!("Audio output is unavailable: {error}"))?,
            );
        }
        let player = Player::connect_new(
            self.sounds
                .output
                .as_ref()
                .expect("initialized audio output")
                .mixer(),
        );
        player.append(FunctionSound::new(self.doc.clone(), functions));
        self.sounds.playbacks.push(SoundPlayback { button, player });
        self.status = format!("Playing Sound button {button}");
        Ok(())
    }
}
