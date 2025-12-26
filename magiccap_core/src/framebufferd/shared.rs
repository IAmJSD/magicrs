use std::sync::{Arc, Mutex};
use tokio::net::UnixStream;
use hyper::{Body, Request, Method, body::to_bytes, client::conn};

static TOKIO_LOOP: Mutex<Option<Arc<tokio::runtime::Runtime>>> = Mutex::new(None);
static HTTP2_SENDER: Mutex<Option<conn::http2::SendRequest<Body>>> = Mutex::new(None);

fn get_tokio_runtime() -> Arc<tokio::runtime::Runtime> {
    let mut loop_guard = TOKIO_LOOP.lock().unwrap();
    if loop_guard.is_none() {
        let new_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        *loop_guard = Some(Arc::new(new_runtime));
    }
    loop_guard.as_ref().unwrap().clone()
}

// Executor for hyper to spawn connection tasks
#[derive(Clone)]
struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::spawn(fut);
    }
}

async fn get_or_create_connection() -> Option<conn::http2::SendRequest<Body>> {
    // First, try to get existing connection without holding lock across await
    let existing_sender = {
        let sender_guard = HTTP2_SENDER.lock().unwrap();
        sender_guard.clone()
    };

    // Check if we have a valid connection
    if let Some(mut sender) = existing_sender {
        if sender.ready().await.is_ok() {
            return Some(sender);
        }
    }

    // Create new connection (no lock held here)
    let stream = match UnixStream::connect("/var/run/framebufferd.sock").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error connecting to framebufferd socket: {}", e);
            return None;
        }
    };

    let (sender, conn) = match conn::http2::handshake(TokioExecutor, stream).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error performing HTTP/2 handshake: {}", e);
            return None;
        }
    };

    // Spawn connection task
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Store the new sender
    {
        let mut sender_guard = HTTP2_SENDER.lock().unwrap();
        *sender_guard = Some(sender.clone());
    }
    Some(sender)
}

pub fn do_get_or_patch_request_to_socket<'a>(path: &'a str, patch: bool, token: Option<String>) -> Option<(Vec<u8>, u16, hyper::header::HeaderMap)> {
    let tokio_runtime = get_tokio_runtime();
    let (tx, rx) = std::sync::mpsc::channel();

    // SAFETY: This is safe because the channel ensures the path will be done with before returning.
    let path = unsafe { std::mem::transmute::<&'a str, &'static str>(path) };

    tokio_runtime.spawn(async move {
        // Get or create persistent connection
        let mut sender = match get_or_create_connection().await {
            Some(s) => s,
            None => {
                let _ = tx.send(None);
                return;
            }
        };

        // Build request
        let method = if patch { Method::PATCH } else { Method::GET };
        let mut req = match Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error building request: {}", e);
                let _ = tx.send(None);
                return;
            }
        };
        if let Some(token) = token {
            req.headers_mut().insert("X-Auth-Token", hyper::header::HeaderValue::from_str(&token).unwrap());
        }

        // Send request
        let response = match sender.send_request(req).await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Error making request to framebufferd socket: {}", e);
                // Clear the cached connection on error
                let mut sender_guard = HTTP2_SENDER.lock().unwrap();
                *sender_guard = None;
                let _ = tx.send(None);
                return;
            }
        };

        let status = response.status().as_u16();
        let headers = response.headers().clone();

        // Read response body
        match to_bytes(response.into_body()).await {
            Ok(bytes) => {
                let _ = tx.send(Some((bytes.to_vec(), status, headers)));
            }
            Err(e) => {
                eprintln!("Error reading response body: {}", e);
                let _ = tx.send(None);
            }
        }
    });
    rx.recv().unwrap()
}
