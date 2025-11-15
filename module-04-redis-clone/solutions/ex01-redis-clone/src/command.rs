use crate::db::Db;
use crate::error::{CommandError, DbError};
use crate::resp::RespValue;
use std::time::Duration;

#[derive(Debug)]
pub enum Command {
    // String commands
    Get {
        key: String,
    },
    Set {
        key: String,
        value: Vec<u8>,
        px: Option<u64>,
        ex: Option<u64>,
    },
    Del {
        keys: Vec<String>,
    },
    Exists {
        keys: Vec<String>,
    },
    Expire {
        key: String,
        seconds: u64,
    },
    Ttl {
        key: String,
    },

    // List commands
    LPush {
        key: String,
        values: Vec<Vec<u8>>,
    },
    RPush {
        key: String,
        values: Vec<Vec<u8>>,
    },
    LPop {
        key: String,
        count: Option<usize>,
    },
    RPop {
        key: String,
        count: Option<usize>,
    },
    LRange {
        key: String,
        start: i64,
        stop: i64,
    },
    LLen {
        key: String,
    },

    // Set commands
    SAdd {
        key: String,
        members: Vec<Vec<u8>>,
    },
    SMembers {
        key: String,
    },
    SIsMember {
        key: String,
        member: Vec<u8>,
    },
    SCard {
        key: String,
    },

    // Hash commands
    HSet {
        key: String,
        field: String,
        value: Vec<u8>,
    },
    HGet {
        key: String,
        field: String,
    },
    HGetAll {
        key: String,
    },
    HLen {
        key: String,
    },

    // Server commands
    Ping {
        message: Option<String>,
    },
    Echo {
        message: String,
    },
}

impl Command {
    /// Parse a RESP array into a Command
    pub fn from_resp(resp: RespValue) -> Result<Self, CommandError> {
        let array = resp
            .into_array()
            .map_err(|_| CommandError::InvalidArgument("Expected array".into()))?;

        if array.is_empty() {
            return Err(CommandError::InvalidArgument("Empty command".into()));
        }

        let cmd_name = array[0]
            .as_str()
            .map_err(|_| CommandError::InvalidArgument("Command name must be a string".into()))?
            .to_uppercase();

        match cmd_name.as_str() {
            "GET" => {
                if array.len() != 2 {
                    return Err(CommandError::WrongArity("GET".into()));
                }
                Ok(Command::Get {
                    key: array[1].as_str()?.to_string(),
                })
            }

            "SET" => {
                if array.len() < 3 {
                    return Err(CommandError::WrongArity("SET".into()));
                }

                let key = array[1].as_str()?.to_string();
                let value = array[2].as_bytes()?.to_vec();
                let mut px = None;
                let mut ex = None;

                // Parse optional EX/PX arguments
                let mut i = 3;
                while i < array.len() {
                    let option = array[i].as_str()?.to_uppercase();
                    match option.as_str() {
                        "EX" => {
                            if i + 1 >= array.len() {
                                return Err(CommandError::InvalidArgument("EX needs value".into()));
                            }
                            let seconds = array[i + 1].as_str()?.parse::<u64>().map_err(|_| {
                                CommandError::InvalidArgument("EX value must be integer".into())
                            })?;
                            ex = Some(seconds);
                            i += 2;
                        }
                        "PX" => {
                            if i + 1 >= array.len() {
                                return Err(CommandError::InvalidArgument("PX needs value".into()));
                            }
                            let millis = array[i + 1].as_str()?.parse::<u64>().map_err(|_| {
                                CommandError::InvalidArgument("PX value must be integer".into())
                            })?;
                            px = Some(millis);
                            i += 2;
                        }
                        _ => {
                            return Err(CommandError::InvalidArgument(format!(
                                "Unknown SET option: {}",
                                option
                            )))
                        }
                    }
                }

                Ok(Command::Set { key, value, px, ex })
            }

            "DEL" => {
                if array.len() < 2 {
                    return Err(CommandError::WrongArity("DEL".into()));
                }
                let keys = array[1..]
                    .iter()
                    .map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Command::Del { keys })
            }

            "EXISTS" => {
                if array.len() < 2 {
                    return Err(CommandError::WrongArity("EXISTS".into()));
                }
                let keys = array[1..]
                    .iter()
                    .map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Command::Exists { keys })
            }

            "EXPIRE" => {
                if array.len() != 3 {
                    return Err(CommandError::WrongArity("EXPIRE".into()));
                }
                let key = array[1].as_str()?.to_string();
                let seconds = array[2].as_str()?.parse::<u64>().map_err(|_| {
                    CommandError::InvalidArgument("EXPIRE value must be integer".into())
                })?;
                Ok(Command::Expire { key, seconds })
            }

            "TTL" => {
                if array.len() != 2 {
                    return Err(CommandError::WrongArity("TTL".into()));
                }
                Ok(Command::Ttl {
                    key: array[1].as_str()?.to_string(),
                })
            }

            "LPUSH" => {
                if array.len() < 3 {
                    return Err(CommandError::WrongArity("LPUSH".into()));
                }
                let key = array[1].as_str()?.to_string();
                let values = array[2..]
                    .iter()
                    .map(|v| v.as_bytes().map(|b| b.to_vec()))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Command::LPush { key, values })
            }

            "RPUSH" => {
                if array.len() < 3 {
                    return Err(CommandError::WrongArity("RPUSH".into()));
                }
                let key = array[1].as_str()?.to_string();
                let values = array[2..]
                    .iter()
                    .map(|v| v.as_bytes().map(|b| b.to_vec()))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Command::RPush { key, values })
            }

            "LPOP" => {
                if array.len() < 2 || array.len() > 3 {
                    return Err(CommandError::WrongArity("LPOP".into()));
                }
                let key = array[1].as_str()?.to_string();
                let count = if array.len() == 3 {
                    Some(array[2].as_str()?.parse::<usize>().map_err(|_| {
                        CommandError::InvalidArgument("COUNT must be integer".into())
                    })?)
                } else {
                    None
                };
                Ok(Command::LPop { key, count })
            }

            "RPOP" => {
                if array.len() < 2 || array.len() > 3 {
                    return Err(CommandError::WrongArity("RPOP".into()));
                }
                let key = array[1].as_str()?.to_string();
                let count = if array.len() == 3 {
                    Some(array[2].as_str()?.parse::<usize>().map_err(|_| {
                        CommandError::InvalidArgument("COUNT must be integer".into())
                    })?)
                } else {
                    None
                };
                Ok(Command::RPop { key, count })
            }

            "LRANGE" => {
                if array.len() != 4 {
                    return Err(CommandError::WrongArity("LRANGE".into()));
                }
                let key = array[1].as_str()?.to_string();
                let start = array[2].as_str()?.parse::<i64>().map_err(|_| {
                    CommandError::InvalidArgument("START must be integer".into())
                })?;
                let stop = array[3].as_str()?.parse::<i64>().map_err(|_| {
                    CommandError::InvalidArgument("STOP must be integer".into())
                })?;
                Ok(Command::LRange { key, start, stop })
            }

            "LLEN" => {
                if array.len() != 2 {
                    return Err(CommandError::WrongArity("LLEN".into()));
                }
                Ok(Command::LLen {
                    key: array[1].as_str()?.to_string(),
                })
            }

            "SADD" => {
                if array.len() < 3 {
                    return Err(CommandError::WrongArity("SADD".into()));
                }
                let key = array[1].as_str()?.to_string();
                let members = array[2..]
                    .iter()
                    .map(|v| v.as_bytes().map(|b| b.to_vec()))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Command::SAdd { key, members })
            }

            "SMEMBERS" => {
                if array.len() != 2 {
                    return Err(CommandError::WrongArity("SMEMBERS".into()));
                }
                Ok(Command::SMembers {
                    key: array[1].as_str()?.to_string(),
                })
            }

            "SISMEMBER" => {
                if array.len() != 3 {
                    return Err(CommandError::WrongArity("SISMEMBER".into()));
                }
                let key = array[1].as_str()?.to_string();
                let member = array[2].as_bytes()?.to_vec();
                Ok(Command::SIsMember { key, member })
            }

            "SCARD" => {
                if array.len() != 2 {
                    return Err(CommandError::WrongArity("SCARD".into()));
                }
                Ok(Command::SCard {
                    key: array[1].as_str()?.to_string(),
                })
            }

            "HSET" => {
                if array.len() != 4 {
                    return Err(CommandError::WrongArity("HSET".into()));
                }
                let key = array[1].as_str()?.to_string();
                let field = array[2].as_str()?.to_string();
                let value = array[3].as_bytes()?.to_vec();
                Ok(Command::HSet { key, field, value })
            }

            "HGET" => {
                if array.len() != 3 {
                    return Err(CommandError::WrongArity("HGET".into()));
                }
                let key = array[1].as_str()?.to_string();
                let field = array[2].as_str()?.to_string();
                Ok(Command::HGet { key, field })
            }

            "HGETALL" => {
                if array.len() != 2 {
                    return Err(CommandError::WrongArity("HGETALL".into()));
                }
                Ok(Command::HGetAll {
                    key: array[1].as_str()?.to_string(),
                })
            }

            "HLEN" => {
                if array.len() != 2 {
                    return Err(CommandError::WrongArity("HLEN".into()));
                }
                Ok(Command::HLen {
                    key: array[1].as_str()?.to_string(),
                })
            }

            "PING" => {
                let message = if array.len() > 1 {
                    Some(array[1].as_str()?.to_string())
                } else {
                    None
                };
                Ok(Command::Ping { message })
            }

            "ECHO" => {
                if array.len() != 2 {
                    return Err(CommandError::WrongArity("ECHO".into()));
                }
                Ok(Command::Echo {
                    message: array[1].as_str()?.to_string(),
                })
            }

            _ => Err(CommandError::UnknownCommand(cmd_name)),
        }
    }

    /// Execute the command against the database
    pub async fn execute(self, db: &Db) -> Result<RespValue, DbError> {
        match self {
            Command::Get { key } => match db.get(&key).await? {
                Some(value) => Ok(RespValue::BulkString(Some(value))),
                None => Ok(RespValue::BulkString(None)),
            },

            Command::Set { key, value, px, ex } => {
                db.set(key.clone(), value).await?;

                if let Some(millis) = px {
                    db.expire(&key, Duration::from_millis(millis)).await?;
                } else if let Some(seconds) = ex {
                    db.expire(&key, Duration::from_secs(seconds)).await?;
                }

                Ok(RespValue::SimpleString("OK".to_string()))
            }

            Command::Del { keys } => {
                let mut count = 0;
                for key in keys {
                    if db.del(&key).await? {
                        count += 1;
                    }
                }
                Ok(RespValue::Integer(count))
            }

            Command::Exists { keys } => {
                let mut count = 0;
                for key in keys {
                    if db.exists(&key).await? {
                        count += 1;
                    }
                }
                Ok(RespValue::Integer(count))
            }

            Command::Expire { key, seconds } => {
                let success = db.expire(&key, Duration::from_secs(seconds)).await?;
                Ok(RespValue::Integer(if success { 1 } else { 0 }))
            }

            Command::Ttl { key } => {
                let ttl = db.ttl(&key).await?;
                Ok(RespValue::Integer(ttl))
            }

            Command::LPush { key, values } => {
                let len = db.lpush(&key, values).await?;
                Ok(RespValue::Integer(len as i64))
            }

            Command::RPush { key, values } => {
                let len = db.rpush(&key, values).await?;
                Ok(RespValue::Integer(len as i64))
            }

            Command::LPop { key, count } => match db.lpop(&key, count.unwrap_or(1)).await? {
                Some(values) => {
                    if count.is_some() {
                        let resp_values = values
                            .into_iter()
                            .map(|v| RespValue::BulkString(Some(v)))
                            .collect();
                        Ok(RespValue::Array(Some(resp_values)))
                    } else {
                        Ok(RespValue::BulkString(Some(values.into_iter().next().unwrap())))
                    }
                }
                None => Ok(RespValue::BulkString(None)),
            },

            Command::RPop { key, count } => match db.rpop(&key, count.unwrap_or(1)).await? {
                Some(values) => {
                    if count.is_some() {
                        let resp_values = values
                            .into_iter()
                            .map(|v| RespValue::BulkString(Some(v)))
                            .collect();
                        Ok(RespValue::Array(Some(resp_values)))
                    } else {
                        Ok(RespValue::BulkString(Some(values.into_iter().next().unwrap())))
                    }
                }
                None => Ok(RespValue::BulkString(None)),
            },

            Command::LRange { key, start, stop } => {
                let values = db.lrange(&key, start, stop).await?;
                let resp_values = values
                    .into_iter()
                    .map(|v| RespValue::BulkString(Some(v)))
                    .collect();
                Ok(RespValue::Array(Some(resp_values)))
            }

            Command::LLen { key } => {
                let len = db.llen(&key).await?;
                Ok(RespValue::Integer(len as i64))
            }

            Command::SAdd { key, members } => {
                let count = db.sadd(&key, members).await?;
                Ok(RespValue::Integer(count as i64))
            }

            Command::SMembers { key } => {
                let members = db.smembers(&key).await?;
                let resp_values = members
                    .into_iter()
                    .map(|v| RespValue::BulkString(Some(v)))
                    .collect();
                Ok(RespValue::Array(Some(resp_values)))
            }

            Command::SIsMember { key, member } => {
                let is_member = db.sismember(&key, &member).await?;
                Ok(RespValue::Integer(if is_member { 1 } else { 0 }))
            }

            Command::SCard { key } => {
                let count = db.scard(&key).await?;
                Ok(RespValue::Integer(count as i64))
            }

            Command::HSet { key, field, value } => {
                let is_new = db.hset(&key, field, value).await?;
                Ok(RespValue::Integer(if is_new { 1 } else { 0 }))
            }

            Command::HGet { key, field } => match db.hget(&key, &field).await? {
                Some(value) => Ok(RespValue::BulkString(Some(value))),
                None => Ok(RespValue::BulkString(None)),
            },

            Command::HGetAll { key } => {
                let hash = db.hgetall(&key).await?;
                let mut resp_values = Vec::new();
                for (field, value) in hash {
                    resp_values.push(RespValue::BulkString(Some(field.into_bytes())));
                    resp_values.push(RespValue::BulkString(Some(value)));
                }
                Ok(RespValue::Array(Some(resp_values)))
            }

            Command::HLen { key } => {
                let len = db.hlen(&key).await?;
                Ok(RespValue::Integer(len as i64))
            }

            Command::Ping { message } => match message {
                Some(msg) => Ok(RespValue::BulkString(Some(msg.into_bytes()))),
                None => Ok(RespValue::SimpleString("PONG".to_string())),
            },

            Command::Echo { message } => Ok(RespValue::BulkString(Some(message.into_bytes()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get() {
        let resp = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"GET".to_vec())),
            RespValue::BulkString(Some(b"mykey".to_vec())),
        ]));

        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(cmd, Command::Get { key } if key == "mykey"));
    }

    #[test]
    fn test_parse_set_with_px() {
        let resp = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"SET".to_vec())),
            RespValue::BulkString(Some(b"mykey".to_vec())),
            RespValue::BulkString(Some(b"myvalue".to_vec())),
            RespValue::BulkString(Some(b"PX".to_vec())),
            RespValue::BulkString(Some(b"1000".to_vec())),
        ]));

        let cmd = Command::from_resp(resp).unwrap();
        assert!(
            matches!(cmd, Command::Set { key, value, px, .. }
                if key == "mykey" && value == b"myvalue" && px == Some(1000))
        );
    }

    #[test]
    fn test_parse_del() {
        let resp = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"DEL".to_vec())),
            RespValue::BulkString(Some(b"key1".to_vec())),
            RespValue::BulkString(Some(b"key2".to_vec())),
        ]));

        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(cmd, Command::Del { keys } if keys.len() == 2));
    }
}
