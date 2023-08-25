use specs::prelude::*;
use super::{Viewshed, Position, Map, Player};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (WriteExpect<'a, Map>,
                       Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, Player>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >=0 && p.y < map.height);

                // runs only if entity is Player
                let player: Option<&Player> = player.get(ent);
                if let Some(_player) = player { 
                    for visible_tile in viewshed.visible_tiles.iter() {
                        let idx = map.get_index_at(visible_tile.x, visible_tile.y);
                        map.revealed_tiles[idx] = true;
                    }
                }
            }
        }
    }
}
