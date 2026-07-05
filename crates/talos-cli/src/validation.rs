//! CLI frontend for the shared validation service.

use anyhow::Result;
use clap::{Subcommand, ValueEnum};

#[derive(Subcommand, Clone)]
pub(crate) enum ValidateCommand {
    /// Print a validation plan without executing commands.
    Plan {
        /// Validation profile to plan.
        #[arg(long, value_enum, default_value_t = ValidationProfile::Workspace)]
        profile: ValidationProfile,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Execute an allowlisted validation profile and print durable evidence.
    Run {
        /// Validation profile to execute.
        #[arg(long, value_enum, default_value_t = ValidationProfile::Workspace)]
        profile: ValidationProfile,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ValidationProfile {
    Governance,
    I076,
    Workspace,
}

impl From<ValidationProfile> for talos_conversation::ValidationProfile {
    fn from(value: ValidationProfile) -> Self {
        match value {
            ValidationProfile::Governance => Self::Governance,
            ValidationProfile::I076 => Self::I076,
            ValidationProfile::Workspace => Self::Workspace,
        }
    }
}

pub(crate) fn run_validate_command(command: ValidateCommand) -> Result<()> {
    match command {
        ValidateCommand::Plan { profile, json } => {
            let workspace = std::env::current_dir()?;
            let plan = talos_conversation::collect_validation_plan(&workspace, profile.into());
            if json {
                println!("{}", talos_conversation::render_json_plan(&plan));
            } else {
                print!("{}", talos_conversation::render_text_plan(&plan));
            }
        }
        ValidateCommand::Run { profile, json } => {
            let workspace = std::env::current_dir()?;
            let plan = talos_conversation::collect_validation_plan(&workspace, profile.into());
            let evidence = talos_conversation::run_validation_plan(&workspace, plan);
            if json {
                println!("{}", talos_conversation::render_json_evidence(&evidence));
            } else {
                print!("{}", talos_conversation::render_text_evidence(&evidence));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_profile_maps_to_shared_service_profile() {
        assert_eq!(
            talos_conversation::ValidationProfile::from(ValidationProfile::Governance),
            talos_conversation::ValidationProfile::Governance
        );
        assert_eq!(
            talos_conversation::ValidationProfile::from(ValidationProfile::I076),
            talos_conversation::ValidationProfile::I076
        );
        assert_eq!(
            talos_conversation::ValidationProfile::from(ValidationProfile::Workspace),
            talos_conversation::ValidationProfile::Workspace
        );
    }
}
