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
    format!("{}{}{}{}",file_content, s1, s2, s3)
}
