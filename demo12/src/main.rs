use interprocess::local_socket::ListenerOptions;
use interprocess::local_socket::traits::Listener;
use interprocess::{local_socket::ToFsName, os::windows::local_socket::NamedPipe};
use std::io;
use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    test_receive()?;
    Ok(())
}

fn test_receive() -> io::Result<()> {
    let listener = ListenerOptions::new()
        .name(r"\\.\pipe\example".to_fs_name::<NamedPipe>().unwrap())
        .create_sync()?;
    let mut conn = listener.accept()?;
    let mut buf = [0u8; 1024];
    let size = conn.read(&mut buf)?;
    println!("Client says: {}", String::from_utf8_lossy(&buf[..size]));
    let mut x = 0;
    loop {
        x = x + 1;
        conn.write_all(b"Hello from server!")?;
        sleep(Duration::from_secs(3));
        if x > 10 {
            break;
        }
    }

    Ok(())
}
