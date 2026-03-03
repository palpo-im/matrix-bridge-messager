use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::Arc;
use crate::db::{DatabaseError, ProcessedEvent, NewProcessedEvent};
use crate::db::stores::memory::{SharedMemoryDatabase, sqlite_database};

#[async_trait]
pub trait EventStoreTrait: Send + Sync {
    async fn create(&self, event: NewProcessedEvent) -> Result<ProcessedEvent, DatabaseError>;
    async fn exists(&self, event_id: &str) -> Result<bool, DatabaseError>;
    async fn cleanup_old_events(&self, days: i32) -> Result<u64, DatabaseError>;
}

pub type EventStore = Arc<dyn EventStoreTrait>;

#[cfg(feature = "sqlite")]
pub struct SqliteEventStore {
    _db_path: String,
    db: SharedMemoryDatabase,
}

#[cfg(feature = "sqlite")]
impl SqliteEventStore {
    pub fn new(db_path: String) -> Self {
        let db = sqlite_database(&db_path);
        Self { _db_path: db_path, db }
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl EventStoreTrait for SqliteEventStore {
    async fn create(&self, event: NewProcessedEvent) -> Result<ProcessedEvent, DatabaseError> {
        let mut db = self.db.write();
        if db.processed_event_ids.contains(&event.event_id) {
            return Err(DatabaseError::AlreadyExists(format!(
                "processed event already exists: {}",
                event.event_id
            )));
        }

        let id = db.allocate_event_id();
        let processed = ProcessedEvent {
            id,
            event_id: event.event_id,
            event_type: event.event_type,
            source: event.source,
            processed_at: Utc::now(),
        };
        db.processed_event_ids.insert(processed.event_id.clone());
        db.processed_events.insert(processed.id, processed.clone());
        Ok(processed)
    }

    async fn exists(&self, event_id: &str) -> Result<bool, DatabaseError> {
        let db = self.db.read();
        Ok(db.processed_event_ids.contains(event_id))
    }

    async fn cleanup_old_events(&self, days: i32) -> Result<u64, DatabaseError> {
        let mut db = self.db.write();
        let cutoff = Utc::now() - Duration::days(days.max(0) as i64);
        let old_ids = db
            .processed_events
            .iter()
            .filter(|(_, event)| event.processed_at < cutoff)
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();

        for id in &old_ids {
            if let Some(event) = db.processed_events.remove(id) {
                db.processed_event_ids.remove(&event.event_id);
            }
        }
        Ok(old_ids.len() as u64)
    }
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

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::{EventStoreTrait, SqliteEventStore};
    use crate::db::NewProcessedEvent;

    #[tokio::test]
    async fn sqlite_event_store_create_and_exists() {
        let store = SqliteEventStore::new(format!("test-event-store-{}", uuid::Uuid::new_v4()));
        assert!(
            !store
                .exists("$event1")
                .await
                .expect("exists query should work")
        );

        store
            .create(NewProcessedEvent {
                event_id: "$event1".to_string(),
                event_type: "m.room.message".to_string(),
                source: "matrix".to_string(),
            })
            .await
            .expect("event should be created");

        assert!(
            store
                .exists("$event1")
                .await
                .expect("exists query should work")
        );
    }
}
