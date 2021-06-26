use log::{info, error};
use crate::Ruko;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufWriter};

// see: https://github.com/tokio-rs/mini-redis

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: Vec<u8>,
}

impl Connection {
    fn new(sock: TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(sock),
            buffer: Vec::new(), // FIXME probably should preallocate
        }
    }

    async fn read_frame(&self) -> Result<Option<Frame>, Box<dyn std::error::Error>> {
        loop {
            // Attempt to parse a frame from the buffered data
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // There is not enough buffered data to read a frame. Attempt to
            // read more data from the socket.
            //
            // On success, the number of bytes is returned. `0` indicates "end
            // of stream".
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    /// attempt to parse buffer into frame
    fn parse_frame(&self) -> Result<Option<Frame>, Box<dyn std::error::Error>> {
        let buf = self.buffer;
        match buf.iter().take(3).collect::<&[u8; 3]>() {
            b"GET" => Ok(None),
            b"SET" => Ok(None),
            unknown => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct Frame {}

pub async fn run(listener: TcpListener) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Ruko server.");
    let db = Ruko::new();
    loop {
        let (sock, _) = listener.accept().await?;
        tokio::spawn(async move {
            let conn = Connection::new(sock);
            let frame = conn.read_frame().await.unwrap(); // FIXME
            let cmd = Command::from_frame(frame).unwrap(); // FIXME
        });
    }
}
