use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};

use super::ModuleMethod;
pub fn sound_beep(frequency: Option<u32>, duration: Option<u32>) -> String {
    let freq = frequency.unwrap_or(523).clamp(37, 32767) as f32;
    let dur_ms = duration.unwrap_or(150);

    let Ok((_stream, stream_handle)) = OutputStream::try_default() else {
        return String::new();
    };
    let Ok(sink) = Sink::try_new(&stream_handle) else {
        return String::new();
    };
    let source = SineWave::new(freq)
        .take_duration(Duration::from_millis(dur_ms as u64))
        .amplify(0.2);
    sink.append(source);
    sink.sleep_until_end();
    String::new()
}

pub fn sound_play(filename: &str, wait: bool) -> String {
    let name = filename.trim();

    if let Some(rest) = name.strip_prefix('*') {
        let code: i32 = rest.parse().unwrap_or(-1);
        let freq = match code {
            -1 => 800,
            16 => 330,  // hand / error
            32 => 660,  // question
            48 => 880,  // exclamation
            64 => 1000, // asterisk
            _ => 523,
        };
        return sound_beep(Some(freq), Some(300));
    }

    let Ok(file) = File::open(name) else {
        return String::new();
    };
    let Ok((_stream, stream_handle)) = OutputStream::try_default() else {
        return String::new();
    };
    let Ok(sink) = Sink::try_new(&stream_handle) else {
        return String::new();
    };
    let Ok(decoder) = Decoder::new(BufReader::new(file)) else {
        return String::new();
    };
    sink.append(decoder);
    if wait {
        sink.sleep_until_end();
    } else {
        sink.detach();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(300));
        });
    }
    String::new()
}

pub fn sound_get(component: Option<&str>, control: Option<&str>, _device: Option<u32>) -> String {
    let ctrl = control.unwrap_or("VOLUME").to_uppercase();
    let comp = component.unwrap_or("MASTER").to_uppercase();
    let is_mute_query = ctrl == "MUTE" || ctrl == "ONOFF";

    #[cfg(target_os = "linux")]
    {
        let channel = ahk_component_to_alsa_name(&comp);
        if let Some(result) = alsa_get(&channel, is_mute_query) {
            return result;
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(result) = macos_audio_get(is_mute_query) {
            return result;
        }
    }
    #[cfg(windows)]
    {
        if let Some(result) = windows_audio_get(is_mute_query) {
            return result;
        }
    }
    let _ = (is_mute_query, comp);
    String::new()
}

pub fn sound_set(
    new_setting: &str,
    component: Option<&str>,
    control: Option<&str>,
    _device: Option<u32>,
) -> String {
    let ctrl = control.unwrap_or("VOLUME").to_uppercase();
    let comp = component.unwrap_or("MASTER").to_uppercase();
    let setting = new_setting.trim();

    #[cfg(target_os = "linux")]
    {
        let is_mute = ctrl == "MUTE" || ctrl == "ONOFF";
        let channel = ahk_component_to_alsa_name(&comp);
        alsa_set(&channel, setting, is_mute);
    }
    #[cfg(target_os = "macos")]
    {
        let is_mute = ctrl == "MUTE" || ctrl == "ONOFF";
        macos_audio_set(setting, is_mute);
    }
    #[cfg(windows)]
    {
        let is_mute = ctrl == "MUTE" || ctrl == "ONOFF";
        windows_audio_set(setting, is_mute);
    }
    let _ = (setting, comp, ctrl);
    String::new()
}

pub fn sound_get_wave_volume(device: Option<u32>) -> String {
    sound_get(Some("WAVE"), Some("VOLUME"), device)
}

pub fn sound_set_wave_volume(percent: &str, device: Option<u32>) -> String {
    sound_set(percent, Some("WAVE"), Some("VOLUME"), device)
}

#[cfg(target_os = "linux")]
fn ahk_component_to_alsa_name(comp: &str) -> String {
    match comp {
        "MASTER" | "N/A" => "Master".to_string(),
        "WAVE" => "PCM".to_string(),
        "LINE" => "Line".to_string(),
        "MICROPHONE" | "MIC" => "Capture".to_string(),
        "CD" => "CD".to_string(),
        other => other.to_string(),
    }
}

#[cfg(target_os = "linux")]
fn open_alsa_selem(name: &str) -> Option<(alsa::mixer::Mixer, alsa::mixer::SelemId)> {
    let mixer = alsa::mixer::Mixer::new("default", false).ok()?;
    let id = alsa::mixer::SelemId::new(name, 0);
    Some((mixer, id))
}

#[cfg(target_os = "linux")]
fn alsa_get(channel: &str, is_mute: bool) -> Option<String> {
    let (mixer, id) = open_alsa_selem(channel)?;
    let elem = mixer.find_selem(&id)?;
    if is_mute {
        let switch = elem
            .get_playback_switch(alsa::mixer::SelemChannelId::FrontLeft)
            .ok()?;
        return Some(if switch == 0 { "On" } else { "Off" }.to_string());
    }
    let (min, max) = elem.get_playback_volume_range();
    if max == min {
        return Some("0".to_string());
    }
    let cur = elem
        .get_playback_volume(alsa::mixer::SelemChannelId::FrontLeft)
        .ok()?;
    let pct = ((cur - min) as f64 / (max - min) as f64 * 100.0).round() as i32;
    Some(format!("{pct}"))
}

#[cfg(target_os = "linux")]
fn alsa_set(channel: &str, setting: &str, is_mute: bool) {
    let Some((mixer, id)) = open_alsa_selem(channel) else {
        return;
    };
    let Some(elem) = mixer.find_selem(&id) else {
        return;
    };
    if is_mute {
        let on = matches!(setting, "1" | "On" | "ON" | "on");
        let _ = elem.set_playback_switch_all(if on { 0 } else { 1 });
        return;
    }
    let (min, max) = elem.get_playback_volume_range();
    let range = max - min;
    if range == 0 {
        return;
    }
    let target_pct: f64 = if setting.starts_with('+') || setting.starts_with('-') {
        let cur = elem
            .get_playback_volume(alsa::mixer::SelemChannelId::FrontLeft)
            .unwrap_or(min);
        let cur_pct = (cur - min) as f64 / range as f64 * 100.0;
        let delta: f64 = setting.parse().unwrap_or(0.0);
        (cur_pct + delta).clamp(0.0, 100.0)
    } else {
        setting.parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0)
    };
    let vol = min + (target_pct / 100.0 * range as f64).round() as i64;
    let _ = elem.set_playback_volume_all(vol);
}

#[cfg(target_os = "macos")]
fn macos_default_output_device() -> Option<coreaudio_sys::AudioObjectID> {
    unsafe {
        let mut device_id: coreaudio_sys::AudioObjectID = 0;
        let mut size = std::mem::size_of::<coreaudio_sys::AudioObjectID>() as u32;
        let address = coreaudio_sys::AudioObjectPropertyAddress {
            mSelector: coreaudio_sys::kAudioHardwarePropertyDefaultOutputDevice,
            mScope: coreaudio_sys::kAudioObjectPropertyScopeGlobal,
            mElement: coreaudio_sys::kAudioObjectPropertyElementMain,
        };
        let status = coreaudio_sys::AudioObjectGetPropertyData(
            coreaudio_sys::kAudioObjectSystemObject,
            &address,
            0,
            std::ptr::null(),
            &mut size,
            &mut device_id as *mut _ as *mut std::ffi::c_void,
        );
        if status == 0 {
            Some(device_id)
        } else {
            None
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_get_scalar(
    device_id: coreaudio_sys::AudioObjectID,
    selector: coreaudio_sys::AudioObjectPropertySelector,
    scope: coreaudio_sys::AudioObjectPropertyScope,
) -> Option<f32> {
    unsafe {
        let mut value: f32 = 0.0;
        let mut size = std::mem::size_of::<f32>() as u32;
        let address = coreaudio_sys::AudioObjectPropertyAddress {
            mSelector: selector,
            mScope: scope,
            mElement: coreaudio_sys::kAudioObjectPropertyElementMain,
        };
        let status = coreaudio_sys::AudioObjectGetPropertyData(
            device_id,
            &address,
            0,
            std::ptr::null(),
            &mut size,
            &mut value as *mut _ as *mut std::ffi::c_void,
        );
        if status == 0 {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_get_u32(
    device_id: coreaudio_sys::AudioObjectID,
    selector: coreaudio_sys::AudioObjectPropertySelector,
    scope: coreaudio_sys::AudioObjectPropertyScope,
) -> Option<u32> {
    unsafe {
        let mut value: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        let address = coreaudio_sys::AudioObjectPropertyAddress {
            mSelector: selector,
            mScope: scope,
            mElement: coreaudio_sys::kAudioObjectPropertyElementMain,
        };
        let status = coreaudio_sys::AudioObjectGetPropertyData(
            device_id,
            &address,
            0,
            std::ptr::null(),
            &mut size,
            &mut value as *mut _ as *mut std::ffi::c_void,
        );
        if status == 0 {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_set_scalar(
    device_id: coreaudio_sys::AudioObjectID,
    selector: coreaudio_sys::AudioObjectPropertySelector,
    scope: coreaudio_sys::AudioObjectPropertyScope,
    value: f32,
) -> bool {
    unsafe {
        let data = value;
        let address = coreaudio_sys::AudioObjectPropertyAddress {
            mSelector: selector,
            mScope: scope,
            mElement: coreaudio_sys::kAudioObjectPropertyElementMain,
        };
        coreaudio_sys::AudioObjectSetPropertyData(
            device_id,
            &address,
            0,
            std::ptr::null(),
            std::mem::size_of::<f32>() as u32,
            &data as *const _ as *const std::ffi::c_void,
        ) == 0
    }
}

#[cfg(target_os = "macos")]
fn macos_set_u32(
    device_id: coreaudio_sys::AudioObjectID,
    selector: coreaudio_sys::AudioObjectPropertySelector,
    scope: coreaudio_sys::AudioObjectPropertyScope,
    value: u32,
) -> bool {
    unsafe {
        let data = value;
        let address = coreaudio_sys::AudioObjectPropertyAddress {
            mSelector: selector,
            mScope: scope,
            mElement: coreaudio_sys::kAudioObjectPropertyElementMain,
        };
        coreaudio_sys::AudioObjectSetPropertyData(
            device_id,
            &address,
            0,
            std::ptr::null(),
            std::mem::size_of::<u32>() as u32,
            &data as *const _ as *const std::ffi::c_void,
        ) == 0
    }
}

#[cfg(target_os = "macos")]
fn macos_audio_get(is_mute: bool) -> Option<String> {
    let device_id = macos_default_output_device()?;
    if is_mute {
        let muted = macos_get_u32(
            device_id,
            coreaudio_sys::kAudioDevicePropertyMute,
            coreaudio_sys::kAudioDevicePropertyScopeOutput,
        )?;
        return Some(if muted != 0 { "On" } else { "Off" }.to_string());
    }
    let level = macos_get_scalar(
        device_id,
        coreaudio_sys::kAudioDevicePropertyVolumeScalar,
        coreaudio_sys::kAudioDevicePropertyScopeOutput,
    )?;
    Some(format!(
        "{}",
        (level.clamp(0.0, 1.0) * 100.0).round() as i32
    ))
}

#[cfg(target_os = "macos")]
fn macos_audio_set(setting: &str, is_mute: bool) {
    let Some(device_id) = macos_default_output_device() else {
        return;
    };
    if is_mute {
        let on = matches!(setting, "1" | "On" | "ON" | "on");
        let _ = macos_set_u32(
            device_id,
            coreaudio_sys::kAudioDevicePropertyMute,
            coreaudio_sys::kAudioDevicePropertyScopeOutput,
            if on { 1 } else { 0 },
        );
        return;
    }
    let target_pct: f64 = if setting.starts_with('+') || setting.starts_with('-') {
        let current = macos_get_scalar(
            device_id,
            coreaudio_sys::kAudioDevicePropertyVolumeScalar,
            coreaudio_sys::kAudioDevicePropertyScopeOutput,
        )
        .unwrap_or(0.5);
        let delta: f64 = setting.parse().unwrap_or(0.0);
        (current as f64 * 100.0 + delta).clamp(0.0, 100.0)
    } else {
        setting.parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0)
    };
    let _ = macos_set_scalar(
        device_id,
        coreaudio_sys::kAudioDevicePropertyVolumeScalar,
        coreaudio_sys::kAudioDevicePropertyScopeOutput,
        (target_pct / 100.0) as f32,
    );
}

#[cfg(windows)]
fn get_default_audio_endpoint_volume(
) -> Option<windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume> {
    use windows::Win32::Media::Audio::Endpoints::*;
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok().ok()?;
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        device
            .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
            .ok()
    }
}

#[cfg(windows)]
fn windows_audio_get(is_mute: bool) -> Option<String> {
    let endpoint = get_default_audio_endpoint_volume()?;
    unsafe {
        if is_mute {
            let muted = endpoint.GetMute().ok()?;
            return Some(if muted.as_bool() { "On" } else { "Off" }.to_string());
        }
        let level = endpoint.GetMasterVolumeLevelScalar().ok()?;
        Some(format!("{}", (level * 100.0).round() as i32))
    }
}

#[cfg(windows)]
fn windows_audio_set(setting: &str, is_mute: bool) {
    let Some(endpoint) = get_default_audio_endpoint_volume() else {
        return;
    };
    unsafe {
        if is_mute {
            let on = matches!(setting, "1" | "On" | "ON" | "on");
            let _ = endpoint.SetMute(on, std::ptr::null());
            return;
        }
        if setting.starts_with('+') || setting.starts_with('-') {
            let cur = endpoint.GetMasterVolumeLevelScalar().unwrap_or(0.5);
            let delta: f64 = setting.parse::<f64>().unwrap_or(0.0_f64);
            let target = (cur as f64 + delta / 100.0).clamp(0.0, 1.0) as f32;
            let _ = endpoint.SetMasterVolumeLevelScalar(target, std::ptr::null());
        } else {
            let pct: f64 = setting.parse::<f64>().unwrap_or(0.0_f64).clamp(0.0, 100.0);
            let _ = endpoint.SetMasterVolumeLevelScalar((pct / 100.0) as f32, std::ptr::null());
        }
    }
}

fn sound_beep_compat(args: &[&str]) -> String {
    let frequency = args
        .first()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .filter(|_| !args.first().map_or(true, |v| v.trim().is_empty()));
    let duration = args
        .get(1)
        .and_then(|v| v.trim().parse::<u32>().ok())
        .filter(|_| !args.get(1).map_or(true, |v| v.trim().is_empty()));
    sound_beep(frequency, duration)
}

fn sound_play_compat(args: &[&str]) -> String {
    let filename = args.first().copied().unwrap_or_default();
    let wait_str = args.get(1).copied().unwrap_or_default().trim();
    let wait = matches!(wait_str.to_uppercase().as_str(), "1" | "WAIT");
    sound_play(filename, wait)
}

fn sound_get_compat(args: &[&str]) -> String {
    let component = args.first().copied().filter(|v| !v.trim().is_empty());
    let control = args.get(1).copied().filter(|v| !v.trim().is_empty());
    let device = args.get(2).and_then(|v| v.trim().parse::<u32>().ok());
    sound_get(component, control, device)
}

fn sound_set_compat(args: &[&str]) -> String {
    let new_setting = args.first().copied().unwrap_or_default();
    let component = args.get(1).copied().filter(|v| !v.trim().is_empty());
    let control = args.get(2).copied().filter(|v| !v.trim().is_empty());
    let device = args.get(3).and_then(|v| v.trim().parse::<u32>().ok());
    sound_set(new_setting, component, control, device)
}

fn sound_get_wave_volume_compat(args: &[&str]) -> String {
    let device = args.first().and_then(|v| v.trim().parse::<u32>().ok());
    sound_get_wave_volume(device)
}

fn sound_set_wave_volume_compat(args: &[&str]) -> String {
    let percent = args.first().copied().unwrap_or_default();
    let device = args.get(1).and_then(|v| v.trim().parse::<u32>().ok());
    sound_set_wave_volume(percent, device)
}

pub fn compat_sound_beep(args: &[&str]) -> String {
    sound_beep_compat(args)
}
pub fn compat_sound_play(args: &[&str]) -> String {
    sound_play_compat(args)
}
pub fn compat_sound_get(args: &[&str]) -> String {
    sound_get_compat(args)
}
pub fn compat_sound_set(args: &[&str]) -> String {
    sound_set_compat(args)
}
pub fn compat_sound_get_wave_volume(args: &[&str]) -> String {
    sound_get_wave_volume_compat(args)
}
pub fn compat_sound_set_wave_volume(args: &[&str]) -> String {
    sound_set_wave_volume_compat(args)
}

pub const METHODS: &[ModuleMethod] = &[
    ("SoundBeep", compat_sound_beep),
    ("SoundGet", compat_sound_get),
    ("SoundGetWaveVolume", compat_sound_get_wave_volume),
    ("SoundPlay", compat_sound_play),
    ("SoundSet", compat_sound_set),
    ("SoundSetWaveVolume", compat_sound_set_wave_volume),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_beep_returns_empty() {
        let result = compat_sound_beep(&["523", "50"]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sound_beep_defaults() {
        let result = compat_sound_beep(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sound_play_system_sound_codes() {
        for code in &["*-1", "*16", "*32", "*48", "*64"] {
            let result = sound_play(code, false);
            assert_eq!(result, "", "system sound code {code} should return empty");
        }
    }

    #[test]
    fn test_sound_play_missing_file_returns_empty() {
        let result = sound_play("/nonexistent/path/to/file.wav", false);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sound_get_wave_delegates() {
        let _ = sound_get_wave_volume(None);
    }

    #[test]
    fn test_sound_set_wave_delegates() {
        let _ = sound_set_wave_volume("50", None);
    }

    #[test]
    fn test_compat_sound_get_parses_args() {
        let result = compat_sound_get(&["MASTER", "VOLUME", "1"]);
        let _ = result;
    }

    #[test]
    fn test_compat_sound_set_returns_empty() {
        let result = compat_sound_set(&["50", "MASTER", "VOLUME", "1"]);
        assert_eq!(result, "");
    }
}
