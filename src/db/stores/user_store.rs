use async_trait::async_trait;
use std::sync::Arc;

use crate::db::{DatabaseError, UserMapping, NewUserMapping};

#[async_trait]
pub trait UserStoreTrait: Send + Sync {
    async fn create(&self, user: NewUserMapping) -> Result<UserMapping, DatabaseError>;
    async fn get_by_matrix_id(&self, matrix_user_id: &str) -> Result<Option<UserMapping>, DatabaseError>;
    async fn get_by_phone_number(&self, phone_number: &str) -> Result<Option<UserMapping>, DatabaseError>;
    async fn update(&self, user: UserMapping) -> Result<UserMapping, DatabaseError>;
    async fn delete(&self, matrix_user_id: &str) -> Result<(), DatabaseError>;
    async fn list_all(&self) -> Result<Vec<UserMapping>, DatabaseError>;
}

pub type UserStore = Arc<dyn UserStoreTrait>;

#[cfg(feature = "sqlite")]
pub struct SqliteUserStore {
    db_path: String,
}

#[cfg(feature = "sqlite")]
impl SqliteUserStore {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl UserStoreTrait for SqliteUserStore {
    async fn create(&self, _user: NewUserMapping) -> Result<UserMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn get_by_matrix_id(&self, _matrix_user_id: &str) -> Result<Option<UserMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn get_by_phone_number(&self, _phone_number: &str) -> Result<Option<UserMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn update(&self, _user: UserMapping) -> Result<UserMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn delete(&self, _matrix_user_id: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
    
    async fn list_all(&self) -> Result<Vec<UserMapping>, DatabaseError> {
        Ok(vec![])
    }
}

#[cfg(feature = "postgres")]
pub struct PgUserStore {
    pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>,
}

#[cfg(feature = "postgres")]
impl PgUserStore {
    pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl UserStoreTrait for PgUserStore {
    async fn create(&self, _user: NewUserMapping) -> Result<UserMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn get_by_matrix_id(&self, _matrix_user_id: &str) -> Result<Option<UserMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn get_by_phone_number(&self, _phone_number: &str) -> Result<Option<UserMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn update(&self, _user: UserMapping) -> Result<UserMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn delete(&self, _matrix_user_id: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
    
    async fn list_all(&self) -> Result<Vec<UserMapping>, DatabaseError> {
        Ok(vec![])
    }
}

#[cfg(feature = "mysql")]
pub struct MySqlUserStore {
    pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>,
}

#[cfg(feature = "mysql")]
impl MySqlUserStore {
    pub fn new(pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "mysql")]
#[async_trait]
impl UserStoreTrait for MySqlUserStore {
    async fn create(&self, _user: NewUserMapping) -> Result<UserMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn get_by_matrix_id(&self, _matrix_user_id: &str) -> Result<Option<UserMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn get_by_phone_number(&self, _phone_number: &str) -> Result<Option<UserMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn update(&self, _user: UserMapping) -> Result<UserMapping, DatabaseError> {
        Err(DatabaseError::Query("Not implemented yet".to_string()))
    }
    
    async fn delete(&self, _matrix_user_id: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
    
    async fn list_all(&self) -> Result<Vec<UserMapping>, DatabaseError> {
        Ok(vec![])
    }
}
