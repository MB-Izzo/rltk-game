use crate::{gamelog::GameLog, RunState};

use super::{CombatStats, SufferDamage, Player, Name};
use rltk::console;
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
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
                        let mut run_state  = ecs.write_resource::<RunState>();
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
