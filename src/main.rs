mod recipe;
mod vbox;
mod ssh;

use clap::{Parser, Subcommand};
use dialoguer::Password;
use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Result};

use recipe::LaziRecipe;

// CLI Interfacting (I think naming this section this is funny cause of the redundancy).

#[derive(Parser)]
#[command(name = "lazi")]
#[command(about = "Automated Pentesting VM Deployment Tools", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // deploy subcommand
    Deploy {
        #[arg(help = "The path to the lazi-recipe.yml file")]
        recipe_path: PathBuf,
    },
    TestSsh {
        #[arg(help = "The path to the lazi-recipe.yml file")]
        recipe_path: PathBuf,
    },
    TestVbox {
        #[arg(help = "The path to the lazi-recipe.yml file")]
        recipe_path: PathBuf,
    },
}

// helper to parse recipe fo command
fn parse_recipe(recipe_path: &PathBuf) -> Result<LaziRecipe> {
    let extension = recipe_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if extension != "yaml" && extension != "yml" {
        anyhow::bail!("Error: File must be .yaml or .yml\n>:(");
    }

    println!("Reading recipe from {:?}\n", recipe_path);
    let yaml = fs::read_to_string(recipe_path).context("Failed to read file")?;
    let recipe: LaziRecipe = serde_yaml::from_str(&yaml).context("Failed to parse YAML")?;
    Ok(recipe)
}

// helper to get user pass
fn gpass(recipe: &LaziRecipe) -> Result<String> {
    let mut user_password = String::new();
    if let Some(user_config) = &recipe.user {
        println!("User Configuration Found:\nUsername: {}\n", &user_config.username);
        user_password = Password::new()
            .with_prompt(format!("Enter desired password for VM user '{}'", user_config.username))
            .with_confirmation("Confirm Password", "Password mismatch. Try again.")
            .interact()?;
        println!("Password secured.\n");
    } else {
        println!("No user configuration provided. Will default to root/kali.\n");
    }
    Ok(user_password)
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
       Commands::Deploy { recipe_path } => {
           let recipe = parse_recipe(&recipe_path)?;
           let password = gpass(&recipe)?;

           println!("====================");
           println!("LAZI DEPLOYMENT STARTING");
           println!("====================");

           vbox::vbox_auto(&recipe)?;
           ssh::config_vm(&recipe, &password)?;

           println!("====================");
           println!("DEPLOYMENT COMPLETE!");
           println!("You custom Pentesting VM '{}' is live and ready!", recipe.name); 
           println!("====================");
        }
        Commands::TestVbox { recipe_path } => {
           let recipe = parse_recipe(&recipe_path)?;
           println!("Running VBox test module...");
           vbox::vbox_auto(&recipe)?;
        }
        Commands::TestSsh { recipe_path } => {
            let recipe = parse_recipe(&recipe_path)?;
            let password = gpass(&recipe)?;
            println!("Running SSH test module...");
            ssh::config_vm(&recipe, &password)?;
        }
    }
    Ok(())
}

