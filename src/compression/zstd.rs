use async_compression::tokio::write::ZstdEncoder;
use tokio::io::{self, AsyncWriteExt};
#[derive(Default, Debug)]
pub struct Zstd;

impl Zstd {
    pub async fn encode(
        &self,
        read_buf: [u8; 32768],
        write_buf: &mut Vec<u8>,
        n: usize,
    ) -> io::Result<()> {
        let mut encoder = ZstdEncoder::new(write_buf);
        encoder.write_all(&read_buf[..n]).await?;
        encoder.shutdown().await?;
        Ok(())
    }
}
