/// Disk-backed pager persisting fixed-size pages in a .dat file.
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;

const MAGIC: &[u8; 4] = b"MDB1";
const HEADER_SIZE: usize = 16;

pub struct Pager {
    page_size: usize,
    path: PathBuf,
    pages: Vec<Vec<u8>>,
    length: usize,
}

impl Pager {
    pub fn new<P: Into<PathBuf>>(path: P, page_size: usize) -> Self {
        let path = path.into();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut pager = Pager {
            page_size,
            path,
            pages: Vec::new(),
            length: 0,
        };
        pager.load();
        pager
    }

    /// Allocate a fresh zeroed page and return its index.
    pub fn allocate_page(&mut self) -> usize {
        self.pages.push(vec![0; self.page_size]);
        self.pages.len() - 1
    }

    /// Write data to an existing page, truncating if necessary.
    pub fn write_page(&mut self, index: usize, data: &[u8]) {
        while index >= self.pages.len() {
            self.pages.push(vec![0; self.page_size]);
        }
        let len = data.len().min(self.page_size);
        self.pages[index][..len].copy_from_slice(&data[..len]);
        self.flush();
    }

    /// Return the bytes stored at a page index.
    pub fn read_page(&self, index: usize) -> Option<&[u8]> {
        self.pages.get(index).map(|p| p.as_slice())
    }

    pub fn write_blob(&mut self, data: &[u8]) {
        self.length = data.len();
        if data.is_empty() {
            self.pages.clear();
            self.flush();
            return;
        }
        let page_count = (data.len() + self.page_size - 1) / self.page_size;
        self.pages = Vec::with_capacity(page_count);
        for i in 0..page_count {
            let start = i * self.page_size;
            let end = (start + self.page_size).min(data.len());
            let mut page = vec![0; self.page_size];
            page[..end - start].copy_from_slice(&data[start..end]);
            self.pages.push(page);
        }
        self.flush();
    }

    pub fn read_blob(&self) -> Vec<u8> {
        if self.pages.is_empty() || self.length == 0 {
            return Vec::new();
        }
        let mut buffer = Vec::with_capacity(self.pages.len() * self.page_size);
        for page in &self.pages {
            buffer.extend_from_slice(page);
        }
        buffer.truncate(self.length);
        buffer
    }

    /// Provide simple pager statistics.
    pub fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("pages".to_string(), self.pages.len());
        stats.insert("page_size".to_string(), self.page_size);
        stats
    }

    fn load(&mut self) {
        let data = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(_) => return,
        };
        if data.len() < HEADER_SIZE || &data[..4] != MAGIC {
            return;
        }
        let stored_size = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        if stored_size > 0 {
            self.page_size = stored_size;
        }
        self.length = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
        let payload = &data[HEADER_SIZE..];
        self.pages.clear();
        for chunk in payload.chunks(self.page_size) {
            let mut page = vec![0; self.page_size];
            page[..chunk.len()].copy_from_slice(chunk);
            self.pages.push(page);
        }
    }

    fn flush(&self) {
        if self.pages.is_empty() {
            let _ = fs::write(&self.path, &[] as &[u8]);
            return;
        }
        let mut buffer = Vec::with_capacity(HEADER_SIZE + self.pages.len() * self.page_size);
        buffer.extend_from_slice(MAGIC);
        buffer.extend_from_slice(&(self.page_size as u32).to_le_bytes());
        buffer.extend_from_slice(&(self.length as u64).to_le_bytes());
        for page in &self.pages {
            buffer.extend_from_slice(page);
        }
        let _ = fs::write(&self.path, buffer);
    }
}
