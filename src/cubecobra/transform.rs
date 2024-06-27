use std::error::Error;
use chrono::Utc;
use cubecobra_client::models::{CobraCard, HistoryPost, PackageChange};
use crate::scryfall::ingest::MigrationMap;

pub struct CobraCubeSnapshot {
    pub timestamp: i64,
    pub main: Option<Vec<CobraCard>>,
    pub mayb: Option<Vec<CobraCard>>,
}

fn revert_changelog(
    package: &mut Vec<CobraCard>,
    changes: &Box<PackageChange>,
    migrations: &MigrationMap
) {
    if let Some(adds) = &changes.adds {
        for add in adds {
            let (id, name) = migrations
                .get(&add.card_id)
                .unwrap_or(&(add.card_id.clone(), add.details.name.clone())).clone();
            let index = package.iter().rposition(|card| card.card_id == id || card.details.name == name)
                .expect(&format!("failed to revert add: couldn't find card {} with ID {}", add.details.name, add.card_id));
            package.remove(index);
        }
    }

    if let Some(removes) = &changes.removes {
        // sort by ascending index on revert
        let mut removes_copy = removes.clone();
        removes_copy.sort_by(|a, b| a.index.unwrap_or(0).cmp(&b.index.unwrap_or(0)));
        for remove in removes_copy {
            let card = *(remove.old_card.clone());
            match remove.index {
                Some(index) => {
                    package.insert(index as usize, card);
                }
                None => {
                    package.push(card);
                }
            }
        }
    }

    if let Some(edits) = &changes.edits {
        for edit in edits {
            let index = match edit.index {
                Some(index) => { index as usize }
                None => {
                    let (id, name) = migrations
                        .get(&edit.new_card.card_id)
                        .unwrap_or(&(edit.new_card.card_id.clone(), edit.new_card.details.name.clone())).clone();
                    package.iter().rposition(|card| card.card_id == id || card.details.name == name)
                        .expect(&format!("failed to revert edit: couldn't find card {} with ID {}", edit.new_card.details.name, edit.new_card.card_id))
                }
            };
            package[index] = *edit.old_card.clone();
        }
    }

    if let Some(swaps) = &changes.swaps {
        for swap in swaps {
            let index = match swap.index {
                Some(index) => { index as usize }
                None => {
                    let (id, name) = migrations
                        .get(&swap.card.card_id)
                        .unwrap_or(&(swap.card.card_id.clone(), swap.card.details.name.clone())).clone();
                    package.iter().rposition(|card| card.card_id == id || card.details.name == name)
                        .expect(&format!("failed to revert swap: couldn't find card {} with ID {}", swap.card.details.name, swap.card.card_id))
                }
            };
            package[index] = *swap.old_card.clone();
        }
    }
}

pub fn generate_cubecobra_snapshots(
    mainboard: &mut Vec<CobraCard>,
    maybeboard: &mut Vec<CobraCard>,
    changes: &Vec<HistoryPost>,
    migrations: &MigrationMap
) -> Result<Vec<CobraCubeSnapshot>, Box<dyn Error>> {
    let mut result: Vec<CobraCubeSnapshot> = Vec::new();
    let mut timestamp: i64 = Utc::now().timestamp_millis();
    result.push(CobraCubeSnapshot {
        timestamp,
        main: Some(mainboard.clone()),
        mayb: Some(maybeboard.clone()),
    });

    // generate snapshots back in time
    for change in changes.iter() {
        timestamp = change.date.unwrap_or(timestamp - 1);
        if let Some(changelog) = &change.changelog {
            let mut main: Option<Vec<CobraCard>> = None;
            if let Some(mainboard_changes) = &changelog.mainboard {
                revert_changelog(mainboard, &mainboard_changes, migrations);
                main = Some(mainboard.clone());
            }

            let mut mayb: Option<Vec<CobraCard>> = None;
            if let Some(maybeboard_changes) = &changelog.maybeboard {
                revert_changelog(maybeboard, &maybeboard_changes, migrations);
                mayb = Some(maybeboard.clone());

            }

            if main.is_some() || mayb.is_some() {
                result.push(CobraCubeSnapshot { timestamp, main, mayb })
            }
        }
    }
    let first_snapshot = result.pop().unwrap();
    result.push(CobraCubeSnapshot {
        timestamp: first_snapshot.timestamp,
        main: Some(mainboard.clone()),
        mayb: Some(maybeboard.clone())
    });
    result.reverse();
    Ok(result)
}