//! PTND schemas 4–5 resource index. Pixels/source images are binary ZIP entries;
//! JSON holds descriptors only. SHA-256 addresses verify bytes and deduplicate
//! shared assets. Stable object IDs associate descriptors with canonical shapes.
use petunia_design_color::IccProfile;
use petunia_design_document::{Document, DocumentMutator, ShapeKind};
use petunia_design_foundation::{ObjectId, PetuniaError, SurfaceId};
use petunia_design_raster::{
    AlphaMode, EncodedImage, PixelFormat, RasterLayerKind, Tile, TileCoord, TileMap, TileState,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::{Read, Write},
    sync::Arc,
};

pub(crate) const INDEX_PATH: &str = "resources/index.json";
const MAX_INDEX_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RESOURCE_BYTES: usize = 256 * 1024 * 1024;
const MAX_RESOURCE_ENTRIES: usize = 8192;
const MAX_OBJECT_BINDINGS: usize = 100_000;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceIndex {
    version: u32,
    bindings: Vec<Binding>,
    #[serde(default)]
    profiles: Vec<ProfileBinding>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileBinding {
    surface: SurfaceId,
    name: String,
    asset: String,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum Binding {
    Image {
        object: ObjectId,
        asset: String,
    },
    Raster {
        object: ObjectId,
        width: u32,
        height: u32,
        kind: RasterLayerKind,
        format: PixelFormat,
        alpha_mode: AlphaMode,
        tiles: Vec<TileReference>,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TileReference {
    coord: TileCoord,
    state: TileState,
    asset: String,
}
#[derive(Clone)]
enum Asset {
    Image(Arc<EncodedImage>),
    Tile(Arc<Tile>),
    Profile(IccProfile),
}
impl Asset {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Image(image) => image.as_slice(),
            Self::Tile(tile) => &tile.data,
            Self::Profile(profile) => profile.bytes(),
        }
    }
}
fn key(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut result, "{byte:02x}").expect("string formatting");
    }
    result
}
fn valid_key(key: &str) -> bool {
    key.len() == 64
        && key
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn insert_asset(
    assets: &mut BTreeMap<String, Asset>,
    asset: Asset,
    bytes: &mut usize,
) -> Result<String, PetuniaError> {
    let hash = key(asset.bytes());
    if !assets.contains_key(&hash) {
        *bytes = bytes.saturating_add(asset.bytes().len());
        if *bytes > MAX_RESOURCE_BYTES || assets.len() >= MAX_RESOURCE_ENTRIES {
            return Err(PetuniaError::invalid_input(
                "native binary resource budget exceeded",
            ));
        }
        assets.insert(hash.clone(), asset);
    }
    Ok(hash)
}
/// Streams binaries, returning the descriptor-only document to serialize.
pub(crate) fn write(
    document: &Document,
    zip: &mut zip::ZipWriter<File>,
) -> Result<Document, PetuniaError> {
    let mut metadata = document.clone();
    let mut bindings = Vec::new();
    let mut assets = BTreeMap::new();
    let mut bytes = 0usize;
    let mut replacements = Vec::new();
    let mut profiles = Vec::new();
    for surface in document.surfaces() {
        if let Some(profile) = &surface.cmyk_profile {
            let asset = insert_asset(&mut assets, Asset::Profile(profile.clone()), &mut bytes)?;
            profiles.push(ProfileBinding {
                surface: surface.id,
                name: profile.name().to_owned(),
                asset,
            });
            DocumentMutator::new(&mut metadata).set_surface_cmyk_profile(surface.id, None)?;
        }
    }
    for object in document.surfaces().iter().flat_map(|s| s.objects()) {
        match &object.shape {
            Some(ShapeKind::Image {
                path,
                data: Some(data),
            }) => {
                let asset = insert_asset(&mut assets, Asset::Image(data.clone()), &mut bytes)?;
                bindings.push(Binding::Image {
                    object: object.id,
                    asset,
                });
                replacements.push((
                    object.id,
                    ShapeKind::Image {
                        path: path.clone(),
                        data: None,
                    },
                ));
            }
            Some(ShapeKind::Image { data: None, .. }) => {
                return Err(PetuniaError::capability_unavailable("native save requires an embedded image original; unresolved linked images cannot be saved faithfully"));
            }
            Some(ShapeKind::Raster { layer }) => {
                let mut tiles = Vec::with_capacity(layer.tiles().resident_tile_count());
                for (coord, tile) in layer.tiles().tiles() {
                    let asset = insert_asset(&mut assets, Asset::Tile(tile.clone()), &mut bytes)?;
                    tiles.push(TileReference {
                        coord: *coord,
                        state: tile.state,
                        asset,
                    });
                }
                bindings.push(Binding::Raster {
                    object: object.id,
                    width: layer.width(),
                    height: layer.height(),
                    kind: layer.kind(),
                    format: layer.tiles().format,
                    alpha_mode: layer.tiles().alpha_mode,
                    tiles,
                });
                replacements.push((
                    object.id,
                    ShapeKind::Raster {
                        layer: Arc::new(layer.without_tiles()),
                    },
                ));
            }
            _ => {}
        }
        if bindings.len() > MAX_OBJECT_BINDINGS {
            return Err(PetuniaError::invalid_input(
                "native resource binding budget exceeded",
            ));
        }
    }
    let index = ResourceIndex {
        version: 2,
        bindings,
        profiles,
    };
    // Bound descriptors before ZIP publication. This allocation never contains pixel bytes.
    let index_bytes =
        serde_json::to_vec(&index).map_err(|e| PetuniaError::io(format!("resource index: {e}")))?;
    if index_bytes.len() as u64 > MAX_INDEX_BYTES {
        return Err(PetuniaError::invalid_input(
            "native resource index budget exceeded",
        ));
    }
    DocumentMutator::new(&mut metadata).set_shapes_bulk(replacements)?;
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(INDEX_PATH, options)
        .map_err(|e| PetuniaError::io(format!("resource index entry: {e}")))?;
    zip.write_all(&index_bytes)
        .map_err(|e| PetuniaError::io(format!("write resource index: {e}")))?;
    for (hash, asset) in assets {
        zip.start_file(format!("resources/{hash}.bin"), options)
            .map_err(|e| PetuniaError::io(format!("resource entry: {e}")))?;
        zip.write_all(asset.bytes())
            .map_err(|e| PetuniaError::io(format!("write resource: {e}")))?;
    }
    Ok(metadata)
}
fn read_bytes(
    archive: &mut zip::ZipArchive<File>,
    name: &str,
    limit: u64,
) -> Result<Vec<u8>, PetuniaError> {
    let file = archive
        .by_name(name)
        .map_err(|e| PetuniaError::invalid_input(format!("missing resource `{name}`: {e}")))?;
    if file.size() > limit {
        return Err(PetuniaError::invalid_input(
            "native resource entry exceeds budget",
        ));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(file.size() as usize)
        .map_err(|_| PetuniaError::invalid_input("native resource allocation"))?;
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| PetuniaError::io(format!("read native resource: {e}")))?;
    if bytes.len() as u64 > limit {
        return Err(PetuniaError::invalid_input(
            "native resource decoded size exceeds budget",
        ));
    }
    Ok(bytes)
}
/// Resolves every mandatory resource exactly once. Missing, altered, duplicate
/// or unreferenced binaries are errors; no incomplete document is returned.
pub(crate) fn read(
    document: &mut Document,
    archive: &mut zip::ZipArchive<File>,
) -> Result<(), PetuniaError> {
    if document
        .surfaces()
        .iter()
        .any(|surface| surface.cmyk_profile.is_some())
    {
        return Err(PetuniaError::invalid_input(
            "ICC profiles must be bound binary resources",
        ));
    }
    let mut names = HashSet::new();
    for name in archive.file_names() {
        if !names.insert(name.to_string()) {
            return Err(PetuniaError::invalid_input("duplicate native ZIP entry"));
        }
    }
    let index_bytes = read_bytes(archive, INDEX_PATH, MAX_INDEX_BYTES)?;
    let index: ResourceIndex = serde_json::from_slice(&index_bytes)
        .map_err(|e| PetuniaError::invalid_input(format!("native resource index: {e}")))?;
    if !matches!(index.version, 1 | 2)
        || index.bindings.len() > MAX_OBJECT_BINDINGS
        || index.profiles.len() > 1024
        || (index.version == 1 && !index.profiles.is_empty())
    {
        return Err(PetuniaError::invalid_input(
            "unsupported or oversized native resource index",
        ));
    }
    let mut requirements: BTreeMap<String, usize> = BTreeMap::new();
    let mut objects = HashSet::new();
    for binding in &index.bindings {
        let (id, resources): (ObjectId, Vec<(&String, usize)>) = match binding {
            Binding::Image { object, asset } => (*object, vec![(asset, EncodedImage::MAX_BYTES)]),
            Binding::Raster {
                object,
                tiles,
                format,
                ..
            } => {
                if tiles.len() > petunia_design_raster::tile::MAX_RESIDENT_TILES {
                    return Err(PetuniaError::invalid_input(
                        "native layer tile count exceeded",
                    ));
                }
                (
                    *object,
                    tiles
                        .iter()
                        .map(|tile| {
                            (
                                &tile.asset,
                                petunia_design_raster::TILE_SIZE.pow(2) * format.bytes_per_pixel(),
                            )
                        })
                        .collect(),
                )
            }
        };
        if !objects.insert(id) {
            return Err(PetuniaError::invalid_input(
                "duplicate native resource binding",
            ));
        }
        for (hash, limit) in resources {
            if !valid_key(hash) {
                return Err(PetuniaError::invalid_input(
                    "invalid native resource content key",
                ));
            }
            requirements
                .entry(hash.clone())
                .and_modify(|value| *value = (*value).min(limit))
                .or_insert(limit);
        }
    }
    let mut profiled_surfaces = HashSet::new();
    for profile in &index.profiles {
        if !profiled_surfaces.insert(profile.surface)
            || !valid_key(&profile.asset)
            || profile.name.len() > 512
            || profile.name.is_empty()
            || document
                .surface(profile.surface)
                .map_or(true, |s| s.cmyk_profile.is_some())
        {
            return Err(PetuniaError::invalid_input(
                "invalid or duplicate ICC surface binding",
            ));
        }
        requirements
            .entry(profile.asset.clone())
            .and_modify(|v| *v = (*v).min(4 * 1024 * 1024))
            .or_insert(4 * 1024 * 1024);
    }
    if requirements.len() > MAX_RESOURCE_ENTRIES {
        return Err(PetuniaError::invalid_input(
            "native resource entry count exceeded",
        ));
    }
    let mut expected = HashSet::from([
        "manifest.json".to_owned(),
        "document/document.json".to_owned(),
        INDEX_PATH.to_owned(),
    ]);
    expected.extend(
        requirements
            .keys()
            .map(|key| format!("resources/{key}.bin")),
    );
    if names != expected {
        return Err(PetuniaError::invalid_input(
            "native resource entry set is incomplete or contains unreferenced entries",
        ));
    }
    // Check the aggregate advertised size before any binary allocations.
    let mut total = 0usize;
    for (hash, limit) in &requirements {
        let file = archive
            .by_name(&format!("resources/{hash}.bin"))
            .map_err(|e| PetuniaError::invalid_input(e.to_string()))?;
        if file.size() > *limit as u64 {
            return Err(PetuniaError::invalid_input("native resource size mismatch"));
        }
        total = total.saturating_add(file.size() as usize);
        if total > MAX_RESOURCE_BYTES {
            return Err(PetuniaError::invalid_input(
                "native aggregate resource byte budget exceeded",
            ));
        }
    }
    let mut binaries = HashMap::new();
    for (hash, limit) in requirements {
        let bytes = read_bytes(archive, &format!("resources/{hash}.bin"), limit as u64)?;
        if key(&bytes) != hash {
            return Err(PetuniaError::invalid_input(
                "native resource digest mismatch",
            ));
        }
        binaries.insert(hash, Arc::new(bytes));
    }
    let mut images: HashMap<String, Arc<EncodedImage>> = HashMap::new();
    let mut tile_cache: HashMap<(String, TileCoord, PixelFormat, AlphaMode, TileState), Arc<Tile>> =
        HashMap::new();
    let mut replacements = Vec::new();
    let mut resolved_bytes = 0usize;
    let lookup: HashMap<_, _> = document
        .surfaces()
        .iter()
        .flat_map(|s| s.objects())
        .map(|object| (object.id, object.shape.clone()))
        .collect();
    for binding in index.bindings {
        match binding {
            Binding::Image { object, asset } => {
                let Some(ShapeKind::Image { path, data: None }) =
                    lookup.get(&object).and_then(Option::as_ref)
                else {
                    return Err(PetuniaError::invalid_input(
                        "image resource binding does not match descriptor",
                    ));
                };
                let path = path.clone();
                let image = if let Some(image) = images.get(&asset) {
                    image.clone()
                } else {
                    let bytes = binaries
                        .get(&asset)
                        .ok_or_else(|| PetuniaError::invalid_input("missing image resource"))?
                        .clone();
                    resolved_bytes = resolved_bytes.saturating_add(bytes.len());
                    if resolved_bytes > MAX_RESOURCE_BYTES {
                        return Err(PetuniaError::invalid_input(
                            "native resolved resource budget exceeded",
                        ));
                    }
                    let image = Arc::new(
                        EncodedImage::new(bytes.as_ref().clone())
                            .map_err(|e| PetuniaError::invalid_input(e.to_string()))?,
                    );
                    images.insert(asset, image.clone());
                    image
                };
                replacements.push((
                    object,
                    ShapeKind::Image {
                        path,
                        data: Some(image),
                    },
                ));
            }
            Binding::Raster {
                object,
                width,
                height,
                kind,
                format,
                alpha_mode,
                tiles,
            } => {
                let Some(ShapeKind::Raster { layer }) =
                    lookup.get(&object).and_then(Option::as_ref)
                else {
                    return Err(PetuniaError::invalid_input(
                        "raster resource binding does not match descriptor",
                    ));
                };
                if layer.width() != width
                    || layer.height() != height
                    || layer.kind() != kind
                    || layer.tiles().format != format
                    || layer.tiles().alpha_mode != alpha_mode
                    || layer.tiles().resident_tile_count() != 0
                {
                    return Err(PetuniaError::invalid_input(
                        "raster resource descriptor mismatch",
                    ));
                }
                let mut resolved = Vec::with_capacity(tiles.len());
                for tile in tiles {
                    let cache_key = (
                        tile.asset.clone(),
                        tile.coord,
                        format,
                        alpha_mode,
                        tile.state,
                    );
                    let resource = if let Some(resource) = tile_cache.get(&cache_key) {
                        resource.clone()
                    } else {
                        let data = binaries
                            .get(&tile.asset)
                            .ok_or_else(|| PetuniaError::invalid_input("missing tile resource"))?
                            .clone();
                        let resource = Arc::new(Tile {
                            coord: tile.coord,
                            format,
                            alpha_mode,
                            state: tile.state,
                            data,
                        });
                        tile_cache.insert(cache_key, resource.clone());
                        resource
                    };
                    resolved.push(resource);
                }
                let layer = layer.with_tiles(TileMap::from_tiles(format, alpha_mode, resolved)?)?;
                replacements.push((
                    object,
                    ShapeKind::Raster {
                        layer: Arc::new(layer),
                    },
                ));
            }
        }
    }
    drop(lookup);
    let mut mutator = DocumentMutator::new(document);
    mutator.set_shapes_bulk(replacements)?;
    for binding in index.profiles {
        let bytes = binaries
            .get(&binding.asset)
            .ok_or_else(|| PetuniaError::invalid_input("missing ICC resource"))?
            .clone();
        let profile = IccProfile::new(binding.name, bytes)?;
        mutator.set_surface_cmyk_profile(binding.surface, Some(profile))?;
    }
    for object in document.surfaces().iter().flat_map(|s| s.objects()) {
        if matches!(
            object.shape,
            Some(ShapeKind::Raster { .. }) | Some(ShapeKind::Image { .. })
        ) && !objects.contains(&object.id)
        {
            return Err(PetuniaError::invalid_input(
                "raster descriptor has no resource binding",
            ));
        }
    }
    document.validate()
}
