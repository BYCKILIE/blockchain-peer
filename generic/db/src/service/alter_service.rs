use crate::{dto::block_dto::BlockDTO, repo::alter_repo::AlterRepo};
use base64::{engine::general_purpose, Engine as _};
use blake3;
use chrono::{DateTime, Utc};
use common::memory::last_hash::LastHash;
use crate::dto::connection_dto::ConnectionDTO;

pub struct AlterService {
    pub(crate) repo: AlterRepo,
    pub(crate) last_hash: LastHash,
}

impl AlterService {
    pub async fn create_blocks(
        &mut self,
        raw_blocks: Vec<((String, String, String), DateTime<Utc>)>,
    ) -> Vec<(String, String)> {
        let last_hash = self.last_hash.get();
        
        let mut blocks_dto: Vec<BlockDTO> = Vec::new();
        let mut prev_hash = general_purpose::URL_SAFE_NO_PAD.decode(last_hash).unwrap();

        let mut inserted_hashes: Vec<(String, String)> = Vec::with_capacity(raw_blocks.len());

        for ((simple_hash_b64, organization, payload), created_at) in raw_blocks {
            let prev_hash_b64 = general_purpose::URL_SAFE_NO_PAD.encode(&prev_hash);
            let simple_hash = general_purpose::STANDARD.decode(&*simple_hash_b64).unwrap();

            let mut hasher = blake3::Hasher::new();
            hasher.update(&prev_hash);
            hasher.update(&simple_hash);
            let mut out = [0u8; 64];
            hasher.finalize_xof().fill(&mut out);

            let hash = general_purpose::URL_SAFE_NO_PAD.encode(out);

            blocks_dto.push(BlockDTO {
                hash: hash.clone(),
                previous_hash: prev_hash_b64,
                organization: organization.clone(),
                payload: payload.clone(),
                created_at: created_at.clone(),
            });
            
            inserted_hashes.push((simple_hash_b64, hash));

            prev_hash = out.to_vec();
        }
        
        let _ = self.repo.insert_many(&blocks_dto).await;
        
        let new_last_hash = general_purpose::URL_SAFE_NO_PAD.encode(prev_hash);

        self.last_hash.set(new_last_hash);
        
        inserted_hashes
    }
    
    pub async fn update_connections(
        &mut self,
        raw_connections: Vec<(String, bool)>,
    ) {
        let connections: Vec<ConnectionDTO> = raw_connections.into_iter().map(|(url, is_active)| ConnectionDTO {
            url,
            is_active,
        }).collect();
        let _ = self.repo.update_connections(&connections).await;
    }
}
