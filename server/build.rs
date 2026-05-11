fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rustc-env=DATABASE_URL=mariadb://root:sparganothis@127.0.0.1/sparganothis");
}