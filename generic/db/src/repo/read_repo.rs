use deadpool_postgres::{Pool, PoolError};
use tokio_postgres::Row;

pub struct ReadRepo {
    pub db_pool: Pool,
}

impl ReadRepo {

    pub async fn get_page(&self, offset: i64) -> Result<Vec<Row>, PoolError> {
        let client = self.db_pool.get().await?;
        let stmt = "SELECT hash, previous_hash, organization, payload, created_at FROM blocks ORDER BY created_at DESC OFFSET $1 LIMIT 20";
        let rows = client
            .query(stmt, &[&offset])
            .await?;
        Ok(rows)
    }
    
    pub async fn get_by_organization(&self, organization: String, offset: i64) -> Result<Vec<Row>, PoolError> {
        let client = self.db_pool.get().await?;
        let stmt = "SELECT hash, previous_hash, organization, payload, created_at FROM blocks WHERE organization = $1 ORDER BY created_at DESC OFFSET $2 LIMIT 20";
        let rows = client
            .query(stmt, &[&organization, &offset])
            .await?;
        Ok(rows)
    }

    pub async fn get_connections(&self) -> Result<Vec<Row>, PoolError> {
        let client = self.db_pool.get().await?;
        let stmt = "\
        SELECT url FROM connections";
        let rows = client
            .query(stmt, &[])
            .await?;
        Ok(rows)
    }

    pub async fn get_by_hash(&self, hash: String) -> Result<Option<Row>, PoolError> {
        let client = self.db_pool.get().await?;
        let stmt = "SELECT hash, previous_hash, organization, payload, created_at FROM blocks WHERE hash = $1";
        let row = client
            .query_opt(stmt, &[&hash])
            .await?;
        Ok(row)
    }
    
}
