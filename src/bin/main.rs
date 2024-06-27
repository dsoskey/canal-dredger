use std::fs::remove_dir_all;

use canal_dredger::cubecobra::generate_git_history;
use canal_dredger::cubecobra::ingest::{CubeCobraClient, CubeCobraHttpClient};
use canal_dredger::local::ingest::CubeCobraLocalClient;
use canal_dredger::scryfall::ingest::ScryfallClient;

fn main() {
    // args
    let cube_id = "soskgy";
    let rebuild_repo = true;
    let local = Some(format!("./res/{}", cube_id));
    let repo_root = format!("./tmp/{}", cube_id);

    if rebuild_repo {
        remove_dir_all(&repo_root).unwrap_or(());
    }

    let scryfall = ScryfallClient::new();
    let migrations = scryfall.get_merge_map().unwrap();

    let client: Box<dyn CubeCobraClient> = if let Some(path) = local {
        Box::new(CubeCobraLocalClient::new(path))
    } else {
        Box::new(CubeCobraHttpClient::new())
    };

    generate_git_history(client, &repo_root, cube_id, &migrations).unwrap();
}