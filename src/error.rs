use std::env::VarError;
use std::fmt;
use std::io::Error as IoError;

use diesel::r2d2::PoolError;
use diesel::result::Error as QueryError;
use diesel_migrations::RunMigrationsError;
use serenity::Error as SerenityError;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Io(IoError),
    Database(DatabaseError),
    Serenity(SerenityError),
    Env(&'static str, VarError),
}

#[derive(Debug)]
pub enum DatabaseError {
    Connection(PoolError),
    Query(QueryError),
    Migrations(RunMigrationsError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(e) => e.fmt(fmt),
            Error::Io(e) => write!(fmt, "IO error: {}", e),
            Error::Serenity(e) => e.fmt(fmt),
            Error::Database(DatabaseError::Connection(e)) => e.fmt(fmt),
            Error::Database(DatabaseError::Query(e)) => e.fmt(fmt),
            Error::Database(DatabaseError::Migrations(e)) => e.fmt(fmt),
            Error::Env(key, VarError::NotPresent) => {
                write!(fmt, "Environment variable '{}' not found", key)
            }
            Error::Env(key, VarError::NotUnicode(_)) => {
                write!(fmt, "Environment variable '{}' was not valid unicode", key)
            }
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Message(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Error {
        Error::Message(s.to_string())
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<SerenityError> for Error {
    fn from(e: SerenityError) -> Error {
        Error::Serenity(e)
    }
}

impl From<PoolError> for Error {
    fn from(e: PoolError) -> Error {
        Error::Database(DatabaseError::Connection(e))
    }
}

impl From<QueryError> for Error {
    fn from(e: QueryError) -> Error {
        Error::Database(DatabaseError::Query(e))
    }
}

impl From<RunMigrationsError> for Error {
    fn from(e: RunMigrationsError) -> Error {
        Error::Database(DatabaseError::Migrations(e))
    }
}
