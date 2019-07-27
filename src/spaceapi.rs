use reqwest::StatusCode;
use serde::*;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::time::{Duration, SystemTime};
use std::io::BufReader;

fn get_spaces_web() -> Result<HashMap<String, HashMap<String, f64>>, failure::Error> {
    let client = reqwest::Client::builder()
        .gzip(true)
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut res = client.get("https://directory.spaceapi.io/").send()?;

    let json: Value = res.json()?;

    fs::create_dir_all("./tmp/")?;
    let serialized = serde_json::to_string(&json).unwrap();
    fs::write("./tmp/spaces.json", serialized).expect("Unable to write file");

    let mut spaces = HashMap::new();
    for (key, value) in json.as_object().unwrap().iter() {
        let space: Result<HashMap<String, f64>, failure::Error> = get_space(
            key.as_str().to_string(),
            value.as_str().unwrap().to_string(),
        );
        if space.is_ok() {
            spaces.insert(key.to_string(), space.unwrap());
        }
    }

    //println!("{:?}", json);
    Ok(spaces)
}

pub fn get_spaces() -> Result<HashMap<String, HashMap<String, f64>>, failure::Error> {
    if Path::new("./tmp/spaces.json").exists() {
        let cache_data = fs::metadata("./tmp/spaces.json")?;

        if let Ok(time) = cache_data.modified() {
            let now = SystemTime::now();
            if now.duration_since(time).unwrap() >= Duration::from_secs(86400) {
                return get_spaces_web();
            } else {
                let cache_file = File::open(Path::new("./tmp/spaces.json"))?;
                let reader = BufReader::new(cache_file);
                let json: Value = serde_json::from_reader(reader)?;

                let mut spaces = HashMap::new();
                for (key, value) in json.as_object().unwrap().iter() {
                    info!("{}: {}", key, value);
                    info!("GetSpace");
                    let space: Result<HashMap<String, f64>, failure::Error> = get_space(
                        key.as_str().to_string(),
                        value.as_str().unwrap().to_string(),
                    );
                    if space.is_ok() {
                        spaces.insert(key.to_string(), space.unwrap());
                    }
                }

                //println!("{:?}", json);
                return Ok(spaces);
            }
        } else {
            return get_spaces_web();
        }
    } else {
        return get_spaces_web();
    }
}

#[derive(Serialize, Deserialize)]
pub struct Location {
    address: String,
    lon: f64,
    lat: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Space {
    api: String,
    space: String,
    logo: String,
    url: String,
    location: Location,
    contact: Value,
    issue_report_channels: Vec<String>,
    state: Value,
    open: Option<bool>,
}

fn get_space_web(name: String, url: String) -> Result<HashMap<String, f64>, failure::Error> {
    let client = reqwest::Client::builder()
        .gzip(true)
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut res = client.get(url.as_str()).send()?;

    if res.status() == StatusCode::OK {
        let json: Space = res.json()?;
        fs::create_dir_all("./tmp/spaces/")?;
        let serialized = serde_json::to_string(&json).unwrap();
        let file_name_path = format!("./tmp/spaces/space_{}.json", name.replace("/", "_"));
        fs::write(&file_name_path, serialized)
            .expect(format!("Unable to write file: {:?}", file_name_path).as_str());

        let mut map = HashMap::new();

        map.insert("lat".to_string(), json.location.lat);
        map.insert("lon".to_string(), json.location.lon);
        //println!("{:?}", json);
        return Ok(map);
    }
    Err(failure::err_msg("SPACE API DOWN"))
}

pub fn get_space(name: String, url: String) -> Result<HashMap<String, f64>, failure::Error> {
    let file_name_path = format!("./tmp/spaces/space_{}.json", name.replace("/", "_"));
    let space_file = Path::new(&file_name_path);
    if space_file.exists() {
        info!("1");
        let cache_data = fs::metadata(&space_file)?;

        if let Ok(time) = cache_data.modified() {
            let now = SystemTime::now();
            if now.duration_since(time).unwrap() >= Duration::from_secs(86400) {
                return get_space_web(name, url);
            } else {
                info!("2");
                let cache_file = File::open(&space_file)?;
                let reader = BufReader::new(cache_file);
                let json: Space = serde_json::from_reader(reader)?;

                let mut map = HashMap::new();

                map.insert("lat".to_string(), json.location.lat);
                map.insert("lon".to_string(), json.location.lon);
                //println!("{:?}", json);
                return Ok(map);
            }
        } else {
            Err(failure::err_msg("modified Date Missing"))
        }
    } else {
        return get_space_web(name, url);
    }
}

pub fn find_space(name: &str) -> Result<HashMap<String, f64>, failure::Error> {
    if Path::new("./tmp/spaces").exists() {
        let file_name_path = format!("./tmp/spaces/space_{}.json", name.replace("/", "_"));
        let space_file = Path::new(&file_name_path);
        if space_file.exists() {
            let cache_file = File::open(&space_file)?;
            let reader = BufReader::new(cache_file);
            let json: Space = serde_json::from_reader(reader)?;

            let mut map = HashMap::new();

            map.insert("lat".to_string(), json.location.lat);
            map.insert("lon".to_string(), json.location.lon);
            //println!("{:?}", json);
            return Ok(map);
        } else {
            let all_spaces = get_spaces();
            match all_spaces {
                Ok(x) => {
                    let space: Option<HashMap<String, f64>> =
                        x.into_iter()
                            .find_map(|x| if x.0 == name { Some(x.1) } else { None });
                    if space.is_some() {
                        Ok(space.unwrap())
                    } else {
                        Err(failure::err_msg("Space not found"))
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                    return Err(e);
                }
            }
        }
    } else {
        let all_spaces = get_spaces();
        match all_spaces {
            Ok(x) => {
                let space: Option<HashMap<String, f64>> =
                    x.into_iter()
                        .find_map(|x| if x.0 == name { Some(x.1) } else { None });
                if space.is_some() {
                    Ok(space.unwrap())
                } else {
                    Err(failure::err_msg("Space not found"))
                }
            }
            Err(e) => {
                error!("{:?}", e);
                return Err(e);
            }
        }
    }
}
