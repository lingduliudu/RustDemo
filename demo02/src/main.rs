use eframe::egui;
mod defend;
use defend::is_expire;
/// 定义应用程序状态结构体
struct MyApp {
    // 存储一个状态值，用于与滑块交互
    value: f32,
    input_text: String,
    button_height: f32,
}

// 实现 eframe::App trait，这是 egui 应用的核心
impl eframe::App for MyApp {
    /// 应用程序的核心更新/渲染方法
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 创建一个中央面板来填充整个窗口
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui 最小样例");
            ui.label(format!("当前值: {:.2}", self.value));
            ui.text_edit_singleline(&mut self.input_text);
            let button =
                egui::Button::new("点击我增加高度").min_size(egui::vec2(0.0, self.button_height));
            let response = ui.add(button);
            if response.clicked() {
                self.button_height += 1.0;
                if self.button_height > 100.0 {
                    self.button_height = 100.0;
                }
                self.input_text = format!("当前高度是:{}", self.button_height);
            }
        });
    }
}

// 主函数入口
fn main() -> Result<(), eframe::Error> {
    //is_expire();
    // 配置窗口选项
    let options = eframe::NativeOptions {
        // 初始窗口大小
        // initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    // 启动 eframe
    eframe::run_native(
        "egui 最小样例应用", // 窗口标题
        options,
        Box::new(|_cc| {
            let ctx = &_cc.egui_ctx;
            let font_data = egui::FontData::from_static(include_bytes!("../msyh.ttf"));
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert("chinese_font".to_owned(), font_data);
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese_font".to_owned());

            ctx.set_fonts(fonts);
            Box::new(MyApp {
                value: 50.0, // 初始化值为 50.0
                input_text: "".to_owned(),
                button_height: 70.0,
            })
        }),
    )
}
