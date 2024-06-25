use std::fs::{read_to_string, remove_dir_all};
use std::thread::sleep;
use std::time::Duration;

use cubecobra::apis::default_api::{cube_api_cube_json_cube_id_get, cube_api_history_cube_id_post};
use cubecobra::models::{CobraCube, HistoryInput, HistoryPage, HistoryPost};

use canal_dredger::ingest::cubecobra::generate_git_history;

fn real_main() {
    // get args
    let cube_id = "soskgy";
    let repo_root = format!("./tmp/{}", cube_id);

    let ccconfig = cubecobra::apis::configuration::Configuration::new();
    let cube = cube_api_cube_json_cube_id_get(&ccconfig, cube_id).unwrap();

    println!("cubbo\n\n{:?}\n\n", cube);

    let mut changes: Vec<HistoryPost> = Vec::new();

    let mut page: Option<HistoryInput> = None;
    let mut res = cube_api_history_cube_id_post(&ccconfig, cube.id.as_str(), page).unwrap();
    while res.last_key.is_some() {
        page = Some(HistoryInput::new(res.last_key.unwrap().as_ref().clone()));
        if res.posts.is_some() {
            changes.extend(res.posts.unwrap());
        }

        sleep(Duration::from_millis(50));
        res = cube_api_history_cube_id_post(&ccconfig, cube.id.as_str(), page).unwrap();
    }

    generate_git_history(&repo_root, &cube, &changes).unwrap();
}

// used to manually test generate_git_history
fn main() {
    // arg
    let cube_id = "andymangold";
    let repo_root = format!("./tmp/{}", cube_id);

    // arg
    let rebuild_repo = false;
    if rebuild_repo {
        remove_dir_all(&repo_root).unwrap();
    }

    let cube_path = format!("./res/{}/cube-sample.json", cube_id);
    let cube: CobraCube = serde_json::from_str(
        &read_to_string(cube_path).unwrap()
    ).unwrap();

    let history_path = format!("./res/{}/history-sample.json", cube_id);
    let history: HistoryPage = serde_json::from_str(
        &read_to_string(history_path).unwrap()
    ).unwrap();

    generate_git_history(
        &repo_root,
        &cube,
        &history.posts.unwrap()
    ).unwrap();
}