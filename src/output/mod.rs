//! Output formatting for different formats

mod table;
mod json;
mod csv;

pub use table::TableFormatter;
pub use json::JsonFormatter;
pub use csv::CsvFormatter;

use crate::cli::{Args, OutputFormat};
use crate::freeze_engine::FreezeEngine;
use crate::process::ProcessInfo;

/// Trait for output formatting
pub trait OutputFormatter {
    fn format_processes(&self, processes: &[ProcessInfo], args: &Args);
}

/// Run output display based on format
pub fn run<E, C, Cat>(engine: &mut FreezeEngine<E, C, Cat>, args: &Args)
where
    E: crate::freeze_engine::ProcessEnumerator,
    C: crate::freeze_engine::ProcessController,
    Cat: crate::categorization::ProcessCategorizer,
{
    match args.format {
        OutputFormat::Table => {
            let formatter = TableFormatter;
            formatter.format_processes(&[], args);
        }
        OutputFormat::Json => {
            let formatter = JsonFormatter;
            let safe = engine.find_safe_to_freeze().unwrap_or_default();
            formatter.format_processes(&safe, args);
        }
        OutputFormat::Csv => {
            let formatter = CsvFormatter;
            let safe = engine.find_safe_to_freeze().unwrap_or_default();
            formatter.format_processes(&safe, args);
        }
    }
}
