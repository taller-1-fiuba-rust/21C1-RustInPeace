use crate::Config;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;

pub fn load_config(file_path: String) -> Result<Config, Error> {
    let mut config_values: Vec<String> = [].to_vec();
    let configuration_values = lines_from_file(file_path);
    match configuration_values {
        Ok(strings) => config_values = strings,
        Err(_) => {
            println!("error")
        }
    };
    let mut map: HashMap<String, String> = HashMap::new();
    let mut vec_aux: Vec<String>;
    for line in config_values {
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
    let configuracion = Config::new(verbose, port, timeout, dbfilename, logfile);
    //pongo estos prints para que no tire warning de funciones sin usar (las uso en los tests)
    println!("{}", configuracion.get_dbfilename());
    println!("{}", configuracion.get_verbose());
    println!("{}", configuracion.get_logfile());
    println!("{}", configuracion.get_port());
    println!("{}", configuracion.get_timeout());

    Ok(configuracion)
}

fn lines_from_file(file_path: String) -> Result<Vec<String>, Error> {
    let file = File::open(file_path)?;
    let f = BufReader::new(file);
    let lines: Vec<String> = f.lines().collect::<Result<_, _>>()?;
    Ok(lines)
}

fn get_verbose_ready(map: &HashMap<String, String>) -> usize {
    let mut verbose: usize = 0;
    let verbose_aux = map.get("verbose");
    if let Some(my_string) = verbose_aux {
        let verb_aux = my_string.parse::<usize>();
        match verb_aux {
            Ok(verb) => verbose = verb,
            Err(_) => println!("parsing error"),
        }
    }
    verbose
}

fn get_timeout_ready(map: &HashMap<String, String>) -> u64 {
    let mut timeout: u64 = 200;
    let timeout_aux = map.get("timeout");
    if let Some(my_string) = timeout_aux {
        let tmt_aux = my_string.parse::<u64>();
        match tmt_aux {
            Ok(tmt) => timeout = tmt,
            Err(_) => println!("parsing error"),
        }
    }
    timeout
}

fn get_port_ready(map: &HashMap<String, String>) -> String {
    let mut port: String = "8080".to_string();
    let port_aux = map.get("port");
    if let Some(my_string) = port_aux {
        port = my_string.to_string()
    }
    port
}

fn get_dbfilename_ready(map: &HashMap<String, String>) -> String {
    let mut dbfilename: String = "dump.rdb".to_string();
    let dbfilename_aux = map.get("dbfilename");
    if let Some(my_string) = dbfilename_aux {
        dbfilename = my_string.to_string()
    }
    dbfilename
}

fn get_logfile_ready(map: &HashMap<String, String>) -> String {
    let mut logfile: String = "/var/log/redis/redis-server.log".to_string();
    let logfile_aux = map.get("logfile");
    if let Some(my_string) = logfile_aux {
        logfile = my_string.to_string()
    }
    logfile
}

#[test]
fn test_01_se_lee_un_archivo_de_5_lineas_y_se_devuelve_un_vector_de_5_elementos() {
    let path = "./src/redis.txt";
    let config_values = lines_from_file(path.to_string());
    assert_eq!(config_values.unwrap().len(), 5)
}
#[test]
fn test_02_se_lee_un_archivo_y_se_obtiene_primer_elemento_correctamente() {
    let path = "./src/redis.txt";
    let config_values = lines_from_file(path.to_string());
    assert_eq!(config_values.unwrap()[0], "verbose 1".to_string())
}
#[test]
fn text_03_se_genera_el_config_y_se_obtiene_el_valor_del_port_en_fmt_string() {
    let path = "./src/redis.txt";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let port = conf.get_port();
    assert_eq!(*port, "8080")
}
#[test]
fn text_04_se_genera_el_config_y_se_obtiene_el_valor_del_verbose_en_fmt_usize() {
    let path = "./src/redis.txt";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let verbose = conf.get_verbose();
    assert_eq!(*verbose, 1)
}
#[test]
fn text_05_se_genera_el_config_y_se_obtiene_el_valor_del_timeout_en_fmt_u64() {
    let path = "./src/redis.txt";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let timeout = conf.get_timeout();
    assert_eq!(*timeout, 300)
}
#[test]
fn text_06_se_genera_el_config_y_se_obtiene_el_valor_del_dbfilename_en_fmt_string() {
    let path = "./src/redis.txt";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let dbfilename = conf.get_dbfilename();
    assert_eq!(*dbfilename, "dump.rdb".to_string())
}
#[test]
fn text_07_se_genera_el_config_y_se_obtiene_el_valor_del_logfile_en_fmt_string() {
    let path = "./src/redis.txt";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let logfile = conf.get_logfile();
    assert_eq!(*logfile, "/var/log/redis/redis-server.log".to_string())
}

#[test]
fn text_file_loading_failure() {
    assert!(lines_from_file("test_data/".to_string()).is_err());
}
