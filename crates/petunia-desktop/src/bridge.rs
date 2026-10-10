//! The sole native QObject boundary; snapshots and pixels leave Rust here.
use crate::controller::{DesktopController, DesktopError};
use base64::Engine;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use image::ImageEncoder;
use std::{path::PathBuf, pin::Pin};

// SAFETY: Only the generated CXX-Qt registration functions are invoked.
pub fn initialize_native() {
    cxx_qt::init_crate!(petunia_desktop);
    cxx_qt::init_qml_module!("Petunia.Studio");
}

#[cxx_qt::bridge]
pub mod ffi {
    // SAFETY: QString is the CXX-Qt library's audited Qt value wrapper.
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("capture.h");
        #[namespace = "petunia::desktop"]
        #[rust_name = "capture_studio_window"]
        fn captureStudioWindow(path: &QString, frame_source: &QString) -> bool;
    }
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, state_json, cxx_name = "stateJson")]
        #[qproperty(QString, frame_source, cxx_name = "frameSource")]
        type StudioBackend = super::StudioBackendRust;

        #[qinvokable]
        fn invoke(self: Pin<&mut Self>, command: &QString, payload: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "renderFrame"]
        fn render_frame(self: Pin<&mut Self>, width: i32, height: i32, dpr: f64);
        #[qinvokable]
        #[cxx_name = "iconSource"]
        fn icon_source(&self, key: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "captureWindow"]
        fn capture_window(&self, path: &QString) -> bool;
    }
}

pub struct StudioBackendRust {
    state_json: QString,
    frame_source: QString,
    controller: DesktopController,
}

fn preference_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PETUNIA_CONFIG_DIR") {
        return Some(PathBuf::from(path).join("preferences.json"));
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|path| PathBuf::from(path).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|path| PathBuf::from(path).join(".config")));
    base.map(|path| path.join("petunia-studio/preferences.json"))
}

impl Default for StudioBackendRust {
    fn default() -> Self {
        let mut controller = preference_path()
            .map_or_else(DesktopController::new, DesktopController::with_preferences);
        let state_json = QString::from(controller.state().to_string());
        Self {
            controller,
            state_json,
            frame_source: QString::default(),
        }
    }
}

impl ffi::StudioBackend {
    pub fn capture_window(&self, path: &QString) -> bool {
        // A diagnostic hook is deliberately limited to explicit smoke runs.
        if !std::env::args().any(|arg| arg == "--smoke-test") {
            return false;
        }
        // Keep an owned Qt value across the diagnostic event loop: callbacks
        // may publish a newer Rust frame while Qt finishes the scenegraph.
        let expected_frame = self.rust().frame_source.clone();
        if !ffi::capture_studio_window(path, &expected_frame) {
            return false;
        }
        let source = self.rust().frame_source.to_string();
        if let Some(encoded) = source.strip_prefix("data:image/png;base64,") {
            let frame_path = PathBuf::from(path.to_string()).with_extension("frame.png");
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(encoded) {
                if std::fs::write(frame_path, bytes).is_err() {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }
        true
    }
    fn publish(mut self: Pin<&mut Self>, error: Option<String>) {
        let mut state = self.as_mut().rust_mut().controller.state();
        if let Some(error) = error {
            state["error"] = error.clone().into();
            state["status"] = error.into();
        }
        self.set_state_json(QString::from(state.to_string()));
    }

    pub fn invoke(mut self: Pin<&mut Self>, command: &QString, payload: &QString) -> bool {
        let payload = payload.to_string();
        let result = if payload.len() > 65536 {
            Err(DesktopError::Invalid(
                "Os parâmetros excedem o limite".into(),
            ))
        } else {
            serde_json::from_str(&payload)
                .map_err(DesktopError::from)
                .and_then(|value| {
                    self.as_mut()
                        .rust_mut()
                        .controller
                        .invoke(&command.to_string(), &value)
                })
        };
        let accepted = result.is_ok();
        self.publish(result.err().map(|error| error.to_string()));
        accepted
    }

    pub fn render_frame(mut self: Pin<&mut Self>, width: i32, height: i32, dpr: f64) {
        let result = (|| -> Result<String, DesktopError> {
            let width = u32::try_from(width)
                .map_err(|_| DesktopError::Invalid("Largura inválida".into()))?;
            let height = u32::try_from(height)
                .map_err(|_| DesktopError::Invalid("Altura inválida".into()))?;
            let pixels = self
                .as_mut()
                .rust_mut()
                .controller
                .render_rgba(width, height, dpr)?;
            let mut encoded = Vec::new();
            image::codecs::png::PngEncoder::new(&mut encoded).write_image(
                &pixels,
                (f64::from(width) * dpr).ceil() as u32,
                (f64::from(height) * dpr).ceil() as u32,
                image::ExtendedColorType::Rgba8,
            )?;
            Ok(format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(encoded)
            ))
        })();
        match result {
            Ok(source) => {
                self.as_mut().set_frame_source(QString::from(source));
                self.publish(None);
            }
            Err(error) => self.publish(Some(error.to_string())),
        }
    }

    pub fn icon_source(&self, key: &QString) -> QString {
        let preferences = &self.rust().controller.preferences;
        let tint = match preferences.appearance {
            petunia_ui::Appearance::Light => "#18212B",
            petunia_ui::Appearance::Dark => "#F3F5F7",
            petunia_ui::Appearance::HighContrast => "#FFFFFF",
        };
        QString::from(crate::icons::source(
            &key.to_string(),
            preferences.icon_family,
            preferences.icon_style,
            tint,
        ))
    }
}
