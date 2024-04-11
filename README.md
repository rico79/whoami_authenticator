# Prepare your dev environement (example for Fedora)

## Install Rust and tools
Install rust : [https://www.rust-lang.org/tools/install]

Install rust formatter
```
rustup component add rustfmt
```

Install cargo shuttle and login
```
cargo install cargo-shuttle
cargo shuttle login
```

## Install Postgresql locally
Download and install : [https://www.postgresql.org/download/]
For example on fedora, first clean the pgsql directory in case of former install
```
sudo rm -r /var/lib/pgsql
```
and then install and start the server

```
sudo dnf install postgresql-server
sudo postgresql-setup --initdb
sudo systemctl enable postgresql.service
sudo systemctl start postgresql.service
```

Change the password of postgres user
```
sudo -u postgres psql
\q
```

Create an app user
```
sudo -u postgres psql
CREATE ROLE devapp LOGIN PASSWORD 'devapp';
\du
\q
```

Create the database
```
sudo -u postgres psql
CREATE DATABASE authenticator;
\l
\q
```

If you want to connect from your own user you will have to change the local config to "trust" in the pg_hba.conf
```
sudo gnome-text-editor /var/lib/pgsql/data/pg_hba.conf
```

## VSCode extensions
- rust-analyzer 
- Even Better TOML

## To build and run the app
To run the app locally (localhost)
```
cargo shuttle run
```

To run the app exposed on local network (on 0.0.0.0:XXXX)
```
cargo shuttle run --external [--port XXXX]
```

To deploy the app
```
cargo shuttle deploy
```