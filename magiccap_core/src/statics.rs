use std::{path::PathBuf, sync::{atomic::AtomicBool, RwLock}};
use once_cell::sync::Lazy;
use rayon::{ThreadPool, ThreadPoolBuilder};

// Defines the config folder.
pub static CONFIG_FOLDER: Lazy<PathBuf> = Lazy::new(|| {
    // Get the home directory.
    let homedir = home::home_dir().unwrap();

    // Return the config folder.
    homedir.join(".config").join("magiccap")
});

// Defines if the global kill switch was set.
pub static KILL_SWITCH: AtomicBool = AtomicBool::new(false);

// Defines the thread pool.
static THREAD_POOL: Lazy<RwLock<Option<ThreadPool>>> = Lazy::new(|| {
    RwLock::new(Some(
        ThreadPoolBuilder::new()
            .num_threads(num_cpus::get() * 2)
            .build()
            .unwrap()
    ))
});

// Gets a thread from the pool and runs the function.
pub fn run_thread<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    // Get the thread pool.
    let pool_guard = THREAD_POOL.read().unwrap();
    let pool = match pool_guard.as_ref() {
        Some(pool) => pool,
        None => {
            std::thread::spawn(f);
            return;
        },
    };

    // Spawn the function.
    pool.spawn(f);
}

// Kills the thread pool.
pub fn kill_thread_pool() {
    let mut pool_guard = THREAD_POOL.write().unwrap();
    *pool_guard = None;
}
