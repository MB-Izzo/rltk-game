use crate::{
    components::{Item, WantsToPickupItem, Monster},
    gamelog::GameLog,
    map::{TileType, MAPHEIGHT, MAPWIDTH},
    Aiming,
};

use super::{CombatStats, Map, Player, Position, RunState, State, Viewshed, WantsToMelee};
use rltk::{Point, Rltk};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();

    let map = ecs.fetch::<Map>();

    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    // Since we join the two, it will only run on those that
    // have both position and player components.
    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return;
        }
        let destination_index = map.get_index_at(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_index].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_index] {
            pos.x = min(MAPWIDTH as i32 - 1, max(0, pos.x + delta_x));
            pos.y = min(MAPHEIGHT as i32 - 1, max(0, pos.y + delta_y));
            viewshed.dirty = true;

            // Write new_pos to storage for everyone to access it
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
            // set global aiming pos to playery+1
            let mut aiming_pos = ecs.fetch_mut::<Aiming>();
            aiming_pos.x = pos.x;
            aiming_pos.y = pos.y - 1;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => {
            return RunState::AwaitingInput;
        }
        Some(key) => match key {
            rltk::VirtualKeyCode::Up | rltk::VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs)
            }

            rltk::VirtualKeyCode::Down | rltk::VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.ecs)
            }

            rltk::VirtualKeyCode::Left | rltk::VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.ecs)
            }

            rltk::VirtualKeyCode::Right | rltk::VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.ecs)
            }

            // Diagonals
            rltk::VirtualKeyCode::Y => try_move_player(1, -1, &mut gs.ecs),
            rltk::VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.ecs),
            rltk::VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),
            rltk::VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            rltk::VirtualKeyCode::G => get_item(&mut gs.ecs),

            rltk::VirtualKeyCode::I => return RunState::ShowInventory,

            rltk::VirtualKeyCode::D => return RunState::ShowDropItem,
            rltk::VirtualKeyCode::R => return RunState::ShowRemoveItem,

            rltk::VirtualKeyCode::Escape => return RunState::SaveGame,
            rltk::VirtualKeyCode::S => return skip_turn(&mut gs.ecs),

            rltk::VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }

            _ => {
                return RunState::AwaitingInput;
            }
        },
    }
    RunState::PlayerTurn
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();

    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if player_on_position(&player_pos, position) {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert wants to pickup");
        }
    }
}

fn player_on_position(player_pos: &Point, other_pos: &Position) -> bool {
    player_pos.x == other_pos.x && player_pos.y == other_pos.y
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.get_index_at(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no stairs here".to_string());
        false
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_comps = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let world_map_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_comps.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = world_map_resource.get_index_at(tile.x, tile.y);
        for entitiy_id in world_map_resource.tile_content[idx].iter() {
            let mob = monsters.get(*entitiy_id);
            match mob {
                None => {}
                Some(_) => { can_heal = false }
            }
        }
    }

    if can_heal {
        let mut healths = ecs.write_storage::<CombatStats>();
        let player_hp = healths.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
}
