//! Log gym occupancy history to a JSON file on a persistant volume

use {
    bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable},
    memmap2::MmapMut,
    std::{fs, mem::size_of, path::Path},
};

const HEADER_SIZE: usize = size_of::<Header>();
const RAW_ENTRY_SIZE: usize = size_of::<RawEntry>();

const NUM_ENTRIES: u64 = 7 * 24 * 60;

/// Total file size is the size of the header and 7 days worth of entries at one entry per minute.
const FILE_SIZE: u64 = HEADER_SIZE as u64 + NUM_ENTRIES * RAW_ENTRY_SIZE as u64;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Header {
    write_pos: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    value: u8,
    timestamp: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
struct RawEntry {
    value: u8,
    _pad0: u8,
    _pad1: u16,
    _pad2: u32,
    timestamp: u64,
}

pub struct PersistentHistory {
    mmap: MmapMut,
}

impl PersistentHistory {
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        let file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();

        file.set_len(FILE_SIZE).unwrap();

        let mmap = unsafe { MmapMut::map_mut(&file) }.unwrap();

        let celf = Self { mmap };

        dbg!(celf.get());

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

    pub fn append(&mut self, timestamp: u64, value: u8) {
        let pos = from_bytes::<Header>(&self.mmap[..HEADER_SIZE]).write_pos as usize;
        let start = HEADER_SIZE + pos * RAW_ENTRY_SIZE;

        let raw_entry = from_bytes_mut::<RawEntry>(&mut self.mmap[start..start + RAW_ENTRY_SIZE]);

        raw_entry.timestamp = timestamp;
        raw_entry.value = value;

        let header = from_bytes_mut::<Header>(&mut self.mmap[..HEADER_SIZE]);
        header.write_pos = (header.write_pos + 1) % NUM_ENTRIES;
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

        for i in 1..=500_000 {
            history.append(i, 0xAA);
        }

        let expected = ((500_000 - super::NUM_ENTRIES + 1)..=500_000)
            .into_iter()
            .map(|i| Entry {
                timestamp: i,
                value: 0xAA,
            })
            .collect::<Vec<_>>();

        assert_eq!(history.get().len(), super::NUM_ENTRIES as usize);
        assert_eq!(history.get(), expected);
    }
}
