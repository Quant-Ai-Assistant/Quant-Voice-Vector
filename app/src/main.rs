#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod recorder;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use qvv_core::{Engine, Match, ProfileBuilder, Sample, Source, VoiceprintFile, audio};
use serde::Serialize;
use tauri::{Manager, State};

use recorder::Recorder;

/// Clean speech needed before a profile can be saved.
const TARGET_SPEECH_SECONDS: f32 = 30.0;
const PROFILE_EXTENSION: &str = "qvoice.json";

/// A profile in the sidebar: a draft that is still collecting speech, or a saved one.
struct Entry {
    id: String,
    name: String,
    builder: ProfileBuilder,
    file: Option<VoiceprintFile>,
}

struct App {
    engine: Engine,
    /// Saved profiles live here as `<id>.qvoice.json`. Audio is never written to disk.
    profiles_dir: PathBuf,
    profiles: Mutex<Vec<Entry>>,
    recorder: Mutex<Option<Recorder>>,
    next_draft: AtomicU64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileView {
    id: String,
    name: String,
    ready: bool,
    speech_seconds: f32,
    target_seconds: f32,
    segments: usize,
    rejected: usize,
    can_finish: bool,
    created_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingProgress {
    seconds: f32,
    peak: f32,
}

type CommandResult<T> = Result<T, String>;

impl Entry {
    fn view(&self) -> ProfileView {
        let (speech_seconds, segments) = match &self.file {
            Some(file) => {
                (file.embeddings.iter().map(|s| s.speech_seconds).sum(), file.embeddings.len())
            }
            None => (self.builder.speech_seconds(), self.builder.accepted().len()),
        };
        ProfileView {
            id: self.id.clone(),
            name: self.name.clone(),
            ready: self.file.is_some(),
            speech_seconds,
            target_seconds: TARGET_SPEECH_SECONDS,
            segments,
            rejected: self.builder.rejected(),
            can_finish: self.file.is_none()
                && speech_seconds >= TARGET_SPEECH_SECONDS
                && segments >= qvv_core::profile::MIN_EMBEDDINGS,
            created_at: self.file.as_ref().map(|f| f.profile.created_at.clone()),
        }
    }
}

impl App {
    fn with_entry<T>(
        &self,
        id: &str,
        action: impl FnOnce(&mut Entry) -> CommandResult<T>,
    ) -> CommandResult<T> {
        let mut profiles = self.profiles.lock().unwrap();
        let entry = profiles.iter_mut().find(|e| e.id == id).ok_or("profile not found")?;
        action(entry)
    }

    fn profile_path(&self, id: &str) -> PathBuf {
        self.profiles_dir.join(format!("{id}.{PROFILE_EXTENSION}"))
    }

    fn enroll(&self, id: &str, samples: &[f32], source: Source) -> CommandResult<()> {
        // Vectors are computed before taking the lock: it is the slow part.
        let mut new_samples = Vec::new();
        for chunk in self.engine.chunks(samples).map_err(|e| e.to_string())? {
            new_samples.push(Sample {
                vector: self.engine.embed(&chunk.samples).map_err(|e| e.to_string())?,
                speech_seconds: chunk.seconds(),
                source,
            });
        }
        self.with_entry(id, |entry| {
            if entry.file.is_some() {
                return Err("profile is already saved".into());
            }
            new_samples.into_iter().for_each(|sample| entry.builder.add(sample));
            Ok(())
        })
    }

    fn compare(&self, id: &str, samples: &[f32]) -> CommandResult<Match> {
        let speech = self.engine.speech_only(samples).map_err(|e| e.to_string())?;
        let vector = self.engine.embed(&speech).map_err(|e| e.to_string())?;
        self.with_entry(id, |entry| {
            let file = entry.file.as_ref().ok_or("profile is not saved yet")?;
            Ok(file.compare(&vector))
        })
    }

    fn stop_recording(&self) -> CommandResult<Vec<f32>> {
        let recorder = self.recorder.lock().unwrap().take().ok_or("not recording")?;
        let recording = recorder.finish();
        Ok(audio::interleaved_to_16k(&recording.samples, recording.channels, recording.rate))
    }

    fn view(&self, id: &str) -> CommandResult<ProfileView> {
        self.with_entry(id, |entry| Ok(entry.view()))
    }
}

#[tauri::command]
fn list_profiles(app: State<App>) -> Vec<ProfileView> {
    app.profiles.lock().unwrap().iter().map(Entry::view).collect()
}

#[tauri::command]
fn create_profile(app: State<App>, name: String) -> CommandResult<ProfileView> {
    let name = name.trim();
    if name.is_empty() {
        return Err("profile name is empty".into());
    }
    let entry = Entry {
        id: format!("draft-{}", app.next_draft.fetch_add(1, Ordering::Relaxed)),
        name: name.to_string(),
        builder: ProfileBuilder::default(),
        file: None,
    };
    let view = entry.view();
    app.profiles.lock().unwrap().push(entry);
    Ok(view)
}

#[tauri::command]
fn delete_profile(app: State<App>, id: String) -> CommandResult<()> {
    let mut profiles = app.profiles.lock().unwrap();
    let index = profiles.iter().position(|e| e.id == id).ok_or("profile not found")?;
    if profiles[index].file.is_some() {
        std::fs::remove_file(app.profile_path(&id)).map_err(|e| e.to_string())?;
    }
    profiles.remove(index);
    Ok(())
}

#[tauri::command]
fn reset_profile(app: State<App>, id: String) -> CommandResult<ProfileView> {
    app.with_entry(&id, |entry| {
        entry.builder.clear();
        Ok(entry.view())
    })
}

#[tauri::command(async)]
fn add_files(app: State<App>, id: String, paths: Vec<String>) -> CommandResult<ProfileView> {
    for path in paths {
        let samples = audio::load_file(Path::new(&path)).map_err(|e| format!("{path}: {e}"))?;
        app.enroll(&id, &samples, Source::File)?;
    }
    app.view(&id)
}

#[tauri::command]
fn start_recording(app: State<App>) -> CommandResult<()> {
    let mut recorder = app.recorder.lock().unwrap();
    if recorder.is_none() {
        *recorder = Some(Recorder::start()?);
    }
    Ok(())
}

#[tauri::command]
fn recording_progress(app: State<App>) -> Option<RecordingProgress> {
    let recorder = app.recorder.lock().unwrap();
    recorder.as_ref().map(|r| {
        let (seconds, peak) = r.progress();
        RecordingProgress { seconds, peak }
    })
}

#[tauri::command(async)]
fn stop_recording_enroll(app: State<App>, id: String) -> CommandResult<ProfileView> {
    let samples = app.stop_recording()?;
    app.enroll(&id, &samples, Source::Microphone)?;
    app.view(&id)
}

#[tauri::command(async)]
fn stop_recording_verify(app: State<App>, id: String) -> CommandResult<Match> {
    let samples = app.stop_recording()?;
    app.compare(&id, &samples)
}

#[tauri::command(async)]
fn verify_file(app: State<App>, id: String, path: String) -> CommandResult<Match> {
    let samples = audio::load_file(Path::new(&path)).map_err(|e| e.to_string())?;
    app.compare(&id, &samples)
}

/// Turns a draft into a saved profile in the application data folder.
#[tauri::command]
fn finish_profile(app: State<App>, id: String) -> CommandResult<ProfileView> {
    let mut profiles = app.profiles.lock().unwrap();
    let entry = profiles.iter_mut().find(|e| e.id == id).ok_or("profile not found")?;
    let file = entry.builder.build(&entry.name).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app.profiles_dir).map_err(|e| e.to_string())?;
    std::fs::write(app.profile_path(&file.profile.id), file.to_json())
        .map_err(|e| e.to_string())?;
    entry.id = file.profile.id.clone();
    entry.builder.clear();
    entry.file = Some(file);
    Ok(entry.view())
}

#[tauri::command]
fn export_profile(app: State<App>, id: String, path: String) -> CommandResult<()> {
    app.with_entry(&id, |entry| {
        let file = entry.file.as_ref().ok_or("profile is not saved yet")?;
        std::fs::write(&path, file.to_json()).map_err(|e| e.to_string())
    })
}

fn load_saved(dir: &Path) -> Vec<Entry> {
    let Ok(files) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut entries: Vec<Entry> = files
        .flatten()
        .filter(|f| f.file_name().to_string_lossy().ends_with(PROFILE_EXTENSION))
        .filter_map(|f| std::fs::read_to_string(f.path()).ok())
        // Files made by another model or damaged ones are skipped, not deleted.
        .filter_map(|json| VoiceprintFile::from_json(&json).ok())
        .map(|file| Entry {
            id: file.profile.id.clone(),
            name: file.profile.name.clone(),
            builder: ProfileBuilder::default(),
            file: Some(file),
        })
        .collect();
    entries.sort_by(|a, b| {
        let created = |e: &Entry| e.file.as_ref().map(|f| f.profile.created_at.clone());
        created(a).cmp(&created(b))
    });
    entries
}

fn models_dir(app: &tauri::App) -> tauri::Result<PathBuf> {
    // A debug build runs unbundled, straight from the repository.
    if cfg!(debug_assertions) {
        return Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("../models"));
    }
    Ok(app.path().resource_dir()?.join("models"))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let engine = Engine::load(&models_dir(app)?)?;
            let profiles_dir = app.path().app_data_dir()?.join("profiles");
            app.manage(App {
                engine,
                profiles: Mutex::new(load_saved(&profiles_dir)),
                profiles_dir,
                recorder: Mutex::default(),
                next_draft: AtomicU64::new(1),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_profiles,
            create_profile,
            delete_profile,
            reset_profile,
            add_files,
            start_recording,
            recording_progress,
            stop_recording_enroll,
            stop_recording_verify,
            verify_file,
            finish_profile,
            export_profile,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Quant Voice Vector");
}
