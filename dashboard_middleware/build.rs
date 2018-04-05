extern crate walkdir;
extern crate includedir_codegen;

fn main() {
    include_client_web_files()
}

const DASHBOARD_YEW_DEPLOY_PATH : &'static str = "../target/deploy-final";

fn include_client_web_files() {
    for entry in walkdir::WalkDir::new(DASHBOARD_YEW_DEPLOY_PATH).into_iter().filter_map(|e| e.ok()) {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }
    for entry in walkdir::WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }
    use includedir_codegen::Compression;
    includedir_codegen::start("FILES")
        .dir(DASHBOARD_YEW_DEPLOY_PATH, Compression::Gzip)
        .build("dashboard_frontend.rs")
        .unwrap();
}
