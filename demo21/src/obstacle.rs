use bracket_lib::{prelude::*, random};

use crate::{SCREEN_HEIGHT, player::Player};
pub struct Obstacle {
    pub x: i32,
    pub gap_y: i32,
    pub size: i32,
}
impl Obstacle {
    pub fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Self {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
        }
    }
    pub fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }
    pub fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let dose_x_match = player.x == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let play_below_gap = player.y > self.gap_y + half_size;
        return dose_x_match && (player_above_gap || play_below_gap);
    }
}
