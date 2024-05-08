# Prepare your dev environement (example for Fedora)

## Install Rust and tools
Install rust : [https://www.rust-lang.org/tools/install]

Install rust formatter
```
rustup component add rustfmt
```

Install cargo make
```
cargo install cargo-make
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
CREATE ROLE devapp LOGIN PASSWORD '<db user password>';
\du
\q
```

If you want to connect from your own user you will have to change the local config to "trust" in the pg_hba.conf
```
sudo gnome-text-editor /var/lib/pgsql/data/pg_hba.conf
```

Create the database
```
sudo -u postgres psql
CREATE DATABASE authenticator;
ALTER DATABASE authenticator OWNER TO devapp;
\l
\q
```

## Tailwind
Download the last CLI version : [https://github.com/tailwindlabs/tailwindcss/releases/latest]

Then copy it in this project directory and rename it to "tailwindcss"
```
mv tailwindcss-<end of name file> tailwindcss
```

And remember to give execution rights to the file
```
chmod +x tailwindcss
```

## VSCode extensions
- rust-analyzer 
- Even Better TOML
- Tailwind CSS IntelliSense

## Secrets structure
For the app to run you need to create a Secrets.toml (and a Secrets.dev.toml for local dev) file containing
```
# General
APP_NAME = "whoami"
APP_URL = "http://localhost:8001"
OWNER_MAIL = "Your mail"

# Database
DB_PASSWORD = "Your db user password"

# Mails
MAIL_SMTP = "Your smtp url"
MAIL_USER_NAME = "Your mailer user"
MAIL_PASSWORD = "Your mailer password"

# JWT
JWT_EXPIRE_SECONDS = "3600"
JWT_SECRET = "Your JWT secret"
```

## To build and run the app
To run the app locally (localhost)
```
cargo make run
```

To deploy the app
```
cargo make deploy
```