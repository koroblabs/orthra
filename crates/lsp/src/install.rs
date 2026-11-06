//! Installing the packs a project enables and does not have yet, in the background.
//!
//! A language whose pack is installed rather than built in (uk-UA, from dict_uk) is fetched from
//! the source its `install.toml` pins, checked by SHA-256 and installed, with a progress bar while
//! it happens. The project is then checked again, so its findings in that language arrive with no
//! restart. `auto-install = false` in a settings file keeps the server off the network, and then
//! it only says how to install the pack. A pack that fails is said once and not tried again this
//! session.
//!
//! The fetch names the archive and nothing else: no document goes with it (`orthra-install`).

use std::{path::Path, sync::Arc};

/// Installs pack `tag` under `packs` and says what was installed: `uk-UA 6.8.6 (GPL-3.0, ...)`.
/// The server's fetches from the source; a test's writes a dictionary of its own.
pub(crate) type Installer = Arc<dyn Fn(&str, &Path) -> Result<String, String> + Send + Sync>;

/// The installer that fetches a pack from its source.
pub(crate) fn from_source() -> Installer {
    Arc::new(|tag, packs| {
        let definition = orthra_install::definition(tag).map_err(|err| err.0)?;
        orthra_install::install(tag, orthra_install::fetch, packs).map_err(|err| err.0)?;
        Ok(format!(
            "{tag} {} ({})",
            definition.version,
            definition.licenses().join("; ")
        ))
    })
}
