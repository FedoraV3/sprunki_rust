//! Sprunki, but in Rust.
//!
//! A tiny Incredibox-style beat toy. Every character owns one looping sample.
//! Toggle a character with its key or by clicking on it; all active characters
//! play on a shared bar so they stay in sync.

use macroquad::{
    audio::{load_sound_from_bytes, play_sound_once, stop_sound, Sound},
    prelude::*,
};

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 768;

/// Raw sprite and sample data baked into the binary.
struct CharacterAssets {
    name: &'static str,
    key: KeyCode,
    sprite: &'static [u8],
    sample: &'static [u8],
}

const CHARACTERS: &[CharacterAssets] = &[CharacterAssets {
    name: "Oren",
    key: KeyCode::O,
    sprite: include_bytes!("Sprites/oren.png"),
    sample: include_bytes!("Sounds/kick.wav"),
}];

/// A loaded character on the stage.
struct Character {
    name: &'static str,
    key: KeyCode,
    texture: Texture2D,
    sound: Sound,
    sample_length: f64,
    slot: Rect,
    /// Currently playing on the bar.
    active: bool,
    /// Switched on mid-bar; starts playing when the next bar begins.
    queued: bool,
}

impl Character {
    async fn load(assets: &CharacterAssets, slot: Rect) -> Result<Character, String> {
        let image = Image::from_file_with_format(assets.sprite, Some(ImageFormat::Png))
            .map_err(|e| format!("{}: bad sprite: {e}", assets.name))?;
        let texture = Texture2D::from_image(&image);
        let sound = load_sound_from_bytes(assets.sample)
            .await
            .map_err(|e| format!("{}: bad sample: {e}", assets.name))?;
        let sample_length = wav_duration_seconds(assets.sample)
            .ok_or_else(|| format!("{}: sample is not a readable WAV", assets.name))?;

        Ok(Character {
            name: assets.name,
            key: assets.key,
            texture,
            sound,
            sample_length,
            slot,
            active: false,
            queued: false,
        })
    }

    fn is_on(&self) -> bool {
        self.active || self.queued
    }

    fn silence(&mut self) {
        stop_sound(&self.sound);
        self.active = false;
        self.queued = false;
    }

    fn draw(&self, beat_progress: Option<f64>) {
        let Rect { x, y, w, h } = self.slot;

        // Slot frame.
        let frame = if self.active {
            Color::new(0.95, 0.80, 0.30, 1.0)
        } else if self.queued {
            Color::new(0.60, 0.75, 0.95, 1.0)
        } else {
            Color::new(0.85, 0.85, 0.85, 1.0)
        };
        draw_rectangle(x - 6.0, y - 6.0, w + 12.0, h + 12.0, frame);
        draw_rectangle(x, y, w, h, WHITE);

        // Sprite: full colour when active, faded when idle, bobbing on the beat.
        let tint = if self.active {
            WHITE
        } else {
            Color::new(1.0, 1.0, 1.0, 0.35)
        };
        let bob = match (self.active, beat_progress) {
            (true, Some(p)) => ((p * std::f64::consts::TAU * 2.0).sin() * 4.0) as f32,
            _ => 0.0,
        };
        draw_texture_ex(
            &self.texture,
            x,
            y + bob,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );

        // Label with the hotkey.
        let label = format!("{} [{}]", self.name, key_name(self.key));
        let size = measure_text(&label, None, 24, 1.0);
        draw_text(
            &label,
            x + (w - size.width) / 2.0,
            y + h + 30.0,
            24.0,
            DARKGRAY,
        );
    }
}

/// Shared transport: every active character restarts its sample when a bar begins.
struct Bar {
    length: f64,
    started_at: Option<f64>,
}

impl Bar {
    fn new(length: f64) -> Bar {
        Bar {
            length,
            started_at: None,
        }
    }

    fn running(&self) -> bool {
        self.started_at.is_some()
    }

    /// Fraction of the current bar that has elapsed, if the bar is running.
    fn progress(&self, now: f64) -> Option<f64> {
        self.started_at
            .map(|start| ((now - start) / self.length).clamp(0.0, 1.0))
    }

    /// Returns true when a new bar begins this frame.
    fn tick(&mut self, now: f64) -> bool {
        match self.started_at {
            Some(start) if now - start >= self.length => {
                // Advance by whole bars so a long frame does not drift the clock.
                let bars_elapsed = ((now - start) / self.length).floor();
                self.started_at = Some(start + bars_elapsed * self.length);
                true
            }
            _ => false,
        }
    }

    fn start(&mut self, now: f64) {
        self.started_at = Some(now);
    }

    fn stop(&mut self) {
        self.started_at = None;
    }
}

struct Stage {
    characters: Vec<Character>,
    bar: Bar,
}

impl Stage {
    async fn load() -> Result<Stage, String> {
        let slot_size = vec2(219.0, 240.0);
        let gap = 40.0;
        let columns = (((WINDOW_WIDTH as f32 + gap) / (slot_size.x + gap)).floor() as usize)
            .clamp(1, CHARACTERS.len().max(1));
        let rows = CHARACTERS.len().div_ceil(columns);
        let grid_width = columns as f32 * slot_size.x + (columns as f32 - 1.0) * gap;
        let grid_height = rows as f32 * slot_size.y + (rows as f32 - 1.0) * (gap + 30.0);
        let origin = vec2(
            (WINDOW_WIDTH as f32 - grid_width) / 2.0,
            (WINDOW_HEIGHT as f32 - grid_height) / 2.0,
        );

        let mut characters = Vec::with_capacity(CHARACTERS.len());
        for (i, assets) in CHARACTERS.iter().enumerate() {
            let col = (i % columns) as f32;
            let row = (i / columns) as f32;
            let slot = Rect::new(
                origin.x + col * (slot_size.x + gap),
                origin.y + row * (slot_size.y + gap + 30.0),
                slot_size.x,
                slot_size.y,
            );
            characters.push(Character::load(assets, slot).await?);
        }

        // The bar is as long as the longest sample so nothing gets cut off.
        let length = characters
            .iter()
            .map(|c| c.sample_length)
            .fold(0.0, f64::max);
        if length <= 0.0 {
            return Err("no characters with a playable sample".to_string());
        }

        Ok(Stage {
            characters,
            bar: Bar::new(length),
        })
    }

    fn toggle(&mut self, index: usize, now: f64) {
        let character = &mut self.characters[index];
        if character.is_on() {
            character.silence();
            if !self.characters.iter().any(Character::is_on) {
                self.bar.stop();
            }
            return;
        }

        if self.bar.running() {
            // Someone is already playing: wait for the next bar so we stay in sync.
            character.queued = true;
        } else {
            // First sound in: it defines where the bar starts.
            character.active = true;
            play_sound_once(&character.sound);
            self.bar.start(now);
        }
    }

    fn silence_all(&mut self) {
        for character in &mut self.characters {
            character.silence();
        }
        self.bar.stop();
    }

    fn update(&mut self, now: f64) {
        if self.bar.tick(now) {
            for character in &mut self.characters {
                if character.queued {
                    character.queued = false;
                    character.active = true;
                }
                if character.active {
                    play_sound_once(&character.sound);
                }
            }
        }
    }

    fn handle_input(&mut self, now: f64) {
        if is_key_pressed(KeyCode::Space) {
            self.silence_all();
        }

        for i in 0..self.characters.len() {
            if is_key_pressed(self.characters[i].key) {
                self.toggle(i, now);
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse = Vec2::from(mouse_position());
            if let Some(i) = self.characters.iter().position(|c| c.slot.contains(mouse)) {
                self.toggle(i, now);
            }
        }
    }

    fn draw(&self, now: f64) {
        clear_background(Color::new(0.97, 0.97, 0.98, 1.0));

        let progress = self.bar.progress(now);
        for character in &self.characters {
            character.draw(progress);
        }

        // Transport bar along the bottom.
        let bar_h = 10.0;
        let bar_y = WINDOW_HEIGHT as f32 - bar_h - 50.0;
        draw_rectangle(0.0, bar_y, WINDOW_WIDTH as f32, bar_h, LIGHTGRAY);
        if let Some(p) = progress {
            draw_rectangle(
                0.0,
                bar_y,
                WINDOW_WIDTH as f32 * p as f32,
                bar_h,
                Color::new(0.95, 0.80, 0.30, 1.0),
            );
        }

        let hint = "Click a character or press its key to toggle it.  Space: stop all.  Esc: quit.";
        let size = measure_text(hint, None, 20, 1.0);
        draw_text(
            hint,
            (WINDOW_WIDTH as f32 - size.width) / 2.0,
            WINDOW_HEIGHT as f32 - 20.0,
            20.0,
            GRAY,
        );
    }
}

/// Length in seconds of a PCM WAV file, read straight from its RIFF chunks.
fn wav_duration_seconds(bytes: &[u8]) -> Option<f64> {
    let u32_at = |i: usize| {
        bytes
            .get(i..i + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };

    if bytes.get(0..4)? != b"RIFF" || bytes.get(8..12)? != b"WAVE" {
        return None;
    }

    let mut byte_rate: Option<u32> = None;
    let mut data_len: Option<u32> = None;
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let id = bytes.get(pos..pos + 4)?;
        let size = u32_at(pos + 4)? as usize;
        let body = pos + 8;
        match id {
            b"fmt " => byte_rate = u32_at(body + 8),
            b"data" => data_len = Some(size.min(bytes.len() - body) as u32),
            _ => {}
        }
        // Chunks are padded to an even length.
        pos = body + size + (size & 1);
    }

    let rate = byte_rate.filter(|&r| r > 0)?;
    Some(f64::from(data_len?) / f64::from(rate))
}

fn key_name(key: KeyCode) -> String {
    format!("{key:?}")
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Sprunki But in Rust".to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut stage = match Stage::load().await {
        Ok(stage) => stage,
        Err(err) => {
            eprintln!("failed to load stage: {err}");
            std::process::exit(1);
        }
    };

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        let now = get_time();
        stage.handle_input(now);
        stage.update(now);
        stage.draw(now);

        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_length_of_bundled_kick() {
        let secs = wav_duration_seconds(CHARACTERS[0].sample).expect("kick.wav parses");
        assert!((secs - 4.9687).abs() < 0.01, "got {secs}");
    }

    #[test]
    fn rejects_non_wav() {
        assert_eq!(wav_duration_seconds(b"not a wav at all"), None);
        assert_eq!(wav_duration_seconds(b""), None);
    }

    #[test]
    fn bar_restarts_on_whole_bars_only() {
        let mut bar = Bar::new(2.0);
        assert!(!bar.tick(0.5));
        bar.start(1.0);
        assert!(!bar.tick(2.9));
        assert!(bar.tick(3.0));
        assert_eq!(bar.started_at, Some(3.0));
        // A long stall skips ahead by whole bars without drifting.
        assert!(bar.tick(7.5));
        assert_eq!(bar.started_at, Some(7.0));
        assert!((bar.progress(7.5).unwrap() - 0.25).abs() < 1e-9);
    }
}
