use std::error::Error;
use std::path::Path;

use cubecobra_client::models::CobraCard;
use git2::{Oid, Repository, RepositoryInitOptions};

use output::{commit, write_overview_file, write_package_file};
use transform::generate_cubecobra_snapshots;

use crate::cubecobra::ingest::CubeCobraClient;
use crate::scryfall::ingest::MigrationMap;

pub mod ingest;
pub mod transform;
mod output;


// this is transform + output
pub fn generate_git_history(
    ccclient: Box<dyn CubeCobraClient>,
    repo_root: &str,
    cube_id: &str,
    migrations: &MigrationMap
) -> Result<(), Box<dyn Error>> {
    let cube = ccclient.get_cube(cube_id).unwrap();
    let changes = ccclient.get_full_cube_history(&cube.id).unwrap();
    let repo = Repository::init_opts(&Path::new(&repo_root), RepositoryInitOptions::new()
        .no_reinit(true))
        .expect(&format!("Repo already exists at {}", &repo_root));

    write_overview_file(&repo_root, &cube)?;

    let mut mainboard: Vec<CobraCard> = cube.cards.mainboard.clone();
    mainboard.sort_by(|a, b| a.index.unwrap_or(0).cmp(&b.index.unwrap_or(0)));
    let mainboard_path = format!("{}/{}", repo_root, "mainboard");
    let mainboard_path = Path::new(&mainboard_path);

    let mut maybeboard = cube.cards.maybeboard.clone();
    maybeboard.sort_by(|a, b| a.index.unwrap_or(0).cmp(&b.index.unwrap_or(0)));
    let maybeboard_path = format!("{}/{}", repo_root, "maybeboard");
    let maybeboard_path = Path::new(&maybeboard_path);

    let history = generate_cubecobra_snapshots(
        &mut mainboard, &mut maybeboard, &changes, migrations,
    )?;

    // write changes forward in time
    let mut oid: Option<Oid> = None;
    for change in history.iter() {
        if let Some(main) = &change.main {
            write_package_file(&mainboard_path, main, migrations)?;
        }
        if let Some(mayb) = &change.mayb {
            write_package_file(&maybeboard_path, mayb, migrations)?;
        }

        if change.main.is_some() || change.mayb.is_some() {
            // write_data_file()
            oid = Some(commit(&repo, &cube.owner.username, change.timestamp, "change mode", oid)?);
        }
    }

    Ok(())
}