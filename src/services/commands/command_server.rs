//! Servicio que implementa todos los comandos de tipo Server

use crate::domain::entities::config::Config;
use crate::domain::entities::message::WorkerMessage;
use crate::domain::implementations::database::Database;
use crate::services::utils::glob_pattern;
use crate::services::utils::resp_type::RespType;
use std::net::SocketAddr;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, RwLock};

/// Pasa al cliente a estado "monitor".
///
/// Envía una directiva al server para que el cliente pase a un estado pasivo donde solo reciba una copia en tiempo real de
/// todos los comandos que se envíen al servidor.
pub fn monitor(tx: &Sender<WorkerMessage>, addrs: SocketAddr) {
    tx.send(WorkerMessage::SetMonitor(addrs)).unwrap();
}

/// Devuelve información y estadísticas sobre el servidor.
///
/// Mediante un parámetro opcional se puede especificar el tipo de información.
/// Los parámetros posibles son:
/// * server: Información general sobre el servidor Redis
/// * clients: Información sobre clientes conectados
/// * memory: Información sobre el consumo de memoria
/// * persistence: Información sobre RDB
/// * stats: Estadísticas generales
/// * replication: Información de replicación
/// * cpu: Estadísticas del consumo de CPU
/// * commandstats: Estadisticas de comandos Redis
/// * cluster: Cluster Redis
/// * modules: Modulos
/// * keyspace: Estadisticas relacionadas a la base de datos
/// * errorstats: Estadisticas de errores Redis
/// * all: Todas las secciones de información, excluyendo módulos
/// * everything: Todas las secciones de información, incluyendo módulos
/// Si no se especifica ningún parámetro, se retorna toda la información (all).
pub fn info(cmd: &[RespType], tx: &Sender<WorkerMessage>) -> RespType {
    if cmd.len() == 2 {
        if let RespType::RBulkString(section) = &cmd[1] {
            match section.as_str() {
                "server" => return RespType::RBulkString(get_server_info(tx)),
                "clients" => return RespType::RBulkString(get_clients_info(tx)),
                "memory" => return RespType::RBulkString(get_memory_info()),
                "persistence" => return RespType::RBulkString(get_persistence_info()),
                "stats" => return RespType::RBulkString(get_stats_info(tx)),
                "replication" => return RespType::RBulkString(get_replication_info()),
                "cpu" => return RespType::RBulkString(get_cpu_info()),
                "commandstats" => return RespType::RBulkString(get_commandstats_info()),
                "cluster" => return RespType::RBulkString(get_cluster_info()),
                "modules" => return RespType::RBulkString(get_modules_info()),
                "keyspace" => return RespType::RBulkString(get_keyspace_info()),
                "errorstats" => return RespType::RBulkString(get_errorstats_info()),
                "all" => return RespType::RBulkString(get_all_info(tx)),
                "everything" => return RespType::RBulkString(get_everything_info(tx)),
                _ => return RespType::RNullBulkString(),
            }
        }
    } else if cmd.len() == 1 {
        return RespType::RBulkString(get_all_info(tx));
    }
    RespType::RNullBulkString()
}

/// Devuelve toda la información y estadísticas del servidor, incluidos los módulos.
fn get_everything_info(tx: &Sender<WorkerMessage>) -> String {
    let info = format!(
        "{}{}{}{}{}{}{}{}{}{}{}{}",
        get_server_info(tx),
        get_clients_info(tx),
        get_memory_info(),
        get_persistence_info(),
        get_stats_info(tx),
        get_replication_info(),
        get_cpu_info(),
        get_commandstats_info(),
        get_cluster_info(),
        get_modules_info(),
        get_keyspace_info(),
        get_errorstats_info()
    );
    info
}

/// Devuelve toda la información y estadísticas del servidor, excluyendo los módulos.
fn get_all_info(tx: &Sender<WorkerMessage>) -> String {
    let info = format!(
        "{}{}{}{}{}{}{}{}{}{}{}",
        get_server_info(tx),
        get_clients_info(tx),
        get_memory_info(),
        get_persistence_info(),
        get_stats_info(tx),
        get_replication_info(),
        get_cpu_info(),
        get_commandstats_info(),
        get_cluster_info(),
        get_keyspace_info(),
        get_errorstats_info()
    );
    info
}

/// Devuelve información general del servidor.
fn get_server_info(tx: &Sender<WorkerMessage>) -> String {
    let (info_tx, info_rx) = mpsc::channel();
    tx.send(WorkerMessage::InfoServer(info_tx)).unwrap();
    if let Ok(info) = info_rx.recv() {
        return info;
    }
    String::from("# Server\r\n")
}

/// Devuelve información sobre los clientes conectados al servidor.
fn get_clients_info(tx: &Sender<WorkerMessage>) -> String {
    let (info_tx, info_rx) = mpsc::channel();
    tx.send(WorkerMessage::InfoClients(info_tx)).unwrap();
    if let Ok(info) = info_rx.recv() {
        return info;
    }
    String::from("# Clients\r\n")
}

/// Devuelve estadísticas sobre el uso del servidor.
fn get_stats_info(tx: &Sender<WorkerMessage>) -> String {
    let (info_tx, info_rx) = mpsc::channel();
    tx.send(WorkerMessage::InfoStats(info_tx)).unwrap();
    if let Ok(info) = info_rx.recv() {
        return info;
    }
    String::from("# Stats\r\n")
}

/// Devuelve información sobre el uso de memoria.
fn get_memory_info() -> String {
    String::from("# Memory\r\nused_memory:200001640\r\nused_memory_peak:201552616\r\nused_memory_overhead:49769056\r\nused_memory_startup:809888\r\nallocator_allocated:199981744\r\ntotal_system_memory:16596955136\r\nnumber_of_cached_scripts:0\r\nmaxmemory:200000000\r\nmaxmemory_policy:allkeys-lru\r\nmem_clients_slaves:0\r\nmem_clients_normal:41008\r\nlazyfree_pending_objects:0\r\nlazyfreed_objects:0\r\n")
}

/// Devuelve información sobre RDB
fn get_persistence_info() -> String {
    String::from("# Persistence\r\nloading:0\r\ncurrent_cow_size:0\r\ncurrent_save_keys_processed:0\r\ncurrent_save_keys_total:0\r\nrdb_changes_since_last_save:13896285\r\nrdb_bgsave_in_progress:0\r\nrdb_last_save_time:1622557783\r\nrdb_last_bgsave_status:ok\r\nrdb_last_bgsave_time_sec:-1\r\nrdb_current_bgsave_time_sec:-1\r\nrdb_last_cow_size:0\r\n")
}

/// Devuelve información sobre replicación
fn get_replication_info() -> String {
    String::from("# Replication\r\nrole:master\r\nconnected_slaves:0\r\nmaster_failover_state:no-failover\r\nmaster_replid:435ed80c4d896f3b6e3a19e10951bacd77bef04b\r\nmaster_replid2:0000000000000000000000000000000000000000\r\nmaster_repl_offset:0\r\nsecond_repl_offset:-1\r\nrepl_backlog_active:0\r\nrepl_backlog_size:1048576\r\nrepl_backlog_first_byte_offset:0\r\nrepl_backlog_histlen:0\r\n")
}

/// Devuelve información sobre el uso de CPU
fn get_cpu_info() -> String {
    String::from("# CPU\r\nused_cpu_sys:2986.579746\r\nused_cpu_user:21613.917583\r\nused_cpu_sys_children:0.000000\r\nused_cpu_user_children:0.000000\r\nused_cpu_sys_main_thread:2956.113647\r\nused_cpu_user_main_thread:21569.740440\r\n")
}

/// Devuelve estadísticas de comandos
fn get_commandstats_info() -> String {
    String::from("# Commandstats\r\ncmdstat_TYPE: calls=1231,rejected_calls=123,failed_calls=32")
}

/// Devuelve información de clusters
fn get_cluster_info() -> String {
    String::from("# Cluster\r\ncluster_enabled:0\r\n")
}

/// Devuelve información de módulos
fn get_modules_info() -> String {
    String::from("# Modules\r\n")
}

/// Devuelve estadísticas de la base de datos
fn get_keyspace_info() -> String {
    String::from("# Keyspace\r\ndb0:keys=1012012,expires=1362,avg_ttl=31334306430714\r\n")
}

/// Devuelve estadísticas de errores
fn get_errorstats_info() -> String {
    String::from("# Errorstats\r\nerrorstat_ERR:count=14072\r\nerrorstat_WRONGTYPE:count=4445\r\n")
}

/// Retorna la cantidad de claves almacenadas.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
/// # use proyecto_taller_1::services::commands::command_server;
///
/// # let db = Database::new("dummy_db_dbsize.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()]),
///     KeyAccessTime::Persistent
/// ));
/// database.write().unwrap().add("nombre".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType("fruta".to_string()),
///     KeyAccessTime::Persistent
/// ));
///
/// let dbsize = command_server::dbsize(&database);
/// assert_eq!(dbsize, RespType::RInteger(2));
/// # std::fs::remove_file("dummy_db_dbsize.csv").unwrap();
/// ```
pub fn dbsize(database: &Arc<RwLock<Database>>) -> RespType {
    RespType::RInteger(database.read().unwrap().get_size())
}

/// Elimina todas claves y valores almacenados.
///
/// Elimina todas las claves y valores almacenados en la base de datos
/// Devuelve el mensaje "Erased database"
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
/// # use proyecto_taller_1::services::commands::command_server;
///
/// # let db = Database::new("dummy_db_flushdb.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()]),
///     KeyAccessTime::Persistent
/// ));
/// database.write().unwrap().add("nombre".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType("fruta".to_string()),
///     KeyAccessTime::Persistent
/// ));
///
/// let removed = command_server::flushdb(&database);
/// assert_eq!(removed, RespType::RBulkString("Erased database".to_string()));
/// assert_eq!(command_server::dbsize(&database), RespType::RInteger(0));
/// # std::fs::remove_file("dummy_db_flushdb.csv").unwrap();
/// ```
pub fn flushdb(database: &Arc<RwLock<Database>>) -> RespType {
    let mut new_database = database.write().unwrap();
    new_database.clean_items();
    RespType::RBulkString("Erased database".to_string())
}

/// Retorna los parámetros de configuración del servidor.
///
/// Busca en la configuración el valor del atributo especificado
/// En caso de encontrarlo, lo devuelve, sino devuelve Error.
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::entities::config::Config;
/// # use proyecto_taller_1::services::commands::command_server;
/// # use std::sync::{Arc, RwLock};
///
/// # std::fs::File::create("./src/dummy_config_get.txt").unwrap();
/// let mut config = Config::new("./src/dummy_config_get.txt".to_string());
/// config
/// .set_attribute(String::from("maxmemory"), String::from("2mb"))
/// .unwrap();
///
/// let mut c = Arc::new(RwLock::new(config));
/// let res = command_server::config_get(&c, &vec![RespType::RBulkString(String::from("get")), RespType::RBulkString(String::from("maxmemory"))]);
///
/// assert_eq!(res, RespType::RArray(vec![RespType::RBulkString("maxmemory".to_string()), RespType::RBulkString("2mb".to_string())]));
/// # std::fs::remove_file("./src/dummy_config_get.txt").unwrap();
/// ```
pub fn config_get(config: &Arc<RwLock<Config>>, cmd: &[RespType]) -> RespType {
    if cmd.len() == 2 {
        if let RespType::RBulkString(field_name) = &cmd[1] {
            if let Ok(conf) = config.read() {
                let mut matches = Vec::new();
                conf.get_all_attributes().iter().for_each(|attribute| {
                    if glob_pattern::g_match(
                        field_name.as_bytes(),
                        attribute.0.to_owned().as_bytes(),
                    ) {
                        matches.push(RespType::RBulkString(attribute.0.to_owned()));
                        matches.push(RespType::RBulkString(attribute.1.to_owned()));
                    }
                });
                return RespType::RArray(matches);
            }
            return RespType::RError(String::from("Parameter missing"));
        }
    }
    RespType::RError(String::from("Invalid request"))
}

/// Reconfigura parámetros de configuración.
///
/// Configura el campo `field` con el valor especificado.
/// En caso de exito devuelve "Ok", sino devuelve Error
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::entities::config::Config;
/// # use proyecto_taller_1::services::commands::command_server;
/// # use std::sync::{Arc, RwLock};
///
/// # std::fs::File::create("./src/dummy_config_set.txt").unwrap();
/// let mut config = Config::new("./src/dummy_config_set.txt".to_string());
/// let mut c = Arc::new(RwLock::new(config));
///
/// let res = command_server::config_set(&c, &vec![RespType::RBulkString(String::from("set")), RespType::RBulkString(String::from("maxmemory")), RespType::RBulkString(String::from("2mb"))]);
/// assert_eq!(res, RespType::RSimpleString("Ok".to_string()));
///
/// let res = command_server::config_get(&c, &vec![RespType::RBulkString(String::from("get")), RespType::RBulkString(String::from("maxmemory"))]);
/// assert_eq!(res, RespType::RArray(vec![RespType::RBulkString("maxmemory".to_string()), RespType::RBulkString("2mb".to_string())]));
/// # std::fs::remove_file("./src/dummy_config_set.txt").unwrap();
/// ```
pub fn config_set(config: &Arc<RwLock<Config>>, cmd: &[RespType]) -> RespType {
    if cmd.len() == 3 {
        if let RespType::RBulkString(field) = &cmd[1] {
            if let RespType::RBulkString(value) = &cmd[2] {
                if let Ok(mut conf) = config.write() {
                    match conf.set_attribute(String::from(field), String::from(value)) {
                        Ok(_) => {
                            return RespType::RSimpleString(String::from("Ok"));
                        }
                        Err(e) => {
                            return RespType::RError(e.to_string());
                        }
                    }
                }
            }
        }
    }
    RespType::RError(String::from("Invalid request"))
}
