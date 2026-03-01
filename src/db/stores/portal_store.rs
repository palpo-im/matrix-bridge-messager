use async_trait::async_trait;
use std::sync::Arc;
use crate::db::{DatabaseError, PortalConfig, NewPortalConfig};

#[async_trait]
pub trait PortalStoreTrait: Send + Sync {
    async fn create(&self, config: NewPortalConfig) -> Result<PortalConfig, DatabaseError>;
    async fn get_by_matrix_room(&self, matrix_room_id: &str) -> Result<Option<PortalConfig>, DatabaseError>;
    async fn get_by_phone_number(&self, phone_number: &str) -> Result<Option<PortalConfig>, DatabaseError>;
    async fn update(&self, config: PortalConfig) -> Result<PortalConfig, DatabaseError>;
    async fn delete(&self, matrix_room_id: &str) -> Result<(), DatabaseError>;
    async fn list_all(&self) -> Result<Vec<PortalConfig>, DatabaseError>;
}

pub type PortalStore = Arc<dyn PortalStoreTrait>;

#[cfg(feature = "sqlite")]
pub struct SqlitePortalStore { db_path: String }

#[cfg(feature = "sqlite")]
impl SqlitePortalStore { pub fn new(db_path: String) -> Self { Self { db_path } } }

#[cfg(feature = "sqlite")]
#[async_trait]
impl PortalStoreTrait for SqlitePortalStore {
    async fn create(&self, _: NewPortalConfig) -> Result<PortalConfig, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn get_by_matrix_room(&self, _: &str) -> Result<Option<PortalConfig>, DatabaseError> { Ok(None) }
    async fn get_by_phone_number(&self, _: &str) -> Result<Option<PortalConfig>, DatabaseError> { Ok(None) }
    async fn update(&self, _: PortalConfig) -> Result<PortalConfig, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn delete(&self, _: &str) -> Result<(), DatabaseError> { Ok(()) }
    async fn list_all(&self) -> Result<Vec<PortalConfig>, DatabaseError> { Ok(vec![]) }
}

#[cfg(feature = "postgres")]
pub struct PgPortalStore { pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>> }

#[cfg(feature = "postgres")]
impl PgPortalStore { pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>) -> Self { Self { pool } } }

#[cfg(feature = "postgres")]
#[async_trait]
impl PortalStoreTrait for PgPortalStore {
    async fn create(&self, _: NewPortalConfig) -> Result<PortalConfig, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn get_by_matrix_room(&self, _: &str) -> Result<Option<PortalConfig>, DatabaseError> { Ok(None) }
    async fn get_by_phone_number(&self, _: &str) -> Result<Option<PortalConfig>, DatabaseError> { Ok(None) }
    async fn update(&self, _: PortalConfig) -> Result<PortalConfig, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn delete(&self, _: &str) -> Result<(), DatabaseError> { Ok(()) }
    async fn list_all(&self) -> Result<Vec<PortalConfig>, DatabaseError> { Ok(vec![]) }
}

#[cfg(feature = "mysql")]
pub struct MySqlPortalStore { pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>> }

#[cfg(feature = "mysql")]
impl MySqlPortalStore { pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>) -> Self { Self { pool } } }

#[cfg(feature = "mysql")]
#[async_trait]
impl PortalStoreTrait for MySqlPortalStore {
    async fn create(&self, _: NewPortalConfig) -> Result<PortalConfig, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn get_by_matrix_room(&self, _: &str) -> Result<Option<PortalConfig>, DatabaseError> { Ok(None) }
    async fn get_by_phone_number(&self, _: &str) -> Result<Option<PortalConfig>, DatabaseError> { Ok(None) }
    async fn update(&self, _: PortalConfig) -> Result<PortalConfig, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn delete(&self, _: &str) -> Result<(), DatabaseError> { Ok(()) }
    async fn list_all(&self) -> Result<Vec<PortalConfig>, DatabaseError> { Ok(vec![]) }
}
