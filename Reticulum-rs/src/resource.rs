//! Resource system for Reticulum
//!
//! Resources allow sending arbitrary amounts of data over Reticulum networks
//! by splitting large data into smaller packets that can be reassembled
//! at the destination.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::RnsError;
use crate::hash::{AddressHash, Hash};
use crate::packet::{Packet, PacketContext};

/// Maximum size of a resource in bytes
pub const MAX_RESOURCE_SIZE: usize = 1024 * 1024 * 10; // 10 MB

/// Maximum size of a resource part in bytes
pub const MAX_RESOURCE_PART_SIZE: usize = 1024 * 8; // 8 KB

/// Resource status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceStatus {
    /// Resource is being advertised but not yet transferred
    Advertising,
    /// Resource transfer is in progress
    Transferring,
    /// Resource transfer is complete
    Complete,
    /// Resource transfer failed
    Failed,
    /// Resource transfer was cancelled
    Cancelled,
}

/// Resource metadata
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    /// Unique resource identifier
    pub resource_id: Hash,
    /// Total size of the resource in bytes
    pub total_size: usize,
    /// Number of parts the resource is divided into
    pub total_parts: usize,
    /// Hash of each part for verification
    pub part_hashes: Vec<Hash>,
    /// Original filename (if applicable)
    pub filename: Option<String>,
    /// MIME type (if known)
    pub mime_type: Option<String>,
}

/// Resource part
#[derive(Debug, Clone)]
pub struct ResourcePart {
    /// Resource ID this part belongs to
    pub resource_id: Hash,
    /// Part index (0-based)
    pub part_index: usize,
    /// Part data
    pub data: Vec<u8>,
    /// Hash of this part for verification
    pub hash: Hash,
}

/// Resource transfer state
#[derive(Debug, Clone)]
pub struct ResourceTransfer {
    /// Resource metadata
    pub metadata: ResourceMetadata,
    /// Current status
    pub status: ResourceStatus,
    /// Received parts
    pub received_parts: HashMap<usize, ResourcePart>,
    /// Missing parts
    pub missing_parts: Vec<usize>,
    /// Destination address
    pub destination: AddressHash,
    /// Source address
    pub source: Option<AddressHash>,
    /// Timestamp when transfer started
    pub started_at: std::time::Instant,
}

impl ResourceTransfer {
    /// Create a new resource transfer
    pub fn new(
        metadata: ResourceMetadata,
        destination: AddressHash,
        source: Option<AddressHash>,
    ) -> Self {
        let total_parts = metadata.total_parts;
        let missing_parts: Vec<usize> = (0..total_parts).collect();
        
        Self {
            metadata,
            status: ResourceStatus::Advertising,
            received_parts: HashMap::new(),
            missing_parts,
            destination,
            source,
            started_at: std::time::Instant::now(),
        }
    }
    
    /// Add a received part
    pub fn add_part(&mut self, part: ResourcePart) -> Result<(), RnsError> {
        // Verify part belongs to this resource
        if part.resource_id != self.metadata.resource_id {
            return Err(RnsError::invalid_argument("Part does not belong to this resource"));
        }
        
        // Verify part index is valid
        if part.part_index >= self.metadata.total_parts {
            return Err(RnsError::invalid_argument("Invalid part index"));
        }
        
        // Verify part hash matches expected hash
        let expected_hash = &self.metadata.part_hashes[part.part_index];
        if &part.hash != expected_hash {
            return Err(RnsError::invalid_argument("Part hash verification failed"));
        }
        
        // Add part to received parts
        self.received_parts.insert(part.part_index, part.clone());
        
        // Remove from missing parts
        if let Some(pos) = self.missing_parts.iter().position(|&x| x == part.part_index) {
            self.missing_parts.remove(pos);
        }
        
        // Update status if all parts are received
        if self.missing_parts.is_empty() {
            self.status = ResourceStatus::Complete;
        } else {
            self.status = ResourceStatus::Transferring;
        }
        
        Ok(())
    }
    
    /// Get completion percentage
    pub fn completion_percentage(&self) -> f32 {
        let received = self.received_parts.len() as f32;
        let total = self.metadata.total_parts as f32;
        (received / total) * 100.0
    }
    
    /// Check if transfer is complete
    pub fn is_complete(&self) -> bool {
        self.status == ResourceStatus::Complete
    }
    
    /// Reassemble the resource data
    pub fn reassemble(&self) -> Result<Vec<u8>, RnsError> {
        if !self.is_complete() {
            return Err(RnsError::invalid_argument("Resource transfer is not complete"));
        }
        
        let mut result = Vec::with_capacity(self.metadata.total_size);
        
        // Assemble parts in order
        for i in 0..self.metadata.total_parts {
            if let Some(part) = self.received_parts.get(&i) {
                result.extend_from_slice(&part.data);
            } else {
                return Err(RnsError::invalid_argument(&format!("Missing part {}", i)));
            }
        }
        
        Ok(result)
    }
}

/// Resource manager
pub struct ResourceManager {
    /// Active resource transfers
    transfers: Arc<RwLock<HashMap<Hash, ResourceTransfer>>>,
    /// Callback for completed resources
    completion_callback: Option<Arc<dyn Fn(ResourceTransfer) + Send + Sync>>,
}

impl std::fmt::Debug for ResourceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceManager")
            .field("transfers", &self.transfers)
            .field("completion_callback", &self.completion_callback.is_some())
            .finish()
    }
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            completion_callback: None,
        }
    }
    
    /// Set completion callback
    pub fn set_completion_callback<F>(&mut self, callback: F)
    where
        F: Fn(ResourceTransfer) + Send + Sync + 'static,
    {
        self.completion_callback = Some(Arc::new(callback));
    }
    
    /// Start a new resource transfer
    pub async fn start_transfer(
        &self,
        metadata: ResourceMetadata,
        destination: AddressHash,
        source: Option<AddressHash>,
    ) -> Result<Hash, RnsError> {
        let transfer = ResourceTransfer::new(metadata.clone(), destination, source);
        let resource_id = metadata.resource_id.clone();
        
        let mut transfers = self.transfers.write().await;
        transfers.insert(resource_id.clone(), transfer);
        
        Ok(resource_id)
    }
    
    /// Handle resource advertisement
    pub async fn handle_advertisement(
        &self,
        packet: &Packet,
    ) -> Result<(), RnsError> {
        // Verify packet context
        if packet.context != PacketContext::ResourceAdvertisement {
            return Err(RnsError::invalid_argument("Packet is not a resource advertisement"));
        }
        
        // Parse resource metadata from packet data
        // TODO: Implement actual parsing of resource metadata
        // For now, just log the advertisement
        log::debug!("Received resource advertisement for destination: {}", packet.destination);
        
        Ok(())
    }
    
    /// Handle resource part
    pub async fn handle_resource_part(
        &self,
        packet: &Packet,
    ) -> Result<(), RnsError> {
        // Verify packet context
        if packet.context != PacketContext::Resource {
            return Err(RnsError::invalid_argument("Packet is not a resource part"));
        }
        
        // Parse resource part from packet data
        // TODO: Implement actual parsing of resource parts
        // For now, just log the resource part
        log::debug!("Received resource part for destination: {}", packet.destination);
        
        Ok(())
    }
    
    /// Handle resource request
    pub async fn handle_resource_request(
        &self,
        packet: &Packet,
    ) -> Result<(), RnsError> {
        // Verify packet context
        if packet.context != PacketContext::ResourceRequest {
            return Err(RnsError::invalid_argument("Packet is not a resource request"));
        }
        
        // Parse resource request from packet data
        // TODO: Implement actual parsing of resource requests
        // For now, just log the request
        log::debug!("Received resource request for destination: {}", packet.destination);
        
        Ok(())
    }
    
    /// Get resource transfer by ID
    pub async fn get_transfer(&self, resource_id: &Hash) -> Option<ResourceTransfer> {
        let transfers = self.transfers.read().await;
        transfers.get(resource_id).cloned()
    }
    
    /// Cancel a resource transfer
    pub async fn cancel_transfer(&self, resource_id: &Hash) -> Result<(), RnsError> {
        let mut transfers = self.transfers.write().await;
        
        if let Some(transfer) = transfers.get_mut(resource_id) {
            transfer.status = ResourceStatus::Cancelled;
            Ok(())
        } else {
            Err(RnsError::invalid_argument("Resource transfer not found"))
        }
    }
    
    /// Clean up completed or failed transfers
    pub async fn cleanup(&self, max_age_seconds: u64) {
        let mut transfers = self.transfers.write().await;
        let now = std::time::Instant::now();
        
        transfers.retain(|_, transfer| {
            let age = now.duration_since(transfer.started_at).as_secs();
            
            // Keep transfers that are still active or recently completed
            match transfer.status {
                ResourceStatus::Advertising | ResourceStatus::Transferring => true,
                ResourceStatus::Complete | ResourceStatus::Failed | ResourceStatus::Cancelled => {
                    age < max_age_seconds
                }
            }
        });
    }
}

/// Utility functions for working with resources
pub mod utils {
    use super::*;
    use sha2::Digest;
    
    /// Split data into resource parts
    pub fn split_into_parts(data: &[u8], max_part_size: usize) -> Vec<ResourcePart> {
        let mut parts = Vec::new();
        let mut offset = 0;
        let mut part_index = 0;
        
        while offset < data.len() {
            let end = std::cmp::min(offset + max_part_size, data.len());
            let part_data = data[offset..end].to_vec();
            
            // Calculate hash of part data
            let mut hasher = Hash::generator();
            hasher.update(&part_data);
            let hash = Hash::new(hasher.finalize().into());
            
            // Create resource ID from all part hashes (simplified)
            // In a real implementation, the resource ID would be calculated differently
            let resource_id = hash.clone(); // Simplified
            
            let part = ResourcePart {
                resource_id,
                part_index,
                data: part_data,
                hash,
            };
            
            parts.push(part);
            offset = end;
            part_index += 1;
        }
        
        parts
    }
    
    /// Create resource metadata from data
    pub fn create_metadata(
        data: &[u8],
        filename: Option<String>,
        mime_type: Option<String>,
    ) -> ResourceMetadata {
        let parts = split_into_parts(data, MAX_RESOURCE_PART_SIZE);
        let part_hashes: Vec<Hash> = parts.iter().map(|p| p.hash.clone()).collect();
        
        // Calculate resource ID from all part hashes
        let mut hasher = Hash::generator();
        for hash in &part_hashes {
            hasher.update(hash.as_slice());
        }
        let resource_id = Hash::new(hasher.finalize().into());
        
        ResourceMetadata {
            resource_id,
            total_size: data.len(),
            total_parts: parts.len(),
            part_hashes,
            filename,
            mime_type,
        }
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}