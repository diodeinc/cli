use std::path::PathBuf;

use atopile::{AtopileNormalizer, AtopileProject};
use colored::*;
use expanduser::expanduser;
use inquire::{autocompletion::Replacement, Autocomplete, Confirm, CustomUserError, Text};
use kicad2schematics::schematics_from_kicad_netlist;

#[derive(clap::Args)]
pub struct ConvertArgs {
    #[clap(
        short,
        long,
        help = "Path to the KiCad netlist file (.net) to be converted"
    )]
    netlist: Option<PathBuf>,

    #[clap(
        short,
        long,
        help = "Directory where the converted output will be saved"
    )]
    output_dir: Option<PathBuf>,

    #[clap(
        short,
        long,
        help = "Force overwrite of existing files in the output directory"
    )]
    force: bool,
}

#[derive(Clone, Default)]
pub struct FilePathCompleter {
    input: String,
    paths: Vec<String>,
    lcp: String,
}

impl FilePathCompleter {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input {
            return Ok(());
        }

        self.input = input.to_owned();
        self.paths.clear();

        let input_path = expanduser(input)?;

        let fallback_parent = input_path
            .parent()
            .map(|p| {
                if p.to_string_lossy() == "" {
                    std::path::PathBuf::from(".")
                } else {
                    p.to_owned()
                }
            })
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let scan_dir = if input.ends_with('/') {
            input_path.clone()
        } else {
            fallback_parent.clone()
        };

        let entries = match std::fs::read_dir(scan_dir) {
            Ok(read_dir) => Ok(read_dir),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                std::fs::read_dir(fallback_parent)
            }
            Err(err) => Err(err),
        }?
        .collect::<Result<Vec<_>, _>>()?;

        let mut idx = 0;
        let limit = 15;

        while idx < entries.len() && self.paths.len() < limit {
            let entry = entries.get(idx).unwrap();

            let path = entry.path();
            let path_str = if path.is_dir() {
                format!("{}/", path.to_string_lossy())
            } else {
                path.to_string_lossy().to_string()
            };

            if path
                .to_string_lossy()
                .starts_with(input_path.to_str().unwrap())
                && path != input_path
            {
                self.paths.push(path_str);
            }

            idx = idx.saturating_add(1);
        }

        self.lcp = self.longest_common_prefix();

        Ok(())
    }

    fn longest_common_prefix(&self) -> String {
        let mut ret: String = String::new();

        let mut sorted = self.paths.clone();
        sorted.sort();
        if sorted.is_empty() {
            return ret;
        }

        let mut first_word = sorted.first().unwrap().chars();
        let mut last_word = sorted.last().unwrap().chars();

        loop {
            match (first_word.next(), last_word.next()) {
                (Some(c1), Some(c2)) if c1 == c2 => {
                    ret.push(c1);
                }
                _ => return ret,
            }
        }
    }
}

impl Autocomplete for FilePathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        match self.update_input(input) {
            Ok(()) => Ok(self.paths.clone()),
            Err(_) => Ok(vec![]),
        }
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        if let Err(_) = self.update_input(input) {
            return Ok(Replacement::None);
        }

        Ok(match highlighted_suggestion {
            Some(suggestion) => Replacement::Some(suggestion),
            None => match self.lcp.is_empty() {
                true => Replacement::None,
                false => Replacement::Some(self.lcp.clone()),
            },
        })
    }
}

impl ConvertArgs {
    pub fn complete(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut required_input = false;

        while self.netlist.is_none() {
            required_input = true;
            let netlist = Text::new("Path to KiCad Netlist (.net): ")
                .with_autocomplete(FilePathCompleter::default())
                .prompt()?;
            let path: PathBuf = expanduser(netlist)?;
            if !path.exists() || !path.is_file() {
                println!("File not found: \"{}\"", path.display());
            } else {
                self.netlist = Some(path);
            }
        }

        while self.output_dir.is_none() {
            required_input = true;
            let output_dir = Text::new("Output directory: ")
                .with_autocomplete(FilePathCompleter::default())
                .prompt()?;
            let path: PathBuf = expanduser(output_dir)?;

            if path.is_file() {
                println!("Invalid output directory: \"{}\"", path.display());
            } else {
                self.output_dir = Some(path);
            }
        }

        let output_dir = self.output_dir.as_ref().unwrap();

        if !self.force && output_dir.exists() {
            required_input = true;

            let overwrite = Confirm::new(&format!(
                "Directory already exists: \"{}\". Overwrite?",
                output_dir.display()
            ))
            .with_default(false)
            .prompt()?;

            if overwrite {
                self.force = true;
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("Directory already exists: \"{}\"", output_dir.display()),
                )));
            }
        }

        Ok(required_input)
    }

    pub fn get_command(&self) -> String {
        let mut command = String::new();
        command.push_str(&format!("{} convert ", std::env::args().next().unwrap()));

        if let Some(netlist) = self.netlist.as_ref() {
            command.push_str(&format!("--netlist \"{}\" ", netlist.display()));
        }

        if let Some(output_dir) = self.output_dir.as_ref() {
            command.push_str(&format!("--output-dir \"{}\" ", output_dir.display()));
        }

        if self.force {
            command.push_str("--force ");
        }

        command.trim().to_string()
    }
}

pub fn run(mut args: ConvertArgs) -> Result<(), Box<dyn std::error::Error>> {
    let required_input = args.complete()?;
    if required_input {
        println!("$ {}", args.get_command());
    }

    let project_name = args
        .output_dir
        .as_ref()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    if !args.output_dir.as_ref().unwrap().exists() {
        println!("Output does not exist, calling `ato create`...");

        let mut command = std::process::Command::new("ato");
        command
            .arg("create")
            .arg(project_name)
            .current_dir(args.output_dir.as_ref().unwrap().parent().unwrap());

        let mut child = command.spawn()?;
        let status = child.wait()?;

        if !status.success() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create project: {}", status),
            )));
        }

        println!("Created project!");
    }

    // Read netlist and parse it into schematics.
    let netlist = std::fs::read_to_string(args.netlist.ok_or("netlist file not found")?)?;
    let mut schematics = schematics_from_kicad_netlist(&netlist)?;

    // Normalize the names in the netlist.
    let normalizer = AtopileNormalizer::default();
    schematics.normalize(normalizer)?;

    // Generate the source files.
    let project = AtopileProject::from_schematic(project_name.to_string(), &schematics)?;
    project.generate_to_directory(&args.output_dir.unwrap())?;

    println!("{}", "Conversion completed successfully!".green());

    Ok(())
}
