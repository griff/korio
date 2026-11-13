fn main() {
    let path = std::env::current_dir().unwrap();
    let mut cmd = capnpc::CompilerCommand::new();
    cmd.import_path(path.join("capnp"))
        .src_prefix("capnp/korio")
        .file("capnp/korio/common.capnp")
        .file("capnp/korio/event.capnp")
        .file("capnp/korio/trace.capnp")
        .file("capnp/korio/capture.capnp")
        .default_parent_module(vec!["capnp".into()])
        .run()
        .expect("compiling");
    println!("cargo::rerun-if-changed=capnp");
    println!("cargo::rerun-if-changed=build.rs");
}
