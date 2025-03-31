/// Implementation for the INDIGO protocol v2 as specified by
/// [INDIGO](https://github.com/indigo-astronomy/indigo/blob/master/indigo_docs/PROTOCOLS.md)
/// and [INDI](../doc/INDI.pdf).

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct GetProperties<'a> {
    client: &'a str,
    version: &'a str,
}

#[derive(Debug, Clone)]
pub struct AccessToken {
    tok: u64,
}


// mod test {
//     use serde_xml_rs::{from_str, to_string};

//     const XML_GET_PROPERTIES: &str = "<getProperties client='My Client' version='2.0'/>";
//     fn test_get_properties() {

//     }
// }
