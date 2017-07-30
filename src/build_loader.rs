extern crate gcc;

fn main() {
    gcc::Config::new()
        .flag("-static")
        .flag("-nostdlib")
        .file("src/kernel/execve/loader/loader.c")
        .out_dir("src/kernel/execve/loader")
        .compile_binary("binary_loader_exe");
}
