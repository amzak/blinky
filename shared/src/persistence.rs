use std::sync::Arc;

use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;
use tokio::task::JoinError;

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone, AsRefStr, Hash, Copy)]
pub enum PersistenceUnitKind {
    RtcSyncInfo,
    CalendarEventInfo,
    CalendarSyncInfo,
    TimelyData,
}

#[derive(Debug)]
pub struct PersistenceUnit {
    pub kind: PersistenceUnitKind,
    pub data: Result<Arc<Vec<u8>>, Error>,
}

impl Clone for PersistenceUnit {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            data: self.data.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct PersistenceUnitDto {
    pub kind: PersistenceUnitKind,
    pub data: Vec<u8>,
}

impl PersistenceUnit {
    pub fn new<TObj>(kind: PersistenceUnitKind, obj: &TObj) -> PersistenceUnit
    where
        TObj: Serialize,
    {
        let buf = rmp_serde::to_vec(obj).unwrap();

        PersistenceUnit {
            kind: kind,
            data: Ok(Arc::new(buf)),
        }
    }
}

impl PersistenceUnit {
    pub async fn deserialize<T>(self) -> Result<T, Error>
    where
        for<'a> T: Deserialize<'a> + Send + 'static,
    {
        if self.data.is_err() {
            return Err(self.data.err().unwrap());
        }

        let data_arc = self.data.unwrap();

        let data_arc_clone = data_arc.clone();

        let result: Result<Result<T, rmp_serde::decode::Error>, JoinError> =
            tokio::spawn(async move {
                let slice = data_arc_clone.as_slice();
                let res = rmp_serde::from_slice::<T>(slice);

                res
            })
            .await;

        match result {
            Ok(deserialize_res) => deserialize_res.map_err(|err| Error::from(err.to_string())),
            Err(err) => Result::Err(Error::from(err.to_string())),
        }
    }
}

impl Into<PersistenceUnitDto> for PersistenceUnit {
    fn into(self) -> PersistenceUnitDto {
        let kind = self.kind;

        PersistenceUnitDto {
            kind: kind,
            data: self.data.unwrap().to_vec(),
        }
    }
}
