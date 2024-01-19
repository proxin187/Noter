use raylib::prelude::*;

use std::ffi::CString;
use std::ffi::CStr;


pub struct Sound {
    file: String,
    sound: Music,
}

pub struct Audio {
    handle: audio::RaylibAudio,
    sounds: Vec<Sound>,
    volume: f32,
    is_playing: bool,
}

impl Audio {
    fn load_sound(&mut self, thread: &RaylibThread, file: &str) -> Result<usize, Box<dyn std::error::Error>> {
        self.sounds.push(Sound {
            file: file.split("/").last().unwrap_or("default.mp3").to_string(),
            sound: Music::load_music_stream(thread, file)?,
        });

        let length = self.sounds.len() - 1;
        self.sounds[length].sound.looping = false;

        Ok(length)
    }
}

pub struct Icons {
    play: Texture2D,
    stop: Texture2D,
    volume: Texture2D,
    volume_off: Texture2D,
}

impl Icons {
    pub fn init(rl: &mut raylib::RaylibHandle, thread: &raylib::RaylibThread) -> Result<Icons, Box<dyn std::error::Error>> {
        Ok(Icons {
            play: rl.load_texture(thread, "assets/play_arrow.png")?,
            stop: rl.load_texture(thread, "assets/pause.png")?,
            volume: rl.load_texture(thread, "assets/volume.png")?,
            volume_off: rl.load_texture(thread, "assets/volume_off.png")?,
        })
    }
}

pub struct Noter<'a> {
    rl: &'a mut raylib::RaylibHandle,
    thread: raylib::RaylibThread,
    audio: Audio,
    icons: Icons,
    selected: i32,
    should_close: bool,
}

impl<'a> Noter<'a> {
    pub fn new(rl: &'a mut raylib::RaylibHandle, thread: raylib::RaylibThread) -> Result<Noter<'a>, Box<dyn std::error::Error>> {
        let handle = audio::RaylibAudio::init_audio_device();
        let icons = Icons::init(rl, &thread)?;

        Ok(Noter {
            rl,
            thread,
            audio: Audio {
                handle,
                sounds: Vec::new(),
                volume: 1.0,
                is_playing: false,
            },
            icons,
            selected: 0,
            should_close: false,
        })
    }

    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let width = self.rl.get_screen_width();
        let height = self.rl.get_screen_height();

        let mut drawer = self.rl.begin_drawing(&self.thread);

        drawer.clear_background(Color::from_hex("2b3a3a")?);

        if self.audio.sounds.is_empty() {
            drawer.gui_set_style(GuiControl::DEFAULT, 16, 50);

            let message = "Drop Here\0";
            drawer.gui_label(
                Rectangle::new(
                    ((width / 2) - (text::measure_text(&message.replace("\0", ""), 50) / 2)) as f32,
                    0.0,
                    width as f32,
                    height as f32
                ),
                Some(CStr::from_bytes_with_nul(message.as_bytes())?)
            );

            drawer.gui_set_style(GuiControl::DEFAULT, 16, 24);
        } else {
            let mut sounds: Vec<CString> = Vec::new();

            for sound in &self.audio.sounds {
                sounds.push(CString::new(sound.file.as_str())?);
            }

            // sounds
            let index = drawer.gui_list_view_ex(
                Rectangle::new(
                    20.0,
                    20.0,
                    width as f32 - 40.0,
                    height as f32 - 55.0
                ),
                &sounds.iter().map(|x| x.as_c_str()).collect::<Vec<&CStr>>(),
                &mut self.selected.clone(),
                &mut self.selected.clone(),
                self.selected,
            );

            if self.audio.sounds.len() > index as usize {
                self.selected = index;
            }

            // bottom-bar
            drawer.gui_slider_bar(
                Rectangle::new(
                    (width / 4) as f32,
                    (height - 25) as f32,
                    (width / 2) as f32,
                    15.0
                ),
                None,
                None,
                self.audio.handle.get_music_time_played(&self.audio.sounds[self.selected as usize].sound),
                0.0,
                self.audio.handle.get_music_time_length(&self.audio.sounds[self.selected as usize].sound),
            );

            // play-icon
            let icon = if !self.audio.is_playing {
                &self.icons.play
            } else {
                &self.icons.stop
            };

            let play = drawer.gui_image_button(
                Rectangle::new(
                    ((width / 4) - icon.width - 5) as f32,
                    (height - icon.height - 5) as f32,
                    icon.width as f32,
                    icon.height as f32,
                ),
                None,
                icon,
            );

            if play {
                self.audio.is_playing = !self.audio.is_playing;
            }

            // volume-bar
            self.audio.volume = drawer.gui_slider_bar(
                Rectangle::new(
                    ((width / 4) * 3) as f32 + 50.0,
                    (height - 25) as f32,
                    100.0,
                    15.0
                ),
                None,
                None,
                self.audio.volume,
                0.0,
                1.0,
            );

            // volume-icon
            // play-icon
            let icon = if self.audio.volume == 0.0 {
                &self.icons.volume_off
            } else {
                &self.icons.volume
            };

            let volume = drawer.gui_image_button(
                Rectangle::new(
                    ((width / 4) * 3) as f32 + 20.0,
                    (height - icon.height - 5) as f32,
                    icon.width as f32,
                    icon.height as f32,
                ),
                None,
                icon,
            );

            // mute/unmute
            if volume {
                if self.audio.volume == 0.0 {
                    self.audio.volume = 1.0;
                } else {
                    self.audio.volume = 0.0;
                }
            }
        }

        Ok(())
    }

    fn handle_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.rl.is_file_dropped() {
            for file in self.rl.get_dropped_files() {
                self.audio.load_sound(&self.thread, &file)?;
            }

            self.rl.clear_dropped_files();
        }

        Ok(())
    }

    fn handle_key(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(key_code) = self.rl.get_key_pressed_number() {
            if key_code == 264 {
                if (self.selected as usize) < self.audio.sounds.len() - 1 {
                    self.selected += 1;
                }
            } else if key_code == 265 {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
        }

        Ok(())
    }

    fn mute_stream(&mut self) {
        for sound in &mut self.audio.sounds {
            if self.audio.handle.is_music_playing(&sound.sound) {
                self.audio.handle.stop_music_stream(&mut sound.sound);
            }
        }
    }

    fn update_stream(&mut self) {
        if self.audio.is_playing && !self.audio.handle.is_music_playing(&self.audio.sounds[self.selected as usize].sound) {
            self.mute_stream();

            self.audio.handle.play_music_stream(&mut self.audio.sounds[self.selected as usize].sound);
        }

        if self.audio.is_playing {
            for sound in &mut self.audio.sounds {
                if self.audio.handle.is_music_playing(&sound.sound) {
                    self.audio.handle.update_music_stream(&mut sound.sound);
                }
            }
        }

        self.audio.handle.set_master_volume(self.audio.volume);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.rl.set_exit_key(Some(KeyboardKey::KEY_NULL));

        self.rl.gui_load_style(Some(CStr::from_bytes_with_nul(b"assets/style_jungle.txt.rgs\0")?));
        self.rl.gui_set_style(GuiControl::DEFAULT, 17, 5);

        let font = self.rl.load_font_ex(&self.thread, "assets/Pixel Intv.otf", 100, FontLoadEx::Default(256))?;
        self.rl.gui_set_font(&font);

        while !self.should_close {
            if self.rl.is_cursor_on_screen() {
                self.should_close = self.rl.window_should_close();
            }

            self.draw()?;
            self.handle_key()?;
            self.handle_files()?;
            self.update_stream();
        }

        Ok(())
    }
}


