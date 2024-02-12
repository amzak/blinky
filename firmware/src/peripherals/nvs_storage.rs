use embedded_svc::storage::RawStorage;
use esp_idf_svc::nvs;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use serde::{Deserialize, Serialize};

pub struct NvsStorage {
    nvs: EspNvs<NvsDefault>,
}

type Error<'a> = &'a str;

const NVS_KEY_MAX_LENGTH: usize = 15;

impl NvsStorage {
    pub fn create(namespace: &str) -> Self {
        let trimmed_key = Self::trim_nvs_key(namespace);

        let partition = nvs::EspDefaultNvsPartition::take().unwrap();
        let nvs = EspNvs::new(partition, trimmed_key, true).unwrap();

        Self { nvs }
    }

    pub fn read<T>(&self, name: &str) -> Result<T, Error>
    where
        for<'a> T: Deserialize<'a>,
    {
        let trimmed_key = Self::trim_nvs_key(name);

        let len_opt = self
            .nvs
            .blob_len(trimmed_key)
            .map_err(|_| Box::<Error>::from("nvs len error"))
            .unwrap();

        if let Some(len) = len_opt {
            let mut buffer = vec![0; len];
            self.nvs.get_raw(trimmed_key, &mut buffer[..]).unwrap();

            return rmp_serde::from_slice(&buffer).map_err(|err| "deserialization error");
        }

        return Err("unable to read nvs");
    }

    fn trim_nvs_key(namespace: &str) -> &str {
        let len = namespace.len();
        let cutoff = if len > NVS_KEY_MAX_LENGTH {
            NVS_KEY_MAX_LENGTH
        } else {
            len
        };

        &namespace[..cutoff]
    }

    pub fn read_bytes(&self, name: &str) -> Result<Vec<u8>, Error> {
        let trimmed_key = Self::trim_nvs_key(name);

        let len_opt = self
            .nvs
            .blob_len(trimmed_key)
            .map_err(|_| Box::<Error>::from("nvs len error"))
            .unwrap();

        if let Some(len) = len_opt {
            let mut buffer = vec![0; len];
            self.nvs.get_raw(trimmed_key, &mut buffer[..]).unwrap();

            return Ok(buffer);
        }

        return Err("unable to read nvs");
    }

    pub fn write<T>(&mut self, name: &str, data: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let trimmed_key = Self::trim_nvs_key(name);

        let buf = rmp_serde::to_vec(data).unwrap();

        self.nvs
            .set_raw(trimmed_key, &buf)
            .map_err(|err| err.to_string())
            .unwrap();

        Ok(())
    }

    pub fn write_bytes<T>(&mut self, name: &str, data: &Vec<u8>) -> Result<(), Error>
    where
        T: Serialize,
    {
        let trimmed_key = Self::trim_nvs_key(name);

        self.nvs
            .set_raw(trimmed_key, data)
            .map_err(|err| err.to_string())
            .unwrap();

        Ok(())
    }
}
