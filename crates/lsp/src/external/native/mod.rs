#![allow(dead_code)]

use futures::Future;

pub(crate) fn spawn<F: Future<Output = ()> + Send + 'static>(fut: F) {
    tokio::spawn(fut);
}
