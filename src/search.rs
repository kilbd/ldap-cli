use itertools::Itertools;
use ldap3::result::Result;
use ldap3::{LdapConnAsync, Scope, SearchEntry};

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
    let (conn, mut ldap) = LdapConnAsync::new("ldaps://example.com:636").await?;
    ldap3::drive!(conn);
    ldap.simple_bind("cn=Admin User,ou=Admin,o=MyOrg,c=US", "redactedpassword")
        .await?;
    let (rs, _res) = ldap
        .search("o=MyOrg,c=US", Scope::Subtree, &query, attrs)
        .await?
        .success()?;
    for entry in rs {
        let result = SearchEntry::construct(entry);
        println!("dn: {}", result.dn);
        for (key, val) in result.attrs.iter().sorted() {
            println!("{key}: {val:?}");
        }
    }
    Ok(ldap.unbind().await?)
}
