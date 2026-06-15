// In-memory LevelDB stubs for browser WASM SDK (signing only; no disk I/O).

pub struct RawBytes(Vec<u8>);

impl std::ops::Deref for RawBytes {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<RawBytes> for Vec<u8> {
    fn from(bytes: RawBytes) -> Self {
        bytes.0
    }
}

pub struct Writebatch {
    ops: Vec<(Vec<u8>, Option<Vec<u8>>)>,
}

impl Writebatch {
    pub fn new() -> Writebatch {
        Writebatch { ops: Vec::new() }
    }

    pub fn put(&mut self, k: &[u8], value: &[u8]) {
        self.ops.push((k.to_vec(), Some(value.to_vec())));
    }

    pub fn delete(&mut self, k: &[u8]) {
        self.ops.push((k.to_vec(), None));
    }
}

pub struct LevelDB {
    data: std::sync::Mutex<std::collections::HashMap<Vec<u8>, Vec<u8>>>,
}

impl LevelDB {
    pub fn open(_dir: &std::path::Path) -> LevelDB {
        LevelDB {
            data: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn get_at(&self, k: &[u8]) -> Option<RawBytes> {
        let guard = self.data.lock().ok()?;
        guard.get(k).map(|v| RawBytes(v.clone()))
    }

    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        self.get_at(k).map(|v| v.0)
    }

    pub fn put(&self, k: &[u8], value: &[u8]) {
        if let Ok(mut guard) = self.data.lock() {
            guard.insert(k.to_vec(), value.to_vec());
        }
    }

    pub fn rm(&self, k: &[u8]) {
        if let Ok(mut guard) = self.data.lock() {
            guard.remove(k);
        }
    }

    pub fn write(&self, batch: &Writebatch) {
        if let Ok(mut guard) = self.data.lock() {
            for (k, v) in &batch.ops {
                match v {
                    Some(val) => {
                        guard.insert(k.clone(), val.clone());
                    }
                    None => {
                        guard.remove(k);
                    }
                }
            }
        }
    }
}