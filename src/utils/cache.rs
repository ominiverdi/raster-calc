// src/utils/cache.rs
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use gdal::Dataset;
use anyhow::Result;

/// Thread-safe cache for GDAL datasets
pub struct RasterCache {
    datasets: Arc<Mutex<HashMap<PathBuf, Arc<Mutex<Dataset>>>>>,
}

impl RasterCache {
    pub fn new() -> Self {
        Self {
            datasets: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn get_dataset<P: AsRef<Path>>(&self, path: P) -> Result<Arc<Mutex<Dataset>>> {
        let path_buf = path.as_ref().to_path_buf();
        
        let mut cache = self.datasets.lock().unwrap();
        
        if let Some(dataset) = cache.get(&path_buf) {
            return Ok(Arc::clone(dataset));
        }
        
        // Not in cache, open and add it
        let dataset = Arc::new(Mutex::new(Dataset::open(path.as_ref())?));
        cache.insert(path_buf, Arc::clone(&dataset));
        
        Ok(dataset)
    }
    
    pub fn clear(&self) {
        let mut cache = self.datasets.lock().unwrap();
        cache.clear();
    }
    
    pub fn len(&self) -> usize {
        let cache = self.datasets.lock().unwrap();
        cache.len()
    }
}