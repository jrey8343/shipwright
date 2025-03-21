use clap::{Parser, Subcommand};
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat, eyre},
};
use guppy::{Version, VersionReq};
use shipwright_cli::{Error, util::ui::UI};
use shipwright_config::{Config, DatabaseConfig, Environment, load_config, parse_env};
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{
    ConnectOptions, Connection, Executor,
    migrate::{Migrate, Migrator},
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};
use tokio::io::{AsyncBufReadExt, stdin};
use url::Url;

#[tokio::main]
async fn main() -> ExitCode {
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    let args = Cli::parse();
    let mut ui = UI::new(&mut stdout, &mut stderr, !args.no_color, !args.quiet);

    match cli(&mut ui, args).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            ui.error(e.to_string().as_str(), &e.into());
            ExitCode::FAILURE
        }
    }
}

#[derive(Parser)]
#[command(author, version, about = "A CLI tool to manage the project's database.", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, help = "Choose the environment (development, test, production).", value_parser = parse_env, default_value = "development")]
    env: Environment,

    #[arg(long, global = true, help = "Disable colored output.")]
    no_color: bool,

    #[arg(long, global = true, help = "Disable debug output.")]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Drop the database")]
    Drop,
    #[command(about = "Create the database")]
    Create,
    #[command(about = "Migrate the database")]
    Migrate,
    #[command(about = "Reset (drop, create, migrate) the database")]
    Reset,
    #[command(about = "Seed the database")]
    Seed,
    #[command(about = "Generate query metadata to support offline compile-time verification")]
    Prepare,
}

#[allow(missing_docs)]
async fn cli(ui: &mut UI<'_>, cli: Cli) -> Result<(), Error> {
    let config: Result<Config, shipwright_config::Error> = load_config(&cli.env);
    match config {
        Ok(config) => {
            match cli.command {
                Commands::Drop => {
                    ui.info(&format!("Dropping {} database…", &cli.env));
                    let db_name = drop(&config.database)
                        .await
                        .context("Could not drop database!")?;
                    ui.success(&format!("Dropped database {} successfully.", db_name));
                    Ok(())
                }
                Commands::Create => {
                    ui.info(&format!("Creating {} database…", &cli.env));
                    let db_name = create(&config.database)
                        .await
                        .context("Could not create database!")?;
                    ui.success(&format!("Created database {} successfully.", db_name));
                    Ok(())
                }
                Commands::Migrate => {
                    ui.info(&format!("Migrating {} database…", &cli.env));
                    ui.indent();
                    let migrations = migrate(ui, &config.database)
                        .await
                        .context("Could not migrate database!");
                    ui.outdent();
                    let migrations = migrations?;
                    ui.success(&format!("{} migrations applied.", migrations));
                    Ok(())
                }
                Commands::Seed => {
                    ui.info(&format!("Seeding {} database…", &cli.env));
                    seed(&config.database)
                        .await
                        .context("Could not seed database!")?;
                    ui.success("Seeded database successfully.");
                    Ok(())
                }
                Commands::Reset => {
                    ui.info(&format!("Resetting {} database…", &cli.env));
                    ui.indent();
                    let result = reset(ui, &config.database)
                        .await
                        .context("Could not reset the database!");
                    ui.outdent();
                    let db_name = result?;
                    ui.success(&format!("Reset database {} successfully.", db_name));
                    Ok(())
                }
                Commands::Prepare => {
                    ensure_sqlx_cli_installed(ui).await?;

                    let cargo = get_cargo_path().expect("Existence of CARGO env var is asserted by calling `ensure_sqlx_cli_installed`");

                    let mut sqlx_prepare_command = {
                        let mut cmd = tokio::process::Command::new(&cargo);

                        cmd.args([
                            "sqlx",
                            "prepare",
                            "--workspace",
                            "--",
                            "--all-targets",
                            "--all-features",
                        ]);

                        cmd.env("DATABASE_URL", &config.database.url);

                        cmd
                    };

                    let o = sqlx_prepare_command
                        .output()
                        .await
                        .wrap_err("Could not run {cargo} sqlx prepare!")?;
                    if !o.status.success() {
                        let error = eyre!(String::from_utf8_lossy(&o.stdout).to_string()).wrap_err("Error generating query metadata. Are you sure the database is running and all migrations are applied?");
                        return Err(error.into());
                    }

                    ui.success("Query data written to db/.sqlx directory; please check this into version control.");
                    Ok(())
                }
            }
        }
        Err(e) => Err(e.into()),
    }
}

async fn drop(config: &DatabaseConfig) -> Result<String, Error> {
    let db_config = get_db_config(config);
    let db_name = db_config.get_filename();

    std::fs::remove_file(db_name).wrap_err("Failed to delete the SQLite database file!")?;

    let db_name = db_name.to_str().wrap_err("Failed to get database name!")?;

    Ok(String::from(db_name))
}

async fn create(config: &DatabaseConfig) -> Result<String, Error> {
    let db_config = get_db_config(config);
    let db_name = db_config
        .get_filename()
        .to_str()
        .wrap_err("Failed to get database name!")?;
    let _connection = get_db_client(config).await;

    Ok(String::from(db_name))
}

async fn migrate(ui: &mut UI<'_>, config: &DatabaseConfig) -> Result<i32, Error> {
    let db_config = get_db_config(config);
    let migrations_path = db_package_root()?.join("migrations");
    let migrator = Migrator::new(Path::new(&migrations_path))
        .await
        .context("Failed to create migrator!")?;
    let mut connection = db_config
        .connect()
        .await
        .context("Failed to connect to database!")?;

    connection
        .ensure_migrations_table()
        .await
        .context("Failed to ensure migrations table!")?;

    let applied_migrations: HashMap<_, _> = connection
        .list_applied_migrations()
        .await
        .context("Failed to list applied migrations!")?
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    let mut applied = 0;
    for migration in migrator.iter() {
        if !applied_migrations.contains_key(&migration.version) {
            connection
                .apply(migration)
                .await
                .context("Failed to apply migration {}!")?;
            ui.log(&format!("Applied migration {}.", migration.version));
            applied += 1;
        }
    }

    Ok(applied)
}

async fn seed(config: &DatabaseConfig) -> Result<(), Error> {
    let mut connection = get_db_client(config).await;

    let statements = fs::read_to_string("./db/seeds.sql")
        .expect("Could not read seeds – make sure db/seeds.sql exists!");

    let mut transaction = connection
        .begin()
        .await
        .context("Failed to start transaction!")?;
    transaction
        .execute(statements.as_str())
        .await
        .context("Failed to execute seeds!")?;
    transaction
        .commit()
        .await
        .context("Failed to commit transaction!")?;

    Ok(())
}

async fn reset(ui: &mut UI<'_>, config: &DatabaseConfig) -> Result<String, Error> {
    ui.log("Dropping database…");
    drop(config).await?;
    ui.log("Recreating database…");
    let db_name = create(config).await?;
    ui.log("Migrating database…");
    ui.indent();
    let migration_result = migrate(ui, config).await;
    ui.outdent();

    match migration_result {
        Ok(_) => Ok(db_name),
        Err(e) => Err(e),
    }
}

fn get_db_config(config: &DatabaseConfig) -> SqliteConnectOptions {
    let db_url = Url::parse(&config.url).expect("Invalid DATABASE_URL!");
    ConnectOptions::from_url(&db_url).expect("Invalid DATABASE_URL!")
}

async fn get_db_client(config: &DatabaseConfig) -> SqliteConnection {
    let db_config = get_db_config(config).create_if_missing(true);
    let connection: SqliteConnection = Connection::connect_with(&db_config).await.unwrap();
    connection
}

fn get_cargo_path() -> Result<String, Error> {
    Ok(std::env::var("CARGO").wrap_err("Please invoke me using Cargo, e.g.: `cargo db <ARGS>`")?)
}

/// Ensure that the correct version of sqlx-cli is installed,
/// and install it if it isn't.
async fn ensure_sqlx_cli_installed(ui: &mut UI<'_>) -> Result<(), Error> {
    /// The version of sqlx-cli required
    const SQLX_CLI_VERSION: &str = "0.8";
    let sqlx_version_req = VersionReq::parse(SQLX_CLI_VERSION)
        .expect("SQLX_CLI_VERSION value is not a valid semver version requirement.");

    /// Get the version of the current sqlx-cli installation, if any.
    async fn installed_sqlx_cli_version(cargo: &str) -> Result<Option<Version>, Error> {
        /// The expected prefix of the version output of sqlx-cli >= 0.8
        const SQLX_CLI_VERSION_STRING_PREFIX: &str = "sqlx-cli-sqlx";
        /// The expected prefix of the version output of sqlx-cli < 0.8
        const SQLX_CLI_VERSION_STRING_PREFIX_OLD: &str = "cargo-sqlx";

        fn error_parsing_version() -> Error {
            eyre!(
                "Error parsing sqlx-cli version. Please install the \
                correct version manually using `cargo install sqlx-cli \
                --version ^{SQLX_CLI_VERSION} --locked`"
            )
            .into()
        }

        let mut cargo_sqlx_command = {
            let mut cmd = tokio::process::Command::new(cargo);
            cmd.args(["sqlx", "--version"]);
            cmd
        };

        let out = cargo_sqlx_command.output().await?;
        if !out.status.success() {
            // Failed to run the command for some reason,
            // we conclude that sqlx-cli is not installed.
            return Ok(None);
        }

        let Ok(stdout) = String::from_utf8(out.stdout) else {
            return Err(error_parsing_version());
        };

        let Some(version) = stdout
            .strip_prefix(SQLX_CLI_VERSION_STRING_PREFIX)
            .or_else(|| stdout.strip_prefix(SQLX_CLI_VERSION_STRING_PREFIX_OLD))
            .map(str::trim)
        else {
            return Err(error_parsing_version());
        };

        let Ok(version) = Version::parse(version) else {
            return Err(error_parsing_version());
        };

        Ok(Some(version))
    }

    let cargo = get_cargo_path()?;

    let current_version = installed_sqlx_cli_version(&cargo).await?;
    if let Some(version) = &current_version {
        if sqlx_version_req.matches(version) {
            // sqlx-cli is already installed and of the correct version, nothing to do
            return Ok(());
        }
    }

    let curr_vers_msg = current_version
        .map(|v| format!("The currently installed version is {v}."))
        .unwrap_or_else(|| "sqlx-cli is currently not installed.".to_string());
    ui.info(&format!(
        "This command requires a version of sqlx-cli that is \
        compatible with version {SQLX_CLI_VERSION}, which is not installed yet. \
        {curr_vers_msg} \
        Would you like to install the latest compatible version now? [Y/n]"
    ));

    // Read user answer
    {
        let mut buf = String::new();
        let mut reader = tokio::io::BufReader::new(stdin());
        loop {
            reader.read_line(&mut buf).await?;
            let line = buf.to_ascii_lowercase();
            let line = line.trim_end();
            if matches!(line, "" | "y" | "yes") {
                ui.info("Starting installation of sqlx-cli...");
                break;
            } else if matches!(line, "n" | "no") {
                return Err(eyre!("Installation of sqlx-cli canceled.").into());
            };
            ui.info("Please enter y or n");
            buf.clear();
        }
    }

    let mut cargo_install_command = {
        let mut cmd = tokio::process::Command::new(&cargo);
        cmd.args([
            "install",
            "sqlx-cli",
            "--version",
            &format!("^{SQLX_CLI_VERSION}"),
            "--locked",
            // Install unoptimized version,
            // making the process much faster.
            // sqlx-cli doesn't really need to be
            // performant anyway for our purposes
            "--debug",
        ]);
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        cmd
    };

    let mut child = cargo_install_command.spawn()?;

    let status = child.wait().await?;
    if !status.success() {
        return Err(
            eyre!("Something went wrong when installing sqlx-cli. Please check output").into(),
        );
    }

    match installed_sqlx_cli_version(&cargo).await {
        Ok(Some(v)) if sqlx_version_req.matches(&v) => {
            ui.success(&format!("Successfully installed sqlx-cli {v}"));
            Ok(())
        }
        Ok(Some(v)) => Err(eyre!("Could not update sqlx cli. Current version: {v}").into()),
        Ok(None) => Err(eyre!("sqlx-cli was not detected after installation").into()),
        Err(e) => Err(e),
    }
}

/// Find the root of the db package in the shipwright workspace.
fn db_package_root() -> Result<PathBuf, Error> {
    Ok(PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .wrap_err("This command needs to be invoked using cargo")?,
    )
    .join("..")
    .join("db")
    .canonicalize()?)
}
