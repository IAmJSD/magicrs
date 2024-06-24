use std::sync::Mutex;
use javascriptcore::{Context, Value};

// Defines the cached context.
#[derive(Default)]
pub struct CacheContext {
    timeouts: Mutex<HashMap<f32, Arc<VirtualMachine>>>,
    intervals: Mutex<HashMap<f32, Arc<VirtualMachine>>>,    
    method_caching: Mutex<Vec<Value>>,
}

// Defines the function used to upload content.
pub fn upload(
    ctx: Context, method: &Value, cached_ctx: &CacheContext, filename: String,
    data: Vec<u8>,
) -> Result<String, String> {
    
}
