use std::collections::HashSet;

use crate::server;
use color_eyre::eyre::{eyre, Result};
use ldap3::Mod;

pub async fn modify(
    dn: String,
    attr: String,
    vals: Vec<String>,
    rm: bool,
    replace: bool,
) -> Result<()> {
    if rm && replace {
        return Err(eyre!("Please choose to 'rm' or 'replace', but not both."));
    }
    let (mut ldap, _) = server::server_connection().await?;
    let mut values: HashSet<String> = HashSet::new();
    vals.iter().for_each(|val| {
        values.insert(val.clone());
    });
    let mut mods: Vec<Mod<String>> = vec![];
    if rm {
        mods.push(Mod::Delete(attr, values));
    } else if replace {
        mods.push(Mod::Replace(attr, values));
    } else {
        mods.push(Mod::Add(attr, values));
    }
    ldap.modify(&dn, mods).await?.success()?;
    Ok(ldap.unbind().await?)
}
