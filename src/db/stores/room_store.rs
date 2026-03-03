use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

use crate::db::{DatabaseError, RoomMapping, NewRoomMapping};
use crate::db::stores::memory::{SharedMemoryDatabase, sqlite_database};

#[async_trait]
pub trait RoomStoreTrait: Send + Sync {
    async fn create(&self, room: NewRoomMapping) -> Result<RoomMapping, DatabaseError>;
    async fn get_by_matrix_id(&self, matrix_room_id: &str) -> Result<Option<RoomMapping>, DatabaseError>;
    async fn get_by_phone_number(&self, phone_number: &str) -> Result<Option<RoomMapping>, DatabaseError>;
    async fn update(&self, room: RoomMapping) -> Result<RoomMapping, DatabaseError>;
    async fn delete(&self, matrix_room_id: &str) -> Result<(), DatabaseError>;
    async fn list_all(&self) -> Result<Vec<RoomMapping>, DatabaseError>;
}

pub type RoomStore = Arc<dyn RoomStoreTrait>;

#[cfg(feature = "sqlite")]
pub struct SqliteRoomStore {
    _db_path: String,
    db: SharedMemoryDatabase,
}

#[cfg(feature = "sqlite")]
impl SqliteRoomStore {
    pub fn new(db_path: String) -> Self {
        let db = sqlite_database(&db_path);
        Self { _db_path: db_path, db }
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl RoomStoreTrait for SqliteRoomStore {
    async fn create(&self, room: NewRoomMapping) -> Result<RoomMapping, DatabaseError> {
        let mut db = self.db.write();

        if db.room_by_matrix.contains_key(&room.matrix_room_id) {
            return Err(DatabaseError::AlreadyExists(format!(
                "matrix room mapping already exists: {}",
                room.matrix_room_id
            )));
        }
        if db.room_by_phone.contains_key(&room.phone_number) {
            return Err(DatabaseError::AlreadyExists(format!(
                "phone mapping already exists: {}",
                room.phone_number
            )));
        }

        let now = Utc::now();
        let id = db.allocate_room_id();
        let mapping = RoomMapping {
            id,
            matrix_room_id: room.matrix_room_id,
            phone_number: room.phone_number,
            portal_name: room.portal_name,
            portal_avatar: room.portal_avatar,
            created_at: now,
            updated_at: now,
        };

        db.room_by_matrix
            .insert(mapping.matrix_room_id.clone(), mapping.id);
        db.room_by_phone
            .insert(mapping.phone_number.clone(), mapping.id);
        db.rooms.insert(mapping.id, mapping.clone());

        Ok(mapping)
    }
    
    async fn get_by_matrix_id(&self, matrix_room_id: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        let db = self.db.read();
        let Some(id) = db.room_by_matrix.get(matrix_room_id) else {
            return Ok(None);
        };
        Ok(db.rooms.get(id).cloned())
    }
    
    async fn get_by_phone_number(&self, phone_number: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        let db = self.db.read();
        let Some(id) = db.room_by_phone.get(phone_number) else {
            return Ok(None);
        };
        Ok(db.rooms.get(id).cloned())
    }
    
    async fn update(&self, mut room: RoomMapping) -> Result<RoomMapping, DatabaseError> {
        let mut db = self.db.write();
        let existing = db
            .rooms
            .get(&room.id)
            .cloned()
            .ok_or_else(|| DatabaseError::NotFound(format!("room id {} not found", room.id)))?;

        if existing.matrix_room_id != room.matrix_room_id
            && db.room_by_matrix.contains_key(&room.matrix_room_id)
        {
            return Err(DatabaseError::AlreadyExists(format!(
                "matrix room mapping already exists: {}",
                room.matrix_room_id
            )));
        }

        if existing.phone_number != room.phone_number && db.room_by_phone.contains_key(&room.phone_number)
        {
            return Err(DatabaseError::AlreadyExists(format!(
                "phone mapping already exists: {}",
                room.phone_number
            )));
        }

        db.room_by_matrix.remove(&existing.matrix_room_id);
        db.room_by_phone.remove(&existing.phone_number);

        room.updated_at = Utc::now();
        db.room_by_matrix.insert(room.matrix_room_id.clone(), room.id);
        db.room_by_phone.insert(room.phone_number.clone(), room.id);
        db.rooms.insert(room.id, room.clone());
        Ok(room)
    }
    
    async fn delete(&self, matrix_room_id: &str) -> Result<(), DatabaseError> {
        let mut db = self.db.write();
        let Some(id) = db.room_by_matrix.remove(matrix_room_id) else {
            return Ok(());
        };
        if let Some(removed) = db.rooms.remove(&id) {
            db.room_by_phone.remove(&removed.phone_number);
        }
        Ok(())
    }
    
    async fn list_all(&self) -> Result<Vec<RoomMapping>, DatabaseError> {
        let db = self.db.read();
        let mut rows = db.rooms.values().cloned().collect::<Vec<_>>();
        rows.sort_by_key(|row| row.id);
        Ok(rows)
    }
}

#[cfg(feature = "postgres")]
pub struct PgRoomStore {
    pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>,
}

#[cfg(feature = "postgres")]
impl PgRoomStore {
    pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl RoomStoreTrait for PgRoomStore {
    async fn create(&self, _room: NewRoomMapping) -> Result<RoomMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn get_by_matrix_id(&self, _matrix_room_id: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn get_by_phone_number(&self, _phone_number: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn update(&self, _room: RoomMapping) -> Result<RoomMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn delete(&self, _matrix_room_id: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
    
    async fn list_all(&self) -> Result<Vec<RoomMapping>, DatabaseError> {
        Ok(vec![])
    }
}

#[cfg(feature = "mysql")]
pub struct MySqlRoomStore {
    pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>,
}

#[cfg(feature = "mysql")]
impl MySqlRoomStore {
    pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "mysql")]
#[async_trait]
impl RoomStoreTrait for MySqlRoomStore {
    async fn create(&self, _room: NewRoomMapping) -> Result<RoomMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn get_by_matrix_id(&self, _matrix_room_id: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn get_by_phone_number(&self, _phone_number: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn update(&self, _room: RoomMapping) -> Result<RoomMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn delete(&self, _matrix_room_id: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
    
    async fn list_all(&self) -> Result<Vec<RoomMapping>, DatabaseError> {
        Ok(vec![])
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::{RoomStoreTrait, SqliteRoomStore};
    use crate::db::NewRoomMapping;

    #[tokio::test]
    async fn sqlite_room_store_crud() {
        let store = SqliteRoomStore::new(format!("test-room-store-{}", uuid::Uuid::new_v4()));
        let created = store
            .create(NewRoomMapping {
                matrix_room_id: "!room:example.com".to_string(),
                phone_number: "+1234567890".to_string(),
                portal_name: Some("Alice".to_string()),
                portal_avatar: None,
            })
            .await
            .expect("room mapping should be created");

        let by_room = store
            .get_by_matrix_id("!room:example.com")
            .await
            .expect("query by room id should work")
            .expect("room should exist");
        assert_eq!(by_room.id, created.id);

        let by_phone = store
            .get_by_phone_number("+1234567890")
            .await
            .expect("query by phone should work")
            .expect("room should exist");
        assert_eq!(by_phone.id, created.id);

        let mut updated = created.clone();
        updated.portal_name = Some("Alice Updated".to_string());
        let updated = store
            .update(updated)
            .await
            .expect("update should succeed");
        assert_eq!(updated.portal_name.as_deref(), Some("Alice Updated"));

        let all = store.list_all().await.expect("list should work");
        assert_eq!(all.len(), 1);

        store
            .delete("!room:example.com")
            .await
            .expect("delete should succeed");
        let after_delete = store
            .get_by_matrix_id("!room:example.com")
            .await
            .expect("query after delete should work");
        assert!(after_delete.is_none());
    }
}
