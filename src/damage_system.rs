use std::collections::HashMap;

use crate::{components::{TeleportsSymetrically, Viewshed}, gamelog::GameLog, RunState};

use super::{CombatStats, Map, Name, Player, Position, SufferDamage};
use rltk::console;
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage, positions, mut map, entities) = data;

        for (stats, damage, entity) in (&mut stats, &damage, &entities).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
            let pos = positions.get(entity);
            if let Some(pos) = pos {
                let idx = map.get_index_at(pos.x, pos.y);
                map.bloodstains.insert(idx);
            }
        }

        damage.clear();
    }
}

pub struct TeleportedSymSystem {}
impl<'a> System<'a> for TeleportedSymSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, TeleportsSymetrically>,
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut positions, mut tp_syms, entities, mut map, mut viewsheds) = data;
        
        let mut middle_positions: HashMap<Entity, Position> = HashMap::new();
        {
            for (entity, tp_sym) in (&entities, &tp_syms).join() {
                let mid_pos = positions.get(tp_sym.from).unwrap();
                middle_positions.insert(entity, mid_pos.clone());
            }
        }

        for (entity, mid_pos) in middle_positions {
            let pos = positions.get_mut(entity).unwrap();
            let vs = viewsheds.get_mut(entity).unwrap();
            let delta_x = pos.x - mid_pos.x;
            let delta_y = pos.y - mid_pos.y;

            let new_x = mid_pos.x - delta_x;
            let new_y = mid_pos.y - delta_y;
            pos.x = new_x;
            pos.y = new_y;
            let idx = map.get_index_at(new_x, new_y);
            map.blocked[idx] = true;
            map.tile_content[idx].push(entity);
            vs.dirty = true;

        }
        tp_syms.clear();



        /*
        for (tp_sym, entity) in (&mut tp_syms, &entities).join() {
            let from = positions.get(tp_sym.from).unwrap().clone();

            let entity_pos = positions.get_mut(entity).unwrap();

            entity_pos.x = from.x;
            entity_pos.y = from.y + 4;
        }
        tp_syms.clear();
        */
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut deads: Vec<Entity> = Vec::new();
    // Scope for borrow checker
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();

        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp <= 0 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("{} is dead", &victim_name.name));
                        }
                        deads.push(entity);
                    }
                    Some(_) => {
                        let mut run_state = ecs.write_resource::<RunState>();
                        *run_state = RunState::GameOver;
                    }
                }
            }
        }
    }
    for victim in deads {
        ecs.delete_entity(victim)
            .expect("Unable to delete dead enitity");
    }
}
