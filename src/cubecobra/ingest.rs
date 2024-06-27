use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use cubecobra_client::apis::configuration::Configuration;
use cubecobra_client::apis::default_api::{cube_api_cube_json_cube_id_get, cube_api_history_cube_id_post};
use cubecobra_client::models::{CobraCube, HistoryInput, HistoryPost};

pub trait CubeCobraClient {
    fn get_cube(&self, cube_id: &str) -> Result<CobraCube, Box<dyn Error>>;
    fn get_full_cube_history(&self, cube_id: &str) -> Result<Vec<HistoryPost>, Box<dyn Error>>;
}
pub struct CubeCobraHttpClient {
    configuration: Configuration
}

impl CubeCobraHttpClient {
    pub fn new() -> CubeCobraHttpClient {
        let configuration = Configuration::new();
        CubeCobraHttpClient {
            configuration
        }
    }
}

impl CubeCobraClient for CubeCobraHttpClient {
    fn get_cube(&self, cube_id: &str) -> Result<CobraCube, Box<dyn Error>> {
        cube_api_cube_json_cube_id_get(&self.configuration, cube_id)
            .map_err(|i| i.into())
    }

    fn get_full_cube_history(&self, cube_id: &str) -> Result<Vec<HistoryPost>, Box<dyn Error>> {
        let mut result: Vec<HistoryPost> = Vec::new();

        let mut page: Option<HistoryInput> = None;
        let mut res = cube_api_history_cube_id_post(&self.configuration, cube_id, page)?;
        while res.last_key.is_some() {
            page = Some(HistoryInput::new(*res.last_key.unwrap().clone()));
            if let Some(posts) = res.posts {
                result.extend(posts);
            }

            sleep(Duration::from_millis(50));
            res = cube_api_history_cube_id_post(&self.configuration, cube_id, page)?;
        }

        Ok(result)
    }
}
