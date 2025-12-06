//! Vietnamese Unicode character mapping

use super::keys;

/// Convert key + tone + mark to Vietnamese character
/// tone: 0=none, 1=hat(^), 2=breve(˘)
/// mark: 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
pub fn to_char(key: u16, caps: bool, tone: u8, mark: u8) -> Option<char> {
    let base = match key {
        keys::A => Some(if tone == 1 { 'â' } else if tone == 2 { 'ă' } else { 'a' }),
        keys::E => Some(if tone == 1 { 'ê' } else { 'e' }),
        keys::I => Some('i'),
        keys::O => Some(if tone == 1 { 'ô' } else if tone == 2 { 'ơ' } else { 'o' }),
        keys::U => Some(if tone == 2 { 'ư' } else { 'u' }),
        keys::Y => Some('y'),
        keys::D => Some('d'),
        _ => None,
    }?;

    let ch = apply_mark(base, mark);
    Some(if caps { to_upper(ch) } else { ch })
}

/// Get đ/Đ character
pub fn get_d(caps: bool) -> char {
    if caps { 'Đ' } else { 'đ' }
}

/// Apply mark to base vowel
fn apply_mark(base: char, mark: u8) -> char {
    if mark == 0 {
        return base;
    }

    // Vietnamese vowel table: [base, sắc, huyền, hỏi, ngã, nặng]
    let table: &[(char, [char; 5])] = &[
        ('a', ['á', 'à', 'ả', 'ã', 'ạ']),
        ('ă', ['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ']),
        ('â', ['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ']),
        ('e', ['é', 'è', 'ẻ', 'ẽ', 'ẹ']),
        ('ê', ['ế', 'ề', 'ể', 'ễ', 'ệ']),
        ('i', ['í', 'ì', 'ỉ', 'ĩ', 'ị']),
        ('o', ['ó', 'ò', 'ỏ', 'õ', 'ọ']),
        ('ô', ['ố', 'ồ', 'ổ', 'ỗ', 'ộ']),
        ('ơ', ['ớ', 'ờ', 'ở', 'ỡ', 'ợ']),
        ('u', ['ú', 'ù', 'ủ', 'ũ', 'ụ']),
        ('ư', ['ứ', 'ừ', 'ử', 'ữ', 'ự']),
        ('y', ['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ']),
    ];

    for (b, marks) in table {
        if *b == base {
            let idx = (mark - 1) as usize;
            if idx < 5 {
                return marks[idx];
            }
        }
    }

    base
}

/// Convert lowercase Vietnamese to uppercase
fn to_upper(ch: char) -> char {
    match ch {
        'a' => 'A', 'á' => 'Á', 'à' => 'À', 'ả' => 'Ả', 'ã' => 'Ã', 'ạ' => 'Ạ',
        'ă' => 'Ă', 'ắ' => 'Ắ', 'ằ' => 'Ằ', 'ẳ' => 'Ẳ', 'ẵ' => 'Ẵ', 'ặ' => 'Ặ',
        'â' => 'Â', 'ấ' => 'Ấ', 'ầ' => 'Ầ', 'ẩ' => 'Ẩ', 'ẫ' => 'Ẫ', 'ậ' => 'Ậ',
        'e' => 'E', 'é' => 'É', 'è' => 'È', 'ẻ' => 'Ẻ', 'ẽ' => 'Ẽ', 'ẹ' => 'Ẹ',
        'ê' => 'Ê', 'ế' => 'Ế', 'ề' => 'Ề', 'ể' => 'Ể', 'ễ' => 'Ễ', 'ệ' => 'Ệ',
        'i' => 'I', 'í' => 'Í', 'ì' => 'Ì', 'ỉ' => 'Ỉ', 'ĩ' => 'Ĩ', 'ị' => 'Ị',
        'o' => 'O', 'ó' => 'Ó', 'ò' => 'Ò', 'ỏ' => 'Ỏ', 'õ' => 'Õ', 'ọ' => 'Ọ',
        'ô' => 'Ô', 'ố' => 'Ố', 'ồ' => 'Ồ', 'ổ' => 'Ổ', 'ỗ' => 'Ỗ', 'ộ' => 'Ộ',
        'ơ' => 'Ơ', 'ớ' => 'Ớ', 'ờ' => 'Ờ', 'ở' => 'Ở', 'ỡ' => 'Ỡ', 'ợ' => 'Ợ',
        'u' => 'U', 'ú' => 'Ú', 'ù' => 'Ù', 'ủ' => 'Ủ', 'ũ' => 'Ũ', 'ụ' => 'Ụ',
        'ư' => 'Ư', 'ứ' => 'Ứ', 'ừ' => 'Ừ', 'ử' => 'Ử', 'ữ' => 'Ữ', 'ự' => 'Ự',
        'y' => 'Y', 'ý' => 'Ý', 'ỳ' => 'Ỳ', 'ỷ' => 'Ỷ', 'ỹ' => 'Ỹ', 'ỵ' => 'Ỵ',
        'đ' => 'Đ',
        _ => ch.to_ascii_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(to_char(keys::A, false, 0, 1), Some('á'));
        assert_eq!(to_char(keys::A, true, 0, 1), Some('Á'));
        assert_eq!(to_char(keys::A, false, 1, 0), Some('â'));
        assert_eq!(to_char(keys::A, false, 1, 1), Some('ấ'));
        assert_eq!(to_char(keys::A, false, 2, 0), Some('ă'));
    }

    #[test]
    fn test_d() {
        assert_eq!(get_d(false), 'đ');
        assert_eq!(get_d(true), 'Đ');
    }
}
