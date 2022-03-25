use crate::server::server_connection;
use color_eyre::{
    eyre::{eyre, Result},
    owo_colors::OwoColorize,
};
use itertools::Itertools;
use ldap3::{Scope, SearchEntry};

pub async fn search(filter: Option<String>, attributes: Option<String>) -> Result<()> {
    let query = filter.ok_or_else(|| eyre!("Unable to search without an LDAP filter."))?;
    let attrs: Vec<&str>;
    match attributes.as_ref() {
        Some(list) => attrs = list.split(',').collect::<Vec<&str>>(),
        None => attrs = vec![],
    }
    let (mut ldap, search_base) = server_connection().await?;
    let (rs, _res) = ldap
        .search(&search_base, Scope::Subtree, &query, attrs)
        .await?
        .success()?;
    for entry in rs {
        let result = SearchEntry::construct(entry);
        println!("{} {}", "dn:".green().bold(), result.dn);
        for (key, vals) in result.attrs.iter().sorted() {
            for val in vals {
                println!("{} {val}", format!("{key}:").cyan().bold());
            }
        }
    }
    Ok(ldap.unbind().await?)
}
