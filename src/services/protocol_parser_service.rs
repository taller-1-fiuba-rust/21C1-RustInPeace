use std::error::Error;
use std::fmt;
use crate::entities::error::ParseError;
use crate::entities::datatype_trait::DataType;
use crate::entities::resp_types::RESPTypes;

/// Error parsing
impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::InvalidProtocol(_) => "Protocol error"
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseError occurred")
    }
}

impl DataType for RESPTypes {
    fn deserialize(self) -> String {
        match self {
            RESPTypes::RBulkString(string) => {
                format!("${}\r\n{}\r\n", string.len(), string)
            }
            RESPTypes::RInteger(integer) => {
                format!(":{}\r\n", integer)
            }
            RESPTypes::RSimpleString(string) => {
                format!("+{}\r\n", string)
            }
            RESPTypes::RArray(array) => {
                let mut final_string = String::from("");
                final_string += "*";
                final_string += &array.len().to_string();
                final_string += "\r\n";
                for element in array {
                    final_string += &element.deserialize();
                }
                return final_string;
            }
            RESPTypes::RError(message) => {
                format!("-{}\r\n", message)
            }
        }
    }
}

fn search_crlf(req: &[u8]) -> Result<usize, ParseError> {
    let mut i = 0;
    for b in req {
        if i+1 >= req.len() {
            return Err(ParseError::InvalidProtocol("crlf".to_string()));
        }
        if b == &b'\r' {
            if req[i+1] == b'\n' {
                break;
            } else {
                return Err(ParseError::InvalidProtocol("crlf".to_string()));
            }
            return Ok(i);
        }
        i += 1;
    }
    Ok(i)
}

fn read_word(from: usize, to: usize, req: &[u8]) -> String {
    let slice = &req[from..to];
    String::from_utf8_lossy(slice).to_string()
}

fn read_int(from: usize, to: usize, req: &[u8]) -> usize {
    let s = read_word(from, to, req);
    let i: usize = s.parse().unwrap();
    i
}

//Asumo que aca llega un string siguiendo el protocolo Redis.. igualmente se hacen las validaciones
pub fn parse(request: &[u8]) -> Result<RESPTypes, ParseError> {
    //valido sea array de bulkstrings
    let mut pos = 0;
    // chequeo el byte de la primer posicion para delegar el parseo segun el tipo de dato RESP
    match request[pos] {
        b'*' => {
            //Array
            //*2\r\n$3\r\nkey\r\n:5\r\n
            let final_pos = search_crlf(request);
            match final_pos {
                Ok(i) => {
                    let size = read_int(pos+1, i, request);
                    pos += i+1; //salto el \r\n
                    let mut vec2: Vec<RESPTypes> = Vec::new();
                    let mut contents = &request[pos+1..];
                    let mut idx = 0;
                    while contents.len() > 0 {
                        if contents[idx] == b'$' {
                            //leer hasta el segundo crlf
                            let first_crlf = search_crlf(contents);
                            match first_crlf {
                                Ok(final_pos_1) => {
                                    // final_pos_1 deberia ser == 2
                                    //busco final de segundo crlf
                                    let second_crlf = search_crlf(&contents[final_pos_1+2..]);
                                    match second_crlf {
                                        Ok(final_pos_2) => {
                                            let bulkstr = parse(&contents[..final_pos_1+final_pos_2+4]);
                                            match bulkstr {
                                                Ok(result) => {
                                                    vec2.push(result);
                                                    contents = &contents[final_pos_1+final_pos_2+4..];
                                                }
                                                Err(e) => {println!("error1");break;}
                                            }
                                        }
                                        Err(e) => {println!("error2");break;}
                                    }
                                }
                                Err(e) => {println!("error3");break;}
                            }
                        } else {
                            //leer hasta el primer crlf
                            println!("NO soy un bulkstring");
                            //leer hasta el segundo crlf
                            let first_crlf = search_crlf(contents);
                            match first_crlf {
                                Ok(final_pos_1) => {
                                    // final_pos_1 deberia ser == 2
                                    //busco final de segundo crlf
                                    println!("first crlf:{}", final_pos_1);
                                    let resp = parse(&contents[..final_pos_1+2]);
                                        match resp {
                                            Ok(result) => {
                                                println!("result: {:?}", result);
                                                vec2.push(result);
                                                contents = &contents[final_pos_1+2..];
                                                println!("contents after:{:?}", contents);
                                            }
                                            Err(e) => {println!("error4");break;}
                                        }
                                }
                                Err(e) => {println!("error5");break;}
                            }
                        }
                    }
                    if vec2.len() != size {
                        return Err(ParseError::InvalidProtocol(String::from("Invalid array size")));
                    }
                    Ok(RESPTypes::RArray(vec2))
                }
                Err(e) => Err(e)
            }
        }
        b'+' => {
            // SimpleString
            // desde pos+1 hasta el \r\n es el string que quiero
            let final_pos = search_crlf(request);
            match final_pos {
                Ok(i) => {
                    let s = read_word(pos+1, i, request);
                    Ok(RESPTypes::RSimpleString(s))
                }
                Err(e) => {
                    Err(e)
                }
            }
           
        }
        b'-' => {
            //Error
            //-Error message\r\n
            let final_pos = search_crlf(request);
            match final_pos {
                Ok(i) => {
                    let s = read_word(pos+1, i, request);
                    Ok(RESPTypes::RError(s))
                }
                Err(e) => {
                    Err(e)
                }
            }
        }
        b':' => {
            //Integer
            //:5\r\n
            let final_pos = search_crlf(request);
            match final_pos {
                Ok(i) => {
                    let r = read_int(pos+1, i, request);
                    Ok(RESPTypes::RInteger(r))
                }
                Err(e) => {
                    Err(e)
                }
            }
        }
        b'$' => {
            //BulkString
            //6\r\nfoobar\r\n
            println!("og req {:?}", request);
            let final_pos = search_crlf(request);
            println!("final pos 1: {:?}", final_pos);
            match final_pos {
                Ok(i) => {
                    if i != 2 {
                        //ERROR xq tiene que saltar las pos. del primer nro y del crlf
                        return Err(ParseError::InvalidProtocol("bad1".to_string()));
                    }
                    let int = read_int(pos+1, i, request);
                    println!("cantidad letras 1: {}", int);
                    pos = i+1;
                    let slice = &request[pos+1..];
                    println!("slice {:?}", slice);
                    let p = search_crlf(slice);
                    match p {
                        Ok(pp) => {
                            println!("final pos 2: {}", pp);
                            let s = read_word(0, pp, slice);
                            println!("string final {}", s);
                            // validate s.len == i
                            if s.len() != int {
                                //ERROR
                                return Err(ParseError::InvalidProtocol("bad2".to_string()))
                            }
                            Ok(RESPTypes::RBulkString(s))
                        }
                        Err(e) => Err(e)
                    }
                }
                Err(e) => Err(e)
            }
        }
        _ => {
            //lanzar error de bad protocol o algo asi
            Err(ParseError::InvalidProtocol("".to_string()))
        }
    }
}

pub fn parse_response(response: RESPTypes) -> String {
    return response.deserialize();
}


#[test]
fn parse_request_returns_simple_string_ok() {
    let req = b"+Ok\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RESPTypes::RSimpleString(s) => {
            assert_eq!(s, "Ok".to_string())
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_request_returns_error_ok() {
    let req = b"-Error message\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RESPTypes::RError(s) => {
            assert_eq!(s, "Error message".to_string())
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_request_returns_integer_ok() {
    let req = b":5\r\n";
    let result = parse(req);
    assert!(result.is_ok());
    match result.unwrap() {
        RESPTypes::RInteger(i) => {
            assert_eq!(i, 5)
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_request_returns_bulkstring_ok() {
    let req = b"$6\r\nfoobar\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_ok());
    match result.unwrap() {
        RESPTypes::RBulkString(s) => {
            assert_eq!(s, "foobar".to_string())
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_request_returns_array_of_bulkstrings_ok() {
    let req = b"*3\r\n$6\r\nfoobar\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_ok());
    match result.unwrap() {
        RESPTypes::RArray(v) => {
            assert_eq!(v, vec![RESPTypes::RBulkString(String::from("foobar")), RESPTypes::RBulkString(String::from("key")), RESPTypes::RBulkString(String::from("value"))])
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_request_returns_array_of_bulkstrings_and_integers_ok() {
    let req = b"*4\r\n$6\r\nfoobar\r\n:5\r\n:10\r\n$5\r\nvalue\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_ok());
    match result.unwrap() {
        RESPTypes::RArray(v) => {
            assert_eq!(v, vec![RESPTypes::RBulkString(String::from("foobar")), RESPTypes::RInteger(5), RESPTypes::RInteger(10), RESPTypes::RBulkString(String::from("value"))])
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_request_returns_array_of_errors_ok() {
    let req = b"*2\r\n-ErrorMessage1\r\n- SomeError Message2\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_ok());
    match result.unwrap() {
        RESPTypes::RArray(v) => {
            assert_eq!(v, vec![RESPTypes::RError(String::from("ErrorMessage1")), RESPTypes::RError(String::from(" SomeError Message2"))])
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_request_returns_error_when_invalid_array_size() {
    let req = b"*4\r\n-ErrorMessage1\r\n- SomeError Message2\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(s) => {
            assert_eq!(s, "Invalid array size".to_string())
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_bulkstring_returns_error_when_invalid_length() {
    let req = b"$5\r\nfoobar\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(s) => {
            assert_eq!(s, "bad2".to_string())
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_bulkstring_returns_error_when_missing_length() {
    let req = b"$\r\nfoobar\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(s) => {
            assert_eq!(s, "bad1".to_string())
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_request_returns_error_when_invalid_crlf() {
    let req = b"$6\r\nfoobar\r";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(s) => {
            assert_eq!(s, "crlf".to_string())
        }
        _ => assert!(false)
    }
}

#[test]
fn parse_response_string_ok() {
    let result = parse_response(RESPTypes::RSimpleString(String::from("test")));
    let expected = "+test\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_bulk_string_ok() {
    let result = parse_response(RESPTypes::RBulkString(String::from("test")));
    let expected = "$4\r\ntest\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_integer_ok() {
    let result = parse_response(RESPTypes::RInteger(5));
    let expected = ":5\r\n";
    assert_eq!(result, expected);
}

#[test]
fn parse_response_generic_error_ok() {
    let result = parse_response(RESPTypes::RError("Error Some error".to_string()));
    let expected = "-Error Some error\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_vector_of_strings_ok() {
    let result = parse_response(RESPTypes::RArray(vec![RESPTypes::RSimpleString("a".to_string()), RESPTypes::RSimpleString("b".to_string())]));
    let expected = "*2\r\n+a\r\n+b\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_vector_of_integers_ok() {
    let result = parse_response(RESPTypes::RArray(vec![RESPTypes::RInteger(2),RESPTypes::RInteger(3),RESPTypes::RInteger(10),RESPTypes::RInteger(11)]));
    let expected = "*4\r\n:2\r\n:3\r\n:10\r\n:11\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_response_vector_of_errors_ok() {
    let result = parse_response(RESPTypes::RArray(vec![RESPTypes::RError("message1".to_string()), RESPTypes::RError("message2".to_string())]));
    let expected = "*2\r\n-message1\r\n-message2\r\n".to_string();
    assert_eq!(result, expected);
}