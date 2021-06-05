use crate::domain::entities::config::Config;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;

pub fn load_config(file_path: String) -> Result<Config, Error> {
    let config_values: Vec<String>;
    let configuration_values = lines_from_file(file_path);
    match configuration_values {
        Ok(strings) => config_values = strings,
        Err(e) => {
            return Err(e);
        }
    };
    let mut vec_aux: Vec<String>;
    let mut verbose = 0;
    let mut timeout = 0;
    let mut port = "8080".to_string();
    let mut dbfilename = "".to_string();
    let mut logfile = "".to_string();

    for line in config_values {
        vec_aux = line.split_whitespace().map(|s| s.to_string()).collect();
        match vec_aux[0].as_str() {
            "verbose" => {
                if vec_aux.len() == 2 {
                    verbose = parse_verbose(vec_aux[1].to_string());
                }
            }
            "timeout" => {
                if vec_aux.len() == 2 {
                    timeout = parse_timeout(vec_aux[1].to_string())
                }
            }
            "port" => {
                if vec_aux.len() == 2 {
                    port = vec_aux[1].to_string();
                }
            }
            "dbfilename" => {
                if vec_aux.len() == 2 {
                    dbfilename = vec_aux[1].to_string()
                }
            }
            "logfile" => {
                if vec_aux.len() == 2 {
                    logfile = vec_aux[1].to_string()
                }
            }
            _ => {
                println!("no field")
            }
        }
    }
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

fn parse_verbose(string: String) -> usize {
    let mut verbose: usize = 1;
    let verb_aux = string.parse::<usize>();
    match verb_aux {
        Ok(verb) => verbose = verb,
        Err(_) => println!("parsing error"),
    }
    verbose
}

fn parse_timeout(string: String) -> u64 {
    let mut timeout: u64 = 200;
    let timeout_aux = string.parse::<u64>();
    match timeout_aux {
        Ok(tmt) => timeout = tmt,
        Err(_) => println!("parsing error"),
    }
    timeout
}

#[test]
fn test_01_se_lee_un_archivo_de_5_lineas_y_se_devuelve_un_vector_de_5_elementos() {
    let path = "./src/redis.conf";
    let config_values = lines_from_file(path.to_string());
    assert_eq!(config_values.unwrap().len(), 5)
}
#[test]
fn test_02_se_lee_un_archivo_y_se_obtiene_primer_elemento_correctamente() {
    let path = "./src/redis.conf";
    let config_values = lines_from_file(path.to_string());
    assert_eq!(config_values.unwrap()[0], "verbose 1".to_string())
}
#[test]
fn text_03_se_genera_el_config_y_se_obtiene_el_valor_del_port_en_fmt_string() {
    let path = "./src/redis.conf";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let port = conf.get_port();
    assert_eq!(*port, "8080")
}
#[test]
fn text_04_se_genera_el_config_y_se_obtiene_el_valor_del_verbose_en_fmt_usize() {
    let path = "./src/redis.conf";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let verbose = conf.get_verbose();
    assert_eq!(*verbose, 1)
}
#[test]
fn text_05_se_genera_el_config_y_se_obtiene_el_valor_del_timeout_en_fmt_u64() {
    let path = "./src/redis.conf";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let timeout = conf.get_timeout();
    assert_eq!(*timeout, 300)
}
#[test]
fn text_06_se_genera_el_config_y_se_obtiene_el_valor_del_dbfilename_en_fmt_string() {
    let path = "./src/redis.conf";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let dbfilename = conf.get_dbfilename();
    assert_eq!(*dbfilename, "dump.rdb".to_string())
}
#[test]
fn text_07_se_genera_el_config_y_se_obtiene_el_valor_del_logfile_en_fmt_string() {
    let path = "./src/redis.conf";
    let config = load_config(path.to_string());
    let conf = config.unwrap();
    let logfile = conf.get_logfile();
    assert_eq!(*logfile, "/var/log/redis/redis-server.log".to_string())
}

#[test]
fn text_file_loading_failure() {
    assert!(lines_from_file("test_data/".to_string()).is_err());
}
