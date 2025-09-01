use std::io::{self, Result};

#[derive(Debug, Clone)]
pub struct BtreeMetadata {
    magic: [u8; 4],        // Magic bytes "BTRE" for validation
    version: u32,          // Snapshot format version
    pub root_page_id: u32, // Which page contains the root
    pub page_size: u32,    // Page size used
    pub num_pages: u32,    // Total number of pages
    created_at: u64,       // Timestamp for validation
}

impl BtreeMetadata {
    const MAGIC: [u8; 4] = [b'B', b'T', b'R', b'E'];
    const VERSION: u32 = 1;

    pub fn new(root_page_id: u32, page_size: u32, num_pages: u32) -> Self {
        BtreeMetadata {
            magic: Self::MAGIC,
            version: Self::VERSION,
            root_page_id,
            page_size,
            num_pages,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.extend_from_slice(&self.magic); // Magic bytes (4 bytes)

        buf.extend_from_slice(&self.version.to_le_bytes()); // Version (4 bytes)

        buf.extend_from_slice(&self.root_page_id.to_le_bytes()); // Root page ID (4 bytes)

        buf.extend_from_slice(&self.page_size.to_le_bytes()); // Page size (4 bytes)

        buf.extend_from_slice(&self.num_pages.to_le_bytes()); // Number of pages (4 bytes)

        buf.extend_from_slice(&self.created_at.to_le_bytes()); // Timestamp (8 bytes)

        while buf.len() < 4096 {
            buf.push(0); // Padding to page size
        }

        buf
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 28 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Metadata too small",
            ));
        }

        let magic = [data[0], data[1], data[2], data[3]];
        if magic != Self::MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic bytes",
            ));
        }

        let version = read_u32_le(data, 4);
        let root_page_id = read_u32_le(data, 8);
        let page_size = read_u32_le(data, 12);
        let num_pages = read_u32_le(data, 16);
        let created_at = read_u64_le(data, 20);

        Ok(BtreeMetadata {
            magic,
            version,
            root_page_id,
            page_size,
            num_pages,
            created_at,
        })
    }
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn read_u64_le(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}
