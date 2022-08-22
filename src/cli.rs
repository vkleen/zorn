use anyhow::Result;

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
);

pub(crate) fn run() -> Result<()> {
    let cli = <Cli as clap::Parser>::parse();

    dispatch_cmd(cli.cmd)
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
                            $name::[<$cmd:camel>](a) => $cmd::run(a).await,
                        )*
                    }
                }
            }
        }
    }
    pub(crate) use build_commands;
}
