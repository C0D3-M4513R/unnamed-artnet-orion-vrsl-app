#[inline]
pub const fn deg_to_microarcseconds(deg:u32) -> u64 {
    deg as u64
        *60 //to arcseconds
        *60 //to arcminutes
        *1000 //to milliarcseconds
        *1000 //to microarcsecond
}

pub const fn scale_deg(input: u64, input_range: u64, output_range:u64) -> u64 {
    #[allow(clippy::cast_possible_truncation)]
    {
        (input as u128 * output_range as u128 / input_range as u128) as u64
    }
}
