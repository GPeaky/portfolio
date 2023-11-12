use mimalloc::MiMalloc;
use warp::Filter;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    let spa = warp::any().and(warp::fs::file("./dist/index.html"));
    let assets = warp::fs::dir("./dist");

    let routes = assets.or(spa);

    warp::serve(routes).run(([0, 0, 0, 0], 4321)).await;
}
