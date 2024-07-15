use crate::http::http;
use crate::socket::socket;

mod socket;
mod http;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    tokio::join!(
        socket(),
        http()
    );
}
