mod function;

pub use function::{decrypt, encrypt};

// use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
//
// lazy_static::lazy_static! {
//     static ref ARGON2: Argon2<'static> = {
//         let mut params = ParamsBuilder::new();
//         params
//             .m_cost(4096)
//             .unwrap()
//             .t_cost(24)
//             .unwrap()
//             .p_cost(8)
//             .unwrap()
//             .output_len(32)
//             .unwrap();
//         let params = params.params().unwrap();
//         Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
//     };
// }
