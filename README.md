# LDAP CLI

A command-line interface for quick queries of an LDAP directory and for modifications of entry attributes.

## Installation

Head over to the [Releases page](https://github.com/kilbd/ldap-cli/releases) to see available binaries. Currently only macOS is supported.

The easiest way to get the binary and avoid macOS's GateKeeper is with `curl`. I'd recommend soft-linking the extracted binary to a directory on your path, such as `/usr/local/bin`. Here's an example for the latest Apple Silicon version:

```shell
$ mkdir ~/bin
$ cd ~/bin
$ curl -L --output ldap.tar.gz https://github.com/kilbd/ldap-cli/releases/latest/download/ldap-aarch64-apple-darwin.tar.gz
$ tar xf ldap.tar.gz
$ sudo ln -s ~/bin/ldap /usr/local/bin/ldap
$ rm ldap.tar.gz
```

## Usage

The `ldap` command and all subcommands offer help when you use the `--help` or `-h` flag.

```shell
$ ldap --help
```

### Set Up

To begin using `ldap`, you'll want to start by adding configuration details for your LDAP server(s). Commands for managing these saved configurations are under the `server` subcommand. To add your first server config and name it `my-server`, run:

```shell
$ ldap server add my-server
```

You'll then be asked for the server host, port, bind DN, password, and search base.

You can view the names of all saved configurations with the `server list` command:

```shell
$ ldap server list
```

To switch to a different configuration, specify the name with `server use`:

```shell
$ ldap server use other-server
```

For configurations you no longer need, remove them with `server rm`:

```shell
$ ldap server rm my-server
```

### Search

As most of my interaction with LDAP is to search entries, that is the top level command. At a minimum, you need to provide an LDAP filter to use for searching. As LDAP filters use parentheses, ampersands, and pipes, you'll want to quote your filter:

```shell
$ ldap '(uid=kilbd)'
```

By default, all attributes are returned. Use the `--attrs` or `-a` flag to specify attributes to return.

```shell
$ ldap -a cn,mail '(uid=kilbd)'
```

You can also specify an output format, currently limited to LDIF and CSV. Output still goes to stdout, but you can easily redirect to a file.

```shell
$ ldap -a cn,mail -f csv '(mail=*@gmail.com)' > gmail_users.csv
```

### Modify

The other common use case for this tool is to make small adjustments to entries, namely modifying attribute values. You'll use the `modify` subcommand for this purpose.

To **add** a value to existing values, specify the attribute (`-a`) and new value (`-v`) with flags and the entry DN as the final argument.

```shell
$ ldap modify -a objectclass -v developer uid=kilbd,o=GitHub,c=US
```

For multi-value attributes, you can add multiple values by using multiple `-v` flags.

```shell
$ ldap modify -a objectclass -v developer -v moderator uid=kilbd,o=GitHub,c=US
```

To **replace ALL** values of an attribute, add the `--replace` flag.

```shell
$ ldap modify -a cn -v 'Daffy Duck' --replace uid=kilbd,o=GitHub,c=US
```

To **delete** an attribute value, use the `--rm` flag.

```shell
$ ldap modify -a objectclass -v moderator --rm uid=kilbd,o=GitHub,c=US
```

If you don't specify the value to delete when using `--rm`, then _the attribute and all values will be deleted_. You've been warned!

```shell
$ ldap modify -a objectclass --rm uid=kilbd,o=GitHub,c=US
```

Yikes.
