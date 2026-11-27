use anyhow::{Context, Result, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::tls::Stream;

const MAGIC: &[u8; 8] = b"CFA6LPAA";
const HEADER_LEN: usize = 40;
const CHUNK_LEN: usize = 64 * 1024;

const OP_STATUS: u64 = 0x01;
const OP_MAKE_DIR: u64 = 0x09;
const OP_FILE_OPEN: u64 = 0x0D;
const OP_FILE_OPEN_RESULT: u64 = 0x0E;
const OP_FILE_WRITE: u64 = 0x10;
const OP_FILE_CLOSE: u64 = 0x14;

const MODE_WRITE: u64 = 3;
const STATUS_OK: u64 = 0;
const STATUS_EXISTS: u64 = 16;

#[derive(Debug)]
struct Header {
    entire_length: u64,
    this_length: u64,
    packet_num: u64,
    operation: u64,
}

impl Header {
    fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut bytes = [0u8; HEADER_LEN];

        bytes[0..8].copy_from_slice(MAGIC);
        bytes[8..16].copy_from_slice(&self.entire_length.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.this_length.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.packet_num.to_le_bytes());
        bytes[32..40].copy_from_slice(&self.operation.to_le_bytes());

        bytes
    }

    fn from_bytes(bytes: &[u8; HEADER_LEN]) -> Result<Self> {
        if &bytes[0..8] != MAGIC {
            bail!("afc: bad magic in response");
        }

        Ok(Self {
            entire_length: read_u64(&bytes[8..16]),
            this_length: read_u64(&bytes[16..24]),
            packet_num: read_u64(&bytes[24..32]),
            operation: read_u64(&bytes[32..40]),
        })
    }
}

fn read_u64(bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[0..8]);
    u64::from_le_bytes(buf)
}

pub struct Afc {
    stream: Box<dyn Stream>,
    packet_num: u64,
}

impl Afc {
    pub fn new(stream: Box<dyn Stream>) -> Self {
        Self {
            stream,
            packet_num: 0,
        }
    }

    async fn send(&mut self, operation: u64, args: &[u8], data: &[u8]) -> Result<()> {
        let this_length = (HEADER_LEN + args.len()) as u64;
        let header = Header {
            entire_length: this_length + data.len() as u64,
            this_length,
            packet_num: self.packet_num,
            operation,
        };
        self.packet_num += 1;

        self.stream.write_all(&header.to_bytes()).await?;
        self.stream.write_all(args).await?;
        self.stream.write_all(data).await?;
        self.stream.flush().await?;

        Ok(())
    }

    async fn recv(&mut self) -> Result<(u64, Vec<u8>)> {
        let mut bytes = [0u8; HEADER_LEN];
        self.stream.read_exact(&mut bytes).await?;
        let header = Header::from_bytes(&bytes)?;

        let len = header
            .entire_length
            .checked_sub(HEADER_LEN as u64)
            .context("afc: bad length in response")?;

        let mut payload = vec![0u8; len as usize];
        self.stream.read_exact(&mut payload).await?;

        Ok((header.operation, payload))
    }

    async fn status(&mut self) -> Result<u64> {
        let (operation, payload) = self.recv().await?;

        if operation != OP_STATUS || payload.len() < 8 {
            bail!("afc: unexpected response, operation {operation}");
        }

        Ok(read_u64(&payload))
    }

    async fn expect_ok(&mut self, action: &str) -> Result<()> {
        let code = self.status().await?;

        if code != STATUS_OK {
            bail!("afc: {action} failed with status {code}");
        }

        Ok(())
    }

    pub async fn make_dir(&mut self, path: &str) -> Result<()> {
        let mut args = path.as_bytes().to_vec();
        args.push(0);

        self.send(OP_MAKE_DIR, &args, &[]).await?;
        let code = self.status().await?;

        if code != STATUS_OK && code != STATUS_EXISTS {
            bail!("afc: make dir {path} failed with status {code}");
        }

        Ok(())
    }

    async fn open(&mut self, path: &str) -> Result<u64> {
        let mut args = MODE_WRITE.to_le_bytes().to_vec();
        args.extend_from_slice(path.as_bytes());
        args.push(0);

        self.send(OP_FILE_OPEN, &args, &[]).await?;
        let (operation, payload) = self.recv().await?;

        if operation == OP_STATUS && payload.len() >= 8 {
            bail!("afc: open {path} failed with status {}", read_u64(&payload));
        }
        if operation != OP_FILE_OPEN_RESULT || payload.len() < 8 {
            bail!("afc: unexpected response, operation {operation}");
        }

        Ok(read_u64(&payload))
    }

    pub async fn upload(&mut self, local: &str, remote: &str) -> Result<()> {
        let mut file = tokio::fs::File::open(local)
            .await
            .with_context(|| format!("cannot open {local}"))?;
        let total = file.metadata().await?.len();

        let handle = self.open(remote).await?;
        let args = handle.to_le_bytes();

        let mut chunk = vec![0u8; CHUNK_LEN];
        let mut sent: u64 = 0;
        let mut last_percent = u64::MAX;

        loop {
            let n = file.read(&mut chunk).await?;
            if n == 0 {
                break;
            }

            self.send(OP_FILE_WRITE, &args, &chunk[..n]).await?;
            self.expect_ok("write").await?;

            sent += n as u64;
            let percent = if total == 0 {
                100
            } else {
                sent.checked_mul(100)
                    .and_then(|v| v.checked_div(total))
                    .unwrap_or(0)
            };
            if percent != last_percent {
                eprint!("\ruploading {percent}%");
                last_percent = percent;
            }
        }
        eprintln!();

        self.send(OP_FILE_CLOSE, &args, &[]).await?;
        self.expect_ok("close").await?;

        Ok(())
    }
}
