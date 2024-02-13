macro_rules! profile_scope {
    ($($tt:expr),+) => {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!($($tt),+);
    };
}
pub(crate) use profile_scope;