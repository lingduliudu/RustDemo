#![windows_subsystem = "windows"]
use slint::{ModelRc, VecModel};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

slint::include_modules!();

#[derive(Clone, Debug)]
struct Row {
    code: String,
    desc: String,
}

const FILES: &[&str] = &["F:/dnf/zhuangbei.txt", "F:/dnf/xiaohaopin.txt"];

fn load_all() -> Vec<Row> {
    let mut out = Vec::new();
    for path in FILES {
        let path = PathBuf::from(path);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // 按第一个空格切分：第一列 code，后面全是说明
            let mut parts = line.splitn(2, |c: char| c.is_whitespace());
            let code = parts.next().unwrap_or("").trim().to_string();
            let desc = parts.next().unwrap_or("").trim().to_string();
            if code.is_empty() {
                continue;
            }
            out.push(Row { code, desc });
        }
    }
    out
}

fn to_model(rows: &[Row]) -> ModelRc<RowData> {
    let m: Vec<RowData> = rows
        .iter()
        .map(|r| RowData {
            code: r.code.clone().into(),
            desc: r.desc.clone().into(),
        })
        .collect();
    ModelRc::new(VecModel::from(m))
}

fn main() -> anyhow::Result<()> {
    let app = App::new()?;
    let all = Arc::new(load_all());
    let weak = app.as_weak();

    // 初始显示全部
    app.set_results(to_model(&all));
    app.set_status(format!("共 {} 条", all.len()).into());
    app.set_toast("".into());

    // 搜索：关键词为空 = 全部；否则 code 或 desc 包含关键词（忽略大小写）
    {
        let all = all.clone();
        app.on_search(move |kw: slint::SharedString| {
            let kw = kw.trim().to_lowercase();
            let filtered: Vec<Row> = if kw.is_empty() {
                (*all).clone()
            } else {
                all.iter()
                    .filter(|r| {
                        r.code.to_lowercase().contains(&kw) || r.desc.to_lowercase().contains(&kw)
                    })
                    .cloned()
                    .collect()
            };
            if let Some(app) = weak.upgrade() {
                app.set_results(to_model(&filtered));
                app.set_status(format!("找到 {} 条 / 共 {} 条", filtered.len(), all.len()).into());
                app.set_toast("".into());
            }
        });
    }

    // 双击复制：用 arboard 写剪贴板
    {
        let weak2 = app.as_weak();
        app.on_copy_code(move |code: slint::SharedString| {
            let code_str = code.to_string();
            let msg =
                match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(code_str.clone())) {
                    Ok(_) => format!("已复制：{}", code_str),
                    Err(e) => format!("复制失败：{}，请手动复制 {}", e, code_str),
                };
            if let Some(app) = weak2.upgrade() {
                app.set_toast(msg.into());
            }
        });
    }

    app.run()?;
    Ok(())
}
