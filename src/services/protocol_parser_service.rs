use crate::entities::datatype_trait::DataType;
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
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error occurred while parsing")
    }
}

impl DataType for RespTypes {
    fn deserialize(self) -> String {
        match self {
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
                    final_string += &element.deserialize();
                }
                final_string
            }
            RespTypes::RError(message) => {
                format!("-{}\r\n", message)
            }
        }
    }
}

pub fn parse_response(response: RespTypes) -> String {
    response.deserialize()
}

pub fn parse_request(request: &[u8]) -> Result<RespTypes, ParseError> {
    parse(request)
}

/// Recibe un arreglo de bytes (req) y devuelve la posición del primer CRLF que encuentre
/// Chequea que el CRLF esté bien formado, si no lo está devuelve Error
/// -poner ejemplos-
fn search_crlf(request: &[u8]) -> Result<usize, ParseError> {
    let mut i = 0;
    for byte in request {
        if i + 1 >= request.len() {
            return Err(ParseError::InvalidProtocol(
                "Missing CRFL or Message contains invalid CRFL [\r must be followed by \n]"
                    .to_string(),
            ));
        }
        if byte == &b'\r' {
            if request[i + 1] != b'\n' {
                return Err(ParseError::InvalidProtocol(
                    "Message contains invalid CRFL [\r must be followed by \n]".to_string(),
                ));
            }
            return Ok(i);
        }
        i += 1;
    }
    Ok(i)
}

/// Recibe un arreglo de bytes (req) y dos numeros enteros, from y to, que indican las posiciones
/// desde donde comenzar a leer y hasta donde leer del arreglo req.
/// Devuelve los datos leidos en forma de String
/// -ejemplos-
fn read_word(from: usize, to: usize, req: &[u8]) -> Result<String, ParseError> {
    if from > to {
        return Err(ParseError::UnexpectedError(
            "Invalid slice of bytes".to_string(),
        ));
    }
    let slice = &req[from..to];
    Ok(String::from_utf8_lossy(slice).to_string())
}

/// Recibe un arreglo de bytes (req) y dos numeros enteros, from y to, que indican las posiciones
/// desde donde comenzar a leer y hasta donde leer del arreglo req.
/// Devuelve los datos leidos transformados a tipo de dato usize
/// -ejemplos-
fn read_int(from: usize, to: usize, req: &[u8]) -> Result<usize, ParseError> {
    match read_word(from, to, req) {
        Ok(string) => match string.parse() {
            Ok(i) => Ok(i),
            Err(_) => Err(ParseError::IntParseError(
                "Error while parsing string to int".to_string(),
            )),
        },
        Err(e) => Err(e),
    }
}

//Asumo que aca llega un string siguiendo el protocolo Redis.. igualmente se hacen las validaciones
/// Recibe un arreglo de bytes y lo transforma a un tipo de dato RESPTypes siguiendo el protocolo RESP
/// Devuelve un Result<RESPTypes, ParseError>
/// -ejemplos-
pub fn parse(request: &[u8]) -> Result<RespTypes, ParseError> {
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

pub fn parse_integer(request: &[u8]) -> Result<RespTypes, ParseError> {
    let pos = 0;
    let final_pos = search_crlf(request);
    match final_pos {
        Ok(i) => match read_int(pos + 1, i, request) {
            Ok(r) => Ok(RespTypes::RInteger(r)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn parse_error(request: &[u8]) -> Result<RespTypes, ParseError> {
    let pos = 0;
    let final_pos = search_crlf(request);
    match final_pos {
        Ok(i) => match read_word(pos + 1, i, request) {
            Ok(s) => Ok(RespTypes::RError(s)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn parse_simple_string(request: &[u8]) -> Result<RespTypes, ParseError> {
    let pos = 0;
    let final_pos = search_crlf(request);
    match final_pos {
        Ok(i) => match read_word(pos + 1, i, request) {
            Ok(s) => Ok(RespTypes::RSimpleString(s)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn parse_bulkstring(request: &[u8]) -> Result<RespTypes, ParseError> {
    let mut pos = 0;
    let final_pos = search_crlf(request);
    match final_pos {
        Ok(i) => {
            if i != 2 {
                return Err(ParseError::UnexpectedError(
                    "String size must be followed by CRFL".to_string(),
                ));
            }
            let int = read_int(pos + 1, i, request).unwrap_or(0);
            pos = i + 1;
            let slice = &request[pos + 1..];
            let p = search_crlf(slice);
            match p {
                Ok(pp) => match read_word(0, pp, slice) {
                    Ok(s) => {
                        if s.len() != int {
                            return Err(ParseError::InvalidSize("String size mismatch".to_string()));
                        }
                        Ok(RespTypes::RBulkString(s))
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub fn parse_array(request: &[u8]) -> Result<RespTypes, ParseError> {
    let mut pos = 0;
    let final_pos = search_crlf(request);
    match final_pos {
        Ok(i) => {
            let size = read_int(pos + 1, i, request).unwrap_or(0);
            pos += i + 1; //salto el \r\n
            let mut vec2: Vec<RespTypes> = Vec::new();
            let mut contents = &request[pos + 1..];
            let idx = 0;
            while !contents.is_empty() {
                if contents[idx] == b'$' {
                    //leer hasta el segundo crlf
                    let first_crlf = search_crlf(contents);
                    match first_crlf {
                        Ok(final_pos_1) => {
                            //busco final de segundo crlf
                            let second_crlf = search_crlf(&contents[final_pos_1 + 2..]);
                            match second_crlf {
                                Ok(final_pos_2) => {
                                    let bulkstr = parse(&contents[..final_pos_1 + final_pos_2 + 4]);
                                    match bulkstr {
                                        Ok(result) => {
                                            vec2.push(result);
                                            contents = &contents[final_pos_1 + final_pos_2 + 4..];
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
                        Ok(final_pos_1) => {
                            //busco final de segundo crlf
                            let resp = parse(&contents[..final_pos_1 + 2]);
                            match resp {
                                Ok(result) => {
                                    vec2.push(result);
                                    contents = &contents[final_pos_1 + 2..];
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
            if vec2.len() != size {
                return Err(ParseError::InvalidSize(String::from("Array size mismatch")));
            }
            Ok(RespTypes::RArray(vec2))
        }
        Err(e) => Err(e),
    }
}

#[test]
fn parse_request_returns_simple_string_ok() {
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
fn parse_request_returns_error_ok() {
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
fn parse_request_returns_integer_ok() {
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
fn parse_request_returns_parse_int_error_when_missing_integer() {
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
fn parse_request_returns_bulkstring_ok() {
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
fn parse_request_returns_array_of_bulkstrings_ok() {
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
fn parse_request_returns_array_of_bulkstrings_and_integers_ok() {
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
fn parse_request_returns_array_of_errors_ok() {
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
fn parse_request_returns_error_when_invalid_array_size() {
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
fn parse_request_returns_error_when_invalid_crlf_ending() {
    let req = b"$6\r\nfoobar\r";
    let result = parse(req);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(s) => {
            assert_eq!(
                s,
                "Missing CRFL or Message contains invalid CRFL [\r must be followed by \n]"
                    .to_string()
            )
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_request_returns_error_when_missing_newline() {
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
fn parse_response_string_ok() {
    let result = parse_response(RespTypes::RSimpleString(String::from("test")));
    let expected = "+test\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_bulk_string_ok() {
    let result = parse_response(RespTypes::RBulkString(String::from("test")));
    let expected = "$4\r\ntest\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_integer_ok() {
    let result = parse_response(RespTypes::RInteger(5));
    let expected = ":5\r\n";
    assert_eq!(result, expected);
}

#[test]
fn parse_response_generic_error_ok() {
    let result = parse_response(RespTypes::RError("Error Some error".to_string()));
    let expected = "-Error Some error\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_vector_of_strings_ok() {
    let result = parse_response(RespTypes::RArray(vec![
        RespTypes::RSimpleString("a".to_string()),
        RespTypes::RSimpleString("b".to_string()),
    ]));
    let expected = "*2\r\n+a\r\n+b\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_vector_of_integers_ok() {
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
fn parse_response_vector_of_errors_ok() {
    let result = parse_response(RespTypes::RArray(vec![
        RespTypes::RError("message1".to_string()),
        RespTypes::RError("message2".to_string()),
    ]));
    let expected = "*2\r\n-message1\r\n-message2\r\n".to_string();
    assert_eq!(result, expected);
}
