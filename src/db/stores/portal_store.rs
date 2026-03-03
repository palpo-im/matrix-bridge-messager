use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use crate::db::{DatabaseError, PortalConfig, NewPortalConfig};
use crate::db::stores::memory::{SharedMemoryDatabase, sqlite_database};

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
pub struct SqlitePortalStore {
    _db_path: String,
    db: SharedMemoryDatabase,
}

#[cfg(feature = "sqlite")]
impl SqlitePortalStore {
    pub fn new(db_path: String) -> Self {
        let db = sqlite_database(&db_path);
        Self { _db_path: db_path, db }
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl PortalStoreTrait for SqlitePortalStore {
    async fn create(&self, config: NewPortalConfig) -> Result<PortalConfig, DatabaseError> {
        let mut db = self.db.write();
        if db.portal_by_matrix_room.contains_key(&config.matrix_room_id) {
            return Err(DatabaseError::AlreadyExists(format!(
                "portal for matrix room already exists: {}",
                config.matrix_room_id
            )));
        }
        if db.portal_by_phone.contains_key(&config.phone_number) {
            return Err(DatabaseError::AlreadyExists(format!(
                "portal for phone already exists: {}",
                config.phone_number
            )));
        }

        let now = Utc::now();
        let id = db.allocate_portal_id();
        let portal = PortalConfig {
            id,
            matrix_room_id: config.matrix_room_id,
            phone_number: config.phone_number,
            auto_bridge: config.auto_bridge,
            bridge_read_receipts: config.bridge_read_receipts,
            bridge_typing: config.bridge_typing,
            created_at: now,
            updated_at: now,
        };
        db.portal_by_matrix_room
            .insert(portal.matrix_room_id.clone(), portal.id);
        db.portal_by_phone
            .insert(portal.phone_number.clone(), portal.id);
        db.portals.insert(portal.id, portal.clone());
        Ok(portal)
    }

    async fn get_by_matrix_room(&self, matrix_room_id: &str) -> Result<Option<PortalConfig>, DatabaseError> {
        let db = self.db.read();
        let Some(id) = db.portal_by_matrix_room.get(matrix_room_id) else {
            return Ok(None);
        };
        Ok(db.portals.get(id).cloned())
    }

    async fn get_by_phone_number(&self, phone_number: &str) -> Result<Option<PortalConfig>, DatabaseError> {
        let db = self.db.read();
        let Some(id) = db.portal_by_phone.get(phone_number) else {
            return Ok(None);
        };
        Ok(db.portals.get(id).cloned())
    }

    async fn update(&self, mut config: PortalConfig) -> Result<PortalConfig, DatabaseError> {
        let mut db = self.db.write();
        let existing = db
            .portals
            .get(&config.id)
            .cloned()
            .ok_or_else(|| DatabaseError::NotFound(format!("portal id {} not found", config.id)))?;

        if existing.matrix_room_id != config.matrix_room_id
            && db.portal_by_matrix_room.contains_key(&config.matrix_room_id)
        {
            return Err(DatabaseError::AlreadyExists(format!(
                "portal for matrix room already exists: {}",
                config.matrix_room_id
            )));
        }
        if existing.phone_number != config.phone_number
            && db.portal_by_phone.contains_key(&config.phone_number)
        {
            return Err(DatabaseError::AlreadyExists(format!(
                "portal for phone already exists: {}",
                config.phone_number
            )));
        }

        db.portal_by_matrix_room.remove(&existing.matrix_room_id);
        db.portal_by_phone.remove(&existing.phone_number);

        config.updated_at = Utc::now();
        db.portal_by_matrix_room
            .insert(config.matrix_room_id.clone(), config.id);
        db.portal_by_phone
            .insert(config.phone_number.clone(), config.id);
        db.portals.insert(config.id, config.clone());

        Ok(config)
    }

    async fn delete(&self, matrix_room_id: &str) -> Result<(), DatabaseError> {
        let mut db = self.db.write();
        let Some(id) = db.portal_by_matrix_room.remove(matrix_room_id) else {
            return Ok(());
        };
        if let Some(removed) = db.portals.remove(&id) {
            db.portal_by_phone.remove(&removed.phone_number);
        }
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<PortalConfig>, DatabaseError> {
        let db = self.db.read();
        let mut rows = db.portals.values().cloned().collect::<Vec<_>>();
        rows.sort_by_key(|row| row.id);
        Ok(rows)
    }
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

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::{PortalStoreTrait, SqlitePortalStore};
    use crate::db::NewPortalConfig;

    #[tokio::test]
    async fn sqlite_portal_store_crud() {
        let store = SqlitePortalStore::new(format!("test-portal-store-{}", uuid::Uuid::new_v4()));
        let created = store
            .create(NewPortalConfig {
                matrix_room_id: "!portal:example.com".to_string(),
                phone_number: "+11111111111".to_string(),
                auto_bridge: true,
                bridge_read_receipts: true,
                bridge_typing: false,
            })
            .await
            .expect("portal should be created");

        assert_eq!(
            store
                .get_by_matrix_room("!portal:example.com")
                .await
                .expect("query should work")
                .expect("portal should exist")
                .id,
            created.id
        );

        let mut updated = created.clone();
        updated.bridge_typing = true;
        let updated = store.update(updated).await.expect("update should succeed");
        assert!(updated.bridge_typing);

        let list = store.list_all().await.expect("list should work");
        assert_eq!(list.len(), 1);

        store
            .delete("!portal:example.com")
            .await
            .expect("delete should work");
        assert!(
            store
                .get_by_matrix_room("!portal:example.com")
                .await
                .expect("query after delete should work")
                .is_none()
        );
    }
}
