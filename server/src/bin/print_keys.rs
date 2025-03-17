use iroh::SecretKey;

fn main() {
    let num = 5;
    println!("\n\n pub const BOOTSTRAP_SECRET_KEYS: [[u8; 32]; {num}] = [");
    for _k in 0..num {
        let key = SecretKey::generate(&mut rand::thread_rng());
        println!("{:?},", key.to_bytes());
    }
    println!("];\n\n");
}
