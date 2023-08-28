use specs::prelude::*;

use crate::{
    components::{
        CombatStats, Consumable, InBackpack, InflictsDamage, Name, Position, Potion,
        ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickupItem, WantsToUseItem,
    },
    gamelog::GameLog,
    map::Map,
};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You picked up {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_use_item,
            names,
            potions,
            mut combat_stats,
            consumables,
            provide_healing,
            inflicts_damage,
            map,
            mut suffer_damage,
        ) = data;

        for (entity, want_use_item, stats) in (&entities, &wants_use_item, &mut combat_stats).join()
        {
            let mut used_item = true;

            let item_heals = provide_healing.get(want_use_item.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You drink the {}, healing {} hp",
                            names.get(want_use_item.item).unwrap().name,
                            healer.heal_amount
                        ));
                    }
                }
            }

            let item_damages = inflicts_damage.get(want_use_item.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    let target_point = want_use_item.target.unwrap();
                    let idx = map.get_index_at(target_point.x, target_point.y);
                    used_item = false;
                    for mob in map.tile_content[idx].iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(want_use_item.item).unwrap();
                            gamelog.entries.push(format!(
                                "You use {} on {}, inflincting {} hp.",
                                item_name.name, mob_name.name, damage.damage
                            ));
                        }
                        used_item = true;
                    }
                }
            }

            if used_item {
                let consumable = consumables.get(want_use_item.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities
                            .delete(want_use_item.item)
                            .expect("Delete consumable failed");
                    }
                }
            }
        }

        wants_use_item.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut in_backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }

            positions
                .insert(
                    to_drop.item,
                    Position {
                        x: dropper_pos.x,
                        y: dropper_pos.y,
                    },
                )
                .expect("Unable to insert position to dropped item");
            in_backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop {}.",
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }
        wants_drop.clear();
    }
}
