use crate::*;
use std::task::{Context, Poll, Waker};
use std::{future::Future, sync::Arc};
use std::{pin::Pin, sync::Mutex};

thread_local! {
    static LIBRARY: KWasmLibrary = KWasmLibrary::new(include_str!("fetch.js"));
}

pub async fn fetch(path: &str) -> Result<Vec<u8>, ()> {
    FetchFuture {
        inner: Arc::new(Mutex::new(Inner {
            _path: path.to_string(),
            running: false,
            result: None,
            waker: None,
        })),
    }
    .await
}

struct Inner {
    _path: String,
    running: bool,
    result: Option<Vec<u8>>,
    waker: Option<Waker>,
}

struct FetchFuture {
    // This needs to be shared with a closure passed to the host
    // that fils in the result and drops the closure later.
    inner: Arc<Mutex<Inner>>,
}

impl<'a> Future for FetchFuture {
    type Output = Result<Vec<u8>, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.inner.lock().unwrap();

        // Begin the task.
        if !inner.running {
            let raw_ptr = Arc::into_raw(self.inner.clone());
            inner.running = true;

            let mut message = [
                inner._path.as_ptr() as u32,
                inner._path.len() as u32,
                raw_ptr as u32,
            ];

            log(&format!("RAW PTR: {:?}", raw_ptr as u32));

            LIBRARY.with(|l| l.message_with_slice(0, &mut message));
        }

        if let Some(v) = inner.result.take() {
            Poll::Ready(Ok(v))
        } else {
            inner.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Called by the host to reserve scratch space to pass data into kwasm.
/// returns a pointer to the allocated data.
#[no_mangle]
extern "C" fn kwasm_complete_fetch(inner_data: u32) {
    log(&format!("INNER DATA: {:?}", inner_data));

    unsafe {
        let arc = Arc::<Mutex<Inner>>::from_raw(inner_data as *const Mutex<Inner>);

        let waker = {
            let mut inner = arc.lock().unwrap();

            DATA_FROM_HOST.with(|d| {
                let data = d.take();
                inner.result = Some(data);
            });
            inner.waker.take().unwrap()
        };
        // Drop the lock before we wake the task that will also try to access the lock.
        waker.wake(); // Wake up our task.
    }
}
