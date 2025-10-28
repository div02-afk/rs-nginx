use async_compression::tokio::write::BrotliEncoder;
use tokio::io::{self, AsyncWriteExt};

#[derive(Default, Debug)]
pub struct Brotli;

impl Brotli {
    pub async fn encode(
        &self,
        read_buf: [u8; 32768],
        write_buf: &mut Vec<u8>,
        n: usize,
    ) -> io::Result<()> {
        let mut encoder = BrotliEncoder::new(write_buf);
        encoder.write_all(&read_buf[..n]).await?;
        encoder.shutdown().await?;
        Ok(())
    }
}
