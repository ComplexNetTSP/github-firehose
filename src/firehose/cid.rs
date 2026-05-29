use serde::{Deserialize, Deserializer};
#[allow(dead_code)]
#[derive(Debug)]
pub struct CidLink(pub String);

impl<'de> Deserialize<'de> for CidLink {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Link {
            #[serde(rename = "$link")]
            link: String,
        }
        let l = Link::deserialize(deserializer)?;
        Ok(CidLink(l.link))
    }
}
