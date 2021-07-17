/// Determina si una cadena de caracteres sigue un patrón glob [glob-style pattern].
///
/// # Examples
///
/// ```
/// use proyecto_taller_1::services::utils::glob_pattern::g_match;
///
/// let pattern = b"*.md";
/// let my_string = b"an_example.md";
/// assert_eq!(g_match(pattern, my_string), true);
/// let pattern = b"*.md";
/// let my_string = b"wrong_example.ad";
/// assert_eq!(g_match(pattern, my_string), false);
/// ```
pub fn g_match(pattern: &[u8], string: &[u8]) -> bool {
    let mut pattern_pos = 0;
    let mut string_pos = 0;

    // * hace match con todo
    if pattern.len() == 1 && pattern[0] == b'*' {
        return true;
    }

    while pattern_pos < pattern.len() {
        match pattern[pattern_pos] {
            b'*' => {
                // * hace match con cualquier numero de caracteres
                // avanzo hasta que no haya mas *s
                while pattern_pos + 1 < pattern.len() && pattern[pattern_pos + 1] == b'*' {
                    pattern_pos += 1;
                }
                if pattern_pos == pattern.len() {
                    return true;
                }

                // verifico recursivamente que lo que siga al asterisco coincida con el string dado
                // por ej. '*.md' hace match con 'cualquiercosa.md'
                for i in string_pos..(string.len() + 1) {
                    if g_match(&pattern[pattern_pos + 1..], &string[i..]) {
                        return true;
                    }
                }
                return false;
            }
            b'?' => {
                // ? hace match con cualquier caracter
            }
            b'!' => {
                // ! niega el patron que le sigue
                pattern_pos += 1;
                if pattern[pattern_pos] == string[string_pos] {
                    return false;
                }
            }
            b'\\' => {
                // '\' hace que el caracter que le siga se lea como un caracter comun, no como
                // caracter especial. Por ejemplo, que se pueda leer el caracter '?' sin considerarlo
                // parte del patron
                pattern_pos += 1;
                if pattern[pattern_pos] != string[string_pos] {
                    return false;
                }
            }
            b'[' => {
                pattern_pos += 1;
                let not = pattern[pattern_pos] == b'!';
                if not {
                    pattern_pos += 1;
                }
                let mut is_match = false;
                loop {
                    if pattern[pattern_pos] == b'\\' {
                        pattern_pos += 1;
                        if pattern[pattern_pos] == string[string_pos] {
                            is_match = true;
                        }
                    } else if pattern[pattern_pos] == b']' {
                        break;
                    } else if pattern_pos >= pattern.len() {
                        pattern_pos += 1;
                        break;
                    } else if pattern.len() >= pattern_pos + 3 && pattern[pattern_pos + 1] == b'-' {
                        //si es un Range
                        let low_bound = pattern[pattern_pos];
                        let upper_bound = pattern[pattern_pos + 2];
                        let char = string[string_pos];
                        pattern_pos += 2;
                        if char >= low_bound && char <= upper_bound {
                            is_match = true;
                        }
                    } else if pattern[pattern_pos] == string[string_pos] {
                        is_match = true;
                    }
                    pattern_pos += 1;
                }
                if not {
                    is_match = !is_match;
                }
                if !is_match {
                    return false;
                }
            }
            _ => {
                if string_pos >= string.len() || pattern[pattern_pos] != string[string_pos] {
                    return false;
                }
            }
        }
        pattern_pos += 1;
        string_pos += 1;
        if string_pos > string.len() {
            return false;
        }
    }
    true
}

#[test]
fn test_01_wildcard_is_match() {
    assert_eq!(g_match(b"*.md", b"banana.md"), true);
}

#[test]
fn test_02_wildcard_is_not_match() {
    assert_eq!(g_match(b"*.md", b"banana.ad"), false);
}

#[test]
fn test_03_pattern_in_brackets_is_match() {
    assert_eq!(g_match(b"[cbr]at", b"cat"), true);
}

#[test]
fn test_04_question_mark_is_match() {
    assert_eq!(g_match(b"?at.md", b"cat.md"), true);
}

#[test]
fn test_05_backslash_is_match() {
    assert_eq!(g_match(b"set\\*.md", b"set*.md"), true);
}

#[test]
fn test_06_multiple_wildcards_is_match() {
    assert_eq!(
        g_match(b"*max-*-entries*", b"hash-max-zipmap-entries"),
        true
    );
}

#[test]
fn test_07_pattern_in_brackets_is_not_match() {
    assert_eq!(g_match(b"[br]", b"cat"), false);
}

#[test]
fn test_08_pattern_in_range_is_match() {
    assert_eq!(g_match(b"[a-e]at", b"cat"), true);
}

#[test]
fn test_09_pattern_in_range_is_not_match() {
    assert_eq!(g_match(b"[n-o]", b"cat"), false);
}
