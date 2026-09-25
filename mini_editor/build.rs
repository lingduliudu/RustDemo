fn main() {
    // 只在 Windows 下设置 exe 图标（资源管理器里看到的图标）
    #[cfg(windows)]
    {
        // 需要一个 .ico 文件，放到 assets/app.ico
        // 如果你没有 assets 目录，先创建
        let ico_path = "assets/app.ico";
        if std::path::Path::new(ico_path).exists() {
            // embed-resource 是最简单的方式
            let mut res = winres::WindowsResource::new();
            res.set_icon(ico_path);
            // 可选：设置文件信息
            res.set("FileDescription", "Mini Editor");
            res.set("ProductName", "Mini Editor");
            res.set("OriginalFilename", "mini_editor.exe");
            if let Err(e) = res.compile() {
                eprintln!("winres compile failed: {}", e);
            }
        }
    }
}
