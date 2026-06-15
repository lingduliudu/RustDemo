use bracket_lib::prelude::*;

pub struct State {
    button_x: i32,
    button_y: i32,
    button_width: i32,
    button_height: i32,
    click_count: i32,
}

impl State {
   pub fn new() -> Self {
        Self {
            button_x:0,
            button_y:40,
            button_width: 100,
            button_height:40,
            click_count: 0,
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls(); // 清屏

        // 1. 获取鼠标状态
        let mouse_pos = ctx.mouse_pos();
        let mouse_x = mouse_pos.0;
        let mouse_y = mouse_pos.1;

        // 判断鼠标是否悬停在按钮上
        let is_hovered = mouse_x >= self.button_x
            && mouse_x < self.button_x + self.button_width
            && mouse_y >= self.button_y
            && mouse_y < self.button_y + self.button_height;

        // 2. 根据悬停状态改变按钮颜色（视觉反馈）
        let mut bg = RGBA::from_u8(0, 0, 255, 255);
        let mut fg = RGBA::from_u8(0, 255, 0, 255);
        if is_hovered {
            bg = RGBA::from_u8(255, 0, 0, 255); // 悬停时背景变灰
            fg = RGBA::from_u8(0, 255, 0, 255);

            // 3. 检测点击
            if ctx.left_click {
                self.click_count += 1;
            }
        }

        // 4. 绘制按钮
        ctx.printer(
            self.button_x,
            self.button_y,
            "[ 点我! ]",
            TextAlign::Left,
            Some(fg)
        );

        // 显示点击次数
        ctx.print(1, 1, format!("点击次数: {}", self.click_count));
    }
}
