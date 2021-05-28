use std::error::Error;
use std::fmt;
use std::convert::TryInto;
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
                    let request_stringified = String::from_utf8_lossy(&request[pos..]).to_string();
                    let vec: Vec<&str> = request_stringified.split("$").collect();
                    let mut vec2: Vec<RESPTypes> = Vec::new();
                    println!("vec stringified: {:?}", vec);
                    for string in vec {
                        let respie = parse(string.as_bytes());
                        match respie {
                            Ok(resp) => { vec2.push(resp) }
                            Err(e) => { return Err(e); }
                        }
                    }
                    //validar size con vec2.len()
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
                            if s.len() != i {
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

// Clients send commands to a Redis server as a RESP Array of Bulk Strings
pub fn parse_request(request: &[u8]) -> Result<Vec<String>, ParseError> {
    // VALIDAR BULK STRINGS 512MB
    let mut pos = 0;
    let request_stringified = String::from_utf8_lossy(request).to_string();
    let mut request_vec = Vec::new();
    for letter in request_stringified.chars() {
        request_vec.push(letter);
    }

    //chequeo que sea un array of bulk strings
    if request_vec[pos] != '*' {
        println!("Error 1");
        return Err(ParseError::InvalidProtocol("Request must start with *".to_string()));
    }

    //chequeo que siempre que haya un /r le siga un /n
    let mut found = false;
    for (count, b) in request.iter().enumerate() {
        if b == &b'\r' {
            found = true;
            if request[count+1] != b'\n' {
                println!("Error 2");
                return Err(ParseError::InvalidProtocol("Invalid CRLF, '\r' must be followed by '\n'".to_string()));
            }
        }
    }

    if !found {
        println!("Error 3");
        return Err(ParseError::InvalidProtocol("CRLF missing".to_string()));
    }

    pos += 1;
    let mut parsed_command: Vec<String> = Vec::new();
    let array_size = request_vec[pos].to_digit(10).unwrap();
    let mut splitted_request: Vec<&[u8]> = request.split(|b| b == &b'$').collect();

    if splitted_request.len() > 1 {
        splitted_request.remove(0);
    } else {
        //revisar este
        println!("Error 4");
        return Err(ParseError::InvalidProtocol("Invalid number of elements".to_string()));
    }
    // valido array size
    if splitted_request.len() != array_size.try_into().unwrap() {
        println!("Error 5");
        return Err(ParseError::InvalidProtocol("Invalid number of elements".to_string()));
    }
    // recorro array para ir decodificando cada elemento
    while (pos+4) < request_vec.len() {
        //valido que haya crlf
        if request[pos+1] != b'\r' {
            println!("Error 6");
            return Err(ParseError::InvalidProtocol("CRLF missing".to_string()));
        }
        pos += 4; // /r/n
        let string_len: usize = request_vec[pos].to_digit(10).unwrap().try_into().unwrap();
        if request[pos+1] != b'\r' {
            println!("Error 6");
            return Err(ParseError::InvalidProtocol("CRLF missing".to_string()));
        }
        pos += 3; // /r/n
        let instruction = &request[pos..pos+string_len];
        let command: String = String::from_utf8_lossy(instruction).to_string().to_lowercase();
        parsed_command.push(command);
        pos += string_len-1;
    }
    if request_vec[request_vec.len()-2] != '\r' {
        println!("Error 7");
        return Err(ParseError::InvalidProtocol("CRLF missing".to_string()));
    }
    println!("command final: {:?}", parsed_command);
    Ok(parsed_command)
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
    let req = b"*2\r\n$6\r\nfoobar\r\n$3\r\nkey\r\n";
    let result = parse(req);
    println!("{:?}", result);
    assert!(result.is_ok());
    match result.unwrap() {
        RESPTypes::RArray(v) => {
            assert_eq!(v, vec![RESPTypes::RBulkString(String::from("foobar")), RESPTypes::RBulkString(String::from("key"))])
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
    let expected = "*2\r\n-Error message1\r\n-Error message2\r\n".to_string();
    assert_eq!(result, expected);
}

#[test]
fn parse_returns_error_when_invalid_command_type() {
    let command = b"+3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
    let result = parse_request(command);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(_) => {}
        _ => assert!(false)
    }
}

#[test]
fn parse_returns_error_when_wrong_number_of_elements() {
    let command = b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n";
    let result = parse_request(command);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(_) => {}
        _ => assert!(false)
    }
}

#[test]
fn parse_returns_error_when_array_of_invalid_type() {
    let command = b"*3\r\n:3\r\nSET\r\n:3\r\nkey\r\n:5\r\nvalue\r\n";
    let result = parse_request(command);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(_) => {}
        _ => assert!(false)
    }
}

#[test]
fn parse_returns_error_when_array_with_some_invalid_type() {
    let command = b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n:5\r\nvalue\r\n";
    let result = parse_request(command);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(_) => {}
        _ => assert!(false)
    }
}

#[test]
fn parse_returns_error_when_invalid_length_of_string() {
    let command = b"*3\r\n$3\r\nSET\r\n$5\r\nkey\r\n$5\r\nvalue\r\n";
    let result = parse_request(command);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(_) => {}
        _ => assert!(false)
    }
}

#[test]
fn parse_returns_error_when_invalid_crlf() {
    let command = b"*3\r\n$3\r\nSET\r\n$3\rkey\r\n$5\r\nvalue\r\n";
    let result = parse_request(command);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(_) => {}
        _ => assert!(false)
    }
}

#[test]
fn parse_returns_error_when_missing_crlf() {
    let command = b"*3\r\n$3\r\nSET$3\r\nkey\r\n$5\r\nvalue\r\n";
    let result = parse_request(command);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(_) => {}
        _ => assert!(false)
    }
}

#[test]
fn parse_returns_error_when_missing_last_crlf() {
    let command = b"*3\r\n$3\r\nSET$3\r\nkey\r\n$5\r\nvalue";
    let result = parse_request(command);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidProtocol(_) => {}
        _ => assert!(false)
    }
}

#[test]
fn parse_returns_ok_when_valid_set_command() {
    let command = b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
    let result = parse_request(command);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["set".to_string(), "key".to_string(), "value".to_string()]);
}

#[test]
fn parse_returns_ok_when_valid_info_command() {
    let command = b"*1\r\n$4\r\nINFO\r\n";
    let result = parse_request(command);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["info".to_string()]);
}

#[test]
fn parse_returns_ok_when_valid_info_with_parameters_command() {
    let command = b"*2\r\n$4\r\nINFO\r\n$6\r\nserver\r\n";
    let result = parse_request(command);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["info".to_string(), "server".to_string()]);
}

