use iroh::SecretKey;

fn main() {
    let key = SecretKey::generate(&mut rand::thread_rng());
    println!("private: {:?}", key.to_string());
    println!("public:  {:?}", key.public().to_string());
}