// user.rs - provides 2 functions - create_user and check_pw

use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{params, Connection, Result};

pub fn create_user(username: &str, password: &str) -> Result<()> {

    // create the database

    let conn = Connection::open("users.db")?;
    conn.execute(
        "INSERT INTO users (username, password) VALUES (?1, ?2)",
        params![username, hash(password, DEFAULT_COST).expect("Failed to hash password")],
    )?;
    Ok(())
}

pub fn check_pw(username: &str, password: &str) -> Result<bool> {
    let conn = Connection::open("users.db")?;
    let hash = conn.query_row(
        "SELECT password FROM users WHERE username = ?1",
        params![username],
        |row| row.get::<usize, String>(0),
    );

    match hash {
        Ok(hash) => Ok(verify(password, &hash).expect("Failed to verify password")),
        Err(_) => Ok(false),
    }
}