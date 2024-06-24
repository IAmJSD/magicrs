mod upload;

use javascriptcore::{Context, ContextExt, Exception, Value, ValueExt, VirtualMachine};
use glib::{gobject_ffi::G_TYPE_NONE, translate::{FromGlibPtrFull, ToGlibPtr}};
use javascriptcore_rs_sys::JSCValue;
use std::{
    collections::HashMap, ffi::CString, os::raw::c_void, sync::{Arc, Mutex},
    thread::{self, sleep}, time
};

// Defines the uploader structure. This can be used to get information about the uploader and to call it.
pub struct Uploader {
    pub name: String,
    pub description: String,
    method: Value,

    cached_ctx: crate::upload::CacheContext,
    holding_vm: VirtualMachine,
}

// Defines the setup data structure.
struct SetupData {
    name: String,
    description: String,
    method: Value,
}

// Create 2 references to an item. One of which can be redeemed later for a reference.
// This is _HIGHLY UNSAFE_. The function that dies first must be the redeemer and they must not race.
unsafe fn tworef<T>(val: T) -> (Box<T>, *mut c_void) {
    let b = Box::new(val);
    let ptr = Box::into_raw(b) as usize;

    (Box::from_raw(ptr as *mut T), ptr as *mut c_void)
}

// Defines the callback from the setup function in the global scope.
#[no_mangle]
extern "C" fn setup_cb(obj: *mut JSCValue, setup_box: *mut c_void) {
    // Setup the variables properly so they can be dropped as expected.
    let obj = unsafe { Value::from_glib_full(obj) };
    let setup_box = unsafe {
        (setup_box as *mut Mutex<Option<SetupData>>).as_ref()
    }.unwrap();

    // Check if this is a object.
    let ctx = Context::current().unwrap();
    if !obj.is_object() {
        ctx.throw_exception(&Exception::new(&ctx, "setup function must have a object input"));
        return;
    }

    // Get the name from the object.
    let name = match obj.object_get_property("name") {
        Some(v) => v,
        None => {
            ctx.throw_exception(&Exception::new(&ctx, "setup function must have a name property"));
            return;
        },
    };
    if !name.is_string() {
        ctx.throw_exception(&Exception::new(&ctx, "name property must be a string"));
        return;
    }
    let name = name.to_string();

    // Get the description from the object.
    let description = match obj.object_get_property("description") {
        Some(v) => v,
        None => {
            ctx.throw_exception(&Exception::new(&ctx, "setup function must have a description property"));
            return;
        },
    };
    if !description.is_string() {
        ctx.throw_exception(&Exception::new(&ctx, "description property must be a string"));
        return;
    }
    let description = description.to_string();

    // Get the method from the object.
    let method = match obj.object_get_property("method") {
        Some(v) => v,
        None => {
            ctx.throw_exception(&Exception::new(&ctx, "setup function must have a method property"));
            return;
        },
    };
    if !method.is_function() {
        ctx.throw_exception(&Exception::new(&ctx, "method property must be a function"));
        return;
    }

    // Set the setup data.
    let mut setup_box = setup_box.lock().unwrap();
    *setup_box = Some(SetupData {
        name,
        description,
        method,
    });
}

impl Uploader {
    // Create a new instance of the uploader.
    pub fn new(code: String) -> Result<Self, String> {
        // Make a new virtual machine which will hold the code we are going to run.
        let vm = VirtualMachine::new();

        // Create the setup function.
        let mut ctx = Context::with_virtual_machine(&vm);
        let setup_cstr = CString::new("setup").unwrap();
        let setup_box: Mutex<Option<SetupData>> = Default::default();
        let (setup_box, setup_ptr) = unsafe {
            tworef(setup_box)
        };
        let setup_fn = unsafe {
            javascriptcore_rs_sys::jsc_value_new_function(
                ctx.to_glib_none().0,
                setup_cstr.as_ptr(),
                Some(
                    std::mem::transmute(setup_cb as extern "C" fn(*mut JSCValue, *mut c_void))
                ),
                setup_ptr,
                None,
                G_TYPE_NONE,
                1,
            )
        };
        ctx.set_value("setup", &unsafe { Value::from_glib_full(setup_fn) });

        // Defines the structure used to pass the VM to the thread.
        struct SendSyncBypass<T> {
            val: T,
        }
        unsafe impl<T> Send for SendSyncBypass<T> {}
        unsafe impl<T> Sync for SendSyncBypass<T> {}        

        // Evaluate the code.
        let bypass = SendSyncBypass { val: ctx };
        let res = thread::spawn(move || {
            // Evaluate the code.
            let bypass_move = bypass;
            let ctx = bypass_move.val;
            ctx.evaluate(&code);

            // Check for errors.
            match ctx.exception() {
                None => Ok(SendSyncBypass { val: ctx }),
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
                        ctx = bypass.val;
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

        // Check the setup data was set.
        let setup_data = match setup_box.lock().unwrap().take() {
            Some(v) => v,
            None => return Err("Setup function was not called".to_string()),
        };

        // Delete the setup function.
        ctx.evaluate("delete this.setup").unwrap();

        // Drop the C string now since it has been used.
        drop(setup_cstr);

        // Build the struct and finish the rest of the building.
        let mut res = Self {
            name: setup_data.name,
            description: setup_data.description,
            method: setup_data.method,
            cached_ctx: Default::default(),
            timeouts: Default::default(),
            intervals: Default::default(),
            holding_vm: vm,
        };
        res.implement_web_standards();
        Ok(res)
    }

    // Handle uploading a file.
    pub fn upload(&self, filename: String, data: Vec<u8>) -> Result<String, String> {
        // Make a clone of the VM and the context.
        let vm = self.holding_vm.clone();
        let ctx = Context::with_virtual_machine(&vm);

        // Handle doing the upload.
        crate::upload::upload(ctx, &self.method, &self.cached_ctx, filename, data)
    }
}
