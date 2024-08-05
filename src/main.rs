use clap::Parser;
use convert::ConvertArgs;
// use diff::DiffArgs;
use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};

mod convert;
// mod diff;

#[derive(Parser)]
#[command(version, about, name = "diode", bin_name = "diode")]
enum DiodeCli {
    Convert(ConvertArgs),
    // Diff(DiffArgs),
}

fn get_inquire_config() -> RenderConfig<'static> {
    let mut config = RenderConfig::default();
    config.prompt_prefix = Styled::new(">").with_fg(Color::DarkGrey);
    config.answered_prompt_prefix = Styled::new(">").with_fg(Color::DarkGrey);
    config.prompt = StyleSheet::new().with_fg(Color::DarkGrey);
    config
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    inquire::set_global_render_config(get_inquire_config());

    let args = DiodeCli::parse();

    match args {
        DiodeCli::Convert(args) => convert::run(args),
        // DiodeCli::Diff(args) => diff::run(args),
    }
}
