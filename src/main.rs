mod app;
mod logger;

use app::App;

fn main() {
    logger::init(logger::LogLevel::Debug);
    let app = App::new();
    app.run();
}
