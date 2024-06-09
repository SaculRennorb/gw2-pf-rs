pub mod pf;
pub mod formats;
pub mod parse;

mod wstr; pub use wstr::WString;
mod filename; pub use filename::FileName;

use gw2_pf_rs_derive;
pub(crate) use gw2_pf_rs_derive::Parse;

pub const fn tcc(_str : &[u8; 2]) -> u16 {
	unsafe{ std::mem::transmute(*_str) }
}

pub const fn fcc(_str : &[u8; 4]) -> u32 {
	unsafe{ std::mem::transmute(*_str) }
}
