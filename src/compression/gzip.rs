use async_compression::tokio::write::GzipEncoder;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

pub enum Encoding {
    GZIP,
    NONE,
}

pub async fn compress_stream<R, W>(mut reader: R, mut writer: W) -> io::Result<()>
where
    R: io::AsyncRead + Unpin,
    W: io::AsyncWrite + Unpin,
{
    let mut gzip_buf = Vec::with_capacity(32 * 1024);
    let mut read_buf = [0u8; 32 * 1024];

    loop {
        let n = reader.read(&mut read_buf).await?;
        if n == 0 {
            break;
        }

        // Compress this chunk into gzip_buf
        gzip_buf.clear();
        {
            let mut encoder = GzipEncoder::new(&mut gzip_buf);
            encoder.write_all(&read_buf[..n]).await?;
            encoder.shutdown().await?;
        }

        // Write chunk length (in hex)
        writer
            .write_all(format!("{:X}\r\n", gzip_buf.len()).as_bytes())
            .await?;

        // Write compressed chunk
        writer.write_all(&gzip_buf).await?;

        // Write CRLF to end this chunk
        writer.write_all(b"\r\n").await?;
    }

    // Final chunk (0-length)
    writer.write_all(b"0\r\n\r\n").await?;
    writer.flush().await?;
    Ok(())
}
