pub const JPEG_END: [u8; 2] = [0xFF, 0xD9];

pub fn find_last_sequence(data: &Vec<u8>, sequence: &[u8; 2]) -> Option<usize> {
    if sequence.len() != 2 || data.len() < 2 {
        return None; // Sequence must be 2 bytes, and data must have at least 2 bytes
    }

    for i in (0..=data.len() - 2).rev() {
        if data[i] == sequence[0] && data[i + 1] == sequence[1] {
            return Some(i);
        }
    }

    None // Sequence not found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_data() {
        let data: Vec<u8> = vec![];
        let sequence: [u8; 2] = [1, 2];
        assert_eq!(find_last_sequence(&data, &sequence), None);
    }

    #[test]
    fn test_shorter_data() {
        let data: Vec<u8> = vec![1];
        let sequence: [u8; 2] = [1, 2];
        assert_eq!(find_last_sequence(&data, &sequence), None);
    }

    #[test]
    fn test_sequence_found_at_end() {
        let data: Vec<u8> = vec![3, 4, 1, 2];
        let sequence: [u8; 2] = [1, 2];
        assert_eq!(find_last_sequence(&data, &sequence), Some(2));
    }

    #[test]
    fn test_sequence_found_in_middle() {
        let data: Vec<u8> = vec![1, 2, 3, 1, 2, 4];
        let sequence: [u8; 2] = [1, 2];
        assert_eq!(find_last_sequence(&data, &sequence), Some(3));
    }

    #[test]
    fn test_sequence_not_found() {
        let data: Vec<u8> = vec![3, 4, 5, 6];
        let sequence: [u8; 2] = [1, 2];
        assert_eq!(find_last_sequence(&data, &sequence), None);
    }

    #[test]
    fn test_data_same_as_sequence() {
        let data: Vec<u8> = vec![1, 2];
        let sequence: [u8; 2] = [1, 2];
        assert_eq!(find_last_sequence(&data, &sequence), Some(0));
    }

    #[test]
    fn test_sequence_occurs_multiple_times_at_start() {
        let data: Vec<u8> = vec![1, 2, 1, 2, 3];
        let sequence: [u8; 2] = [1, 2];
        assert_eq!(find_last_sequence(&data, &sequence), Some(2));
    }
}
