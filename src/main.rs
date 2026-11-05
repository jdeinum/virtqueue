use tracing::error;
use virtio::run;

fn main() {
    tracing_subscriber::fmt::init();
    if let Err(e) = run() {
        error!("Error running program: {e:?}");
    }
}
