mod error;
mod scan;
mod spec;
mod util;
mod zip;

pub use error::{ZipAllError, ZipAllResult};
pub use scan::Scanner;
pub use spec::{ZipMode, ZipSpecification};
pub use zip::{ZipStat, Zipper};

pub static PRODUCT_NAME: &str = "dev.thmsn.zipall";
