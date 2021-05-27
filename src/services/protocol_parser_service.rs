use std::error::Error;
use std::fmt;
use std::convert::TryInto;
use crate::entities::error::ResponseError;
use crate::entities::error::ParseError;
use crate::entities::datatype_trait::DataType;

#[derive(Debug)]
pub enum RESPTypes {
    RSimpleString(String),
    RError(String),
    RInteger(usize),
    RBulkString(String),
    RArray(Vec<RESPTypes>)
}

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

impl Error for ResponseError {
    fn description(&self) -> &str {
        match &*self {
            ResponseError::GenericError(message) => &message
        }
    }
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ResponseError occurred")
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

impl DataType for RESPTypes {
    fn deserialize(self) -> String {
        println!("aca {:?}", self);
        match self {
            RESPTypes::RBulkString(string) => {
                format!("${:?}\r\n{}\r\n", string.len(), string)
            }
            RESPTypes::RInteger(integer) => {
                format!(":{}\r\n", integer)
            }
            RESPTypes::RSimpleString(string) => {
                println!("recibo: {}", string);
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
                format!("-Error {}\r\n", message)
            }
        }
    }
}

pub fn parse_response(response: RESPTypes) -> String {
    return response.deserialize();
}

#[test]
fn parse_response_string_ok() {
    let result = parse_response(RESPTypes::RSimpleString(String::from("test")));
    let expected = "+test\r\n".to_string();
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
    let result = parse_response(RESPTypes::RError("Some error".to_string()));
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

