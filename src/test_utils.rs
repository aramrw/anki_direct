use std::sync::LazyLock;

use crate::AnkiClient;

pub static ANKICLIENT: LazyLock<AnkiClient> = LazyLock::new(AnkiClient::default_latest_sync);

pub(crate) fn display_type<T>() -> String {
    std::any::type_name::<T>().to_string()
}

pub(crate) fn pretty_unwrap<T, E: std::error::Error>(res: Result<T, E>) -> T {
    let Err(e) = res else {
        return res.unwrap();
    };
    panic!("{e}");
}
