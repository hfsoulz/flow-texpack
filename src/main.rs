// flow-texpack: A program that will allow you to generate texture atlas.
// zlib License (see LICENSE)

use flow_texpack::App;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.run().await;
}
