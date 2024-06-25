use std::error::Error;
use std::io;
use std::path::Path;
use std::string::ToString;

use chrono::DateTime;
use cubecobra::models::{CobraCard, CobraCube, HistoryPost, PackageChange};
use git2::{IndexAddOption, Oid, Repository, RepositoryInitOptions, Signature, Time};

const NULL: &str = "~~";
fn write_package_file(package_file: &Path, package: &Vec<CobraCard>) -> Result<(), Box<dyn Error>> {
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
        wtr.write_record(&[
            &card.details.name,
            &card.details.set.clone().unwrap_or(NULL.to_string()),
            &card.details.collector_number.clone().unwrap_or(NULL.to_string()),
            &card.status,
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

fn write_overview_file(repo_root: &str, cube: &CobraCube) -> Result<(), io::Error> {
    std::fs::write::<String, String>(
        format!("{}/{}", repo_root, "README.md"),
        format!("# {}\n\n![{}]({})\n{}\n", cube.name, cube.image_name, cube.image.uri, cube.description)
    )?;

    Ok(())
}

fn apply_changelog(cube: &CobraCube, package: &mut Vec<CobraCard>, changes: &Box<PackageChange>) {
    if let Some(swaps) = &changes.swaps {
        for swap in swaps {
            package[swap.index.clone() as usize] = *swap.card.clone();
        }
    }

    if let Some(edits) = &changes.edits {
        for edit in edits {
            package[edit.index.clone() as usize] = *edit.new_card.clone();
        }
    }

    if let Some(removes) = &changes.removes {
        // sort by descending index on apply
        let mut removes_copy = removes.clone();
        removes_copy.sort_by(|a, b| b.index.cmp(&a.index));
        for remove in removes {
            package.remove(remove.index.clone() as usize);
        }
    }

    if let Some(adds) = &changes.adds {
        for add in adds {
            let mut card = CobraCard::new(
                cube.default_status.clone(),
                add.card_id.clone(),
                add.added_tmsp.clone(),
                add.details.clone(),
            );

            card.index = Some(package.len() as i32);
            package.push(card);
        }
    }


}

fn revert_changelog(package: &mut Vec<CobraCard>, changes: &Box<PackageChange>) {
    if let Some(adds) = &changes.adds {
        for add in adds {
            let index = package.iter().rposition(|card| card.card_id == add.card_id)
                .expect(&format!("failed to revert add: couldn't find card {} with ID {}", add.details.name, add.card_id));
            package.remove(index);
        }
    }

    if let Some(removes) = &changes.removes {
        // sort by ascending index on revert
        let mut removes_copy = removes.clone();
        removes_copy.sort_by(|a, b| a.index.cmp(&b.index));
        for remove in removes_copy {
            package.insert(remove.index.clone() as usize, *(remove.old_card.clone()));
        }
    }

    if let Some(edits) = &changes.edits {
        for edit in edits {
            package[edit.index.clone() as usize] = *edit.old_card.clone();
        }
    }

    if let Some(swaps) = &changes.swaps {
        for swap in swaps {
            package[swap.index.clone() as usize] = *swap.old_card.clone();
        }
    }
}

fn commit(repo: &Repository, author: &str, timestamp_millis: i64, message: &str, prev_commit_oid: Option<Oid>) -> Result<Oid, git2::Error> {
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

pub fn generate_git_history(
    repo_root: &str,
    cube: &CobraCube,
    changes: &Vec<HistoryPost>
) -> Result<(), Box<dyn Error>> {
    let repo = Repository::init_opts(&Path::new(&repo_root), RepositoryInitOptions::new().no_reinit(true))
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

    let mut timestamp: i64 = 0;

    for change in changes.iter() {
        timestamp = change.date.unwrap_or(0);
        if let Some(changelog) = &change.changelog {
            if let Some(mainboard_changes) = &changelog.mainboard {
                revert_changelog(&mut mainboard, &mainboard_changes);
            }

            if let Some(maybeboard_changes) = &changelog.maybeboard {
                revert_changelog(&mut maybeboard, &maybeboard_changes)
            }
        }
    }

    write_package_file(mainboard_path,  &mainboard)?;
    write_package_file(maybeboard_path, &maybeboard)?;
    // write_data_file()
    let mut oid = commit(&repo, &cube.owner.username, timestamp - 1, "initial cube", None)?;

    for change in changes.iter().rev() {
        timestamp = change.date.unwrap_or(timestamp.clone() + 1);
        if let Some(changelog) = &change.changelog {
            let mut num_pgk_changes: usize = 0;
            if let Some(mainboard_changes) = &changelog.mainboard {
                apply_changelog(cube, &mut mainboard, &mainboard_changes);
                write_package_file(&mainboard_path, &mainboard)?;
                num_pgk_changes +=1;
            }

            if let Some(maybeboard_changes) = &changelog.maybeboard {
                apply_changelog(cube, &mut maybeboard, &maybeboard_changes);
                write_package_file(&maybeboard_path, &maybeboard)?;
                num_pgk_changes +=1;
            }

            if num_pgk_changes > 0 {
                // write_data_file()
                oid = commit(&repo, &cube.owner.username, timestamp, "change mode", Some(oid))?;
            }
        }
    }

    Ok(())
}