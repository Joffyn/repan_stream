#[cfg(feature = "ssr")]
use std::{path::Path};


#[cfg(feature = "ssr")]
use rusqlite::types::FromSql;
#[cfg(feature = "ssr")]
use rusqlite::{ Connection, Error, Result, ToSql};
#[cfg(feature = "ssr")]
use std::fmt;

use leptos::logging::log;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JamQueryResult<T>
{
    pub id: i64,
    pub data: T, 
}

#[derive(Debug)]
pub enum QueryTarget
{
    Date,
    Path,
    Track(i64),
}

#[derive(Debug)]
pub enum QueryAmount
{
    All,
    One(String),
    Month(String),
    MonthDays(String),
    Day(String),
}

#[derive(Debug)]
#[cfg(feature = "ssr")]
pub enum DatabaseError
{
        AlreadyExists,
}

#[cfg(feature = "ssr")]
impl fmt::Display for DatabaseError
{
    #[cfg(feature = "ssr")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        match self 
        {
            DatabaseError::AlreadyExists => write!(f, "Item already exists"),
        }
    }
}

#[cfg(feature = "ssr")]
impl std::error::Error for DatabaseError{}

#[cfg(feature = "ssr")]
impl From<rusqlite::Error> for DatabaseError
{
    #[cfg(feature = "ssr")]
    fn from(_: rusqlite::Error) -> Self
    {
        todo!()
    }
}

#[cfg(feature = "ssr")]
pub struct Database
{
    conn: Connection,
}

#[cfg(feature = "ssr")]
impl Database
{

    #[cfg(feature = "ssr")]
    pub fn new(conn: Connection) -> Self
    {
        Database { conn }
    }

    #[cfg(feature = "ssr")]
    pub fn create_jam_table(&mut self) -> Result<()>
    {

        self.conn.execute_batch(" BEGIN;
                            CREATE TABLE IF NOT EXISTS jams
                          ( id INTEGER PRIMARY KEY AUTOINCREMENT,
                            date TEXT NOT NULL,
                            path TEXT NOT NULL);
                            CREATE TABLE IF NOT EXISTS tracks
                          ( id INTEGER PRIMARY KEY AUTOINCREMENT,
                            jam_id INTEGER NOT NULL,
                            track TEXT NOT NULL,
                            FOREIGN KEY(jam_id) REFERENCES jams(id));
                            COMMIT;",)
        
    }

    #[cfg(feature = "ssr")]
    pub fn query<T>(&mut self, target: QueryTarget, amount: QueryAmount) -> Result<Vec<JamQueryResult<T>>, rusqlite::Error>
    where T: FromSql
    {
        let (sql, params): (String, Vec<rusqlite::types::Value>) = match (target, amount)
        {
            (QueryTarget::Date, QueryAmount::All) =>
            (
                "SELECT date, id FROM jams".to_string(),
                vec![],
            ),
            (QueryTarget::Date, QueryAmount::One(path)) =>
            (
                "SELECT date, id FROM jams WHERE path = ?1".to_string(),
                vec![path.into()],
            ),
            (QueryTarget::Date, QueryAmount::Month(yearmonth)) =>
            (
                "SELECT date, id FROM jams WHERE SUBSTR(date, 1, 4) = ?1;".to_string(),
                vec![yearmonth.into()],
            ),
            (QueryTarget::Date, QueryAmount::MonthDays(yearmonth)) =>
            (
                "SELECT CAST(SUBSTR(date, 5, 2) AS INTEGER) AS day, id FROM jams WHERE SUBSTR(date, 1, 4) = ?1;".to_string(),
                vec![yearmonth.into()],
            ),
            (QueryTarget::Date, QueryAmount::Day(yearmonthday)) =>
            (
                "SELECT date, id FROM jams WHERE SUBSTR(date, 1, 6) = ?1;".to_string(),
                vec![yearmonthday.into()],
            ),
            (QueryTarget::Path, QueryAmount::All) =>
            (
                "SELECT path, id FROM jams".to_string(),
                vec![],
            ),
            (QueryTarget::Path, QueryAmount::One(date)) =>
            (
                "SELECT path, id FROM WHERE jams WHERE date = ?1".to_string(),
                vec![date.into()],
            ),
            (QueryTarget::Track(jam_id), QueryAmount::All) =>
            (
                "SELECT track, id FROM tracks WHERE jam_id = ?1".to_string(),
                vec![jam_id.into()],
            ),
            (QueryTarget::Track(jam_id), QueryAmount::One(stem)) =>
            (
                "SELECT track, id FROM tracks WHERE jam_id = ?1 AND track = ?2".to_string(),
                vec![jam_id.into(), stem.into()],
            ),
            _ => unimplemented!("Not a valid query"),
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let params_refs: Vec<&dyn ToSql> = params.iter().map(|v| v as &dyn ToSql).collect();
        let rows = stmt.query_map
            (&*params_refs, |row| 
                { 
                    Ok( JamQueryResult {
                        data: row.get(0)?,
                        id: row.get(1)?,
                    })
                })?;
        let results: Result<Vec<_>, _> = rows.collect();
        results
    }


}
#[cfg(feature = "ssr")]
pub fn create_database(path_str: &str) -> Result<Database, Error>
{

    let path = Path::new(path_str);

    if !path.exists() || !path.is_dir()
    {
        return  Err(rusqlite::Error::InvalidPath(path.to_path_buf()));
    }

    let full_path = path_str.to_string() + "database.db";

    let db_path = Path::new(&full_path);

    let exists = db_path.is_file();

    let conn = Connection::open(&full_path)?;

    let mut db = Database::new(conn);

    if exists
    {
        return Ok(db);
    }
    db.create_jam_table()?;
    Ok(db)
}


