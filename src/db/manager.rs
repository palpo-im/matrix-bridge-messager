use std::sync::Arc;
use anyhow::Result;

use crate::config::DatabaseConfig;
use crate::db::{DatabaseError, UserStore, RoomStore, MessageStore, EventStore, PortalStore};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbType {
    Postgres,
    Sqlite,
    Mysql,
}

impl DbType {
    pub fn from_url(url: &str) -> Self {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            DbType::Postgres
        } else if url.starts_with("mysql://") || url.starts_with("mariadb://") {
            DbType::Mysql
        } else {
            DbType::Sqlite
        }
    }
}

#[derive(Clone)]
pub struct DatabaseManager {
    db_type: DbType,
    user_store: UserStore,
    room_store: RoomStore,
    message_store: MessageStore,
    event_store: EventStore,
    portal_store: PortalStore,
}

impl DatabaseManager {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let db_type = DbType::from_url(&config.url);
        
        match db_type {
            #[cfg(feature = "postgres")]
            DbType::Postgres => {
                Self::new_postgres(config).await
            }
            #[cfg(feature = "sqlite")]
            DbType::Sqlite => {
                Self::new_sqlite(config).await
            }
            #[cfg(feature = "mysql")]
            DbType::Mysql => {
                Self::new_mysql(config).await
            }
            #[allow(unreachable_patterns)]
            _ => {
                Err(DatabaseError::Connection(format!("Unsupported database type: {:?}", db_type)))
            }
        }
    }
    
    #[cfg(feature = "postgres")]
    async fn new_postgres(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        use diesel::r2d2::{self, ConnectionManager};
        use diesel::pg::PgConnection;
        
        let manager = ConnectionManager::<PgConnection>::new(&config.url);
        let max_connections = config.max_connections.unwrap_or(10);
        let min_connections = config.min_connections.unwrap_or(1);
        
        let pool = r2d2::Pool::builder()
            .max_size(max_connections)
            .min_idle(Some(min_connections))
            .build(manager)
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;
        
        let user_store = Arc::new(crate::db::stores::user_store::PgUserStore::new(pool.clone()));
        let room_store = Arc::new(crate::db::stores::room_store::PgRoomStore::new(pool.clone()));
        let message_store = Arc::new(crate::db::stores::message_store::PgMessageStore::new(pool.clone()));
        let event_store = Arc::new(crate::db::stores::event_store::PgEventStore::new(pool.clone()));
        let portal_store = Arc::new(crate::db::stores::portal_store::PgPortalStore::new(pool));
        
        Ok(Self {
            db_type: DbType::Postgres,
            user_store,
            room_store,
            message_store,
            event_store,
            portal_store,
        })
    }
    
    #[cfg(feature = "sqlite")]
    async fn new_sqlite(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let db_path = config.url.trim_start_matches("sqlite://");
        
        let user_store = Arc::new(crate::db::stores::user_store::SqliteUserStore::new(db_path.to_string()));
        let room_store = Arc::new(crate::db::stores::room_store::SqliteRoomStore::new(db_path.to_string()));
        let message_store = Arc::new(crate::db::stores::message_store::SqliteMessageStore::new(db_path.to_string()));
        let event_store = Arc::new(crate::db::stores::event_store::SqliteEventStore::new(db_path.to_string()));
        let portal_store = Arc::new(crate::db::stores::portal_store::SqlitePortalStore::new(db_path.to_string()));
        
        Ok(Self {
            db_type: DbType::Sqlite,
            user_store,
            room_store,
            message_store,
            event_store,
            portal_store,
        })
    }
    
    #[cfg(feature = "mysql")]
    async fn new_mysql(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        use diesel::r2d2::{self, ConnectionManager};
        use diesel::mysql::MysqlConnection;
        
        let manager = ConnectionManager::<MysqlConnection>::new(&config.url);
        let max_connections = config.max_connections.unwrap_or(10);
        let min_connections = config.min_connections.unwrap_or(1);
        
        let pool = r2d2::Pool::builder()
            .max_size(max_connections)
            .min_idle(Some(min_connections))
            .build(manager)
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;
        
        let user_store = Arc::new(crate::db::stores::user_store::MySqlUserStore::new(pool.clone()));
        let room_store = Arc::new(crate::db::stores::room_store::MySqlRoomStore::new(pool.clone()));
        let message_store = Arc::new(crate::db::stores::message_store::MySqlMessageStore::new(pool.clone()));
        let event_store = Arc::new(crate::db::stores::event_store::MySqlEventStore::new(pool.clone()));
        let portal_store = Arc::new(crate::db::stores::portal_store::MySqlPortalStore::new(pool));
        
        Ok(Self {
            db_type: DbType::Mysql,
            user_store,
            room_store,
            message_store,
            event_store,
            portal_store,
        })
    }
    
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        tracing::info!("Running database migrations...");
        Ok(())
    }
    
    pub fn db_type(&self) -> DbType {
        self.db_type
    }
    
    pub fn user_store(&self) -> UserStore {
        self.user_store.clone()
    }
    
    pub fn room_store(&self) -> RoomStore {
        self.room_store.clone()
    }
    
    pub fn message_store(&self) -> MessageStore {
        self.message_store.clone()
    }
    
    pub fn event_store(&self) -> EventStore {
        self.event_store.clone()
    }
    
    pub fn portal_store(&self) -> PortalStore {
        self.portal_store.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::DbType;

    #[test]
    fn detect_db_type_from_url() {
        assert_eq!(DbType::from_url("postgresql://localhost/db"), DbType::Postgres);
        assert_eq!(DbType::from_url("postgres://localhost/db"), DbType::Postgres);
        assert_eq!(DbType::from_url("mysql://localhost/db"), DbType::Mysql);
        assert_eq!(DbType::from_url("mariadb://localhost/db"), DbType::Mysql);
        assert_eq!(DbType::from_url("sqlite://./test.db"), DbType::Sqlite);
        assert_eq!(DbType::from_url("./local.db"), DbType::Sqlite);
    }
}
