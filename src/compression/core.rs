use std::io::Error;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::compression::{brotli::Brotli, gzip::Gzip, zlib::Zlib, zstd::Zstd};

pub enum Encoding {
    Gzip(Gzip),
    Zlib(Zlib),
    Zstd(Zstd),
    Deflate(Zlib),
    Brotli(Brotli),
    None(),
}

pub fn parse_encoding(s: &str) -> Encoding {
    if s.eq("gzip") {
        Encoding::Gzip(Gzip)
    } else if s.eq("zlib") {
        Encoding::Zlib(Zlib)
    } else if s.eq("zstd") {
        Encoding::Zstd(Zstd)
    } else if s.eq("deflate") {
        Encoding::Deflate(Zlib)
    } else if s.eq("br") {
        Encoding::Brotli(Brotli)
    } else {
        Encoding::None()
    }
}

impl Encoding {
    async fn encode(
        &self,
        read_buf: [u8; 32768],
        write_buf: &mut Vec<u8>,
        n: usize,
    ) -> io::Result<()> {
        match self {
            Encoding::Gzip(g) => g.encode(read_buf, write_buf, n).await,
            Encoding::Zlib(l) => l.encode(read_buf, write_buf, n).await,
            Encoding::Zstd(z) => z.encode(read_buf, write_buf, n).await,
            Encoding::Deflate(d) => d.encode(read_buf, write_buf, n).await,
            Encoding::Brotli(b) => b.encode(read_buf, write_buf, n).await,
            Encoding::None() => Err(Error::other("Can't encode")),
        }
    }
    pub fn string_value(&self) -> &str {
        match self {
            Encoding::Gzip(_) => "gzip",
            Encoding::Zlib(_) => "zlib",
            Encoding::Zstd(_) => "zstd",
            Encoding::Deflate(_) => "deflate",
            Encoding::Brotli(_) => "br",
            Encoding::None() => "",
        }
    }
}

pub async fn compress_stream<R, W>(
    mut reader: R,
    mut writer: W,
    encoding: Encoding,
) -> io::Result<()>
where
    R: io::AsyncRead + Unpin,
    W: io::AsyncWrite + Unpin,
{
    let mut write_buf = Vec::with_capacity(32 * 1024);
    let mut read_buf: [u8; 32768] = [0u8; 32 * 1024];

    loop {
        let n = reader.read(&mut read_buf).await?;
        if n == 0 {
            break;
        }

        // Compress this chunk into write_buf
        write_buf.clear();
        encoding.encode(read_buf, &mut write_buf, n).await?;
        // Write chunk length (in hex)
        writer
            .write_all(format!("{:X}\r\n", write_buf.len()).as_bytes())
            .await?;

        // Write compressed chunk
        writer.write_all(&write_buf).await?;

        // Write CRLF to end this chunk
        writer.write_all(b"\r\n").await?;
    }

    // Final chunk (0-length)
    writer.write_all(b"0\r\n\r\n").await?;
    writer.flush().await?;
    Ok(())
}
