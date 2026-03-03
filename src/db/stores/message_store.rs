use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use crate::db::{DatabaseError, MessageMapping, NewMessageMapping};
use crate::db::stores::memory::{SharedMemoryDatabase, sqlite_database};

#[async_trait]
pub trait MessageStoreTrait: Send + Sync {
    async fn create(&self, message: NewMessageMapping) -> Result<MessageMapping, DatabaseError>;
    async fn get_by_message_id(&self, message_id: &str) -> Result<Option<MessageMapping>, DatabaseError>;
    async fn get_by_matrix_event(&self, matrix_event_id: &str) -> Result<Option<MessageMapping>, DatabaseError>;
    async fn delete(&self, message_id: &str) -> Result<(), DatabaseError>;
}

pub type MessageStore = Arc<dyn MessageStoreTrait>;

#[cfg(feature = "sqlite")]
pub struct SqliteMessageStore {
    _db_path: String,
    db: SharedMemoryDatabase,
}

#[cfg(feature = "sqlite")]
impl SqliteMessageStore {
    pub fn new(db_path: String) -> Self {
        let db = sqlite_database(&db_path);
        Self { _db_path: db_path, db }
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl MessageStoreTrait for SqliteMessageStore {
    async fn create(&self, message: NewMessageMapping) -> Result<MessageMapping, DatabaseError> {
        let mut db = self.db.write();
        if db.message_by_id.contains_key(&message.message_id) {
            return Err(DatabaseError::AlreadyExists(format!(
                "message mapping already exists: {}",
                message.message_id
            )));
        }
        if db.message_by_matrix_event.contains_key(&message.matrix_event_id) {
            return Err(DatabaseError::AlreadyExists(format!(
                "matrix event mapping already exists: {}",
                message.matrix_event_id
            )));
        }

        let id = db.allocate_message_id();
        let mapping = MessageMapping {
            id,
            message_id: message.message_id,
            matrix_room_id: message.matrix_room_id,
            matrix_event_id: message.matrix_event_id,
            direction: message.direction,
            created_at: Utc::now(),
        };

        db.message_by_id
            .insert(mapping.message_id.clone(), mapping.id);
        db.message_by_matrix_event
            .insert(mapping.matrix_event_id.clone(), mapping.id);
        db.messages.insert(mapping.id, mapping.clone());
        Ok(mapping)
    }

    async fn get_by_message_id(&self, message_id: &str) -> Result<Option<MessageMapping>, DatabaseError> {
        let db = self.db.read();
        let Some(id) = db.message_by_id.get(message_id) else {
            return Ok(None);
        };
        Ok(db.messages.get(id).cloned())
    }

    async fn get_by_matrix_event(&self, matrix_event_id: &str) -> Result<Option<MessageMapping>, DatabaseError> {
        let db = self.db.read();
        let Some(id) = db.message_by_matrix_event.get(matrix_event_id) else {
            return Ok(None);
        };
        Ok(db.messages.get(id).cloned())
    }

    async fn delete(&self, message_id: &str) -> Result<(), DatabaseError> {
        let mut db = self.db.write();
        let Some(id) = db.message_by_id.remove(message_id) else {
            return Ok(());
        };
        if let Some(removed) = db.messages.remove(&id) {
            db.message_by_matrix_event.remove(&removed.matrix_event_id);
        }
        Ok(())
    }
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

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::{MessageStoreTrait, SqliteMessageStore};
    use crate::db::NewMessageMapping;

    #[tokio::test]
    async fn sqlite_message_store_crud() {
        let store =
            SqliteMessageStore::new(format!("test-message-store-{}", uuid::Uuid::new_v4()));
        let created = store
            .create(NewMessageMapping {
                message_id: "msg-1".to_string(),
                matrix_room_id: "!room:example.com".to_string(),
                matrix_event_id: "$event:example.com".to_string(),
                direction: "matrix_to_sms".to_string(),
            })
            .await
            .expect("message mapping should be created");
        assert_eq!(created.message_id, "msg-1");

        assert!(store
            .get_by_message_id("msg-1")
            .await
            .expect("query should work")
            .is_some());
        assert!(store
            .get_by_matrix_event("$event:example.com")
            .await
            .expect("query should work")
            .is_some());

        store.delete("msg-1").await.expect("delete should succeed");
        assert!(store
            .get_by_message_id("msg-1")
            .await
            .expect("query should work")
            .is_none());
    }
}
