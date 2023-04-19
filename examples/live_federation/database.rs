use crate::{objects::person::DbUser, Error};
use anyhow::anyhow;
use std::sync::{Arc, Mutex};

pub type DatabaseHandle = Arc<Database>;

/// Our "database" which contains all known users (local and federated)
pub struct Database {
    pub users: Mutex<Vec<DbUser>>,
}

impl Database {
    pub fn local_user(&self, name: &str) -> DbUser {
        let lock = self.users.lock().unwrap();
        for user in lock.iter() {
            if user.name == name {
                return user.clone();
            }
        }
        lock.first().unwrap().clone()
    }

    pub fn read_user(&self, name: &str) -> Result<DbUser, Error> {
        let db_user = self.local_user(name);
        if name == db_user.name {
            Ok(db_user)
        } else {
            Err(anyhow!("Invalid user {name}").into())
        }
    }
}
