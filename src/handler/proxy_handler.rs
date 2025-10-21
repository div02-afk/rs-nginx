use std::io::Error;

use tokio::{io::copy, net::TcpStream, try_join};

pub async fn handle_proxy(
    request_stream: &mut TcpStream,
    proxy_address: &String,
) -> Result<(), Error> {
    let client_stream_result = TcpStream::connect(proxy_address).await;
    if let Ok(client_stream) = client_stream_result {
        let mut client_stream = client_stream;
        let (mut client_read_stream, mut client_write_stream) = client_stream.split();
        let (mut request_read_stream, mut request_write_stream) = request_stream.split();
        // println!("RW streams created");
        let request_client = copy(&mut request_read_stream, &mut client_write_stream);
        let client_request = copy(&mut client_read_stream, &mut request_write_stream);

        let _ = try_join!(request_client, client_request);
        Ok(())
    } else {
        //throw error
        Err(client_stream_result.err().unwrap())
    }
}
