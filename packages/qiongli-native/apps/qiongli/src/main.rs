use std::env;
use std::io::{self, BufReader, IsTerminal};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1).collect::<Vec<_>>();
    #[cfg(feature = "desktop")]
    if args.is_empty() {
        return match qiongli::run_desktop_application() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: {}", error.reason_code());
                ExitCode::FAILURE
            }
        };
    }

    if args.is_empty() && io::stdin().is_terminal() && io::stdout().is_terminal() {
        args = ["install", "migrate", "--interactive"]
            .map(Into::into)
            .to_vec();
    }
    let environment = qiongli::CommandEnvironment::from_process();
    let content = match qiongli::embedded_content() {
        Ok(content) => content,
        Err(_) => return render_output(qiongli::failed_embedded_content_output()),
    };
    match qiongli::prepare_action(args, &environment, &content) {
        qiongli::ProductAction::Output(output) => render_output(output),
        qiongli::ProductAction::ReviewCliInstallations => {
            match qiongli::review_cli_installations(&environment) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        qiongli::ProductAction::ServeLiteMcpStdio => {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut reader = BufReader::new(stdin.lock());
            let mut writer = stdout.lock();
            match qiongli::serve_lite_mcp(&mut reader, &mut writer, &environment, &content) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("error: {}", error.reason_code());
                    ExitCode::FAILURE
                }
            }
        }
        qiongli::ProductAction::ServeFullMcpStdio => {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut reader = BufReader::new(stdin.lock());
            let mut writer = stdout.lock();
            match qiongli::serve_full_mcp(&mut reader, &mut writer, &environment, &content) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("error: {}", error.reason_code());
                    ExitCode::FAILURE
                }
            }
        }
        qiongli::ProductAction::LaunchDesktop => {
            if qiongli::run_desktop(environment, content).is_ok() {
                ExitCode::SUCCESS
            } else {
                eprintln!("error: {}", qiongli::DESKTOP_STARTUP_ERROR_CODE);
                ExitCode::FAILURE
            }
        }
        qiongli::ProductAction::LaunchDesktopWithCandidate(session) => {
            if qiongli::run_desktop_with_candidate_sessions(environment, content, vec![*session])
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                eprintln!("error: {}", qiongli::DESKTOP_STARTUP_ERROR_CODE);
                ExitCode::FAILURE
            }
        }
    }
}

fn render_output(output: qiongli::CliOutput) -> ExitCode {
    if !output.stdout().is_empty() {
        print!("{}", output.stdout());
    }
    if !output.stderr().is_empty() {
        eprint!("{}", output.stderr());
    }
    ExitCode::from(output.exit_code())
}
