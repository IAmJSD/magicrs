use javascriptcore::{Context, ContextExt, Value, ValueExt, VirtualMachine};
use std::{
    collections::HashMap,
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex},
    thread::{self, sleep}, time,
};

pub struct Uploader {
    holding_vm: VirtualMachine,
    timeouts: Mutex<HashMap<f32, Arc<VirtualMachine>>>,
    intervals: Mutex<HashMap<f32, Arc<VirtualMachine>>>,
}

impl Uploader {
    // Implement the timeout function.
    fn implement_timeouts(&self) {
        let ctx = Context::with_virtual_machine(&self.holding_vm);
        ctx.set_value(name, value)
    }

    // Create a new instance of the uploader.
    pub fn new(code: &str) -> Result<Self, String> {
        // Make a new virtual machine which will hold the code we are going to run.
        let mut vm = VirtualMachine::new();

        // Create the setup function.
        //let setup_fn = 
        //Context::with_virtual_machine(&vm).set_value("setup", setup_fn);

        // Defines the structure used to pass the VM to the thread.
        struct SendSyncBypass<T> {
            val: T,
        }
        unsafe impl<T> Send for SendSyncBypass<T> {}
        unsafe impl<T> Sync for SendSyncBypass<T> {}        

        // Evaluate the code.
        let bypass = SendSyncBypass { val: vm };
        let res = thread::spawn(move || {
            // Evaluate the code.
            let bypass_move = bypass;
            let ctx = Context::with_virtual_machine(&bypass_move.val);

            // Evaluate the JS.
            ctx.evaluate(code);

            // Check for errors.
            match ctx.exception() {
                None => Ok(bypass_move),
                Some(err) => Err(format!("Code failed to evaluate: {}", err.to_string())),
            }
        });

        // Wait for the result. If it takes more than 100 millis, bail.
        let mut ticks = 0;
        loop {
            // Check if the thread is still alive.
            if res.is_finished() {
                // Return or break here since this would mean the VM is done.
                match res.join().unwrap() {
                    Ok(bypass) => {
                        vm = bypass.val;
                        break;
                    },
                    Err(err) => return Err(err),
                };
            }

            // If the tick count is 50, kill the VM and return an error.
            if ticks == 50 {
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                unsafe {
                    use std::os::unix::thread::JoinHandleExt;
                    use libc::pthread_cancel;

                    pthread_cancel(res.into_pthread_t());
                }
                #[cfg(target_os = "windows")]
                todo!("implement windows");
                return Err("timeout exceeded 100ms".to_owned());
            }
            ticks += 1;

            // Wait 2ms.
            sleep(time::Duration::from_millis(2));
        }

        // Delete the setup function.
        //vm.delete_function("setup");

        // Create the timeouts and intervals.
    }
}
