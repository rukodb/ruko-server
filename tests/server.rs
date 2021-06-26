use ruko_server::server;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// A basic "hello world" style test. A server instance is started in a
/// background task. A client TCP connection is then established and raw redis
/// commands are sent to the server. The response is evaluated at the byte
/// level.
#[tokio::test]
async fn key_value_get_set() {
    // start the server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move { server::run(listener).await.expect("fixme") });

    let mut stream = TcpStream::connect(addr).await.unwrap();

    // GET 'hello'
    stream
        .write_all(b"GET*hello*")
        .await
        .unwrap();

    // Read nil response
    let mut response = [0; 3];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"ERR", &response);

    // SET 'hello' = 'world'
    stream
        .write_all(b"SET*hello*world*")
        .await
        .unwrap();

    // Read OK
    let mut response = [0; 2];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"OK", &response);

    // GET 'hello'
    stream
        .write_all(b"GET*hello*")
        .await
        .unwrap();

    // Shutdown the write half
    // stream.shutdown().await.unwrap();

    // Read "world" response
    let mut response = [0; 9];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"OK*world*", &response);

    // Receive `None`
    assert_eq!(0, stream.read(&mut response).await.unwrap());
}
