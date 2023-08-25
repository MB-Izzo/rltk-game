use super::{Player, Position, State, TileType, Map};
use rltk::Rltk;
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    // Since we join the two, it will only run on those that
    // have both position and player components.
    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_index = map.get_index_at(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[destination_index] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
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
            _ => {}
        },
        None => {}
    }
}
