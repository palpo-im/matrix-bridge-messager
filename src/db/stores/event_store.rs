use async_trait::async_trait;
use std::sync::Arc;
use crate::db::{DatabaseError, ProcessedEvent, NewProcessedEvent};

#[async_trait]
pub trait EventStoreTrait: Send + Sync {
    async fn create(&self, event: NewProcessedEvent) -> Result<ProcessedEvent, DatabaseError>;
    async fn exists(&self, event_id: &str) -> Result<bool, DatabaseError>;
    async fn cleanup_old_events(&self, days: i32) -> Result<u64, DatabaseError>;
}

pub type EventStore = Arc<dyn EventStoreTrait>;

#[cfg(feature = "sqlite")]
pub struct SqliteEventStore { db_path: String }

#[cfg(feature = "sqlite")]
impl SqliteEventStore { pub fn new(db_path: String) -> Self { Self { db_path } } }

#[cfg(feature = "sqlite")]
#[async_trait]
impl EventStoreTrait for SqliteEventStore {
    async fn create(&self, _: NewProcessedEvent) -> Result<ProcessedEvent, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn exists(&self, _: &str) -> Result<bool, DatabaseError> { Ok(false) }
    async fn cleanup_old_events(&self, _: i32) -> Result<u64, DatabaseError> { Ok(0) }
}

#[cfg(feature = "postgres")]
pub struct PgEventStore { pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>> }

#[cfg(feature = "postgres")]
impl PgEventStore { pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>) -> Self { Self { pool } } }

#[cfg(feature = "postgres")]
#[async_trait]
impl EventStoreTrait for PgEventStore {
    async fn create(&self, _: NewProcessedEvent) -> Result<ProcessedEvent, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn exists(&self, _: &str) -> Result<bool, DatabaseError> { Ok(false) }
    async fn cleanup_old_events(&self, _: i32) -> Result<u64, DatabaseError> { Ok(0) }
}

#[cfg(feature = "mysql")]
pub struct MySqlEventStore { pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>> }

#[cfg(feature = "mysql")]
impl MySqlEventStore { pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>) -> Self { Self { pool } } }

#[cfg(feature = "mysql")]
#[async_trait]
impl EventStoreTrait for MySqlEventStore {
    async fn create(&self, _: NewProcessedEvent) -> Result<ProcessedEvent, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn exists(&self, _: &str) -> Result<bool, DatabaseError> { Ok(false) }
    async fn cleanup_old_events(&self, _: i32) -> Result<u64, DatabaseError> { Ok(0) }
}
