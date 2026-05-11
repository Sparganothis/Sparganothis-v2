fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rustc-env=DATABASE_URL=mariadb://root:tetris@127.0.0.1/tetris");
}