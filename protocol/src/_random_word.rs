use iroh::PublicKey;

fn get_nickname_from_seed(seed: u32) -> String {
    let all =  random_word::all(random_word::Lang::Fr);

    let idx = seed as usize % all.len();

    all[idx].to_string()
}

pub fn get_nickname_from_pubkey(pubkey: PublicKey) -> String {
    let seed = pubkey.as_bytes().to_vec();
    let seed = seed.chunks(4).fold(0_u32, |acc, b| {
        let b2 = b.try_into().unwrap();
        acc ^  u32::from_le_bytes(b2)
    });

    get_nickname_from_seed(seed)
}


