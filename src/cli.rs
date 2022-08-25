use anyhow::Result;
use tracing::metadata::LevelFilter;

const BINNAME: &str = clap::crate_name!();

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(subcommand)]
    cmd: Cmd,
}

command_builder::build_commands!(
    Cmd {
        #[clap(setting = clap::AppSettings::Hidden)]
        /// Generate shell completions
        GenCompletions {
            #[clap(arg_enum)]
            /// The target shell
            shell: clap_complete::Shell,
        },
    },
    {
        Cmd::GenCompletions{shell} => {
            gen_completions(shell);
            Ok(())
        }
    },
    encrypt
);

pub(crate) fn run() -> Result<()> {
    let cli = <Cli as clap::Parser>::parse();

    init_tracing(&cli);
    dispatch_cmd(cli.cmd)
}

fn init_tracing(_: &Cli) {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter, registry};

    registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::builder()
            .with_regex(false)
            .with_default_directive(LevelFilter::ERROR.into())
            .from_env_lossy())
        .init();
}

fn gen_completions(shell: impl clap_complete::Generator) {
    clap_complete::generate(
        shell,
        &mut <Cli as clap::IntoApp>::into_app(),
        BINNAME,
        &mut std::io::stdout(),
    )
}

mod command_builder {
    macro_rules! build_commands {
        (
            $name:ident { $($manual_fields:tt)* },
            { $($manual_dispatch:tt)* },
            $($cmd:ident),* $(,)?
        ) => {
            $(
                mod $cmd;
            )*

            paste::paste! {
                #[derive(Debug, clap::Subcommand)]
                #[clap(about, version)]
                enum $name {
                    $($manual_fields)*
                    $(
                        #[clap(about = $cmd::ABOUT)]
                        [<$cmd:camel>]($cmd::CmdArgs),
                    )*
                }

                fn [<dispatch_ $name:snake>](a: $name) -> anyhow::Result<()> {
                    match a {
                        $($manual_dispatch)*
                        $(
                            $name::[<$cmd:camel>](a) => a.run(),
                        )*
                    }
                }
            }
        }
    }
    pub(crate) use build_commands;
}
