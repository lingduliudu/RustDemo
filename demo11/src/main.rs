use interprocess::os::windows::named_pipe::PipeStream;
use interprocess::os::windows::named_pipe::pipe_mode::Bytes;
use std::io::{self, Read, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    println!("Client connected.");
    test_client()?;
    Ok(())
}

fn test_client() -> io::Result<()> {
    let mut conn: PipeStream<Bytes, Bytes> = PipeStream::connect_by_path(r"\\.\pipe\example")?;
    conn.write_all(b"Hello from client!")?;
    let mut x = 0;
    loop {
        x = x + 1;
        let mut buf = [0u8; 1024];
        let read = conn.read(&mut buf)?;
        if read == 0 {
            println!("server closed the pipe");
            break;
        }
        println!("Server replied: {}", String::from_utf8_lossy(&buf[..read]),);
    }
    Ok(())
}
