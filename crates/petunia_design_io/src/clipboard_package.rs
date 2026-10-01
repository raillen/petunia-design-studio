//! Lossless native clipboard fragments reuse PTND binary-resource validation.
//! Temporary files are private to this operation and removed with their directory.
use petunia_design_document::{Document, DocumentMutator, DocumentObject};
use petunia_design_foundation::{PetuniaError, SurfaceId};
use std::io::Read;
const MAX_BYTES: usize = 16 * 1024 * 1024;
fn invalid(reason: &str) -> PetuniaError {
    PetuniaError::invalid_input(reason)
}
pub fn encode_clipboard_fragment(
    objects: Vec<DocumentObject>,
    frame: [f64; 4],
) -> Result<Vec<u8>, PetuniaError> {
    if objects.len() > 10_000 {
        return Err(invalid("clipboard object budget exceeded"));
    }
    let mut document = Document::new();
    let surface = SurfaceId::new(1);
    {
        let mut mutator = DocumentMutator::new(&mut document);
        mutator.add_surface(surface, "Clipboard")?;
        mutator.set_surface_geometry(surface, [frame[0], frame[1]], [frame[2], frame[3]])?;
        mutator.add_objects_bulk(surface, objects)?;
    }
    let directory = tempfile::Builder::new()
        .prefix("petunia-clipboard-")
        .tempdir()
        .map_err(|e| PetuniaError::io(e.to_string()))?;
    let path = directory.path().join("fragment.PTND");
    crate::save_package(&document, &path)?;
    let file = std::fs::File::open(&path).map_err(|e| PetuniaError::io(e.to_string()))?;
    if file
        .metadata()
        .map_err(|e| PetuniaError::io(e.to_string()))?
        .len()
        > MAX_BYTES as u64
    {
        return Err(invalid("native clipboard transfer exceeds 16 MiB"));
    }
    let mut bytes = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| PetuniaError::io(e.to_string()))?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("native clipboard transfer exceeds 16 MiB"));
    }
    Ok(bytes)
}
pub fn decode_clipboard_fragment(bytes: &[u8]) -> Result<Document, PetuniaError> {
    if bytes.len() > MAX_BYTES {
        return Err(invalid("native clipboard transfer exceeds 16 MiB"));
    }
    let directory = tempfile::Builder::new()
        .prefix("petunia-clipboard-")
        .tempdir()
        .map_err(|e| PetuniaError::io(e.to_string()))?;
    let path = directory.path().join("fragment.PTND");
    std::fs::write(&path, bytes).map_err(|e| PetuniaError::io(e.to_string()))?;
    let opened = crate::open_package(&path)?;
    let document = opened.document;
    if document.surfaces().len() != 1 || document.surfaces()[0].objects().len() > 10_000 {
        return Err(invalid("invalid native clipboard fragment scope"));
    }
    Ok(document)
}
