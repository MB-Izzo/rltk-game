use rltk::{VirtualKeyCode, Rltk};
use specs::prelude::*;
use super::{Position, Player, TileType, get_index_at, State};
use std::cmp::{min, max};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    // Since we join the two, it will only run on those that
    // have both position and player components.
    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_index = get_index_at(pos.x + delta_x, pos.y + delta_y);
        if map[destination_index] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        Some(key) => match key {
            rltk::VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            rltk::VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            rltk::VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            rltk::VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            _ => {}
        },
        None => {}
    }
}
