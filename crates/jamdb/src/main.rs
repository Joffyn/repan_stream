use std::{collections::HashMap, env, path::Path};

use jamdb::{Jam, add_jam, create_jam_table, walk_directories};
use rusqlite::Connection;


fn main() 
{
    let args: Vec<String> = env::args().collect();
    //dbg!(args);

    //let path = Path::new("test.txt");
    //let not_path = Path::new("not.txt");

    //let res = path.exists();
    //let not_res = not_path.exists();

    //println!("test.txt exists is: {}",res); 
    //println!("not.txt exists is: {}", not_res); 

    if args.len() < 2 
    {
        println!("Not enough arguments");
        return;
    }
    else if args.len() > 2
    {
        println!("Too many arguments");
        return;
    }
    let jam_dir = Path::new(args[1].as_str());
    if !jam_dir.exists() || !jam_dir.is_dir()
    {

        println!("Path given is not valid or is not a directory");
        return;
    }
    let mut jam_map: HashMap<String, Jam> = HashMap::new();
    walk_directories(jam_dir, &mut jam_map);

    let db_path = Path::new("jams.db");
    if db_path.exists()
    {
        println!("Database already exists!");
        return;
    }
    let mut conn = Connection::open(db_path).unwrap();

    create_jam_table(&mut conn);
    for jam in jam_map
    {
        match add_jam(&mut conn, &jam.1)
        {
            Ok(()) => println!("Added {}", jam.0.as_str()),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
}
