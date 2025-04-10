use clap::{Parser, Subcommand};
use color_eyre::eyre::{Context, ContextCompat, eyre};
use cruet::{
    case::{snake::to_snake_case, to_class_case},
    string::{pluralize::to_plural, singularize::to_singular},
};
use guppy::{MetadataCommand, graph::PackageGraph};
use liquid::Template;
use shipwright_cli::{
    Error,
    util::{
        query::{Field, generate_sql, generate_struct_fields, parse_cli_fields},
        ui::UI,
    },
};
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::process::ExitCode;
use std::time::SystemTime;

static BLUEPRINTS_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/blueprints");

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
#[command(author, version, about = "A CLI tool to generate project files.", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Disable colored output.")]
    no_color: bool,

    #[arg(long, global = true, help = "Disable debug output.")]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Generate a middleware")]
    Middleware {
        #[arg(help = "The name of the middleware.")]
        name: String,
    },
    #[command(about = "Generate a controller")]
    Controller {
        #[arg(help = "The name of the controller.")]
        name: String,
        #[arg(help = "Column definitions like: 'id:uuid^', 'name:string256!', 'avatar:references=avatars(id)'", num_args = 0..)]
        fields: Vec<String>,
    },
    #[command(about = "Generate a test for a controller")]
    ControllerTest {
        #[arg(help = "The name of the controller.")]
        name: String,
        #[arg(help = "Column definitions like: 'id:uuid^', 'name:string256!', 'avatar:references=avatars(id)'", num_args = 0..)]
        fields: Vec<String>,
    },
    #[command(about = "Generate a migration")]
    Migration {
        #[arg(help = "The table name.")]
        table: String,
        #[arg(help = "Column definitions like: 'id:uuid^', 'name:string256!', 'avatar:references=avatars(id)'", num_args = 0..)]
        fields: Vec<String>,
    },
    #[command(about = "Generate an entity")]
    Entity {
        #[arg(help = "The name of the entity.")]
        name: String,
        #[arg(help = "Column definitions like: 'id:uuid^', 'name:string256!', 'avatar:references=avatars(id)'", num_args = 0..)]
        fields: Vec<String>,
    },
    #[command(about = "Generate a view")]
    View {
        #[arg(help = "The name of the view.")]
        name: String,
    },
    #[command(
        about = "Generate a complete scaffold (migration, entity, controller, test, and view)"
    )]
    Scaffold {
        #[arg(help = "The name of the resource.")]
        name: String,
        #[arg(help = "Column definitions like: 'id:uuid^', 'name:string256!', 'avatar:references=avatars(id)'", num_args = 0..)]
        fields: Vec<String>,
    },
}

#[allow(missing_docs)]
async fn cli(ui: &mut UI<'_>, cli: Cli) -> Result<(), Error> {
    match cli.command {
        Commands::Middleware { name } => {
            ui.info("Generating middleware…");
            let file_name = generate_middleware(name)
                .await
                .wrap_err("Could not generate middleware!")?;
            ui.success(&format!("Generated middleware {}.", &file_name));
            Ok(())
        }
        Commands::Controller { name, fields } => {
            ui.info("Generating controller…");
            let file_name = generate_controller(name.clone())
                .await
                .wrap_err("Could not generate controller!")?;
            ui.success(&format!("Generated controller {}.", &file_name));
            ui.info("Do not forget to route the controller's actions in ./web/src/routes.rs!");
            ui.info("Generating test for controller…");
            let file_name = generate_controller_test(name, parse_cli_fields(fields)?)
                .await
                .wrap_err("Could not generate test for controller!")?;
            ui.success(&format!("Generated test for controller {}.", &file_name));
            Ok(())
        }
        Commands::ControllerTest { name, fields } => {
            ui.info("Generating test for controller…");
            let file_name = generate_controller_test(name, parse_cli_fields(fields)?)
                .await
                .wrap_err("Could not generate test for controller!")?;
            ui.success(&format!("Generated test for controller {}.", &file_name));
            Ok(())
        }
        Commands::Migration { table, fields } => {
            ui.info("Generating migration…");
            let table_name = to_plural(&table);
            let migration_name = format!("create_{}_table", table_name);
            let file_name = generate_migration(migration_name, table, parse_cli_fields(fields)?)
                .await
                .wrap_err("Could not generate migration!")?;
            ui.success(&format!("Generated migration {}.", &file_name));
            Ok(())
        }
        Commands::Entity { name, fields } => {
            ui.info("Generating entity…");
            let struct_name = generate_entity(name, parse_cli_fields(fields)?)
                .await
                .wrap_err("Could not generate entity!")?;
            ui.success(&format!("Generated entity {}.", &struct_name));
            Ok(())
        }
        Commands::View { name } => {
            ui.info("Generating view…");
            let file_name = generate_view(name)
                .await
                .wrap_err("Could not generate view!")?;
            ui.success(&format!("Generated view {}.", &file_name));
            Ok(())
        }
        Commands::Scaffold { name, fields } => {
            let parsed_fields = parse_cli_fields(fields)?;
            let name_snake = to_snake_case(&name).to_lowercase();
            let name_plural = to_plural(&name_snake);

            // Generate migration
            ui.info("Generating migration…");
            let migration_name = format!("create_{}_table", name_plural);
            let file_name =
                generate_migration(migration_name, name_snake.clone(), parsed_fields.clone())
                    .await
                    .wrap_err("Could not generate migration!")?;
            ui.success(&format!("Generated migration {}.", &file_name));

            // Generate entity
            ui.info("Generating entity…");
            let struct_name = generate_entity(name.clone(), parsed_fields.clone())
                .await
                .wrap_err("Could not generate entity!")?;
            ui.success(&format!("Generated entity {}.", &struct_name));

            // Generate controller
            ui.info("Generating controller…");
            let file_name = generate_controller(name.clone())
                .await
                .wrap_err("Could not generate controller!")?;
            ui.success(&format!("Generated controller {}.", &file_name));
            ui.info("Do not forget to route the controller's actions in ./web/src/routes.rs!");

            // Generate controller test
            ui.info("Generating test for controller…");
            let file_name = generate_controller_test(name.clone(), parsed_fields.clone())
                .await
                .wrap_err("Could not generate test for controller!")?;
            ui.success(&format!("Generated test for controller {}.", &file_name));

            // Generate view
            ui.info("Generating view…");
            let file_name = generate_view(name.clone())
                .await
                .wrap_err("Could not generate view!")?;
            ui.success(&format!("Generated view {}.", &file_name));

            ui.success(&format!("Successfully scaffolded resource '{}'!", name));
            Ok(())
        }
    }
}

async fn generate_middleware(name: String) -> Result<String, Error> {
    let name = to_snake_case(&name).to_lowercase();

    let template = get_liquid_template("middleware/file.rs")?;
    let variables = liquid::object!({
        "name": name
    });
    let output = template
        .render(&variables)
        .wrap_err("Failed to render Liquid template")?;

    let file_path = format!("./web/src/middlewares/{}.rs", name);
    create_project_file(&file_path, output.as_bytes())?;
    append_to_project_file(
        "./web/src/middlewares/mod.rs",
        &format!("pub mod {};", name),
    )?;

    Ok(file_path)
}

async fn generate_controller(name: String) -> Result<String, Error> {
    let name = to_snake_case(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let name_singular = to_singular(&name);
    let struct_name = to_class_case(&name_singular);
    let db_crate_name = get_member_package_name("db")?;
    let db_crate_name = to_snake_case(&db_crate_name);

    let template = get_liquid_template("controller/file.rs")?;
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name_singular,
        "entity_plural_name": name_plural,
        "db_crate_name": db_crate_name,
    });
    let output = template
        .render(&variables)
        .wrap_err("Failed to render Liquid template")?;

    let file_path = format!("./web/src/controllers/{}.rs", name);
    create_project_file(&file_path, output.as_bytes())?;
    append_to_project_file(
        "./web/src/controllers/mod.rs",
        &format!("pub mod {};", name),
    )?;

    Ok(file_path)
}

async fn generate_controller_test(name: String, fields: Vec<Field>) -> Result<String, Error> {
    let name = to_snake_case(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let name_singular = to_singular(&name);
    let struct_name = to_class_case(&name_singular);
    let web_crate_name = get_member_package_name("web")?;
    let web_crate_name = to_snake_case(&web_crate_name);
    let db_crate_name = get_member_package_name("db")?;

    let (entity_struct_fields, changeset_struct_fields) = generate_struct_fields(&fields);

    let template = get_liquid_template("controller/test.rs")?;
    let variables = liquid::object!({
        "name": name,
        "entity_struct_name": struct_name,
        "entity_singular_name": name_singular,
        "entity_plural_name": name_plural,
        "web_crate_name": web_crate_name,
        "db_crate_name": db_crate_name,
        "entity_struct_fields": entity_struct_fields,
        "changeset_struct_fields": changeset_struct_fields,
    });
    let output = template
        .render(&variables)
        .wrap_err("Failed to render Liquid template")?;

    let file_path = format!("./web/tests/integration/{name}_test.rs");
    create_project_file(&file_path, output.as_bytes())?;
    append_to_project_file(
        "./web/tests/integration/main.rs",
        &format!("mod {name}_test;"),
    )?;

    Ok(file_path)
}

async fn generate_migration(
    name: String,
    table: String,
    fields: Vec<Field>,
) -> Result<String, Error> {
    let table_name = to_plural(&table);
    let generated_sql = generate_sql(&table_name, fields).await?;

    let template = get_liquid_template("migration/file.sql")?;

    let variables = liquid::object!({
        "generated_sql": generated_sql,
    });
    let output = template
        .render(&variables)
        .wrap_err("Failed to render Liquid template")?;

    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let file_name = format!("{}__{}.sql", timestamp.as_secs(), name);
    let path = format!("./db/migrations/{}", file_name);
    create_project_file(&path, output.as_bytes())?;

    Ok(path)
}

async fn generate_entity(name: String, fields: Vec<Field>) -> Result<String, Error> {
    let name = to_singular(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let struct_name = to_class_case(&name);

    let (entity_struct_fields, changeset_struct_fields) = generate_struct_fields(&fields);

    let template = get_liquid_template("entity/file.rs")?;

    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name,
        "entity_plural_name": name_plural,
        "entity_struct_fields": entity_struct_fields,
        "changeset_struct_fields": changeset_struct_fields,
    });

    let output = template
        .render(&variables)
        .wrap_err("Failed to render Liquid template")?;

    let file_path = format!("./db/src/entities/{}.rs", name_plural);
    create_project_file(&file_path, output.as_bytes())?;
    append_to_project_file(
        "./db/src/entities/mod.rs",
        &format!("pub mod {};", name_plural),
    )?;

    Ok(struct_name)
}

async fn generate_view(name: String) -> Result<String, Error> {
    let name = to_snake_case(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let name_singular = to_singular(&name);
    let struct_name = to_class_case(&name_singular);
    let db_crate_name = get_member_package_name("db")?;
    let db_crate_name = to_snake_case(&db_crate_name);

    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name_singular,
        "entity_plural_name": name_plural,
        "db_crate_name": db_crate_name,
    });

    // Generate Rust view file
    let template = get_liquid_template("view/file.rs")?;
    let output = template
        .render(&variables)
        .wrap_err("Failed to render Liquid template")?;

    let file_path = format!("./web/src/views/{}.rs", name_plural);
    create_project_file(&file_path, output.as_bytes())?;
    append_to_project_file(
        "./web/src/views/mod.rs",
        &format!("pub mod {};", name_plural),
    )?;

    // Create templates directory if it doesn't exist
    let templates_dir = format!("./ui/assets/templates/{}", name_plural);
    fs::create_dir_all(&templates_dir).wrap_err("Failed to create templates directory")?;

    // Generate index.html template
    let index_template = get_liquid_template("view/templates/index.html")?;
    let index_output = index_template
        .render(&variables)
        .wrap_err("Failed to render index template")?;
    create_project_file(
        &format!("{}/index.html", templates_dir),
        index_output.as_bytes(),
    )?;

    // Generate show.html template
    let show_template = get_liquid_template("view/templates/show.html")?;
    let show_output = show_template
        .render(&variables)
        .wrap_err("Failed to render show template")?;
    create_project_file(
        &format!("{}/show.html", templates_dir),
        show_output.as_bytes(),
    )?;

    // Generate update.html template
    let update_template = get_liquid_template("view/templates/update.html")?;
    let update_output = update_template
        .render(&variables)
        .wrap_err("Failed to render update template")?;
    create_project_file(
        &format!("{}/update.html", templates_dir),
        update_output.as_bytes(),
    )?;

    Ok(file_path)
}

fn get_liquid_template(path: &str) -> Result<Template, Error> {
    let blueprint = BLUEPRINTS_DIR
        .get_file(path)
        .wrap_err(format!("Failed to get blueprint {}!", path))?;
    let template_source = blueprint
        .contents_utf8()
        .wrap_err(format!("Failed to read blueprint {}!", path))?;
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(template_source)
        .wrap_err("Failed to parse blueprint as Liquid template")?;

    Ok(template)
}

fn create_project_file(path: &str, contents: &[u8]) -> Result<(), Error> {
    let mut file = File::create(path).wrap_err(format!(r#"Could not create file "{}""#, path))?;
    file.write_all(contents)
        .wrap_err(format!(r#"Could not write file "{}""#, path))?;

    Ok(())
}

fn append_to_project_file(path: &str, contents: &str) -> Result<(), Error> {
    let file_contents =
        fs::read_to_string(path).wrap_err(format!(r#"Could not read file "{}"!"#, path))?;
    let file_contents = file_contents.trim();

    let mut options = OpenOptions::new();
    options.write(true);

    if file_contents.is_empty() {
        options.truncate(true);
    } else {
        options.append(true);
    }

    let mut file = options
        .open(path)
        .wrap_err(format!(r#"Could not open file "{}"!"#, path))?;

    writeln!(file, "{}", contents).wrap_err(format!(r#"Failed to append to file "{}"!"#, path))?;

    Ok(())
}

fn get_member_package_name(path: &str) -> Result<String, Error> {
    let mut cmd = MetadataCommand::new();
    let package_graph = PackageGraph::from_command(cmd.manifest_path("./Cargo.toml")).unwrap();
    let workspace = package_graph.workspace();
    for member in workspace.iter_by_path() {
        let (member_path, metadata) = member;
        if member_path == path {
            return Ok(String::from(metadata.name()));
        }
    }
    Err(eyre!("Could not find workspace member at path: {}", path).into())
}
