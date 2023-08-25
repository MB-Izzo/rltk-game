use rltk::{Rltk, RGB};

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;

    for tile in map.iter() {
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('#'),
                );
            }
        }
        // move coords
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

pub fn get_index_at(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

pub fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80 * 50];

    // boundary walls
    for x in 0..80 {
        map[get_index_at(x, 0)] = TileType::Wall;
        map[get_index_at(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[get_index_at(0, y)] = TileType::Wall;
        map[get_index_at(79, y)] = TileType::Wall;
    }

    let mut rng = rltk::RandomNumberGenerator::new();
    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let index = get_index_at(x, y);
        if index != get_index_at(40, 25) {
            map[index] = TileType::Wall;
        }
    }
    map
}
