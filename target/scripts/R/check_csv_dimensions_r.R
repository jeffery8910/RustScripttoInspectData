# scripts/R/check_csv_dimensions_r.R
# Suppress package startup messages for cleaner output when called from Rust
suppressPackageStartupMessages(library(readr))
suppressPackageStartupMessages(library(dplyr)) # For select and bind_rows

# --- Argument Parsing ---
args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 3) {
  cat("Usage: Rscript check_csv_dimensions_r.R <csv_file_path> <output_csv_path> <encoding_to_test>\n", file = stderr())
  stop("Incorrect number of arguments provided. Expected 3.", call. = FALSE)
}

csv_file_path <- args[1]
output_csv_path <- args[2]
encoding_to_test <- args[3]

# --- Logging ---
cat("R: Processing file:", csv_file_path, "with encoding:", encoding_to_test, "\n", file = stdout())

# --- Main Logic ---
result_data <- tryCatch({
  # Attempt to read the CSV file with the specified encoding
  # guess_max is set to a high value, adjust if necessary for very large files or specific needs
  # progress = FALSE and show_col_types = FALSE for cleaner script output
  df <- readr::read_csv(
    csv_file_path,
    locale = readr::locale(encoding = encoding_to_test),
    guess_max = 100000, # Increased guess_max as common practice for diverse CSVs
    show_col_types = FALSE,
    progress = FALSE,
    name_repair = "minimal" # To avoid messages about new names
  )

  # If successful, gather dimensions
  rows <- nrow(df)
  cols <- ncol(df)
  
  list(
    tool = "R_readr",
    file_path = basename(csv_file_path), # Store only the basename, consistent with Python
    encoding_tested = encoding_to_test,
    status = "Success",
    rows = as.integer(rows), # Ensure integer type
    cols = as.integer(cols),
    cells = as.integer(rows * cols),
    error_message = NA_character_ # NA will be written as "" by write_csv with na = ""
  )
}, error = function(e) {
  # If an error occurs (e.g., encoding error, file not found)
  cat("R: Error reading file:", csv_file_path, "with encoding:", encoding_to_test, "-", conditionMessage(e), "\n", file = stderr())
  list(
    tool = "R_readr",
    file_path = basename(csv_file_path),
    encoding_tested = encoding_to_test,
    status = "Failure",
    rows = NA_integer_, # Use NA_integer_ for missing numeric values
    cols = NA_integer_,
    cells = NA_integer_,
    error_message = as.character(conditionMessage(e))
  )
})

# --- Output to CSV ---
# Convert the list to a data frame.
# We expect only one row from this R script per run.
final_df <- dplyr::bind_rows(list(result_data)) # Encapsulate in list for bind_rows

# Ensure the column order matches Rust's ScriptOutput struct
# This also handles cases where some columns might be all NA (e.g. if tryCatch failed early)
# and ensures they are of the correct type for consistent CSV output.
expected_cols <- c("tool", "file_path", "encoding_tested", "status", "rows", "cols", "cells", "error_message")

# Add any missing columns with NAs of appropriate type (though result_data should have them all)
for (col_name in expected_cols) {
  if (!col_name %in% names(final_df)) {
    if (col_name %in% c("rows", "cols", "cells")) {
      final_df[[col_name]] <- NA_integer_
    } else {
      final_df[[col_name]] <- NA_character_
    }
  }
}

# Select columns in the correct order
final_df <- final_df %>%
  dplyr::select(all_of(expected_cols))

# Write the results to the output CSV file specified by Rust
# Using na = "" ensures that NA values are written as empty strings,
# which Rust's `csv` crate can easily deserialize into `Option::None`.
tryCatch({
  readr::write_csv(final_df, output_csv_path, na = "")
  cat("R: Results for encoding", encoding_to_test, "saved to:", output_csv_path, "\n", file = stdout())
}, error = function(e) {
  cat("R: CRITICAL - Failed to write output CSV to", output_csv_path, "-", conditionMessage(e), "\n", file = stderr())
  # If we can't write the output, Rust won't get the result.
  # This error should be caught by Rust based on script's exit code or lack of output file.
  quit(save = "no", status = 1, runLast = FALSE) # Exit with an error status
})

# Implicitly exit with status 0 if no errors up to this point.
