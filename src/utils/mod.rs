pub mod logging;
pub mod security;
pub mod validation;

pub fn init() {
    logging::init_tracing();
}
