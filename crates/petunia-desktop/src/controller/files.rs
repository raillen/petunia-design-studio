//! Native paths and explicit file-write policies.
use super::*;
use petunia_engine::persistence::SavePolicy;
use std::io::Write;
use std::path::Path;

impl DesktopController {
    pub(super) fn file_command(
        &mut self,
        command: &str,
        payload: &Value,
    ) -> Result<(), DesktopError> {
        self.require_idle()?;
        match command {
            "new" | "open" => {
                if self.session.is_dirty() && !boolean_or(payload, "discard", false)? {
                    return Err(DesktopError::UnsavedDocument);
                }
                if command == "open" {
                    let path = native_path(string(payload, "path")?)?;
                    if path
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .is_some_and(|extension| {
                            extension.eq_ignore_ascii_case("svg")
                                || extension.eq_ignore_ascii_case("png")
                        })
                    {
                        return invalid("Use um arquivo PTND. Importação SVG/PNG ainda não está conectada a este comando.");
                    }
                    self.session.open(&path)?;
                    self.status = "Documento aberto".into();
                } else {
                    let view = *self.session.view();
                    let shortcuts = self.session.shortcuts().clone();
                    let workspace = self.session.workspace().clone();
                    self.session = StudioSession::new(
                        payload
                            .get("title")
                            .and_then(Value::as_str)
                            .unwrap_or("Sem título"),
                    );
                    *self.session.view_mut() = view;
                    *self.session.shortcuts_mut() = shortcuts;
                    *self.session.workspace_mut() = workspace;
                    self.status = "Novo documento".into();
                }
                self.overlays.clear();
                self.fit();
                Ok(())
            }
            "save" => {
                self.session.save()?;
                self.status = "Documento salvo".into();
                Ok(())
            }
            "saveAs" => {
                let path = native_path(string(payload, "path")?)?;
                let policy = if boolean_or(payload, "overwrite", false)? {
                    SavePolicy::Overwrite
                } else {
                    SavePolicy::CreateNew
                };
                self.session.save_as(&path, policy)?;
                self.status = "Documento salvo".into();
                Ok(())
            }
            "exportPng" => {
                let path = native_path(string(payload, "path")?)?;
                let overwrite = boolean_or(payload, "overwrite", false)?;
                self.export_png(&path, overwrite)?;
                self.status = "PNG exportado; documento permanece editável".into();
                Ok(())
            }
            _ => invalid("Comando de arquivo desconhecido"),
        }
    }

    fn export_png(&self, path: &Path, overwrite: bool) -> Result<(), DesktopError> {
        let (width, height) = self.page_size();
        if width <= 0.0 || height <= 0.0 || width * height > (16 << 20) as f64 {
            return invalid("Página excede o limite de exportação PNG");
        }
        let width = width.ceil() as u32;
        let height = height.ceil() as u32;
        let (snapshot, _) = self
            .session
            .compile_snapshot(petunia_render_model::RenderQuality::Authoring);
        let frame = petunia_engine::compile::headless_page_frame(
            snapshot,
            self.session.active_page(),
            width,
            height,
            1.0,
        )
        .ok_or_else(|| DesktopError::Invalid("Página ativa inexistente".into()))?;
        let mut renderer = petunia_render::SoftwareRenderer::new(64 << 20, 64 << 20);
        let options = petunia_render::RenderOptions {
            background: petunia_core::ColorRgba::TRANSPARENT,
            ..Default::default()
        };
        let (pixels, _) = petunia_render::RenderBackend::render(&mut renderer, &frame, &options)
            .map_err(|error| DesktopError::Invalid(error.to_string()))?;
        let mut bytes = Vec::new();
        image::ImageEncoder::write_image(
            image::codecs::png::PngEncoder::new(&mut bytes),
            &pixels,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )?;
        // Write and sync a sibling before publishing. Failed writes never
        // truncate existing artwork; create-new cannot race an existing target.
        let directory = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let temporary = directory.join(format!(".petunia-export-{}.png", uuid::Uuid::new_v4()));
        let result = (|| -> Result<(), DesktopError> {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            if overwrite {
                std::fs::rename(&temporary, path)?;
            } else {
                std::fs::hard_link(&temporary, path)?;
            }
            Ok(())
        })();
        if temporary.exists() {
            let _ = std::fs::remove_file(&temporary);
        }
        result?;
        Ok(())
    }

    pub(super) fn page_size(&self) -> (f64, f64) {
        let size = self
            .session
            .document()
            .pages
            .get(self.session.active_page())
            .map(|page| page.spec.size)
            .unwrap_or_else(|| self.session.document().default_page_size());
        (size.width, size.height)
    }
}

/// Qt FileDialog returns local file URLs. Reject remote schemes instead of
/// interpreting them as filenames; decode UTF-8 percent escapes exactly once.
fn native_path(text: &str) -> Result<PathBuf, DesktopError> {
    if text.is_empty() || text.contains('\0') {
        return invalid("Escolha um caminho de arquivo válido");
    }
    let value = if let Some(url) = text.strip_prefix("file://") {
        let path = if url.starts_with('/') {
            url
        } else if let Some(local) = url.strip_prefix("localhost/") {
            return decode_path(&format!("/{local}"));
        } else {
            return invalid("Somente arquivos locais são aceitos");
        };
        return decode_path(path);
    } else if text.contains("://") {
        return invalid("Somente arquivos locais são aceitos");
    } else {
        text
    };
    Ok(PathBuf::from(value))
}

fn decode_path(text: &str) -> Result<PathBuf, DesktopError> {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return invalid("URL de arquivo incompleta");
            }
            let high = char::from(bytes[index + 1])
                .to_digit(16)
                .ok_or_else(|| DesktopError::Invalid("Escape inválido na URL".into()))?;
            let low = char::from(bytes[index + 2])
                .to_digit(16)
                .ok_or_else(|| DesktopError::Invalid("Escape inválido na URL".into()))?;
            decoded.push((high * 16 + low) as u8);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    let path = String::from_utf8(decoded)
        .map_err(|_| DesktopError::Invalid("Caminho precisa ser UTF-8 válido".into()))?;
    if path.contains('\0') {
        return invalid("Caminho contém caractere nulo");
    }
    #[cfg(target_os = "windows")]
    let path = if path.as_bytes().get(2) == Some(&b':') {
        path[1..].to_string()
    } else {
        path
    };
    Ok(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_paths_decode_without_accepting_remote_urls() {
        assert_eq!(
            native_path("file:///tmp/Arte%20%C3%A1.ptnd").unwrap(),
            PathBuf::from("/tmp/Arte á.ptnd")
        );
        assert_eq!(
            native_path("/tmp/100%.ptnd").unwrap(),
            PathBuf::from("/tmp/100%.ptnd")
        );
        assert!(native_path("file://remote/tmp/file.ptnd").is_err());
        assert!(native_path("https://example.test/art.ptnd").is_err());
        assert!(native_path("file:///tmp/%00.ptnd").is_err());
        assert!(native_path("file:///tmp/%x0.ptnd").is_err());
    }
}
