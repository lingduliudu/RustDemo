use crate::global_cache::{PORT};
pub fn get_after(file_content: String) -> String {
    let s1 = String::from(
        r#"
    <script>
        const WS_ADDRESS_lingduliudu_001 = "ws://127.0.0.1:"#,
    );
    let mut x: u16 = 0;
    unsafe {
        x = PORT;
    }
    let s2 = format!("{}", x);
    let s3 = String::from(
        r#"/ws/10";
            let socket_lingduliudu_001;
            let reconnectAttempts = 0;
            function connectWebSocket(){
                socket_lingduliudu_001 = new WebSocket(WS_ADDRESS_lingduliudu_001);
                socket_lingduliudu_001.onopen = function(e) {
                };
                socket_lingduliudu_001.onmessage = function(event) {
                    window.location.reload();
                };
                socket_lingduliudu_001.onclose = function(event) {
                };
                socket_lingduliudu_001.onerror = function(error) {
                };
            }
            window.onload = connectWebSocket;
        </script>
"#,
    );

    let safe = serde_json::to_string(&file_content).unwrap();
    let pre_content =  format!(r#"
     <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/github-markdown-css/github-markdown.min.css"></link>
     <div class="markdown-body" id="content"></div>
     <script src="https://cdn.jsdelivr.net/npm/markdown-it/dist/markdown-it.min.js"></script>
     <script>
     const md = window.markdownit();
     const html = md.render({});
     document.getElementById('content').innerHTML =  html;
     </script>"#,safe);
    format!("{}{}{}{}",pre_content, s1, s2, s3)
}
