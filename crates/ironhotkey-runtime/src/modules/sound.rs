use super::{stub_log, ModuleMethod};

pub fn sound_beep(args: &[&str]) -> String {
    stub_log("sound", "SoundBeep", args)
}
pub fn sound_get(args: &[&str]) -> String {
    stub_log("sound", "SoundGet", args)
}
pub fn sound_get_wave_volume(args: &[&str]) -> String {
    stub_log("sound", "SoundGetWaveVolume", args)
}
pub fn sound_play(args: &[&str]) -> String {
    stub_log("sound", "SoundPlay", args)
}
pub fn sound_set(args: &[&str]) -> String {
    stub_log("sound", "SoundSet", args)
}
pub fn sound_set_wave_volume(args: &[&str]) -> String {
    stub_log("sound", "SoundSetWaveVolume", args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("SoundBeep", sound_beep),
    ("SoundGet", sound_get),
    ("SoundGetWaveVolume", sound_get_wave_volume),
    ("SoundPlay", sound_play),
    ("SoundSet", sound_set),
    ("SoundSetWaveVolume", sound_set_wave_volume),
];
