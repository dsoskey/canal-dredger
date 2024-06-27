use std::error::Error;
use std::io;
use std::path::Path;
use chrono::DateTime;
use cubecobra_client::models::{CobraCard, CobraCube};
use git2::{IndexAddOption, Oid, Repository, Signature, Time};
use crate::scryfall::ingest::MigrationMap;

const NULL: &str = "~~";
pub fn write_package_file(package_file: &Path, package: &Vec<CobraCard>, migrations: &MigrationMap) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(package_file)?;

    wtr.write_record(&[
        "Name",
        "Set",
        "CollectorNumber",
        "Status",
        "Tags",
        "Finish",
        "Cmc",
        "Colors",
        "ColorCategory",
        "Rarity",
        "TypeLine",
    ])?;

    for card in package {
        let (_, name) = migrations
            .get(&card.card_id)
            .unwrap_or(&(card.card_id.clone(), card.details.name.clone())).clone();
        wtr.write_record(&[
            &name,
            &card.details.set.clone().unwrap_or(NULL.to_string()),
            &card.details.collector_number.clone().unwrap_or(NULL.to_string()),
            &card.status.clone().unwrap_or(NULL.to_string()),
            &card.tags.clone().map_or(NULL.to_string(), |tags| if tags.len() > 0 {
                tags.join(",")
            } else {
                NULL.to_string()
            }),
            &card.finish.clone().unwrap_or(NULL.to_string()),
            &card.cmc.clone().unwrap_or(None)
                .unwrap_or(Box::from(serde_json::Value::String(NULL.to_string())))
                .to_string(),
            &card.colors.clone().map_or(NULL.to_string(), |tags| if tags.len() > 0 {
                tags.join(",")
            } else {
                NULL.to_string()
            }),
            &card.color_category.clone().unwrap_or(None).unwrap_or(NULL.to_string()),
            &card.rarity.clone().unwrap_or(None).unwrap_or(NULL.to_string()),
            &card.type_line.clone().unwrap_or(NULL.to_string()),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn write_overview_file(repo_root: &str, cube: &CobraCube) -> Result<(), io::Error> {
    std::fs::write::<String, String>(
        format!("{}/{}", repo_root, "README.md"),
        format!("# {}\n\n![{}]({})\n{}\n", cube.name, cube.image_name, cube.image.uri, cube.description)
    )?;

    Ok(())
}

pub fn commit(repo: &Repository, author: &str, timestamp_millis: i64, message: &str, prev_commit_oid: Option<Oid>) -> Result<Oid, git2::Error> {
    let timestamp_seconds = timestamp_millis / 1000;
    let time = Time::new(timestamp_seconds, 0);
    let dt = DateTime::from_timestamp(timestamp_seconds.clone(), 0).unwrap();
    let mut repo_index = repo.index()?;
    let message = format!("{}\n{}", dt.to_rfc2822(), message);
    println!("committing at {} ({})", dt.to_rfc2822(), timestamp_millis);

    repo_index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    let oid = repo_index.write_tree()?;
    let tree = repo.find_tree(oid)?;

    let signature = Signature::new(
        author,
        "email@example.com",
        &time
    )?;

    if let Some(oid) = prev_commit_oid {
        let parent = repo.find_commit(oid)?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &[&parent],
        )
    } else {
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &[],
        )
    }
}

