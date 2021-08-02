use crate::services::parser_service;
use super::utils::resp_type::RespType;

/// Obtiene el body de una solicitud HTTP.
pub fn parse_request(req: &[u8]) -> String {
    println!("Request: {}", String::from_utf8_lossy(&req[..]));

    let mut parameters = Vec::new();
    let mut pos = 0;
    while pos < req.len() {
        let crlf_pos = parser_service::search_crlf(&req[pos..]);
        if crlf_pos.is_err() {
            parameters.push(String::from_utf8_lossy(&req[pos..]));
            pos = req.len();
        } else {
            let c = crlf_pos.unwrap();
            parameters.push(String::from_utf8_lossy(&req[pos..c+pos]));
            pos += c;
            pos += 2;
        }
    }

    let cmd = parameters.last().unwrap()[4..].to_string().replace("\u{0}", "");
    return cmd;
}

/// Transforma el body de una solicitud en un string separado por espacios en blanco.
pub fn get_body_as_string(string: &String) -> String {
    return string.to_string().replace("+", " ");
}

/// Transforma el body de una solicitud en un vector de strings.
pub fn get_body_as_resp(string: String) -> RespType {
    return RespType::RArray(string.split('+').collect::<Vec<&str>>().iter().map(|e| RespType::RBulkString(e.to_string())).collect());
}


#[test]
fn test_parse_request() {
    let req = b"POST / HTTP/1.1\r\nHost: 127.0.0.1:8080\r\nConnection: keep-alive\r\nContent-Length: 11\r\nCache-Control: max-age=0\r\nsec-ch-ua: \" Not;A Brand\";v=\"99\", \"Google Chrome\";v=\"91\", \"Chromium\";v=\"91\"\r\nsec-ch-ua-mobile: ?0\r\nUpgrade-Insecure-Requests: 1\r\nOrigin: http://127.0.0.1:8080\r\nContent-Type: application/x-www-form-urlencoded\r\nUser-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9\r\nSec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-User: ?1\r\nSec-Fetch-Dest: document\r\nReferer: http://127.0.0.1:8080/\r\nAccept-Encoding: gzip, deflate, br\r\nAccept-Language: es-ES,es;q=0.9,en;q=0.8\r\n\r\ncmd=set+nombre+juan\r\n";
    let cmd = parse_request(req);
    assert_eq!(cmd, "set+nombre+juan".to_string());
}



