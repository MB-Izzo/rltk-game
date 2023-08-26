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
    for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
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
                wants_to_melee.insert(entity, WantsToMelee { target: *potential_target }).expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_index] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            viewshed.dirty = true;

            // Write new_pos to storage for everyone to access it
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
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
            _ => {
                return RunState::AwaitingInput;
            }
        },
    }
    RunState::PlayerTurn
}
