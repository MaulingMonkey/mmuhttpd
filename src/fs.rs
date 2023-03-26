pub mod dir {
    use std::collections::HashMap;
    use std::ffi::*;
    use std::path::*;
    use std::sync::*;
    use std::time::SystemTime;



    #[derive(Default)] pub struct Cache {
        snapshots: Mutex<HashMap<PathBuf, Arc<Snapshot>>>,
    }

    impl Cache {
        pub fn new() -> Self { Default::default() }

        pub fn read_dir(&self, path: impl AsRef<Path> + Into<PathBuf>) -> Option<Arc<Snapshot>> { // XXX: this is a little awkward
            let modified = std::fs::metadata(path.as_ref()).ok()?.modified().ok()?; // XXX
            let mut snapshots = self.snapshots.lock().expect("bug: Mutex poisoned");
            if let Some(snapshot) = snapshots.get(path.as_ref()) {
                if modified == snapshot.modified {
                    return Some(Arc::clone(snapshot));
                }
            }

            // This might be long I/O while holding a lock.  This is semi-intentional:
            // • no sense having multiple threads reading the same dir.
            // • might be better to modify V={Arc<Snapshot> → Mutex<Option<Arc<Snapshot>>>} and drop the overall lock
            // • OTOH this isn't meant to be a maximum performance production httpd
            let snapshot = Arc::new(Snapshot::new(modified, path.as_ref()).ok()?); // XXX

            snapshots.insert(path.into(), Arc::clone(&snapshot));
            Some(snapshot)
        }
    }



    pub struct Snapshot {
        modified:   SystemTime,
        path:       PathBuf,
        entries:    Vec<Entry>,
        by_name:    HashMap<OsString, usize>, // indexes entries
    }

    impl Default for Snapshot {
        fn default() -> Self {
            Self {
                modified:   SystemTime::UNIX_EPOCH,
                path:       Default::default(),
                entries:    Default::default(),
                by_name:    Default::default(),
            }
        }
    }

    impl Snapshot {
        pub fn new(modified: SystemTime, path: impl Into<PathBuf>) -> std::io::Result<Self> {
            let path = path.into();
            let mut snapshot = Self { modified, path, .. Default::default() };
            for e in std::fs::read_dir(&snapshot.path)? {
                let e : Entry = e?.into();
                snapshot.by_name.insert(e.name_os().into(), snapshot.entries.len());
                snapshot.entries.push(e);
            }
            Ok(snapshot)
        }

        pub fn path(&self) -> &Path { self.path.as_path() }

        pub fn by_name<'e>(&'e self, name: &(impl AsRef<OsStr> + ?Sized)) -> Option<&'e Entry> {
            let index = *self.by_name.get(name.as_ref())?;
            debug_assert!(index < self.entries.len());
            self.entries.get(index)
        }

        pub fn entries<'e>(&'e self) -> impl Iterator<Item = &'e Entry> { self.entries.iter() }
    }



    pub struct Entry {
        name_os:    Option<OsString>, // None = name_lossy
        name_lossy: String,
        path:       PathBuf,
        flags:      EntryFlag,
    }

    impl Entry {
        pub fn name_os      (&self) -> &OsStr   { self.name_os.as_deref().unwrap_or(OsStr::new(&self.name_lossy)) }
        pub fn name_lossy   (&self) -> &str     { &self.name_lossy }
        pub fn path         (&self) -> &Path    { &self.path }
        pub fn is_dir       (&self) -> bool     { self.flags & EntryFlag::IS_DIR  != EntryFlag::NONE }
        pub fn is_file      (&self) -> bool     { self.flags & EntryFlag::IS_FILE != EntryFlag::NONE }
    }

    impl From<std::fs::DirEntry> for Entry {
        fn from(de: std::fs::DirEntry) -> Self {
            let name_os = de.file_name();
            let (name_lossy, name_os) = match name_os.into_string() {
                Ok(name_lossy)  => (name_lossy, None),
                Err(name_os)    => (name_os.to_string_lossy().into_owned(), Some(name_os)),
            };
            let mut flags = EntryFlag::NONE;
            if let Ok(file_type) = de.file_type() {
                if file_type.is_dir()  { flags |= EntryFlag::IS_DIR  }
                if file_type.is_file() { flags |= EntryFlag::IS_FILE }
            }
            Self { name_os, name_lossy, path: de.path(), flags }
        }
    }



    #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] struct EntryFlag(u8);
    impl EntryFlag {
        pub const NONE      : EntryFlag = EntryFlag(0);
        pub const IS_FILE   : EntryFlag = EntryFlag(1 << 0);
        pub const IS_DIR    : EntryFlag = EntryFlag(1 << 1);
    }
    impl core::ops::BitAnd for EntryFlag { type Output = Self; fn bitand(self, rhs: Self) -> Self::Output { Self(self.0 & rhs.0) } }
    impl core::ops::BitOr  for EntryFlag { type Output = Self; fn bitor (self, rhs: Self) -> Self::Output { Self(self.0 | rhs.0) } }
    impl core::ops::BitAndAssign for EntryFlag { fn bitand_assign(&mut self, rhs: Self) { self.0 &= rhs.0 } }
    impl core::ops::BitOrAssign  for EntryFlag { fn bitor_assign (&mut self, rhs: Self) { self.0 |= rhs.0 } }
}
