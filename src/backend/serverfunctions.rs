#[cfg(feature = "ssr")]
use leptos::prelude::*;

use crate::backend::database::database::create_database;
#[cfg(feature = "ssr")]
use crate::backend::database::database::{self, QueryAmount, JamQueryResult, QueryTarget};

const DB_PATH: &str = "/home/joffy/repan_stream/";
//const DB_PATH: &str = "D:\\dev\\audio-stream\\";

#[server(GetAllJams)]
pub async fn get_all_jams() -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{
    let mut db = create_database(DB_PATH)?;

    let res = db.query(QueryTarget::Date, QueryAmount::All)?;
    Ok(res)
}
#[server(GetAllJamsFromMonth)]
pub async fn get_all_jams_month(yearmonth: String) -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{
    let mut db = create_database(DB_PATH)?;

    let res = db.query(QueryTarget::Date, QueryAmount::Month(yearmonth))?;
    Ok(res)
}
#[server(GetAllJamsFromMonthAsDays)]
pub async fn get_all_days_with_jams(yearmonth: String)
-> Result<Vec<JamQueryResult<u32>>, ServerFnError>
{
    let mut db = create_database(DB_PATH)?;

    let res = db.query(QueryTarget::Date, QueryAmount::MonthDays(yearmonth))?;
    Ok(res)
}

#[server(GetAllJamsFromDay)]
pub async fn get_all_jams_from_day(ymd: String) -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{

    let mut db = create_database(DB_PATH)?;

    let res = db.query(QueryTarget::Date, QueryAmount::Day(ymd))?;
    Ok(res)
}
#[server(GetJam)]
pub async fn get_jam() -> Result<JamQueryResult<String>, ServerFnError>
{
    let mut db = create_database(DB_PATH)?;

    let res = db.query(QueryTarget::Date, QueryAmount::All)?;
    Ok(res.first().expect("At least one jam to exist").clone())
}
#[server(GetTracks)]
pub async fn get_tracks(id: i64) -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{
    let mut db = create_database(DB_PATH)?;

    let res = db.query(QueryTarget::Track(id), QueryAmount::All)?;
    Ok(res)
}
#[server(GetTrackList)]
pub async fn get_track_list(jam_id: i64) -> Result<Vec<JamQueryResult<String>>, ServerFnError>
{
    let mut db = create_database(DB_PATH)?;

    let res = db.query(QueryTarget::Track(jam_id), QueryAmount::All)?;
    Ok(res)
}
//#[server(GetFullJam)]
//async fn get_full_jam() -> Result<Vec<JamQueryResult>, ServerFnError>
//{
//    let mut db = database::create_database("/home/joffy/audio-stream/")?;
//
//    let res = db.query(QueryTarget::Date, QueryAmount::All)?;
//
//    let tracks = db.query(QueryTarget::Track(), amount)
//    Ok(res)
//}

//pub fn add_jam(&mut self, jam: &Jam) -> Result<(), DatabaseError>
//{

//    let mut check_if_exists = self.conn.prepare("SELECT EXISTS(SELECT 1 FROM jams WHERE date = ?1)")?;

//    if check_if_exists.query_row([&jam.date], |row| row.get(0))?
//    {
//        println!("Attempted to add jam: {} that already exists", &jam.date);
//        return Err(DatabaseError::AlreadyExists);
//    }


//    self.conn.execute("INSERT INTO jams (date, path) VALUES (:date, :path)", 
//        &[(":date", &jam.date), (":path", &jam.path)])?;

//    let jam_id = self.conn.last_insert_rowid();

//    let mut statement = self.conn.prepare("INSERT INTO tracks (jam_id, track) VALUES (?1, ?2)")?;

//    for track in &jam.tracks
//    {
//        statement.execute(params![jam_id, track])?;
//    }
//    Ok(())
//}

//pub fn get_jam_from_date(&mut self, date: &str) -> Result<Jam, Error>
//{
//    let mut statement = self.conn.prepare("SELECT id, date, path FROM jams WHERE date = ?1").expect("Querying from date went wrong");

//    let jam_data = statement.query_one(params![date], |row|
//        { 
//            Ok((
//                row.get::<_, i32>(0)?,
//                row.get::<_, String>(1)?,
//                row.get::<_, String>(2)?,
//            ))
//        })?;

//    let mut trackstmt = self.conn.prepare("SELECT track FROM tracks WHERE jam_id = ?1")?;

//    let track_data = trackstmt.query_map(params![jam_data.0], |row| row.get(0))?;

//    let mut tracks = Vec::new();

//    for track in track_data
//    {
//        tracks.push(track?);
//    }
//    Ok(Jam { date: jam_data.1, path: jam_data.2, tracks})
//}
