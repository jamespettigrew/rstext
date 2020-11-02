pub fn line_break_offsets(s: &str) -> Vec<usize> {
    s.bytes()
        .enumerate()
        .filter_map(|(i, b)| match b {
            0x0A => Some(i),
            _ => None,
        })
        .collect()
}

pub fn next_char_idx(s: &str, byte_offset: usize) -> Option<usize> {
    s[byte_offset..]
        .char_indices()
        .skip_while(|(i, _)| *i == 0)
        .next()
        .map(|(i, _)| byte_offset + i)
}

pub fn prev_char_idx(s: &str, byte_offset: usize) -> Option<usize> {
    s[..byte_offset]
        .char_indices()
        .rev()
        // .skip_while(|(i, _)| *i == 0)
        .next()
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_break_offsets_correct() {
        let mut line = String::from("");
        let mut offsets = line_break_offsets(&line);
        assert_eq!(vec![0usize; 0], offsets);

        line = String::from("abc\ndef\nghijk\nl");
        offsets = line_break_offsets(&line);
        assert_eq!(vec![3, 7, 13], offsets);
    }

    #[test]
    fn next_char_idx_correct() {
        let s = String::from("abcdef");
        let idx = next_char_idx(&s, 3);
        assert_eq!(idx, Some(4));
    }

    #[test]
    fn next_char_idx_end_correct() {
        let s = String::from("abcdef");
        let idx = next_char_idx(&s, 6);
        assert_eq!(idx, None);
    }

    #[test]
    fn prev_char_idx_correct() {
        let s = String::from("abcdef");
        let idx = prev_char_idx(&s, 3);
        assert_eq!(idx, Some(2));
    }

    #[test]
    fn prev_char_idx_start_correct() {
        let s = String::from("abcdef");
        let idx = prev_char_idx(&s, 0);
        assert_eq!(idx, None);
    }
}