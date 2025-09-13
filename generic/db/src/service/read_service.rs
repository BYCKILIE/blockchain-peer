use deadpool_postgres::PoolError;
use crate::dto::block_dto::BlockDTO;
use crate::dto::connection_dto::ConnectionDTO;
use crate::repo::read_repo::ReadRepo;

pub struct ReadService {
    pub repo: ReadRepo,
}

impl ReadService {
    
    pub async fn get_page(&self, offset: i64) -> Result<Vec<BlockDTO>, PoolError> {
        let rows = self.repo.get_page(offset).await?;
        Ok(rows.into_iter().map(BlockDTO::from_row).collect())
    }
    
    pub async fn get_by_organization(&self, organization: String, offset: i64) -> Result<Vec<BlockDTO>, PoolError> {
        let rows = self.repo.get_by_organization(organization, offset).await?;
        Ok(rows.into_iter().map(BlockDTO::from_row).collect())
    }

    pub async fn get_connections(&self) -> Result<Vec<ConnectionDTO>, PoolError> {
        let rows = self.repo.get_connections().await?;
        Ok(rows.into_iter().map(ConnectionDTO::from_row).collect())
    }
    
    pub async fn get_by_hash(&self, hash: String) -> Result<Option<BlockDTO>, PoolError> {
        let row = self.repo.get_by_hash(hash).await?;
        if let Some(block) = row {
            return Ok(Some(BlockDTO::from_row(block)))
        }
        Ok(None)
    }
    
}
