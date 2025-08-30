use super::Item;
use std::{
    fs::File,
    io::{Read, Result, Seek, Write},
    vec,
};
pub type PageID = u32;

#[derive(Debug)]
pub enum Page {
    Internal {
        id: PageID,
        keys: Vec<i32>,
        children: Vec<PageID>,
    },
    Leaf {
        id: PageID,
        items: Vec<Item>,
    },
}

#[derive(Debug)]
pub struct Pager {
    pub file: File,
    pub page_size: usize,
    pub num_pages: PageID,
}

impl Pager {
    pub fn allocate_page(&mut self) -> std::io::Result<PageID> {
        let new_id = self.num_pages;

        if self.page_size == 0 || self.page_size > 1024 * 1024 {
            // Max 1MB
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid page size",
            ));
        }
        self.num_pages += 1;

        self.file.seek(std::io::SeekFrom::End(0))?;
        let write_result = self.file.write_all(&vec![0u8; self.page_size]);

        if write_result.is_err() {
            self.num_pages -= 1;
            write_result?
        }

        Ok(new_id)
    }
}

impl Pager {
    pub fn write_page(&mut self, page: &Page) -> Result<()> {
        let mut buf = vec![0u8; self.page_size];
        let page_id: u32;
        match page {
            Page::Internal { id, keys, children } => {
                page_id = *id;

                buf[0] = 1; // 1-> internal type
                let key_count = keys.len() as u32;
                buf[1..5].copy_from_slice(&key_count.to_le_bytes()); // next 4 bytes-> keycount in little endian format

                let mut offset = 5;
                for k in keys {
                    buf[offset..offset + 4].copy_from_slice(&k.to_le_bytes());
                    offset += 4;
                }

                for c in children {
                    buf[offset..offset + 4].copy_from_slice(&c.to_le_bytes());
                    offset += 4;
                }
            }

            Page::Leaf { id, items } => {
                page_id = *id;

                buf[0] = 0; // 0-> Leaf type
                let items_count = items.len() as u32;
                buf[1..5].copy_from_slice(&items_count.to_le_bytes());

                let mut offset = 5;
                for item in items {
                    buf[offset..offset + 4].copy_from_slice(&item.key.to_le_bytes());
                    offset += 4;

                    let val_bytes = item.val.as_bytes();
                    let len = val_bytes.len() as u32;
                    buf[offset..offset + 4].copy_from_slice(&len.to_le_bytes());
                    offset += 4;
                    buf[offset..offset + len as usize].copy_from_slice(val_bytes);
                    offset += len as usize;
                }
            }
        }
        self.file.seek(std::io::SeekFrom::Start(
            (page_id as u64) * self.page_size as u64,
        ))?;
        self.file.write_all(&buf)?;
        Ok(())
    }
}

impl Pager {
    pub fn read_page(&mut self, page_id: PageID) -> Result<Page> {
        let mut buf = vec![0u8; self.page_size];
        self.file.seek(std::io::SeekFrom::Start(
            (page_id as u64) * (self.page_size as u64),
        ))?;
        self.file.read_exact(&mut buf)?;

        let page_type = buf[0];
        if page_type == 1 {
            let keycount = u32::from_le_bytes(buf[1..5].try_into().unwrap());

            if keycount > ((self.page_size - 5) / 8) as u32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Invalid key count: {} (max allowed: {})",
                        keycount,
                        (self.page_size - 5) / 8
                    ),
                ));
            }

            let mut offset = 5;

            let mut keys = Vec::with_capacity(keycount as usize);
            for _ in 0..keycount {
                if offset + 4 > self.page_size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Buffer overflow while reading keys",
                    ));
                }

                keys.push(i32::from_le_bytes(
                    buf[offset..offset + 4].try_into().unwrap(),
                ));
                offset += 4;
            }

            let mut children = Vec::with_capacity((keycount + 1) as usize);
            for _ in 0..=keycount {
                if offset + 4 > self.page_size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Buffer overflow while reading children",
                    ));
                }

                children.push(u32::from_le_bytes(
                    buf[offset..offset + 4].try_into().unwrap(),
                ));
                offset += 4;
            }

            Ok(Page::Internal {
                id: page_id,
                keys,
                children,
            })
        } else if page_type == 0 {
            let items_count = u32::from_le_bytes(buf[1..5].try_into().unwrap());

            if items_count > ((self.page_size - 5) / 20) as u32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Invalid items count: {} (max allowed: {})",
                        items_count,
                        (self.page_size - 5) / 20
                    ),
                ));
            }

            let mut items = Vec::with_capacity(items_count as usize);
            let mut offset = 5;

            for _ in 0..items_count {
                if offset + 4 > self.page_size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Buffer overflow while reading item key",
                    ));
                }

                let key = i32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap());
                offset += 4;

                if offset + 4 > self.page_size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Buffer overflow while reading value length",
                    ));
                }

                let val_len =
                    u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;

                if val_len > self.page_size || offset + val_len > self.page_size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Invalid value length: {} (buffer size: {})",
                            val_len, self.page_size
                        ),
                    ));
                }

                let val = String::from_utf8_lossy(&buf[offset..offset + val_len]).to_string();
                offset += val_len;

                items.push(Item { key, val });
            }

            Ok(Page::Leaf { id: page_id, items })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown page type: {page_type}"),
            ))
        }
    }
}
