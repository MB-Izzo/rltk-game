use super::{CombatStats, SufferDamage, Player};
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
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp <= 0 {
                let player = players.get(entity);
                match player {
                    None => deads.push(entity),
                    Some(_) => console::log("You are dead"),
                }
            }
        }
    }
    for victim in deads {
        ecs.delete_entity(victim)
            .expect("Unable to delete dead enitity");
    }
}
