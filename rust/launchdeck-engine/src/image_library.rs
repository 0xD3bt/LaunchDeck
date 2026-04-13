#![allow(non_snake_case, dead_code)]

use crate::paths;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: String,
    pub fileName: String,
    pub name: String,
    pub tags: Vec<String>,
    pub category: String,
    pub isFavorite: bool,
    pub createdAt: u128,
    pub updatedAt: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageLibrary {
    pub images: Vec<ImageRecord>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SerializedImageRecord {
    pub id: String,
    pub fileName: String,
    pub name: String,
    pub tags: Vec<String>,
    pub category: String,
    pub isFavorite: bool,
    pub createdAt: u128,
    pub updatedAt: u128,
    pub previewUrl: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageLibraryPayload {
    pub ok: bool,
    pub images: Vec<SerializedImageRecord>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone)]
struct CachedImageLibrary {
    modified_ms: u128,
    library: ImageLibrary,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn sanitize_base_name(value: &str, fallback: &str) -> String {
    let base = value
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || char == '-' || char == '_' {
                char
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if base.is_empty() {
        fallback.to_string()
    } else {
        base
    }
}

fn supported_extension(content_type: &str) -> Option<&'static str> {
    match content_type.trim().to_ascii_lowercase().as_str() {
        "image/png" => Some(".png"),
        "image/jpeg" | "image/jpg" => Some(".jpg"),
        "image/avif" => Some(".avif"),
        "image/webp" => Some(".webp"),
        "image/gif" => Some(".gif"),
        _ => None,
    }
}

pub fn ensure_local_dirs() -> Result<(), String> {
    fs::create_dir_all(paths::local_root_dir()).map_err(|error| error.to_string())?;
    fs::create_dir_all(paths::uploads_dir()).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn read_image_library() -> Result<ImageLibrary, String> {
    ensure_local_dirs()?;
    let path = paths::image_library_path();
    let modified_ms = fs::metadata(&path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let cache = image_library_cache();
    if let Ok(guard) = cache.lock()
        && let Some(cached) = guard.as_ref()
        && cached.modified_ms == modified_ms
    {
        return Ok(cached.library.clone());
    }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(ImageLibrary::default());
    }
    let parsed: ImageLibrary = serde_json::from_str(&raw).unwrap_or_default();
    let mut categories = parsed
        .categories
        .into_iter()
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>();
    let mut images = Vec::new();
    let mut image_categories = BTreeSet::new();
    for entry in parsed.images {
        let file_name = entry.fileName.trim().to_string();
        if file_name.is_empty() {
            continue;
        }
        let category = entry.category.trim().to_string();
        if !category.is_empty() {
            image_categories.insert(category.clone());
        }
        images.push(ImageRecord {
            id: if entry.id.trim().is_empty() {
                file_name.clone()
            } else {
                entry.id
            },
            fileName: file_name.clone(),
            name: if entry.name.trim().is_empty() {
                Path::new(&file_name)
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("image")
                    .to_string()
            } else {
                entry.name.trim().to_string()
            },
            tags: entry
                .tags
                .into_iter()
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty())
                .collect(),
            category,
            isFavorite: entry.isFavorite,
            createdAt: if entry.createdAt == 0 {
                now_ms()
            } else {
                entry.createdAt
            },
            updatedAt: if entry.updatedAt == 0 {
                if entry.createdAt == 0 {
                    now_ms()
                } else {
                    entry.createdAt
                }
            } else {
                entry.updatedAt
            },
        });
    }
    categories.extend(image_categories);
    categories.sort_by_key(|entry| entry.to_lowercase());
    categories.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    let library = ImageLibrary { images, categories };
    if let Ok(mut guard) = cache.lock() {
        *guard = Some(CachedImageLibrary {
            modified_ms,
            library: library.clone(),
        });
    }
    Ok(library)
}

pub fn write_image_library(library: &ImageLibrary) -> Result<(), String> {
    ensure_local_dirs()?;
    fs::write(
        paths::image_library_path(),
        serde_json::to_vec_pretty(library).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    invalidate_image_library_cache();
    Ok(())
}

pub fn serialize_image_record(record: &ImageRecord) -> SerializedImageRecord {
    SerializedImageRecord {
        id: record.id.clone(),
        fileName: record.fileName.clone(),
        name: record.name.clone(),
        tags: record.tags.clone(),
        category: record.category.clone(),
        isFavorite: record.isFavorite,
        createdAt: record.createdAt,
        updatedAt: record.updatedAt,
        previewUrl: format!("/uploads/{}", record.fileName),
    }
}

fn create_image_record(file_name: &str, original_name: &str) -> ImageRecord {
    let base_name = Path::new(original_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_else(|| {
            Path::new(file_name)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("image")
        });
    ImageRecord {
        id: format!("{}-{}", now_ms(), &uuid::Uuid::new_v4().to_string()[..6]),
        fileName: file_name.to_string(),
        name: base_name.to_string(),
        tags: vec![],
        category: String::new(),
        isFavorite: false,
        createdAt: now_ms(),
        updatedAt: now_ms(),
    }
}

pub fn build_image_library_payload(
    search: &str,
    category: &str,
    favorites_only: bool,
) -> Result<ImageLibraryPayload, String> {
    let library = read_image_library()?;
    let normalized_search = search.trim().to_lowercase();
    let normalized_category = category.trim().to_lowercase();
    let mut images = library
        .images
        .into_iter()
        .filter(|record| paths::uploads_dir().join(&record.fileName).exists())
        .filter(|record| {
            if favorites_only && !record.isFavorite {
                return false;
            }
            if !normalized_category.is_empty()
                && normalized_category != "all"
                && normalized_category != "favorites"
                && record.category.trim().to_lowercase() != normalized_category
            {
                return false;
            }
            if normalized_search.is_empty() {
                return true;
            }
            let haystack = format!(
                "{} {} {} {}",
                record.name,
                record.category,
                record.tags.join(" "),
                record.fileName
            )
            .to_lowercase();
            haystack.contains(&normalized_search)
        })
        .collect::<Vec<_>>();
    images.sort_by(|left, right| right.updatedAt.cmp(&left.updatedAt));
    Ok(ImageLibraryPayload {
        ok: true,
        images: images.iter().map(serialize_image_record).collect(),
        categories: library.categories,
    })
}

pub fn save_data_url_image(
    data_url: &str,
    original_name: &str,
) -> Result<SerializedImageRecord, String> {
    let Some((content_type, encoded)) = data_url
        .strip_prefix("data:")
        .and_then(|value| value.split_once(";base64,"))
    else {
        return Err("Invalid image payload.".to_string());
    };
    let Some(extension) = supported_extension(content_type) else {
        return Err("Only png, jpg/jpeg, avif, webp, and gif images are supported.".to_string());
    };
    let bytes = BASE64
        .decode(encoded.as_bytes())
        .map_err(|_| "Invalid image payload.".to_string())?;
    save_image_bytes(&bytes, extension, original_name, None)
}

pub fn save_image_bytes(
    bytes: &[u8],
    extension: &str,
    original_name: &str,
    record_name: Option<&str>,
) -> Result<SerializedImageRecord, String> {
    ensure_local_dirs()?;
    let safe_base_name = sanitize_base_name(
        Path::new(original_name)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("image"),
        "image",
    );
    let file_name = format!(
        "{}-{}{}",
        chrono_like_timestamp(),
        safe_base_name,
        extension
    );
    let file_path = paths::uploads_dir().join(&file_name);
    fs::write(&file_path, bytes).map_err(|error| error.to_string())?;
    let mut library = read_image_library()?;
    let mut record = create_image_record(&file_name, original_name);
    if let Some(name) = record_name {
        if !name.trim().is_empty() {
            record.name = name.trim().to_string();
        }
    }
    library.images.insert(0, record.clone());
    write_image_library(&library)?;
    Ok(serialize_image_record(&record))
}

fn chrono_like_timestamp() -> String {
    let now = now_ms();
    now.to_string()
}

fn image_library_cache() -> &'static Mutex<Option<CachedImageLibrary>> {
    static CACHE: OnceLock<Mutex<Option<CachedImageLibrary>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

fn invalidate_image_library_cache() {
    if let Ok(mut guard) = image_library_cache().lock() {
        *guard = None;
    }
}

pub fn update_image(
    id: &str,
    name: Option<&str>,
    tags: Option<Vec<String>>,
    category: Option<&str>,
    is_favorite: Option<bool>,
) -> Result<(ImageLibraryPayload, SerializedImageRecord), String> {
    let mut library = read_image_library()?;
    let record = library
        .images
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| "Image not found.".to_string())?;
    if let Some(value) = name {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            record.name = trimmed.to_string();
        }
    }
    if let Some(values) = tags {
        record.tags = values
            .into_iter()
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .take(24)
            .collect();
    }
    if let Some(value) = category {
        record.category = value.trim().to_string();
        if !record.category.is_empty()
            && !library
                .categories
                .iter()
                .any(|entry| entry.eq_ignore_ascii_case(&record.category))
        {
            library.categories.push(record.category.clone());
            library.categories.sort_by_key(|entry| entry.to_lowercase());
        }
    }
    if let Some(value) = is_favorite {
        record.isFavorite = value;
    }
    record.updatedAt = now_ms();
    let serialized = serialize_image_record(record);
    write_image_library(&library)?;
    Ok((build_image_library_payload("", "", false)?, serialized))
}

pub fn create_category(name: &str) -> Result<(ImageLibraryPayload, String), String> {
    let normalized = name.trim().split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err("Category name is required.".to_string());
    }
    if normalized.chars().count() > 32 {
        return Err("Category name must be 32 characters or fewer.".to_string());
    }
    let mut library = read_image_library()?;
    let existing = library
        .categories
        .iter()
        .find(|entry| entry.eq_ignore_ascii_case(&normalized))
        .cloned();
    let category = existing.unwrap_or_else(|| normalized.clone());
    if !library
        .categories
        .iter()
        .any(|entry| entry.eq_ignore_ascii_case(&category))
    {
        library.categories.push(category.clone());
        library.categories.sort_by_key(|entry| entry.to_lowercase());
        write_image_library(&library)?;
    }
    Ok((build_image_library_payload("", "", false)?, category))
}

pub fn delete_image(id: &str) -> Result<ImageLibraryPayload, String> {
    let mut library = read_image_library()?;
    let index = library
        .images
        .iter()
        .position(|entry| entry.id == id)
        .ok_or_else(|| "Image not found.".to_string())?;
    let removed = library.images.remove(index);
    let file_path = paths::uploads_dir().join(&removed.fileName);
    if file_path.exists() {
        let _ = fs::remove_file(file_path);
    }
    write_image_library(&library)?;
    build_image_library_payload("", "", false)
}
