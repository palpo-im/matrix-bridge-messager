use async_trait::async_trait;
use std::sync::Arc;

use crate::db::{DatabaseError, RoomMapping, NewRoomMapping};

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
    db_path: String,
}

#[cfg(feature = "sqlite")]
impl SqliteRoomStore {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl RoomStoreTrait for SqliteRoomStore {
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
