lazy_static::lazy_static! {
    static ref ARGON2: argon2::Argon2<'static> = {
        let mut params = argon2::ParamsBuilder::new();
        params
            .m_cost(4096)
            .unwrap()
            .t_cost(24)
            .unwrap()
            .p_cost(8)
            .unwrap()
            .output_len(32)
            .unwrap();
        let params = params.params().unwrap();
        argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::ersion::V0x13, params)
    };
}

const WORD_LIST: [&str; 2048] = soter_macros::word_list!("english.txt");

pub struct SeedPhrase;
