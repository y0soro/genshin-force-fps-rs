pub mod logger;
pub mod process;

pub mod utils {
    pub fn str_to_w_vec(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(::core::iter::once(0)).collect()
    }

    pub fn w_to_str(wide: &[u16]) -> String {
        let i = wide.iter().cloned().take_while(|&c| c != 0);
        char::decode_utf16(i)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect()
    }
}
