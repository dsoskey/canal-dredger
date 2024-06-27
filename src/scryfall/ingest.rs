use std::collections::HashMap;
use std::error::Error;
use std::fs::{read_to_string, write};
use std::thread::sleep;
use std::time::Duration;
use mtgql::apis::configuration::Configuration;
use mtgql::apis::default_api::migrations_get;
use mtgql::models::{Migration, MigrationMetadata};
use mtgql::models::migration::MigrationStrategy;

// old -> new(id, name)
pub type MigrationMap = HashMap<String, (String, String)>;
pub struct ScryfallClient {
    config: Configuration,
    cache_path: String,
}

impl ScryfallClient {
    pub fn new() -> ScryfallClient {
        ScryfallClient{ config: Configuration::new(), cache_path: "./res/scryfall".to_string() }
    }

    pub fn get_merge_map(&self) -> Result<MigrationMap, Box<dyn Error>> {
        let scryfall_path = format!("{}/merges.json", self.cache_path);
        let local_scryfall = read_to_string(&scryfall_path);
        let mut result = match local_scryfall {
            Ok(blob) => {
                serde_json::from_str::<MigrationMap>(&blob)?
            }
            Err(_) => {
                let output = self.get_merge_map_inner()?;
                write(&scryfall_path, serde_json::to_string(&output)?)?;
                output
            }
        };

        let override_path = format!("{}/manual-migrations.json", &self.cache_path);
        let manual_overrides = read_to_string(&override_path).unwrap_or("{}".to_string());
        let manual_overrides: MigrationMap = serde_json::from_str(&manual_overrides).unwrap_or(HashMap::new());
        for (k, v) in manual_overrides.iter() {
            result.insert(k.clone(), v.clone());
        };

        Ok(result)
    }

    fn get_merge_map_inner(&self) -> Result<MigrationMap, Box<dyn Error>> {
        let mut result: MigrationMap = HashMap::new();

        let migrations = self.get_all_migrations()?;

        for migration in migrations {
            if migration.migration_strategy == MigrationStrategy::Merge {
                if let Some(new_scryfall_id) = &migration.new_scryfall_id {
                    result.insert(
                        migration.old_scryfall_id.clone(),
                        (new_scryfall_id.clone(), migration.metadata.unwrap_or(MigrationMetadata{ name: "Unknown Card".to_string() }).name)
                    );
                }
            } else {
                result.insert(
                    migration.old_scryfall_id.clone(),
                    (migration.old_scryfall_id.clone(), migration.metadata.unwrap_or(MigrationMetadata{ name: "Unknown Card".to_string() }).name)
                );
            }
        }

        Ok(result)
    }

    pub fn get_all_migrations(&self) -> Result<Vec<Migration>, Box<dyn Error>> {
        let mut result: Vec<Migration> = Vec::new();

        let mut page: i32 = 1;
        let mut response = migrations_get(&self.config, page)?;
        result.extend(response.data);
        while response.has_more {
            page+=1;

            sleep(Duration::from_millis(50));
            response = migrations_get(&self.config, page)?;
            result.extend(response.data);
        }

        Ok(result)
    }
}