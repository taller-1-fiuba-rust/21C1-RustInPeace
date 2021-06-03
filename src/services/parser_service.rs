//! Servicio para transformar un mensaje que llega por stream a un tipo de dato RESP y viceversa !

use super::utils::resp_type::RespType;
use crate::shared_errors::parse_error::ParseError;

/// Recibe una response de tipo RespType y lo traduce a un String respetando el protocolo RESP
/// -ejemplos-
pub fn parse_response(response: RespType) -> String {
    // trait display -> en resp_type.rs
    match response {
        RespType::RBulkString(string) => {
            format!("${}\r\n{}\r\n", string.len(), string)
        }
        RespType::RInteger(integer) => {
            format!(":{}\r\n", integer)
        }
        RespType::RSimpleString(string) => {
            format!("+{}\r\n", string)
        }
        RespType::RArray(array) => {
            let mut final_string = String::from("");
            final_string += "*";
            final_string += &array.len().to_string();
            final_string += "\r\n";
            for element in array {
                final_string += &parse_response(element);
            }
            final_string
        }
        RespType::RError(message) => {
            format!("-{}\r\n", message)
        }
        RespType::RNullBulkString() => "$-1\r\n".to_string(),
        RespType::RNullArray() => "*-1\r\n".to_string(),
    }
}

/// Recibe una request, la traduce segun el protocolo RESP a un tipo de dato RespType
/// Verifica que sea un array de bulkstrings, si no lo es arroja error InvalidRequest
/// -ejemplos-
pub fn parse_request(request: &[u8]) -> Result<RespType, ParseError> {
    match parse(request) {
        Ok(parsed_request) => {
            if is_array_of_bulkstring(&parsed_request) {
                Ok(parsed_request)
            } else {
                Err(ParseError::InvalidRequest(
                    "Request must be an array of bulkstrings".to_string(),
                ))
            }
        }
        Err(e) => Err(e),
    }
}

/// A partir de una request ya traducida a un RespType, valida que sea un array de bulkstrings
/// Devuelve true si lo es, false si no
fn is_array_of_bulkstring(parsed_request: &RespType) -> bool {
    if let RespType::RArray(array) = parsed_request {
        for element in array {
            if let RespType::RBulkString(_) = element {
            } else {
                return false;
            }
        }
    } else {
        return false;
    }
    true
}

/// Recibe un arreglo de bytes (req) y devuelve la posición del primer CRLF que encuentre
/// Chequea que el CRLF esté bien formado, si no lo está devuelve Error
fn search_crlf(request: &[u8]) -> Result<usize, ParseError> {
    let mut pos = 0;
    for byte in request {
        if pos + 1 >= request.len() {
            return Err(ParseError::InvalidProtocol(
                "Missing CRFL or Message contains invalid CRFL [\r must be followed by \n]"
                    .to_string(),
            ));
        }
        if byte == &b'\r' {
            if request[pos + 1] != b'\n' {
                return Err(ParseError::InvalidProtocol(
                    "Message contains invalid CRFL [\r must be followed by \n]".to_string(),
                ));
            }
            return Ok(pos);
        }
        pos += 1;
    }
    Ok(pos)
}

/// Recibe un arreglo de bytes (request) y dos numeros enteros, from y to, que indican las posiciones
/// desde donde comenzar a leer y hasta donde leer del arreglo request.
/// Devuelve los datos leidos en forma de String
fn read_word(from: usize, to: usize, request: &[u8]) -> Result<String, ParseError> {
    if from > to {
        return Err(ParseError::UnexpectedError(
            "Invalid slice of bytes".to_string(),
        ));
    }
    let slice = &request[from..to];
    Ok(String::from_utf8_lossy(slice).to_string())
}

/// Recibe un arreglo de bytes (request) y dos numeros enteros, from y to, que indican las posiciones
/// desde donde comenzar a leer y hasta donde leer del arreglo request.
/// Devuelve los datos leidos transformados a tipo de dato usize
fn read_int(from: usize, to: usize, request: &[u8]) -> Result<usize, ParseError> {
    let word = read_word(from, to, request)?;
    match word.parse() {
        Ok(int) => Ok(int),
        Err(_) => Err(ParseError::IntParseError(
            "Error while parsing string to int".to_string(),
        )),
    }
}

/// Recibe un arreglo de bytes y lo transforma a un tipo de dato RESPType siguiendo el protocolo RESP
/// Devuelve un Result<RESPType, ParseError>
/// -ejemplos-
fn parse(request: &[u8]) -> Result<RespType, ParseError> {
    if String::from_utf8_lossy(&request[request.len() - 2..]) != "\r\n" {
        return Err(ParseError::InvalidProtocol(
            "CRFL missing at the end of command".to_string(),
        ));
    }
    match request[0] {
        b'*' => Ok(parse_array(request)?),
        b'+' => Ok(parse_simple_string(request)?),
        b'-' => Ok(parse_error(request)?),
        b':' => Ok(parse_integer(request)?),
        b'$' => Ok(parse_bulkstring(request)?),
        _ => Err(ParseError::InvalidProtocol(
            "First byte must be one of the following: $, +, :, *, -".to_string(),
        )),
    }
}

/// Recibe un arreglo de bytes y lee el segundo byte como un numero entero
/// Devuelve RespType::RInteger(numero)
fn parse_integer(request: &[u8]) -> Result<RespType, ParseError> {
    let pos = 0;
    let crlf_pos = search_crlf(request)?;
    let int = read_int(pos + 1, crlf_pos, request)?;
    Ok(RespType::RInteger(int))
}

/// Recibe un arreglo de bytes y lee desde la segunda posicion como una cadena de caracteres
/// Devuelve RespType::RError(string)
fn parse_error(request: &[u8]) -> Result<RespType, ParseError> {
    let pos = 0;
    let crlf_pos = search_crlf(request)?;
    let word = read_word(pos + 1, crlf_pos, request)?;
    Ok(RespType::RError(word))
}

/// Recibe un arreglo de bytes y lee desde la segunda posicion como una cadena de caracteres
/// Devuelve RespType::RESimpleString(string)
fn parse_simple_string(request: &[u8]) -> Result<RespType, ParseError> {
    let pos = 0;
    let crlf_pos = search_crlf(request)?;
    let word = read_word(pos + 1, crlf_pos, request)?;
    Ok(RespType::RSimpleString(word))
}

/// Recibe un arreglo de bytes, en la segunda posicion lee un numero entero que indica el largo de la palabra
/// luego lee la cadena de caracteres validando que sea del mismo largo indicado
/// Devuelve RespType::RBulkString(string)
fn parse_bulkstring(request: &[u8]) -> Result<RespType, ParseError> {
    let mut pos = 0;
    let crlf = search_crlf(request)?;
    if crlf == 3 {
        return check_if_bulkstring_null_type(pos, crlf, request);
    }
    if crlf != 2 {
        return Err(ParseError::UnexpectedError(
            "String size must be followed by CRFL".to_string(),
        ));
    }
    let size = read_int(pos + 1, crlf, request).unwrap_or(0);
    pos = crlf + 1;
    let slice = &request[pos + 1..];
    let next_crlf = search_crlf(slice)?;
    let word = read_word(0, next_crlf, slice)?;
    if word.len() != size {
        return Err(ParseError::InvalidSize("String size mismatch".to_string()));
    }
    Ok(RespType::RBulkString(word))
}

/// Dado un arreglo de bytes, una posicion inicial y una posicion inicial, verifica si es un tipo de dato bulkstring
/// nulo segun el procolo RESP
/// Devuelve true si lo es, false si no
fn check_if_bulkstring_null_type(
    from: usize,
    to: usize,
    request: &[u8],
) -> Result<RespType, ParseError> {
    let word = read_word(from + 1, to, request)?;
    if word == "-1" {
        Ok(RespType::RNullBulkString())
    } else {
        Err(ParseError::UnexpectedError(
            "String size must be followed by CRFL".to_string(),
        ))
    }
}

/// Dado un arreglo de bytes, una posicion inicial y una posicion inicial, verifica si es un tipo de dato array
/// nulo segun el procolo RESP
/// Devuelve true si lo es, false si no
fn check_if_array_null_type(
    from: usize,
    to: usize,
    request: &[u8],
) -> Result<RespType, ParseError> {
    let word = read_word(from + 1, to, request)?;
    if word == "-1" {
        Ok(RespType::RNullArray())
    } else {
        Err(ParseError::UnexpectedError(
            "String size must be followed by CRFL".to_string(),
        ))
    }
}

/// Returns length of the given request
fn get_request_len(request: &[u8]) -> usize {
    match request[0] {
        b'*' => get_array_len(request),
        b'$' => get_bulkstring_len(request),
        b':' => get_integer_len(request),
        b'+' => get_string_len(request),
        b'-' => get_string_len(request),
        _ => 0,
    }
}

/// Recibe un arreglo de bytes y lo traduce a un dato de tipo RespType::RArray(Vec<RespType>)
fn parse_array(request: &[u8]) -> Result<RespType, ParseError> {
    let mut pos = 0;
    let crlf = search_crlf(request)?;
    if crlf == 3 {
        return check_if_array_null_type(pos, crlf, request);
    }
    let size = read_int(pos + 1, crlf, request).unwrap_or(0);
    pos += crlf + 2; //salto el \r\n
    let mut vec: Vec<RespType> = Vec::new();
    let mut contents = &request[pos..];
    while !contents.is_empty() {
        let request_len = get_request_len(contents);
        let parsed_request = parse(&contents[..request_len]).unwrap();
        vec.push(parsed_request);
        contents = &contents[request_len..];
    }
    if vec.len() != size {
        return Err(ParseError::InvalidSize(String::from("Array size mismatch")));
    }
    Ok(RespType::RArray(vec))
}

/// Returns length of array request
fn get_array_len(request: &[u8]) -> usize {
    let mut len = 0;
    match search_crlf(request) {
        Ok(crlf_pos) => {
            let size = read_int(len + 1, crlf_pos, request).unwrap_or(0);
            let mut element_len = 0;
            len += crlf_pos + 2;
            let mut i = 0;
            while i < size {
                element_len = get_request_len(&request[crlf_pos + 2 + element_len..]);
                len += element_len;
                i += 1;
            }
            len
        }
        Err(_e) => len,
    }
}

/// Returns length of bulkstring request
fn get_bulkstring_len(request: &[u8]) -> usize {
    let mut len = 0;
    match search_crlf(request) {
        Ok(crfl) => {
            len += 1; //$
            let size = read_int(len, crfl, request).unwrap_or(0);
            len += 1; //size
            match search_crlf(&request[crfl + 1..]) {
                Ok(_second_crfl) => {
                    len += size + 4; //+ 4 bytes crfl
                    len
                }
                Err(_e) => len,
            }
        }
        Err(_e) => len,
    }
}

/// Returns length of integer request
fn get_integer_len(request: &[u8]) -> usize {
    let mut len = 0;
    match search_crlf(request) {
        Ok(crlf) => {
            len += 2; // \r\n
            for _i in &request[..crlf] {
                len += 1;
            }
            len
        }
        Err(_e) => len,
    }
}

/// Returns length of string request (can be simple string or error request)
fn get_string_len(request: &[u8]) -> usize {
    let mut len = 0;
    match search_crlf(request) {
        Ok(crlf) => {
            len += 1;
            match read_word(len, crlf, request) {
                Ok(word) => {
                    len += word.len();
                    len += 2; // \r\n
                    len
                }
                Err(_e) => len,
            }
        }
        Err(_e) => len,
    }
}

#[test]
fn parse_returns_ok_when_given_valid_simple_string() {
    let req = b"+Ok\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RSimpleString(s) => {
            assert_eq!(s, "Ok".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_valid_resp_error() {
    let req = b"-Error message\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RError(s) => {
            assert_eq!(s, "Error message".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_valid_integer() {
    let req = b":5\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RInteger(i) => {
            assert_eq!(i, 5)
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_parse_int_error_when_missing_integer() {
    let req = b":\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::IntParseError(s) => {
            assert_eq!(s, String::from("Error while parsing string to int"))
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_valid_bulkstring() {
    let req = b"$6\r\nfoobar\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RBulkString(s) => {
            assert_eq!(s, "foobar".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_valid_nullbulkstring() {
    let req = b"$-1\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RNullBulkString() => assert!(true),
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_valid_nullarray() {
    let req = b"*-1\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RNullArray() => assert!(true),
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_empty_bulkstring() {
    let req = b"$0\r\n\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RBulkString(s) => {
            assert_eq!(s, "".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_empty_array() {
    let req = b"*0\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RArray(v) => {
            assert_eq!(v, vec![])
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_array_of_bulkstrings() {
    let req = b"*3\r\n$6\r\nfoobar\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RArray(v) => {
            assert_eq!(
                v,
                vec![
                    RespType::RBulkString(String::from("foobar")),
                    RespType::RBulkString(String::from("key")),
                    RespType::RBulkString(String::from("value"))
                ]
            )
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_request_returns_ok_when_given_valid_array_of_bulkstrings() {
    let req = b"*3\r\n$6\r\nfoobar\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
    let result = parse_request(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RArray(v) => {
            assert_eq!(
                v,
                vec![
                    RespType::RBulkString(String::from("foobar")),
                    RespType::RBulkString(String::from("key")),
                    RespType::RBulkString(String::from("value"))
                ]
            )
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_array_of_bulkstrings_and_integers() {
    let req = b"*4\r\n$6\r\nfoobar\r\n:5\r\n:10\r\n$5\r\nvalue\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RArray(v) => {
            assert_eq!(
                v,
                vec![
                    RespType::RBulkString(String::from("foobar")),
                    RespType::RInteger(5),
                    RespType::RInteger(10),
                    RespType::RBulkString(String::from("value"))
                ]
            )
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_array_of_errors() {
    let req = b"*2\r\n-ErrorMessage1\r\n- SomeError Message2\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RArray(v) => {
            assert_eq!(
                v,
                vec![
                    RespType::RError(String::from("ErrorMessage1")),
                    RespType::RError(String::from(" SomeError Message2"))
                ]
            )
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_array_of_arrays() {
    let req = b"*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Foo\r\n-Bar\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_ok());
    match result.unwrap() {
        RespType::RArray(v) => {
            assert_eq!(
                v,
                vec![
                    RespType::RArray(vec![
                        RespType::RInteger(1),
                        RespType::RInteger(2),
                        RespType::RInteger(3)
                    ]),
                    RespType::RArray(vec![
                        RespType::RSimpleString(String::from("Foo")),
                        RespType::RError(String::from("Bar"))
                    ])
                ]
            )
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_error_when_invalid_array_size() {
    let req = b"*4\r\n-ErrorMessage1\r\n- SomeError Message2\r\n";
    let result = parse(req);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidSize(s) => {
            assert_eq!(s, "Array size mismatch".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_bulkstring_returns_error_when_invalid_length() {
    let req = b"$5\r\nfoobar\r\n";
    let result = parse(req);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidSize(s) => {
            assert_eq!(s, "String size mismatch".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_bulkstring_returns_error_when_missing_length() {
    let req = b"$\r\nfoobar\r\n";
    let result = parse(req);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::UnexpectedError(s) => {
            assert_eq!(s, "String size must be followed by CRFL".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_error_when_missing_newline() {
    let req = b"$6\rfoobar\r\n";
    let result = parse(req);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(s) => {
            assert_eq!(
                s,
                "Message contains invalid CRFL [\r must be followed by \n]".to_string()
            )
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_error_when_missing_last_crfl() {
    let req = b"$6\r\nfoobar\r\njhkhb";
    let result = parse(req);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(s) => {
            assert_eq!(s, "CRFL missing at the end of command".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_request_returns_error_when_not_given_array() {
    let req = b"$6\r\nfoobar\r\n";
    let result = parse_request(req);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidRequest(s) => {
            assert_eq!(s, "Request must be an array of bulkstrings".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_request_returns_error_when_given_array_of_integers() {
    let req = b"*2\r\n:5\r\n:7\r\n";
    let result = parse_request(req);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidRequest(s) => {
            assert_eq!(s, "Request must be an array of bulkstrings".to_string())
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_response_returns_ok_when_given_string() {
    let result = parse_response(RespType::RSimpleString(String::from("test")));
    let expected = "+test\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_bulkstring() {
    let result = parse_response(RespType::RBulkString(String::from("test")));
    let expected = "$4\r\ntest\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_empty_bulkstring() {
    let result = parse_response(RespType::RBulkString(String::from("")));
    let expected = "$0\r\n\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_empty_array() {
    let result = parse_response(RespType::RArray(vec![]));
    let expected = "*0\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_null_bulkstring() {
    let result = parse_response(RespType::RNullBulkString());
    let expected = "$-1\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_null_array() {
    let result = parse_response(RespType::RNullArray());
    let expected = "*-1\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_integer() {
    let result = parse_response(RespType::RInteger(5));
    let expected = ":5\r\n";
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_error() {
    let result = parse_response(RespType::RError("Error Some error".to_string()));
    let expected = "-Error Some error\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_array_of_strings() {
    let result = parse_response(RespType::RArray(vec![
        RespType::RSimpleString("a".to_string()),
        RespType::RSimpleString("b".to_string()),
    ]));
    let expected = "*2\r\n+a\r\n+b\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_array_of_integers() {
    let result = parse_response(RespType::RArray(vec![
        RespType::RInteger(2),
        RespType::RInteger(3),
        RespType::RInteger(10),
        RespType::RInteger(11),
    ]));
    let expected = "*4\r\n:2\r\n:3\r\n:10\r\n:11\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_array_of_errors() {
    let result = parse_response(RespType::RArray(vec![
        RespType::RError("message1".to_string()),
        RespType::RError("message2".to_string()),
    ]));
    let expected = "*2\r\n-message1\r\n-message2\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_array_of_arrays() {
    let result = parse_response(RespType::RArray(vec![
        RespType::RError("message1".to_string()),
        RespType::RError("message2".to_string()),
        RespType::RArray(vec![RespType::RInteger(7)]),
    ]));
    let expected = "*3\r\n-message1\r\n-message2\r\n*1\r\n:7\r\n".to_string();
    assert_eq!(result, expected);
}
