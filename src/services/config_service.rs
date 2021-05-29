use std::io::Error;
use std::fs::File;
use std::io::Read;
use std::io::BufReader;
use std::collections::HashMap;
use crate::Config;
use std::io::BufRead;


pub fn load_config(config_values:Vec<String>)->Result<Config,Error>{
    let mut map: HashMap<String, String> = HashMap::new();
    let mut vec_aux:Vec<String>;
    for line in config_values{
        vec_aux = line.split_whitespace().map(|s| s.to_string()).collect();
        let key = vec_aux[0].to_string();
        let value = vec_aux[1].to_string();
        map.entry(key).or_insert(value);
    }
    let verbose = get_verbose_ready(&map);
    let port = get_port_ready(&map);
    let timeout = get_timeout_ready(&map);
    let dbfilename = get_dbfilename_ready(&map);
    let logfile = get_logfile_ready(&map);
    let configuracion = Config::new(verbose, port, timeout,
    dbfilename, logfile);
    Ok(configuracion)
}

fn lines_from_file(file_path: String) -> Result<Vec<String>,Error> {
    let file = File::open(file_path)?;
    let f = BufReader::new(file);
    let lines: Vec<String> = f.lines().collect::<Result<_, _>>()?;
    Ok(lines)
}

fn get_verbose_ready(map:&HashMap<String, String>) -> usize {
    let mut verbose:usize = 0;
    let verbose_aux = map.get("verbose");
    match verbose_aux {
        Some(my_string)=>{ 
            let verb_aux = my_string.parse::<usize>();
            match verb_aux {
                Ok(verb) => verbose = verb,
                Err(_) => println!("parse error")
            }
        }
        None => {
        }
    }
    verbose
}

fn get_timeout_ready(map:&HashMap<String, String>) -> u64 {
    let mut timeout:u64 = 200;
    let timeout_aux = map.get("timeout");
    match timeout_aux {
        Some(my_string)=>{ 
            let tmt_aux = my_string.parse::<u64>();
            match tmt_aux {
                Ok(tmt) => timeout = tmt,
                Err(_) => println!("parse error")
            }
        }
        None => {
        }
    }
    timeout
}

fn get_port_ready(map:&HashMap<String, String>) -> String {
    let mut port:String = "8080".to_string();
    let port_aux = map.get("port");
    match port_aux {
        Some(my_string)=> {
            port = my_string.to_string() 
        }
        None => {
        }
    }
    port
}

fn get_dbfilename_ready(map:&HashMap<String, String>) -> String {
    let mut dbfilename:String = "dump.rdb".to_string();
    let dbfilename_aux = map.get("dbfilename");
    match dbfilename_aux {
        Some(my_string)=> {
            dbfilename = my_string.to_string() 
        }
        None => {
        }
    }
    dbfilename
}

fn get_logfile_ready(map:&HashMap<String, String>) -> String {
    let mut logfile:String = "/var/log/redis/redis-server.log".to_string();
    let logfile_aux = map.get("logfile");
    match logfile_aux {
        Some(my_string)=> {
            logfile = my_string.to_string() 
        }
        None => {
        }
    }
    logfile
}


#[test]
fn test_01_se_lee_un_archivo_de_5_lineas_y_se_devuelve_un_vector_de_5_elementos(){
    let path = "C:/Users/Ron Heyes/UBA/TALLER DE PROGRAMACION/TP/RustInPeace/src/redis.txt";
    let config_values = lines_from_file(path.to_string());
    assert_eq!(config_values.unwrap().len(),5)
}
#[test]
fn test_02_se_lee_un_archivo_y_se_obtiene_primer_elemento_correctamente(){
    let path = "C:/Users/Ron Heyes/UBA/TALLER DE PROGRAMACION/TP/RustInPeace/src/redis.txt";
    let config_values = lines_from_file(path.to_string());
    assert_eq!(config_values.unwrap()[0],"verbose 1".to_string())
}
#[test]
fn text_03_se_genera_el_config_y_se_obtiene_el_valor_del_port_en_fmt_string(){
    let path = "C:/Users/Ron Heyes/UBA/TALLER DE PROGRAMACION/TP/RustInPeace/src/redis.txt";
    let config_values = lines_from_file(path.to_string()).unwrap();
    let config = load_config(config_values);
    let port = config.unwrap().get_port();
    assert_eq!(port,"8080")
}
#[test]
fn text_04_se_genera_el_config_y_se_obtiene_el_valor_del_verbose_en_fmt_size(){
    let path = "C:/Users/Ron Heyes/UBA/TALLER DE PROGRAMACION/TP/RustInPeace/src/redis.txt";
    let config_values = lines_from_file(path.to_string()).unwrap();
    let config = load_config(config_values);
    let verbose = config.unwrap().get_verbose();
    assert_eq!(verbose,1)
}
#[test]
fn text_05_se_genera_el_config_y_se_obtiene_el_valor_del_timeout_en_fmt_u64(){
    let path = "C:/Users/Ron Heyes/UBA/TALLER DE PROGRAMACION/TP/RustInPeace/src/redis.txt";
    let config_values = lines_from_file(path.to_string()).unwrap();
    let config = load_config(config_values);
    let timeout = config.unwrap().get_timeout();
    assert_eq!(timeout,300)
}
#[test]
fn text_06_se_genera_el_config_y_se_obtiene_el_valor_del_dbfilename_en_fmt_string(){
    let path = "C:/Users/Ron Heyes/UBA/TALLER DE PROGRAMACION/TP/RustInPeace/src/redis.txt";
    let config_values = lines_from_file(path.to_string()).unwrap();
    let config = load_config(config_values);
    let dbfilename = config.unwrap().get_dbfilename();
    assert_eq!(dbfilename,"dump.rdb".to_string())
}
#[test]
fn text_07_se_genera_el_config_y_se_obtiene_el_valor_del_logfile_en_fmt_string(){
    let path = "C:/Users/Ron Heyes/UBA/TALLER DE PROGRAMACION/TP/RustInPeace/src/redis.txt";
    let config_values = lines_from_file(path.to_string()).unwrap();
    let config = load_config(config_values);
    let logfile = config.unwrap().get_logfile();
    assert_eq!(logfile,"/var/log/redis/redis-server.log".to_string())
}

#[test]
fn text_file_loading_failure(){
    assert!(lines_from_file("test_data/".to_string()).is_err());
}
