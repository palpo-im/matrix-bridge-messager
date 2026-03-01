pub mod logging;

pub fn init() {
    logging::init_tracing();
}
