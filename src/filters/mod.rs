// All operations related to filter domain management are handled in this module.
// including blacklisting for ads, and whitelisting domains to avoid TLS interception.

mod domain_filter;
pub mod utils;

pub use domain_filter::ListConfigType;
pub use utils::*;
