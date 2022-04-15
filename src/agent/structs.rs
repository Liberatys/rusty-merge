use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Request {
    pub protocol: u32,
    pub body: RequestBody,
}

/// Represents the kinds of requests understood by the agent.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "body")]
pub enum RequestBody {
    Push(String),
    Pop(String),
    ForceProcess,
    Clear,
    List,
    Quit,
}

/// Represents the kinds of responses sent by the agent.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "body")]
pub enum Response {
    /// A successful request, with some request-specific response data.
    Success(String),

    List(Vec<String>),

    /// A failed request, of `FailureKind`.
    Failure(FailureKind),
}

/// Represents the kinds of failures encoded by a `kbs2` `Response`.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "body")]
pub enum FailureKind {
    /// The request failed because one or more I/O operations failed.
    Io(String),

    /// The request failed because it was malformed.
    Malformed(String),

    /// The request failed because key unwrapping failed.
    Unwrap(String),

    /// The request failed because the agent and client don't speak the same protocol version.
    VersionMismatch(u32),

    Auth,
}

/// A convenience trait for marshaling and unmarshaling `RequestBody`s and `Response`s
/// through Rust's `Read` and `Write` traits.
pub trait Message {
    fn read<R: Read>(reader: R) -> Result<Self>
    where
        Self: DeserializeOwned,
    {
        #[allow(clippy::unwrap_used)]
        let data: Result<Vec<_>, _> = reader
            .bytes()
            .take_while(|b| b.is_ok() && *b.as_ref().unwrap() != b'\n')
            .collect();
        let data = data?;
        let res = serde_json::from_slice(&data)?;

        Ok(res)
    }

    fn write<W: Write>(&self, mut writer: W) -> Result<()>
    where
        Self: Serialize,
    {
        serde_json::to_writer(&mut writer, &self)?;
        writer.write_all(&[b'\n'])?;
        writer.flush()?;

        Ok(())
    }
}

impl Message for Request {}
impl Message for Response {}
