use std::{fs, os::unix::net::{UnixListener, UnixStream}, thread};

// Acquires the application lock. If another instance has the lock, we call the closure on the
// other instance and status code 0 exit the current instance.
pub fn acquire_application_lock<F: Fn() + Send + Sync + 'static>(closure: F) {
    // Get the path to ~/.config/magiccap/instance.sock.
    let homedir = home::home_dir().unwrap();
    let lock_path = homedir.join(".config").join("magiccap").join("instance.sock");

    // Attempt to connect to the Unix socket.
    if let Ok(_) = UnixStream::connect(&lock_path) {
        // We are already running. Exit with status code 0.
        std::process::exit(0);
    } else {
        // Make sure the file is deleted.
        let _ = fs::remove_file(&lock_path);
    }

    // Start a Unix listener.
    let ln = UnixListener::bind(lock_path).unwrap();
    thread::spawn(move || {
        for stream in ln.incoming() {
            if let Ok(_) = stream {
                // Call the closure since another instance was ran.
                closure();
            }
        }
    });
}
