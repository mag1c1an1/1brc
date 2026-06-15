use tokio::runtime;

async fn inner() {
    todo!()
}

// use tokio
pub fn __main() {
    let rt = runtime::Builder::new_multi_thread().build().unwrap();
    rt.block_on(inner())
}
