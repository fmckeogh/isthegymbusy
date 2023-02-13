//! Log gym occupancy history to a JSON file on a persistant volume

use {
    bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable},
    memmap2::MmapMut,
    std::{cmp::Ordering, fs, mem::size_of, path::Path},
    tracing::{info, warn},
};

const HEADER_SIZE: usize = size_of::<Header>();
const RAW_ENTRY_SIZE: usize = size_of::<RawEntry>();

/// 1 year worth of entries at 1 per minute
const NUM_ENTRIES: u64 = 365 * 24 * 60;

/// Total file size is the size of the header
const FILE_SIZE: u64 = HEADER_SIZE as u64 + NUM_ENTRIES * RAW_ENTRY_SIZE as u64;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Header {
    write_pos: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    pub value: u8,
    pub timestamp: i64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
struct RawEntry {
    value: u8,
    _pad0: u8,
    _pad1: u16,
    _pad2: u32,
    timestamp: i64,
}

pub struct PersistentHistory {
    mmap: MmapMut,
}

impl PersistentHistory {
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        info!("opening history file");

        let file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();

        let current_len = file.metadata().unwrap().len();
        match current_len.cmp(&FILE_SIZE) {
            Ordering::Less => {
                warn!("history file size ({current_len} bytes) less than FILE_SIZE ({FILE_SIZE} bytes), expanding...");
                file.set_len(FILE_SIZE).unwrap();
            }
            Ordering::Equal => (),
            Ordering::Greater => {
                panic!("history file size ({current_len} bytes) greater than FILE_SIZE ({FILE_SIZE} bytes)");
            }
        }

        let mmap = unsafe { MmapMut::map_mut(&file) }.unwrap();

        let celf = Self { mmap };

        info!(
            "file size {current_len} bytes, {}/{NUM_ENTRIES} entries occupied",
            celf.get().len()
        );

        celf
    }

    pub fn get(&self) -> Vec<Entry> {
        let mut entries = self.mmap[HEADER_SIZE..]
            .chunks_exact(RAW_ENTRY_SIZE)
            .map(|chunk| *from_bytes::<RawEntry>(chunk))
            .filter(|entry| entry.timestamp != 0)
            .map(
                |RawEntry {
                     value, timestamp, ..
                 }| Entry { value, timestamp },
            )
            .collect::<Vec<_>>();

        entries.sort_by_key(|&Entry { timestamp, .. }| timestamp);

        entries
    }

    pub fn append(&mut self, timestamp: i64, value: u8) {
        let pos = from_bytes::<Header>(&self.mmap[..HEADER_SIZE]).write_pos as usize;
        let start = HEADER_SIZE + pos * RAW_ENTRY_SIZE;

        // write next timestamp and value, then flush
        {
            let raw_entry =
                from_bytes_mut::<RawEntry>(&mut self.mmap[start..start + RAW_ENTRY_SIZE]);

            raw_entry.timestamp = timestamp;
            raw_entry.value = value;

            #[cfg(not(test))]
            self.mmap.flush_range(start, RAW_ENTRY_SIZE).unwrap();
        }

        // only after successful write increment write_pos, a crash at any time cannot result in invalid data being written only loss of 1 entry
        {
            let header = from_bytes_mut::<Header>(&mut self.mmap[..HEADER_SIZE]);
            header.write_pos = (header.write_pos + 1) % NUM_ENTRIES;

            #[cfg(not(test))]
            self.mmap.flush_range(0, HEADER_SIZE).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::history::{Entry, PersistentHistory},
        std::path::PathBuf,
        tempdir::TempDir,
    };

    fn get_test_path() -> (TempDir, PathBuf) {
        let dir = TempDir::new("history").unwrap();

        let mut path = dbg!(dir.path().to_owned());
        path.push("history");

        (dir, path)
    }

    #[test]
    fn single() {
        let (_dir, path) = get_test_path();
        let mut history = PersistentHistory::open(path);

        history.append(0xFFFF, 0xAB);

        assert_eq!(
            history.get(),
            vec![Entry {
                timestamp: 0xFFFF,
                value: 0xAB
            }]
        )
    }

    #[test]
    fn many() {
        let (_dir, path) = get_test_path();
        let mut history = PersistentHistory::open(path);
        let mut expected = vec![];

        for i in 1..=1000 {
            history.append(i, 0xAA);
            expected.push(Entry {
                timestamp: i,
                value: 0xAA,
            })
        }

        assert_eq!(history.get(), expected)
    }

    #[test]
    fn loop_around() {
        let (_dir, path) = get_test_path();
        let mut history = PersistentHistory::open(path);

        for i in 1..=1_000_000 {
            history.append(i, 0xAA);
        }

        let expected = ((1_000_000 - super::NUM_ENTRIES as i64 + 1)..=1_000_000)
            .into_iter()
            .map(|i| Entry {
                timestamp: i,
                value: 0xAA,
            })
            .collect::<Vec<_>>();

        assert_eq!(history.get(), expected);
    }

    #[test]
    fn reopen() {
        let expected = ((1_000_000 - super::NUM_ENTRIES as i64 + 1)..=1_000_000)
            .into_iter()
            .map(|i| Entry {
                timestamp: i,
                value: rand::random(),
            })
            .collect::<Vec<_>>();

        let (_dir, path) = get_test_path();

        {
            let mut history = PersistentHistory::open(&path);

            // add random data that will be overwritten
            for _ in 1..=100_000 {
                history.append(rand::random::<i64>() + 1, rand::random());
            }

            for entry in &expected {
                history.append(entry.timestamp, entry.value)
            }

            assert_eq!(history.get(), expected);
        }

        {
            let history = PersistentHistory::open(&path);
            assert_eq!(history.get(), expected);
        }
    }
}
