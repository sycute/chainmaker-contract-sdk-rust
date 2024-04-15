pub const ZERO_ADDR: &str = "0000000000000000000000000000000000000000";
pub const ZERO_ADDR_WITH_PREFIX: &str = "0x0000000000000000000000000000000000000000";
pub const ZERO_ADDR_SKI_HEX: &str = "";

pub fn is_zero_ski_hex_address(address: &str) -> bool {
    address == ZERO_ADDR_SKI_HEX
}
