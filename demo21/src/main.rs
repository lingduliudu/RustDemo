#![allow(unused)]
//#![windows_subsystem = "windows"]
use bracket_lib::{prelude::*, terminal::VirtualKeyCode::O};
mod player;
use player::Player;
mod obstacle;
use obstacle::Obstacle;
mod button;
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 70;
const FRAME_DURATION: f32 = 75.0;

enum GameMode {
    Menu,
    Playing,
    End,
}
/**************************************************************
* Description:  状态
* Author: yuanhao
* Versions: V1
**************************************************************/
struct State {
    pub player: Player,
    frame_time: f32,
    mode: GameMode,
    score: i32,
    obstacle: Obstacle,
}
impl State {
    fn new() -> Self {
        Self {
            player: Player::new(5, 25),
            frame_time: 0.0,
            mode: GameMode::Menu,
            score: 0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
        }
    }
    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "welcome to flappy dragon");
        ctx.print_centered(8, "(P) play game");
        ctx.print_centered(9, "(Q) quit game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Q => ctx.quitting = true,
                VirtualKeyCode::P => self.play(ctx),
                _ => {}
            }
        }
    }
    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "game over");
        ctx.print_centered(6, &format!("Your Score is {}",self.score));
        ctx.print_centered(8, "(P) play game");
        ctx.print_centered(9, "(Q) quit game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Q => ctx.quitting = true,
                VirtualKeyCode::P => self.play(ctx),
                _ => {}
            }
        }
    }
    fn play(&mut self, ctx: &mut BTerm) {
        self.mode = GameMode::Playing;
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.render(ctx);
        self.obstacle.render(ctx, self.player.x);
        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("score:{}", self.score));

        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }
        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }
    fn restart(&mut self, ctx: &mut BTerm) {
        self.mode = GameMode::Playing;
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
    }
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}
fn main() -> BError {
    let context = BTermBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)?
        .with_title("Flappy Dragon")
        .build()?;
    return main_loop(context, button::State::new());
}
