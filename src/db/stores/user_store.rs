use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

use crate::db::{DatabaseError, UserMapping, NewUserMapping};
use crate::db::stores::memory::{SharedMemoryDatabase, sqlite_database};

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
    _db_path: String,
    db: SharedMemoryDatabase,
}

#[cfg(feature = "sqlite")]
impl SqliteUserStore {
    pub fn new(db_path: String) -> Self {
        let db = sqlite_database(&db_path);
        Self { _db_path: db_path, db }
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl UserStoreTrait for SqliteUserStore {
    async fn create(&self, user: NewUserMapping) -> Result<UserMapping, DatabaseError> {
        let mut db = self.db.write();

        if db.user_by_matrix.contains_key(&user.matrix_user_id) {
            return Err(DatabaseError::AlreadyExists(format!(
                "matrix user mapping already exists: {}",
                user.matrix_user_id
            )));
        }
        if db.user_by_phone.contains_key(&user.phone_number) {
            return Err(DatabaseError::AlreadyExists(format!(
                "phone mapping already exists: {}",
                user.phone_number
            )));
        }

        let now = Utc::now();
        let id = db.allocate_user_id();
        let mapping = UserMapping {
            id,
            matrix_user_id: user.matrix_user_id,
            phone_number: user.phone_number,
            contact_name: user.contact_name,
            contact_avatar: user.contact_avatar,
            created_at: now,
            updated_at: now,
        };

        db.user_by_matrix
            .insert(mapping.matrix_user_id.clone(), mapping.id);
        db.user_by_phone
            .insert(mapping.phone_number.clone(), mapping.id);
        db.users.insert(mapping.id, mapping.clone());

        Ok(mapping)
    }
    
    async fn get_by_matrix_id(&self, matrix_user_id: &str) -> Result<Option<UserMapping>, DatabaseError> {
        let db = self.db.read();
        let Some(id) = db.user_by_matrix.get(matrix_user_id) else {
            return Ok(None);
        };
        Ok(db.users.get(id).cloned())
    }
    
    async fn get_by_phone_number(&self, phone_number: &str) -> Result<Option<UserMapping>, DatabaseError> {
        let db = self.db.read();
        let Some(id) = db.user_by_phone.get(phone_number) else {
            return Ok(None);
        };
        Ok(db.users.get(id).cloned())
    }
    
    async fn update(&self, mut user: UserMapping) -> Result<UserMapping, DatabaseError> {
        let mut db = self.db.write();
        let existing = db
            .users
            .get(&user.id)
            .cloned()
            .ok_or_else(|| DatabaseError::NotFound(format!("user id {} not found", user.id)))?;

        if existing.matrix_user_id != user.matrix_user_id
            && db.user_by_matrix.contains_key(&user.matrix_user_id)
        {
            return Err(DatabaseError::AlreadyExists(format!(
                "matrix user mapping already exists: {}",
                user.matrix_user_id
            )));
        }
        if existing.phone_number != user.phone_number && db.user_by_phone.contains_key(&user.phone_number)
        {
            return Err(DatabaseError::AlreadyExists(format!(
                "phone mapping already exists: {}",
                user.phone_number
            )));
        }

        db.user_by_matrix.remove(&existing.matrix_user_id);
        db.user_by_phone.remove(&existing.phone_number);

        user.updated_at = Utc::now();
        db.user_by_matrix.insert(user.matrix_user_id.clone(), user.id);
        db.user_by_phone.insert(user.phone_number.clone(), user.id);
        db.users.insert(user.id, user.clone());

        Ok(user)
    }
    
    async fn delete(&self, matrix_user_id: &str) -> Result<(), DatabaseError> {
        let mut db = self.db.write();
        let Some(id) = db.user_by_matrix.remove(matrix_user_id) else {
            return Ok(());
        };
        if let Some(removed) = db.users.remove(&id) {
            db.user_by_phone.remove(&removed.phone_number);
        }
        Ok(())
    }
    
    async fn list_all(&self) -> Result<Vec<UserMapping>, DatabaseError> {
        let db = self.db.read();
        let mut rows = db.users.values().cloned().collect::<Vec<_>>();
        rows.sort_by_key(|row| row.id);
        Ok(rows)
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

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::{SqliteUserStore, UserStoreTrait};
    use crate::db::NewUserMapping;

    #[tokio::test]
    async fn sqlite_user_store_crud() {
        let store = SqliteUserStore::new(format!("test-user-store-{}", uuid::Uuid::new_v4()));
        let created = store
            .create(NewUserMapping {
                matrix_user_id: "@alice:example.com".to_string(),
                phone_number: "+19876543210".to_string(),
                contact_name: Some("Alice".to_string()),
                contact_avatar: None,
            })
            .await
            .expect("user mapping should be created");

        assert_eq!(
            store
                .get_by_matrix_id("@alice:example.com")
                .await
                .expect("query by matrix id should work")
                .expect("mapping should exist")
                .id,
            created.id
        );
        assert_eq!(
            store
                .get_by_phone_number("+19876543210")
                .await
                .expect("query by phone should work")
                .expect("mapping should exist")
                .id,
            created.id
        );

        let mut updated = created.clone();
        updated.contact_name = Some("Alice 2".to_string());
        let updated = store.update(updated).await.expect("update should succeed");
        assert_eq!(updated.contact_name.as_deref(), Some("Alice 2"));

        store
            .delete("@alice:example.com")
            .await
            .expect("delete should succeed");
        assert!(
            store
                .get_by_matrix_id("@alice:example.com")
                .await
                .expect("query after delete should work")
                .is_none()
        );
    }
}
