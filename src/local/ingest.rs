use std::error::Error;
use std::fs::read_to_string;
use cubecobra_client::models::{CobraCube, HistoryPage, HistoryPost};
use crate::cubecobra::ingest::CubeCobraClient;

pub struct CubeCobraLocalClient {
    cache_path: String
}

impl CubeCobraLocalClient {
    pub fn new(cache_path: String) -> CubeCobraLocalClient {
        CubeCobraLocalClient {
            cache_path
        }
    }
}

impl CubeCobraClient for CubeCobraLocalClient {
    fn get_cube(&self, _id_encoded_in_cache_path: &str) -> Result<CobraCube, Box<dyn Error>> {
        let cube_path = format!("{}/cube.json", self.cache_path);
        let file_contents = read_to_string(&cube_path)?;
        Ok(serde_json::from_str::<CobraCube>(&file_contents)
            .map_err(|e| Box::new(e))?)
    }

    fn get_full_cube_history(&self, _id_encoded_in_cache_path: &str) -> Result<Vec<HistoryPost>, Box<dyn Error>> {
        let history_path = format!("{}/history.json", self.cache_path);
        Ok(serde_json::from_str::<HistoryPage>(&read_to_string(history_path)?)?.posts.unwrap())
    }
}