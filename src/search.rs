use std::collections::HashSet;

use crate::server::server_connection;
use clap::ArgEnum;
use color_eyre::{
    eyre::{eyre, Result},
    owo_colors::OwoColorize,
};
use csv::Writer;
use itertools::Itertools;
use ldap3::{Scope, SearchEntry};

#[derive(Clone, Debug, ArgEnum)]
pub enum Output {
    Ldif,
    Csv,
}

pub async fn search(
    filter: Option<String>,
    attributes: Option<String>,
    format: Option<Output>,
) -> Result<()> {
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
    let mut results: Vec<SearchEntry> = vec![];
    for entry in rs {
        results.push(SearchEntry::construct(entry));
    }
    match format {
        Some(fmt) => match fmt {
            Output::Ldif => print_ldif(results),
            Output::Csv => print_csv(results)?,
        },
        None => print_stdout(results),
    }
    Ok(ldap.unbind().await?)
}

fn print_stdout(results: Vec<SearchEntry>) {
    results.iter().for_each(|result| {
        println!("{} {}", "dn:".green().bold(), result.dn);
        for (key, vals) in result.attrs.iter().sorted() {
            for val in vals {
                println!("{} {val}", format!("{key}:").cyan().bold());
            }
        }
        println!("-----");
    });
    let mut plural = "ies";
    if results.len() == 1 {
        plural = "y";
    }
    println!("Found {} entr{}.", results.len(), plural);
}

fn print_ldif(results: Vec<SearchEntry>) {
    results.iter().for_each(|result| {
        println!("dn: {}", result.dn);
        for (key, vals) in result.attrs.iter().sorted() {
            for val in vals {
                println!("{key}: {val}");
            }
        }
    });
    println!();
}

fn print_csv(results: Vec<SearchEntry>) -> Result<()> {
    let mut all_attribs = HashSet::new();
    for result in results.iter() {
        for key in result.attrs.keys() {
            all_attribs.insert(key.clone());
        }
    }
    let mut attribs = all_attribs.iter().collect::<Vec<&String>>();
    attribs.sort();
    let dn_head = String::from("dn");
    let mut headers = vec![&dn_head];
    attribs.iter().for_each(|attr| headers.push(attr));
    let mut writer = Writer::from_writer(std::io::stdout());
    writer.write_record(headers)?;
    for result in results.iter() {
        let mut row = attribs
            .iter()
            .map(|attr| match result.attrs.get(*attr) {
                Some(val) => val.join(","),
                None => String::from(""),
            })
            .collect::<Vec<String>>();
        row.insert(0, result.dn.clone());
        writer.write_record(row)?;
    }
    writer.flush()?;
    Ok(())
}
