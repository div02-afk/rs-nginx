use std::io::Error;

use tokio::{io::copy, net::TcpStream, try_join};

pub async fn handle_proxy(
    request_stream: &mut TcpStream,
    proxy_address: &String,
) -> Result<(), Error> {
    println!("Connecting to: {}", proxy_address.clone());
    let client_stream_result = TcpStream::connect(proxy_address).await;
    if client_stream_result.is_ok() {
        let mut client_stream = client_stream_result.unwrap();
        let (mut client_read_stream, mut client_write_stream) = client_stream.split();
        let (mut request_read_stream, mut request_write_stream) = request_stream.split();
        println!("RW streams created");
        let request_client = copy(&mut request_read_stream, &mut client_write_stream);
        let client_request = copy(&mut client_read_stream, &mut request_write_stream);

        let _ = try_join!(request_client, client_request);
        return Ok(());
    } else {
        //throw error
        return Err(client_stream_result.err().unwrap());
    }
}
