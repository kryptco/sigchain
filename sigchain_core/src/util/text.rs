pub fn ellipsize_center(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.into();
    }
    let in_ = s.chars();
    let mut out = String::new();
    out.extend(in_.clone().take(max_len/2));
    out.extend("...".chars());
    out.extend(in_.rev().take(max_len/2).collect::<Vec<_>>().into_iter().rev());
    out
}
