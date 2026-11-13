fn main() {
    let path = std::env::current_dir().unwrap();
    let mut cmd = capnpc::CompilerCommand::new();
    match std::env::var_os("SCHEMA_ROOT") {
        Some(schema_root) => {
            cmd.import_path(schema_root);
        }
        None => {
            cmd.import_path("../korio-capnp/capnp");
        }
    }
    cmd.import_path(path.join("capnp"))
        .crate_provides("korio_capnp::capnp", [0xb40eafabe6638c3c])
        .src_prefix("capnp/korio")
        .file("capnp/korio/koriosc.capnp")
        .default_parent_module(vec!["capnp".into()])
        .run()
        .expect("compiling");
    println!("cargo::rerun-if-changed=capnp");
    println!("cargo::rerun-if-changed=build.rs");
}
