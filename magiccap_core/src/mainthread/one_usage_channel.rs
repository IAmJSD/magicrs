use std::{marker::PhantomData, sync::{Mutex, MutexGuard}};

// Defines a channel that can only be used once. This handles the intracacies of
// sharing a channel across threads without the expense of a full channeling system.
struct OneUsageChannel<T>
where
    T: Send,
{
    mu: Mutex<Option<T>>,
    lock_guard: Option<usize>,
}

// Defines a sender struct for the channel.
pub struct Sender<'a, T>
where
    T: Send,
{
    channel: &'a mut OneUsageChannel<T>,
}

// Defines the send function.
impl<'a, T> Sender<'a, T>
where
    T: Send,
{
    pub fn send(&mut self, value: T) {
        self.channel.send(value);
    }
}

// Defines a receiver struct for the channel.
pub struct Receiver<'a, T>
where
    T: Send,
{
    channel: OneUsageChannel<T>,
    lifetime: PhantomData<&'a ()>,
}

// Defines the receive function.
impl<'a, T> Receiver<'a, T>
where
    T: Send,
{
    pub fn wait(&self) -> T {
        self.channel.wait()
    }
}

// Creates a new one usage channel. Returns a tuple with the sender and receiver. Note
// the sender _CAN_ be unsafe, but only if it is completely misused and flooded with events.
// Both sides _MUST_ be used once and only once.
pub fn channel<'a, T>() -> (Sender<'a, T>, Receiver<'a, T>)
where
    T: Send,
{
    // Create the channel as the owner of the mutex.
    let mut channel: OneUsageChannel<T> = OneUsageChannel {
        mu: Mutex::new(None),
        lock_guard: None,
    };

    // Create the lock guard and box it and get the pointer. This is VERY unsafe, but is okay
    // in such a controlled environment.
    let lock_guard = channel.mu.lock().unwrap();
    let lock_ptr = Box::into_raw(Box::new(lock_guard)) as usize;
    channel.lock_guard = Some(lock_ptr);

    // Do some unsafe stuff to break Rust rules and get a mutable reference to the channel.
    let mut_ref = &mut channel;
    let mut_ref_cpy = unsafe {
        std::mem::transmute::<&mut OneUsageChannel<T>, &'a mut OneUsageChannel<T>>(mut_ref)
    };

    // Return the sender and receiver.
    (
        Sender { channel: mut_ref_cpy },
        Receiver { channel, lifetime: PhantomData },
    )
}

// Defines the send function.
impl<T> OneUsageChannel<T>
where
    T: Send,
{
    // Waits for a value within the channel. This can only be called once.
    fn wait(&self) -> T {
        // Get the lock guard.
        let mut lock_guard = self.mu.lock().unwrap();

        // Get the value.
        match lock_guard.take() {
            Some(value) => value,
            None => panic!("The channel has already been used."),
        }
    }

    // Sends a value to the channel. This can only be called once.
    fn send(&mut self, value: T) {
        // Get the lock guard pointer.
        let ptr = match self.lock_guard.take() {
            Some(ptr) => ptr,
            None => panic!("The channel has already been used."),
        };

        // Turn the lock guard back into a box.
        let mut lock_guard = unsafe {
            Box::from_raw(ptr as *mut MutexGuard<Option<T>>)
        };

        // Set the value inside the lock guard.
        lock_guard.replace(value);
    }
}

// Allow this channel to be sent across threads.
unsafe impl<T> Send for OneUsageChannel<T> where T: Send {}

// Don't worry about thread safety for this channel.
unsafe impl<T> Sync for OneUsageChannel<T> where T: Send {}

// Handle the drop of the channel.
impl<T> Drop for OneUsageChannel<T>
where
    T: Send,
{
    fn drop(&mut self) {
        // Get the lock guard pointer.
        let ptr = match self.lock_guard.take() {
            Some(ptr) => ptr,
            None => return,
        };

        // Turn the lock guard back into a box.
        let lock_guard = unsafe {
            Box::from_raw(ptr as *mut MutexGuard<Option<T>>)
        };

        // Drop the lock guard.
        drop(lock_guard);
    }
}
