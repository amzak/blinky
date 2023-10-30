use embedded_svc::storage::RawStorage;
use esp_idf_svc::nvs;
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use serde::{Deserialize, Serialize};

pub struct NvsStorage {
    nvs: EspNvs<NvsDefault>,
}

type Error<'a> = &'a str;

impl NvsStorage {
    pub fn create(namespace: &str) -> Self {
        let partition = nvs::EspDefaultNvsPartition::take().unwrap();
        let nvs = EspNvs::new(partition, namespace, true).unwrap();

        Self {
            nvs
        }
    }

    pub fn read<T>(&self, name: &str) -> Result<T, Error>
        where for<'a> T: Deserialize<'a> {

        let len_opt = self.nvs.len(name).map_err(|_| Box::<Error>::from("nvs len error")).unwrap();

        if let Some(len) = len_opt {
            let mut buffer = vec![0; len];
            self.nvs.get_raw(name, &mut buffer[..]).unwrap();

            return  rmp_serde::from_slice(&buffer).map_err(|err| { "deserialization error" });
        }

        return Err("unable to read nvs");
    }

    pub fn write<T>(&mut self, name: &str, data: &T) -> Result<(), Error>
    where T: Serialize {
        //let buf: Vec<u8> = postcard::to_allocvec(data).unwrap();

        let buf = rmp_serde::to_vec(data).unwrap();

        self.nvs.set_raw(name, &buf).map_err(|err| err.to_string()).unwrap();

        Ok(())
    }
}