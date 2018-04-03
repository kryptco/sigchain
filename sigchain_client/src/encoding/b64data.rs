extern crate base64;
extern crate serde;

use serde::ser::Serializer;
use serde::de::Deserializer;
pub use serde::Deserialize;
pub use serde::Serialize;

/// Encodes/decodes a Vec<u8> as a base64 encoded string (instead of serde's default as a list of bytes)

pub fn serialize<S, T>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
where S: Serializer,
      T: AsRef<[u8]>
{
    serializer.serialize_str(&base64::encode(data))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where D: Deserializer<'de>
{
    let b64 = String::deserialize(deserializer)?;
    Ok(base64::decode(&b64).map_err(serde::de::Error::custom)?) 
}

pub mod option {
    extern crate base64;
    extern crate serde;
    use serde::ser::Serializer;
    use serde::de::Deserializer;
    use serde::de::Deserialize;
    pub fn serialize<S>(data: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
              {
                  match data {
                      &Some(ref data) => {
                          serializer.serialize_some(&base64::encode(data))
                      },
                      &None => {
                          serializer.serialize_none()
                      }
                  }
              }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
        where D: Deserializer<'de>
        {
            let b64 = Option::<String>::deserialize(deserializer)?;
            if let Some(b64) = b64 {
                Ok(Some(base64::decode(&b64).map_err(serde::de::Error::custom)?))
            } else {
                Ok(None)
            }
        }
}

pub mod vec {
    extern crate base64;
    extern crate serde;
    use serde::ser::Serializer;
    use serde::de::Deserializer;
    use serde::de::Deserialize;
    pub fn serialize<S>(data_vec: &Vec<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use serde::ser::SerializeSeq;
        let mut ser_seq = serializer.serialize_seq(Some(data_vec.len()))?;
        for data in data_vec {
            ser_seq.serialize_element(&base64::encode(data))?;
        }
        ser_seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
        where D: Deserializer<'de>
    {
        let mut result = vec![];
        let decodings = Vec::<String>::deserialize(deserializer)?.into_iter().map(|b64_string| {
            base64::decode(&b64_string)
        });
        for decoding in decodings {
            let data = decoding.map_err(serde::de::Error::custom)?;
            result.push(data);
        }
        Ok(result)
    }
}
