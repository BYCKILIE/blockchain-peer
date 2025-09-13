use deadpool_postgres::{Pool, PoolError};
use crate::dto::block_dto::BlockDTO;
use crate::dto::connection_dto::ConnectionDTO;

pub struct AlterRepo {
    pub db_pool: Pool,
}

impl AlterRepo {
    pub async fn insert_many(
        &mut self,
        blocks: &[BlockDTO],
    ) -> Result<Vec<String>, PoolError> {
        let mut client =  self.db_pool.get().await?;
        let tx = client.transaction().await?;

        let stmt = tx.prepare(
            "INSERT INTO blocks
         (hash, previous_hash, organization, payload, created_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING hash"
        ).await?;

        let mut inserted_hashes: Vec<String> = Vec::with_capacity(blocks.len());

        for block in blocks {
            let row = tx
                .query_one(
                    &stmt,
                    &[
                        &block.hash,
                        &block.previous_hash,
                        &block.organization,
                        &block.payload,
                        &block.created_at,
                    ],
                )
                .await?;
            let returned_hash: String = row.get(0);
            inserted_hashes.push(returned_hash);
        }
        
        tx.commit().await?;
        Ok(inserted_hashes)
    }
    
    pub async fn update_connections(&mut self, updates: &[ConnectionDTO]) -> Result<(), PoolError> {
        let mut client =  self.db_pool.get().await?;
        let tx = client.transaction().await?;

        let insert_stmt = tx.prepare(
            "INSERT INTO connections
         (url)
         VALUES ($1)"
        ).await?;

        let remove_stmt = tx.prepare(
            "DELETE FROM connections
         WHERE url = $1"
        ).await?;

        for update in updates {
            if update.is_active {
                tx.execute(
                    &insert_stmt,
                    &[
                        &update.url
                    ],
                ).await?;   
            } else {
                tx.execute(
                    &remove_stmt,
                    &[
                        &update.url
                    ],
                ).await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }
}
