use std::sync::{Arc, RwLock};

use crate::{domain::implementations::database::Database, services::utils::resp_type::RespType};

pub fn scard(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            return RespType::RInteger(db.get_len_of_set(key));
        }
    }
    RespType::RInteger(0)
}

pub fn sismember(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(member) = &cmd[2] {
                return RespType::RInteger(db.is_member_of_set(key, member));
            }
        }
    }
    RespType::RInteger(0)
}

pub fn smembers(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut final_members = Vec::new();
            let mut db = database.write().unwrap();
            let members = db.get_members_of_set(key);
            members
                .iter()
                .for_each(|member| final_members.push(RespType::RBulkString(member.to_string())));
            return RespType::RArray(final_members);
        }
    }
    RespType::RInteger(0)
}
