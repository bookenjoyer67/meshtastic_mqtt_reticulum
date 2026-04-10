//! Enhanced file and image transfer capabilities
//!
//! This module provides improved file transfer functionality with support for:
//! - Image compression and resizing
//! - File chunking for large files
//! - Progress tracking
//! - Multiple file formats
//! - Metadata preservation

use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write, Cursor};
use anyhow::Result;
use log::info;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use image::{ImageFormat, imageops};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// File transfer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferConfig {
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Enable image compression
    pub enable_image_compression: bool,
    /// Maximum image dimensions (width, height)
    pub max_image_dimensions: (u32, u32),
    /// Image quality (0-100)
    pub image_quality: u8,
    /// Chunk size for large files
    pub chunk_size: usize,
    /// Enable file encryption
    pub enable_encryption: bool,
    /// Default download directory
    pub download_directory: String,
}

impl Default for FileTransferConfig {
    fn default() -> Self {
        FileTransferConfig {
            max_file_size: 10 * 1024 * 1024, // 10 MB
            enable_image_compression: true,
            max_image_dimensions: (1920, 1080), // Full HD
            image_quality: 85,
            chunk_size: 64 * 1024, // 64 KB chunks
            enable_encryption: true,
            download_directory: "downloads".to_string(),
        }
    }
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub filename: String,
    pub size: u64,
    pub file_type: FileType,
    pub hash: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub mime_type: String,
    pub dimensions: Option<(u32, u32)>, // For images
    pub duration: Option<f32>, // For audio/video
}

/// File type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Text,
    Image(ImageFormatType),
    Audio,
    Video,
    Document,
    Archive,
    Binary,
    Other(String),
}

/// Image format type that can be serialized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormatType {
    Jpeg,
    Png,
    Gif,
    Bmp,
    WebP,
    Tiff,
    Ico,
    Pnm,
    Tga,
    Dds,
    Farbfeld,
    Unknown,
}

impl From<ImageFormat> for ImageFormatType {
    fn from(format: ImageFormat) -> Self {
        match format {
            ImageFormat::Jpeg => ImageFormatType::Jpeg,
            ImageFormat::Png => ImageFormatType::Png,
            ImageFormat::Gif => ImageFormatType::Gif,
            ImageFormat::Bmp => ImageFormatType::Bmp,
            ImageFormat::WebP => ImageFormatType::WebP,
            ImageFormat::Tiff => ImageFormatType::Tiff,
            ImageFormat::Ico => ImageFormatType::Ico,
            ImageFormat::Pnm => ImageFormatType::Pnm,
            ImageFormat::Tga => ImageFormatType::Tga,
            ImageFormat::Dds => ImageFormatType::Dds,
            ImageFormat::Farbfeld => ImageFormatType::Farbfeld,
            _ => ImageFormatType::Unknown,
        }
    }
}

impl From<ImageFormatType> for ImageFormat {
    fn from(format: ImageFormatType) -> Self {
        match format {
            ImageFormatType::Jpeg => ImageFormat::Jpeg,
            ImageFormatType::Png => ImageFormat::Png,
            ImageFormatType::Gif => ImageFormat::Gif,
            ImageFormatType::Bmp => ImageFormat::Bmp,
            ImageFormatType::WebP => ImageFormat::WebP,
            ImageFormatType::Tiff => ImageFormat::Tiff,
            ImageFormatType::Ico => ImageFormat::Ico,
            ImageFormatType::Pnm => ImageFormat::Pnm,
            ImageFormatType::Tga => ImageFormat::Tga,
            ImageFormatType::Dds => ImageFormat::Dds,
            ImageFormatType::Farbfeld => ImageFormat::Farbfeld,
            ImageFormatType::Unknown => ImageFormat::Png, // Default to PNG
        }
    }
}

/// File transfer status
#[derive(Debug, Clone)]
pub enum TransferStatus {
    Pending,
    InProgress { bytes_transferred: u64, total_bytes: u64 },
    Completed,
    Failed { error: String },
    Cancelled,
}

/// File transfer request
#[derive(Debug, Clone)]
pub struct FileTransferRequest {
    pub source_path: PathBuf,
    pub destination_hash: String,
    pub metadata: FileMetadata,
    pub status: TransferStatus,
    pub transfer_id: String,
    pub start_time: DateTime<Utc>,
}

/// Enhanced file transfer manager
pub struct FileTransferManager {
    config: FileTransferConfig,
    active_transfers: Vec<FileTransferRequest>,
    download_dir: PathBuf,
}

impl FileTransferManager {
    /// Create a new file transfer manager
    pub fn new(config: FileTransferConfig) -> Result<Self> {
        // Create download directory
        let download_dir = PathBuf::from(&config.download_directory);
        fs::create_dir_all(&download_dir)?;
        
        Ok(Self {
            config,
            active_transfers: Vec::new(),
            download_dir,
        })
    }
    
    /// Prepare a file for transfer (compress images, chunk files, etc.)
    pub async fn prepare_file(&self, file_path: &Path) -> Result<Vec<Vec<u8>>> {
        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();
        
        // Check file size limit
        if file_size > self.config.max_file_size {
            return Err(anyhow::anyhow!(
                "File size {} exceeds maximum {} bytes",
                file_size,
                self.config.max_file_size
            ));
        }
        
        // Read file
        let mut file = fs::File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        // Process based on file type
        let file_type = self.detect_file_type(file_path, &buffer);
        
        match file_type {
            FileType::Image(format) => {
                // Process image
                let processed_image = self.process_image(&buffer, format).await?;
                buffer = processed_image;
            }
            _ => {
                // For other file types, just use as-is
            }
        }
        
        // Split into chunks if needed
        let chunks = self.split_into_chunks(&buffer);
        
        Ok(chunks)
    }
    
    /// Detect file type
    fn detect_file_type(&self, path: &Path, data: &[u8]) -> FileType {
        // Try to detect from extension first
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            
            match ext_str.as_str() {
                "txt" | "md" | "json" | "xml" | "yaml" | "yml" | "toml" => return FileType::Text,
                "jpg" | "jpeg" => return FileType::Image(ImageFormatType::Jpeg),
                "png" => return FileType::Image(ImageFormatType::Png),
                "gif" => return FileType::Image(ImageFormatType::Gif),
                "bmp" => return FileType::Image(ImageFormatType::Bmp),
                "webp" => return FileType::Image(ImageFormatType::WebP),
                "mp3" | "wav" | "ogg" | "flac" => return FileType::Audio,
                "mp4" | "avi" | "mkv" | "mov" => return FileType::Video,
                "pdf" | "doc" | "docx" | "xls" | "xlsx" => return FileType::Document,
                "zip" | "tar" | "gz" | "7z" => return FileType::Archive,
                _ => {}
            }
        }
        
        // Try to detect from magic bytes
        if data.len() >= 4 {
            match &data[0..4] {
                [0x89, 0x50, 0x4E, 0x47] => return FileType::Image(ImageFormatType::Png),
                [0xFF, 0xD8, 0xFF, _] => return FileType::Image(ImageFormatType::Jpeg),
                [0x47, 0x49, 0x46, 0x38] => return FileType::Image(ImageFormatType::Gif),
                [0x42, 0x4D, _, _] => return FileType::Image(ImageFormatType::Bmp),
                [0x52, 0x49, 0x46, 0x46] if data.len() >= 12 && &data[8..12] == b"WEBP" => 
                    return FileType::Image(ImageFormatType::WebP),
                _ => {}
            }
        }
        
        // Default to binary
        FileType::Binary
    }
    
    /// Process image (compress, resize, etc.)
    async fn process_image(&self, data: &[u8], format: ImageFormatType) -> Result<Vec<u8>> {
        if !self.config.enable_image_compression {
            return Ok(data.to_vec());
        }
        
        // Load image
        let img = image::load_from_memory(data)?;
        
        // Resize if needed
        let (max_width, max_height) = self.config.max_image_dimensions;
        let img = if img.width() > max_width || img.height() > max_height {
            img.resize(max_width, max_height, imageops::FilterType::Lanczos3)
        } else {
            img
        };
        
        // Convert ImageFormatType to ImageFormat
        let image_format: ImageFormat = format.into();
        
        // Encode with compression
        let mut output = Vec::new();
        match image_format {
            ImageFormat::Jpeg => {
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, self.config.image_quality);
                encoder.encode_image(&img)?;
            }
            ImageFormat::Png => {
                // PNG compression level 6 (default)
                let mut cursor = Cursor::new(&mut output);
                img.write_to(&mut cursor, ImageFormat::Png)?;
            }
            ImageFormat::WebP => {
                // WebP encoding
                let mut cursor = Cursor::new(&mut output);
                img.write_to(&mut cursor, ImageFormat::WebP)?;
            }
            _ => {
                // For other formats, just write as-is
                let mut cursor = Cursor::new(&mut output);
                img.write_to(&mut cursor, image_format)?;
            }
        }
        
        Ok(output)
    }
    
    /// Split data into chunks
    fn split_into_chunks(&self, data: &[u8]) -> Vec<Vec<u8>> {
        let chunk_size = self.config.chunk_size;
        let mut chunks = Vec::new();
        
        for chunk in data.chunks(chunk_size) {
            chunks.push(chunk.to_vec());
        }
        
        chunks
    }
    
    /// Create file metadata
    pub fn create_metadata(&self, file_path: &Path, data: &[u8]) -> Result<FileMetadata> {
        let metadata = fs::metadata(file_path)?;
        let filename = file_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let file_type = self.detect_file_type(file_path, data);
        
        // Get dimensions for images
        let dimensions = match file_type {
            FileType::Image(_) => {
                if let Ok(img) = image::load_from_memory(data) {
                    Some((img.width(), img.height()))
                } else {
                    None
                }
            }
            _ => None,
        };
        
        // Calculate hash
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = format!("{:x}", hasher.finalize());
        
        Ok(FileMetadata {
            filename,
            size: data.len() as u64,
            file_type: file_type.clone(),
            hash,
            created: metadata.created()?.into(),
            modified: metadata.modified()?.into(),
            mime_type: self.get_mime_type(file_path, &file_type),
            dimensions,
            duration: None, // Would need audio/video processing
        })
    }
    
    /// Get MIME type for file
    fn get_mime_type(&self, path: &Path, file_type: &FileType) -> String {
        match file_type {
            FileType::Text => "text/plain".to_string(),
            FileType::Image(format) => match format {
                ImageFormatType::Jpeg => "image/jpeg".to_string(),
                ImageFormatType::Png => "image/png".to_string(),
                ImageFormatType::Gif => "image/gif".to_string(),
                ImageFormatType::Bmp => "image/bmp".to_string(),
                ImageFormatType::WebP => "image/webp".to_string(),
                ImageFormatType::Tiff => "image/tiff".to_string(),
                ImageFormatType::Ico => "image/x-icon".to_string(),
                _ => "image/*".to_string(),
            },
            FileType::Audio => "audio/*".to_string(),
            FileType::Video => "video/*".to_string(),
            FileType::Document => {
                if let Some(ext) = path.extension() {
                    match ext.to_string_lossy().to_lowercase().as_str() {
                        "pdf" => "application/pdf".to_string(),
                        "doc" | "docx" => "application/msword".to_string(),
                        "xls" | "xlsx" => "application/vnd.ms-excel".to_string(),
                        _ => "application/octet-stream".to_string(),
                    }
                } else {
                    "application/octet-stream".to_string()
                }
            }
            FileType::Archive => "application/zip".to_string(),
            FileType::Binary => "application/octet-stream".to_string(),
            FileType::Other(_) => "application/octet-stream".to_string(),
        }
    }
    
    /// Save received file
    pub fn save_file(&self, filename: &str, data: &[u8]) -> Result<PathBuf> {
        let mut file_path = self.download_dir.join(filename);
        
        // Handle duplicate filenames
        let mut counter = 1;
        while file_path.exists() {
            let stem = file_path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| filename.to_string());
            let extension = file_path.extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_else(|| "".to_string());
            
            file_path = self.download_dir.join(format!("{}_{}{}", stem, counter, extension));
            counter += 1;
        }
        
        // Write file
        let mut file = fs::File::create(&file_path)?;
        file.write_all(data)?;
        
        info!("File saved: {}", file_path.display());
        Ok(file_path)
    }
    
    /// Create thumbnail for image
    pub fn create_thumbnail(&self, image_data: &[u8], max_size: (u32, u32)) -> Result<Vec<u8>> {
        let img = image::load_from_memory(image_data)?;
        let thumbnail = img.thumbnail(max_size.0, max_size.1);
        
        let mut output = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        thumbnail.write_to(&mut cursor, ImageFormat::Jpeg)?;
        
        Ok(output)
    }
    
    /// Convert image to base64 for embedding in messages
    pub fn image_to_base64(&self, image_data: &[u8], format: ImageFormatType) -> Result<String> {
        let mime_type = match format {
            ImageFormatType::Jpeg => "image/jpeg",
            ImageFormatType::Png => "image/png",
            ImageFormatType::Gif => "image/gif",
            ImageFormatType::Bmp => "image/bmp",
            ImageFormatType::WebP => "image/webp",
            ImageFormatType::Tiff => "image/tiff",
            ImageFormatType::Ico => "image/x-icon",
            _ => "image/*",
        };
        
        let base64_data = BASE64.encode(image_data);
        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }
}

/// File transfer protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileTransferMessage {
    Initiate {
        transfer_id: String,
        metadata: FileMetadata,
        total_chunks: usize,
    },
    Chunk {
        transfer_id: String,
        chunk_index: usize,
        total_chunks: usize,
        data: String, // Base64 encoded
        hash: String, // SHA256 of chunk
    },
    Ack {
        transfer_id: String,
        chunk_index: usize,
        received: bool,
    },
    Complete {
        transfer_id: String,
        success: bool,
        error: Option<String>,
    },
    Cancel {
        transfer_id: String,
        reason: String,
    },
}

/// File transfer protocol handler
pub struct FileTransferProtocol {
    config: FileTransferConfig,
    transfer_manager: FileTransferManager,
}

impl FileTransferProtocol {
    /// Create new protocol handler
    pub fn new(config: FileTransferConfig) -> Result<Self> {
        let transfer_manager = FileTransferManager::new(config.clone())?;
        
        Ok(Self {
            config,
            transfer_manager,
        })
    }
    
    /// Encode file transfer message
    pub fn encode_message(&self, message: &FileTransferMessage) -> Result<String> {
        serde_json::to_string(message).map_err(|e| e.into())
    }
    
    /// Decode file transfer message
    pub fn decode_message(&self, data: &str) -> Result<FileTransferMessage> {
        serde_json::from_str(data).map_err(|e| e.into())
    }
    
    /// Prepare file for sending
    pub async fn prepare_send(&self, file_path: &Path) -> Result<(FileMetadata, Vec<Vec<u8>>)> {
        let chunks = self.transfer_manager.prepare_file(file_path).await?;
        let metadata = self.transfer_manager.create_metadata(file_path, &chunks.concat())?;
        
        Ok((metadata, chunks))
    }
    
    /// Process received chunk
    pub fn process_chunk(&self, chunk_data: &str) -> Result<Vec<u8>> {
        // Decode base64
        let data = BASE64.decode(chunk_data)?;
        Ok(data)
    }
}