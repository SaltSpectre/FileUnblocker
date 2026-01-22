use std::env;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let _ = embed_resource::compile("unblocker.rc", embed_resource::NONE);
    }
}