use anyhow::{Context, Result};
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::generator::QrGenerator;
use crate::models::{AppTheme, EccLevel, ExportFormat};
use crate::utils::{FileOps, UrlValidator};

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<String>,
    pub generated_files: Vec<PathBuf>,
}

pub struct BatchGenerator;

impl BatchGenerator {
    pub fn process_csv<F>(
        csv_path: &Path,
        output_dir: &Path,
        format: ExportFormat,
        ecc: EccLevel,
        theme: AppTheme,
        size: u32,
        progress_callback: F,
    ) -> Result<BatchResult>
    where
        F: Fn(usize, usize, &str),
    {
        FileOps::ensure_dir_exists(output_dir)?;

        let file = File::open(csv_path)
            .with_context(|| format!("Failed to open CSV file: {:?}", csv_path))?;

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let records: Vec<csv::StringRecord> = rdr.records().filter_map(|r| r.ok()).collect();
        let total = records.len();

        let mut result = BatchResult {
            total,
            succeeded: 0,
            failed: 0,
            errors: Vec::new(),
            generated_files: Vec::new(),
        };

        for (idx, record) in records.iter().enumerate() {
            if record.is_empty() {
                continue;
            }

            let raw_url = record.get(0).unwrap_or("").trim();
            let label = record.get(1).unwrap_or("").trim();

            match UrlValidator::validate(raw_url) {
                Ok(valid_url) => match QrGenerator::create_qr(&valid_url, ecc) {
                    Ok(qr) => {
                        let filename = if !label.is_empty() {
                            let safe_label = label
                                .chars()
                                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                                .collect::<String>();
                            format!("qr_{}.{}", safe_label, format.extension())
                        } else {
                            format!("qr_batch_{:03}.{}", idx + 1, format.extension())
                        };

                        let out_path = output_dir.join(filename);
                        if let Err(e) = QrGenerator::save_to_file(
                            &qr, &out_path, format, size, true, theme, false,
                        ) {
                            result.failed += 1;
                            result.errors.push(format!("Row {}: {}", idx + 1, e));
                        } else {
                            result.succeeded += 1;
                            result.generated_files.push(out_path);
                        }
                    }
                    Err(e) => {
                        result.failed += 1;
                        result.errors.push(format!("Row {}: {}", idx + 1, e));
                    }
                },
                Err(e) => {
                    result.failed += 1;
                    result
                        .errors
                        .push(format!("Row {} invalid URL '{}': {}", idx + 1, raw_url, e));
                }
            }

            progress_callback(idx + 1, total, raw_url);
        }

        Ok(result)
    }
}
