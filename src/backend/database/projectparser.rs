use std::collections::HashMap;
use std::path::Path;
use std::fs::{self};
use regex::Regex;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Jam
{
    pub date: String,
    pub path: String,
    pub tracks: Vec<String>,
}

fn walk_directories(dir: &Path, jam_map: &mut HashMap<String, Jam>) 
{
    //Move regex initializion to separate function
    let regex = Regex::new(r"([0-9]{6}_[0-9]{4})").unwrap();
    let trackre = Regex::new(r"[0-9]{2}-.*-").unwrap();
    let pathre: Regex;

    if cfg!(unix) 
    { 
        pathre = Regex::new(r"/.*/").unwrap(); 
    } 
    else if cfg!(windows)
    { 
        pathre = Regex::new(r"[A-G]:.*\\").unwrap(); 
    }
    else 
    {
        println!("Platform not supported");
        return;   
    }

    for entry in fs::read_dir(dir).unwrap()
    {
        let path = entry.unwrap().path();
        if path.is_dir()
        {
            println!("Found directory: {}", path.to_str().unwrap());
            walk_directories(&path, jam_map);
        }
        else if path.is_file() && path.extension().unwrap() == "wav" && regex.is_match(path.to_str().unwrap())
        {

            let jam = regex.find(path.to_str().unwrap()).unwrap().as_str();
            if !trackre.is_match(path.to_str().unwrap())
            {
                continue;
            }
            let mut track = trackre.find(path.to_str().unwrap()).unwrap().as_str().to_string();
            track.pop();
            
            if jam_map.contains_key(jam)
            {

                let jamref = jam_map.get_mut(jam).unwrap();
                jamref.tracks.push(track);
            }
            else 
            {
                let dirpath = pathre.find(path.to_str().unwrap()).unwrap().as_str().to_string();
                let tracks: Vec<String> = vec![track];
                let jam_data = Jam {date: jam.to_string(), path: dirpath, tracks};
                jam_map.insert(jam.to_string(), jam_data);
            }
        }
    }
}

pub fn get_all_jams_from_dirs(directories: &Vec<&Path>) -> Vec<Jam>
{

    
    let mut jam_map: HashMap<String, Jam> = HashMap::new();
    for dir in directories
    {
        if !dir.is_dir()
        {
            println!("Path: {} was not a directory", dir.to_str().unwrap());
            continue;
        }
        walk_directories(dir, &mut jam_map);
    }
    jam_map.into_values().collect()
}
pub fn get_all_jams_from_dirs_json(directories: &Vec<&Path>) -> String
{
    
    let mut jam_map: HashMap<String, Jam> = HashMap::new();
    for dir in directories
    {
        if !dir.is_dir()
        {
            println!("Path: {} was not a directory", dir.to_str().unwrap());
            continue;
        }
        walk_directories(dir, &mut jam_map);
    }
    let result: Vec<Jam> = jam_map.into_values().collect();
    serde_json::to_string_pretty(&result).expect("Couldn't parse to JSON")

}
pub fn scan_and_save_jams(directories: &Vec<&Path>, save_path: &str) -> std::io::Result<()>
{
    let jams = get_all_jams_from_dirs_json(directories);
    
    let mut name = save_path.to_string();
    name.push_str("AllJams.json");


    fs::write(name, jams)?;
    Ok(())

}

