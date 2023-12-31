use specs::prelude::*;

use crate::{
    components::{
        AreaOfEffect, CombatStats, Confusion, Consumable, Equippable, Equipped, InBackpack,
        InflictsDamage, InflictsTeleportsSymetrically, Name, Position, ProvidesHealing,
        SufferDamage, TeleportsSymetrically, WantsToDropItem, WantsToPickupItem, WantsToRemoveItem,
        WantsToUseItem,
    },
    gamelog::GameLog,
    map::Map,
    particle_system::ParticleBuilder,
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
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, Equipped>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, InflictsTeleportsSymetrically>,
        WriteStorage<'a, TeleportsSymetrically>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_use_item,
            names,
            mut combat_stats,
            consumables,
            provide_healing,
            inflicts_damage,
            map,
            mut suffer_damage,
            aoe,
            mut confused,
            mut equipped,
            equippable,
            mut backpack,
            mut particle_builder,
            positions,
            inflicts_tp,
            mut receive_tp,
        ) = data;

        for (entity, want_use_item) in (&entities, &wants_use_item).join() {
            let mut used_item = true;

            let mut targets: Vec<Entity> = Vec::new();
            match want_use_item.target {
                None => {
                    targets.push(*player_entity);
                }
                Some(target) => {
                    let area_effect = aoe.get(want_use_item.item);
                    match area_effect {
                        None => {
                            // single target in tile
                            let idx = map.get_index_at(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            let mut blast_tiles =
                                rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.get_index_at(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                                particle_builder.request(
                                    tile_idx.x,
                                    tile_idx.y,
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('░'),
                                    200.0,
                                );
                            }
                        }
                    }
                }
            }

            let item_equippable = equippable.get(want_use_item.item);
            match item_equippable {
                None => {}
                Some(item_to_equip) => {
                    let target_slot = item_to_equip.slot;
                    let target = targets[0];

                    // remove any item the target has in the item slot
                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in
                        (&entities, &equipped, &names).join()
                    {
                        if already_equipped.owner == target && already_equipped.slot == target_slot
                        {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("You unequip {}.", name.name));
                            }
                        }
                    }

                    // unequip + put in backpack
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack
                            .insert(*item, InBackpack { owner: target })
                            .expect("Unable to insert backpack component");
                    }

                    // remove from backpack and equip item
                    equipped
                        .insert(
                            want_use_item.item,
                            Equipped {
                                owner: target,
                                slot: target_slot,
                            },
                        )
                        .expect("Unable to insert equippable");
                    backpack.remove(want_use_item.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!(
                            "You equip {}.",
                            names.get(want_use_item.item).unwrap().name
                        ));
                    }
                }
            }

            let item_heals = provide_healing.get(want_use_item.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    used_item = false;
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!(
                                    "You use {}, healing {} hp",
                                    names.get(want_use_item.item).unwrap().name,
                                    healer.heal_amount
                                ));
                            }
                            used_item = true;

                            let pos = positions.get(*target);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::GREEN),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('♥'),
                                    200.0,
                                );
                            }
                        }
                    }
                }
            }

            let item_damages = inflicts_damage.get(want_use_item.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    used_item = false;
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(want_use_item.item).unwrap();
                            gamelog.entries.push(format!(
                                "You use {} on {}, inflincting {} hp.",
                                item_name.name, mob_name.name, damage.damage
                            ));

                            let pos = positions.get(*mob);

                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::RED),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                );
                            }
                        }
                    }
                    used_item = true;
                }
            }

            let item_teleports_victim = inflicts_tp.get(want_use_item.item);
            match item_teleports_victim {
                None => {}
                Some(tp) => {
                    used_item = false;
                    for mob in targets.iter() {
                        receive_tp
                            .insert(*mob, TeleportsSymetrically { from: entity })
                            .expect("failed to insert tp");
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(want_use_item.item).unwrap();
                            gamelog.entries.push(format!(
                                "You use {} on {}, Teleporting them",
                                item_name.name, mob_name.name
                            ));
                        }
                    }
                    used_item = true;
                }
            }

            let mut confused_victims = Vec::new();
            {
                let causes_confusion = confused.get(want_use_item.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        used_item = false;
                        for mob in targets.iter() {
                            confused_victims.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(want_use_item.item).unwrap();
                                gamelog.entries.push(format!(
                                    "You use {} on {}, confusing them",
                                    item_name.name, mob_name.name
                                ));
                                let pos = positions.get(*mob);

                                if let Some(pos) = pos {
                                    particle_builder.request(
                                        pos.x,
                                        pos.y,
                                        rltk::RGB::named(rltk::PURPLE),
                                        rltk::RGB::named(rltk::BLACK),
                                        rltk::to_cp437('?'),
                                        200.0,
                                    );
                                }
                            }
                        }
                        used_item = true;
                    }
                }
            }
            for mob in confused_victims.iter() {
                confused
                    .insert(mob.0, Confusion { turns: mob.1 })
                    .expect("Unable to insert confused status");
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

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert removed item in backpack");
        }

        wants_remove.clear();
    }
}
