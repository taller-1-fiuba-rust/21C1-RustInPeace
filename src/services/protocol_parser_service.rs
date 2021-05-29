use crate::entities::error::ParseError;
use crate::entities::resp_types::RespTypes;
use std::error::Error;
use std::fmt;

/// Error parsing
impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::InvalidProtocol(_) => "RESP Protocol error",
            ParseError::IntParseError(_) => "Error occured while parsing int",
            ParseError::InvalidSize(_) => "Size mismatch",
            ParseError::UnexpectedError(_) => "Unexpected error while parsing",
            ParseError::InvalidRequest(_) => "Invalid request error",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error occurred while parsing")
    }
}

pub fn parse_response(response: RespTypes) -> String {
    match response {
        RespTypes::RBulkString(string) => {
            format!("${}\r\n{}\r\n", string.len(), string)
        }
        RespTypes::RInteger(integer) => {
            format!(":{}\r\n", integer)
        }
        RespTypes::RSimpleString(string) => {
            format!("+{}\r\n", string)
        }
        RespTypes::RArray(array) => {
            let mut final_string = String::from("");
            final_string += "*";
            final_string += &array.len().to_string();
            final_string += "\r\n";
            for element in array {
                final_string += &parse_response(element);
            }
            final_string
        }
        RespTypes::RError(message) => {
            format!("-{}\r\n", message)
        }
        RespTypes::RNullBulkString() => {
            format!("$-1\r\n")
        }
        RespTypes::RNullArray() => {
            format!("*-1\r\n")
        }
    }
}

pub fn parse_request(request: &[u8]) -> Result<RespTypes, ParseError> {
    match parse(request) {
        Ok(parsed_request) => {
            if is_array_of_bulkstring(&parsed_request) {
                return Ok(parsed_request);
            } else {
                return Err(ParseError::InvalidRequest(
                    "Request must be an array of bulkstrings".to_string(),
                ));
            }
        }
        Err(e) => Err(e),
    }
}

fn is_array_of_bulkstring(parsed_request: &RespTypes) -> bool {
    if let RespTypes::RArray(array) = parsed_request {
        for element in array {
            if let RespTypes::RBulkString(_) = element {
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
/// -poner ejemplos-
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

/// Recibe un arreglo de bytes (req) y dos numeros enteros, from y to, que indican las posiciones
/// desde donde comenzar a leer y hasta donde leer del arreglo req.
/// Devuelve los datos leidos en forma de String
/// -ejemplos-
fn read_word(from: usize, to: usize, request: &[u8]) -> Result<String, ParseError> {
    if from > to {
        return Err(ParseError::UnexpectedError(
            "Invalid slice of bytes".to_string(),
        ));
    }
    let slice = &request[from..to];
    Ok(String::from_utf8_lossy(slice).to_string())
}

/// Recibe un arreglo de bytes (req) y dos numeros enteros, from y to, que indican las posiciones
/// desde donde comenzar a leer y hasta donde leer del arreglo req.
/// Devuelve los datos leidos transformados a tipo de dato usize
/// -ejemplos-
fn read_int(from: usize, to: usize, request: &[u8]) -> Result<usize, ParseError> {
    match read_word(from, to, request) {
        Ok(string) => match string.parse() {
            Ok(int) => Ok(int),
            Err(_) => Err(ParseError::IntParseError(
                "Error while parsing string to int".to_string(),
            )),
        },
        Err(e) => Err(e),
    }
}

/// Recibe un arreglo de bytes y lo transforma a un tipo de dato RESPTypes siguiendo el protocolo RESP
/// Devuelve un Result<RESPTypes, ParseError>
/// -ejemplos-
fn parse(request: &[u8]) -> Result<RespTypes, ParseError> {
    if String::from_utf8_lossy(&request[request.len() - 2..]) != "\r\n" {
        return Err(ParseError::InvalidProtocol(
            "CRFL missing at the end of command".to_string(),
        ));
    }
    match request[0] {
        b'*' => match parse_array(request) {
            Ok(array) => Ok(array),
            Err(e) => Err(e),
        },
        b'+' => match parse_simple_string(request) {
            Ok(simple_string) => Ok(simple_string),
            Err(e) => Err(e),
        },
        b'-' => match parse_error(request) {
            Ok(error) => Ok(error),
            Err(e) => Err(e),
        },
        b':' => match parse_integer(request) {
            Ok(integer) => Ok(integer),
            Err(e) => Err(e),
        },
        b'$' => match parse_bulkstring(request) {
            Ok(bulkstring) => Ok(bulkstring),
            Err(e) => Err(e),
        },
        _ => Err(ParseError::InvalidProtocol(
            "First byte must be one of the following: $, +, :, *, -".to_string(),
        )),
    }
}

fn parse_integer(request: &[u8]) -> Result<RespTypes, ParseError> {
    let pos = 0;
    let crlf_pos = search_crlf(request);
    match crlf_pos {
        Ok(final_pos) => match read_int(pos + 1, final_pos, request) {
            Ok(int) => Ok(RespTypes::RInteger(int)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

fn parse_error(request: &[u8]) -> Result<RespTypes, ParseError> {
    let pos = 0;
    let crlf_pos = search_crlf(request);
    match crlf_pos {
        Ok(final_pos) => match read_word(pos + 1, final_pos, request) {
            Ok(word) => Ok(RespTypes::RError(word)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

fn parse_simple_string(request: &[u8]) -> Result<RespTypes, ParseError> {
    let pos = 0;
    let crlf_pos = search_crlf(request);
    match crlf_pos {
        Ok(final_pos) => match read_word(pos + 1, final_pos, request) {
            Ok(word) => Ok(RespTypes::RSimpleString(word)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

fn parse_bulkstring(request: &[u8]) -> Result<RespTypes, ParseError> {
    let mut pos = 0;
    let crlf = search_crlf(request);
    match crlf {
        Ok(crlf_pos) => {
            if crlf_pos == 3 {
                match check_if_resp_null_type(pos, crlf_pos, request) {
                    Ok(is_null) => {
                        if is_null {
                            return Ok(RespTypes::RNullBulkString());
                        } else {
                            return Err(ParseError::UnexpectedError(
                                "String size must be followed by CRFL".to_string(),
                            ));
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            if crlf_pos != 2 {
                return Err(ParseError::UnexpectedError(
                    "String size must be followed by CRFL".to_string(),
                ));
            }
            let size = read_int(pos + 1, crlf_pos, request).unwrap_or(0);
            pos = crlf_pos + 1;
            let slice = &request[pos + 1..];
            let next_crlf = search_crlf(slice);
            match next_crlf {
                Ok(next_crlf_pos) => match read_word(0, next_crlf_pos, slice) {
                    Ok(word) => {
                        if word.len() != size {
                            return Err(ParseError::InvalidSize(
                                "String size mismatch".to_string(),
                            ));
                        }
                        Ok(RespTypes::RBulkString(word))
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

fn check_if_resp_null_type(from: usize, to: usize, request: &[u8]) -> Result<bool, ParseError> {
    match read_word(from + 1, to, request) {
        Ok(word) => {
            if word == "-1" {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

fn parse_array(request: &[u8]) -> Result<RespTypes, ParseError> {
    let mut pos = 0;
    let crlf = search_crlf(request);
    match crlf {
        Ok(crlf_pos) => {
            if crlf_pos == 3 {
                match check_if_resp_null_type(pos, crlf_pos, request) {
                    Ok(is_null) => {
                        if is_null {
                            return Ok(RespTypes::RNullArray());
                        } else {
                            return Err(ParseError::UnexpectedError(
                                "Array size must be followed by CRFL".to_string(),
                            ));
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            let size = read_int(pos + 1, crlf_pos, request).unwrap_or(0);
            pos += crlf_pos + 1; //salto el \r\n
            let mut vec: Vec<RespTypes> = Vec::new();
            let mut contents = &request[pos + 1..];
            let idx = 0;
            while !contents.is_empty() {
                if contents[idx] == b'$' {
                    //leer hasta el segundo crlf
                    let first_crlf = search_crlf(contents);
                    match first_crlf {
                        Ok(first_crlf_pos) => {
                            //busco final de segundo crlf
                            let second_crlf = search_crlf(&contents[first_crlf_pos + 2..]);
                            match second_crlf {
                                Ok(second_crlf_pos) => {
                                    let bulkstr =
                                        parse(&contents[..first_crlf_pos + second_crlf_pos + 4]);
                                    match bulkstr {
                                        Ok(result) => {
                                            vec.push(result);
                                            contents =
                                                &contents[first_crlf_pos + second_crlf_pos + 4..];
                                        }
                                        Err(e) => {
                                            return Err(e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                } else {
                    //leer hasta el segundo crlf
                    let first_crlf = search_crlf(contents);
                    match first_crlf {
                        Ok(first_crlf_pos) => {
                            //busco final de segundo crlf
                            let resp = parse(&contents[..first_crlf_pos + 2]);
                            match resp {
                                Ok(result) => {
                                    vec.push(result);
                                    contents = &contents[first_crlf_pos + 2..];
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }
            if vec.len() != size {
                return Err(ParseError::InvalidSize(String::from("Array size mismatch")));
            }
            Ok(RespTypes::RArray(vec))
        }
        Err(e) => Err(e),
    }
}

#[test]
fn parse_returns_ok_when_given_valid_simple_string() {
    let req = b"+Ok\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_ok());
    match result.unwrap() {
        RespTypes::RSimpleString(s) => {
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
        RespTypes::RError(s) => {
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
        RespTypes::RInteger(i) => {
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
        RespTypes::RBulkString(s) => {
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
        RespTypes::RNullBulkString() => assert!(true),
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_valid_nullarray() {
    let req = b"*-1\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespTypes::RNullArray() => assert!(true),
        _ => assert!(false),
    }
}

#[test]
fn parse_returns_ok_when_given_empty_bulkstring() {
    let req = b"$0\r\n\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RespTypes::RBulkString(s) => {
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
        RespTypes::RArray(v) => {
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
        RespTypes::RArray(v) => {
            assert_eq!(
                v,
                vec![
                    RespTypes::RBulkString(String::from("foobar")),
                    RespTypes::RBulkString(String::from("key")),
                    RespTypes::RBulkString(String::from("value"))
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
        RespTypes::RArray(v) => {
            assert_eq!(
                v,
                vec![
                    RespTypes::RBulkString(String::from("foobar")),
                    RespTypes::RBulkString(String::from("key")),
                    RespTypes::RBulkString(String::from("value"))
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
        RespTypes::RArray(v) => {
            assert_eq!(
                v,
                vec![
                    RespTypes::RBulkString(String::from("foobar")),
                    RespTypes::RInteger(5),
                    RespTypes::RInteger(10),
                    RespTypes::RBulkString(String::from("value"))
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
        RespTypes::RArray(v) => {
            assert_eq!(
                v,
                vec![
                    RespTypes::RError(String::from("ErrorMessage1")),
                    RespTypes::RError(String::from(" SomeError Message2"))
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
    let result = parse_response(RespTypes::RSimpleString(String::from("test")));
    let expected = "+test\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_bulkstring() {
    let result = parse_response(RespTypes::RBulkString(String::from("test")));
    let expected = "$4\r\ntest\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_empty_bulkstring() {
    let result = parse_response(RespTypes::RBulkString(String::from("")));
    let expected = "$0\r\n\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_empty_array() {
    let result = parse_response(RespTypes::RArray(vec![]));
    let expected = "*0\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_null_bulkstring() {
    let result = parse_response(RespTypes::RNullBulkString());
    let expected = "$-1\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_null_array() {
    let result = parse_response(RespTypes::RNullArray());
    let expected = "*-1\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_integer() {
    let result = parse_response(RespTypes::RInteger(5));
    let expected = ":5\r\n";
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_error() {
    let result = parse_response(RespTypes::RError("Error Some error".to_string()));
    let expected = "-Error Some error\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_array_of_strings() {
    let result = parse_response(RespTypes::RArray(vec![
        RespTypes::RSimpleString("a".to_string()),
        RespTypes::RSimpleString("b".to_string()),
    ]));
    let expected = "*2\r\n+a\r\n+b\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_array_of_integers() {
    let result = parse_response(RespTypes::RArray(vec![
        RespTypes::RInteger(2),
        RespTypes::RInteger(3),
        RespTypes::RInteger(10),
        RespTypes::RInteger(11),
    ]));
    let expected = "*4\r\n:2\r\n:3\r\n:10\r\n:11\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_returns_ok_when_given_array_of_errors() {
    let result = parse_response(RespTypes::RArray(vec![
        RespTypes::RError("message1".to_string()),
        RespTypes::RError("message2".to_string()),
    ]));
    let expected = "*2\r\n-message1\r\n-message2\r\n".to_string();
    assert_eq!(result, expected);
}
