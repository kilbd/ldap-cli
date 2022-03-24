use crate::server_connection;
use itertools::Itertools;
use ldap3::result::Result;
use ldap3::{Scope, SearchEntry};

pub async fn search(filter: Option<String>, attributes: Option<String>) -> Result<()> {
    let query: String;
    match filter {
        Some(q) => query = q,
        None => {
            println!("Unable to search without an LDAP filter.");
            return Ok(());
        }
    }
    let attrs: Vec<&str>;
    match attributes.as_ref() {
        Some(list) => attrs = list.split(",").collect::<Vec<&str>>(),
        None => attrs = vec![],
    }
    let (mut ldap, search_base) = server_connection().await?;
    let (rs, _res) = ldap
        .search(&search_base, Scope::Subtree, &query, attrs)
        .await?
        .success()?;
    for entry in rs {
        let result = SearchEntry::construct(entry);
        println!("dn: {}", result.dn);
        for (key, vals) in result.attrs.iter().sorted() {
            for val in vals {
                println!("{key}: {val}");
            }
        }
    }
    Ok(ldap.unbind().await?)
}
