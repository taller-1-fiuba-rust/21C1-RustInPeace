/// Determina si un string sigue un patrón glob [glob-style pattern]
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
                    if g_match(
                        &pattern[pattern_pos + 1..],
                        &string[i..]
                    ) {
                        return true;
                    }
                }
                return false;
            }
            b'?' => {
                // ? hace match con cualquier caracter
                // if string_pos >= string.len() {
                //     return false;
                // }
                // string_pos += 1;
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
                // if string_pos >= string.len() {
                //     return false;
                // }
                if pattern[pattern_pos] != string[string_pos] {
                    return false;
                }
                // string_pos += 1;
            }
            b'[' => {
                pattern_pos += 1;
                let not = pattern[pattern_pos] == b'!';
                if not {
                    pattern_pos += 1;
                }
                let mut matched = false;
                loop {
                    if pattern[pattern_pos] == b'\\' {
                        pattern_pos += 1;
                        if pattern[pattern_pos] == string[string_pos] {
                            matched = true;
                        }
                    } else if pattern[pattern_pos] == b']' {
                        break;
                    } else if pattern_pos >= pattern.len() {
                        pattern_pos += 1;
                        break;
                    } else if pattern.len() >= pattern_pos + 3 && pattern[pattern_pos + 1] == b'-' {
                        let mut start = pattern[pattern_pos];
                        let mut end = pattern[pattern_pos + 2];
                        let c = string[string_pos];
                        if start > end {
                            std::mem::swap(&mut start, &mut end);
                        }
                        pattern_pos += 2;
                        if c >= start && c <= end {
                            matched = true;
                        }
                    } else if pattern[pattern_pos] == string[string_pos] {
                        matched = true;
                    }

                    pattern_pos += 1;
                }
                if not {
                    matched = !matched;
                }
                if !matched {
                    return false;
                }
                // string_pos += 1;
            }
            _ => {
                // if string_pos >= string.len() {
                //     return false;
                // }
                if string_pos >= string.len() || pattern[pattern_pos] != string[string_pos] {
                    return false;
                }
                // string_pos += 1;
            }
        }
        pattern_pos += 1;
        string_pos += 1;
        if string_pos == string.len() {
            for i in &pattern[pattern_pos..pattern.len()] {
                if *i != b'*' {
                    break;
                }
            }
            break;
        }
        if string_pos >= string.len() {
            return false;
        }
    }

    true
}

#[test]
fn test_01() {
    assert_eq!(g_match(b"*.md", b"banana.md"), true);
}

#[test]
fn test_02() {
    assert_eq!(g_match(b"*.md", b"banana.ad"), false);
}


#[test]
fn test_03() {
    assert_eq!(g_match(b"*.md", b"banana.ad"), false);
}

#[test]
fn test_04() {
    assert_eq!(g_match(b"?at.md", b"cat.md"), true);
}

#[test]
fn test_05() {
    assert_eq!(g_match(b"set\\*.md", b"set*.md"), true);
}

#[test]
fn test_06() {
    assert_eq!(g_match(b"*max-*-entries*", b"hash-max-zipmap-entries"), true);
}