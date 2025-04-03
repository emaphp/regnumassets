/// String header indicating the beginning of a DDS texture
pub const DDS_STRING: &str = "DDS ";

// TODO: add as feature?
// use itertools::Itertools;
// pub fn barg_to_rgba(data: Vec<u8>) -> Vec<u8> {
//     let mut out = Vec::with_capacity(data.len());
//     let mut it = data.into_iter().tuples::<(u8, u8, u8, u8)>();

//     while let Some(x) = it.next() {
//         let (b, a, r, g) = x;
//         out.extend_from_slice(&[r, g, b, a]);
//     }

//     out
// }
