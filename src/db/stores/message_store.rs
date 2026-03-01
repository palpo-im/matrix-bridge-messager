use async_trait::async_trait;
use std::sync::Arc;
use crate::db::{DatabaseError, MessageMapping, NewMessageMapping};

#[async_trait]
pub trait MessageStoreTrait: Send + Sync {
    async fn create(&self, message: NewMessageMapping) -> Result<MessageMapping, DatabaseError>;
    async fn get_by_message_id(&self, message_id: &str) -> Result<Option<MessageMapping>, DatabaseError>;
    async fn get_by_matrix_event(&self, matrix_event_id: &str) -> Result<Option<MessageMapping>, DatabaseError>;
    async fn delete(&self, message_id: &str) -> Result<(), DatabaseError>;
}

pub type MessageStore = Arc<dyn MessageStoreTrait>;

#[cfg(feature = "sqlite")]
pub struct SqliteMessageStore { db_path: String }

#[cfg(feature = "sqlite")]
impl SqliteMessageStore { pub fn new(db_path: String) -> Self { Self { db_path } } }

#[cfg(feature = "sqlite")]
#[async_trait]
impl MessageStoreTrait for SqliteMessageStore {
    async fn create(&self, _: NewMessageMapping) -> Result<MessageMapping, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn get_by_message_id(&self, _: &str) -> Result<Option<MessageMapping>, DatabaseError> { Ok(None) }
    async fn get_by_matrix_event(&self, _: &str) -> Result<Option<MessageMapping>, DatabaseError> { Ok(None) }
    async fn delete(&self, _: &str) -> Result<(), DatabaseError> { Ok(()) }
}

#[cfg(feature = "postgres")]
pub struct PgMessageStore { pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>> }

#[cfg(feature = "postgres")]
impl PgMessageStore { pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>) -> Self { Self { pool } } }

#[cfg(feature = "postgres")]
#[async_trait]
impl MessageStoreTrait for PgMessageStore {
    async fn create(&self, _: NewMessageMapping) -> Result<MessageMapping, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn get_by_message_id(&self, _: &str) -> Result<Option<MessageMapping>, DatabaseError> { Ok(None) }
    async fn get_by_matrix_event(&self, _: &str) -> Result<Option<MessageMapping>, DatabaseError> { Ok(None) }
    async fn delete(&self, _: &str) -> Result<(), DatabaseError> { Ok(()) }
}

#[cfg(feature = "mysql")]
pub struct MySqlMessageStore { pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>> }

#[cfg(feature = "mysql")]
impl MySqlMessageStore { pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>) -> Self { Self { pool } } }

#[cfg(feature = "mysql")]
#[async_trait]
impl MessageStoreTrait for MySqlMessageStore {
    async fn create(&self, _: NewMessageMapping) -> Result<MessageMapping, DatabaseError> { Err(DatabaseError::Query("Not implemented".into())) }
    async fn get_by_message_id(&self, _: &str) -> Result<Option<MessageMapping>, DatabaseError> { Ok(None) }
    async fn get_by_matrix_event(&self, _: &str) -> Result<Option<MessageMapping>, DatabaseError> { Ok(None) }
    async fn delete(&self, _: &str) -> Result<(), DatabaseError> { Ok(()) }
}
