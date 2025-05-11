# scripts/python/check_csv_dimensions_py.py
import sys
import argparse
import pandas as pd
import duckdb
import csv
import os

def get_csv_dimensions_pandas(file_path, encoding_to_test):
    """
    Attempts to read a CSV file with Pandas using a specific encoding
    and returns its dimensions and status.
    """
    try:
        # low_memory=False can sometimes help with mixed types, though might use more memory
        df = pd.read_csv(file_path, encoding=encoding_to_test, low_memory=False)
        rows = len(df)
        cols = len(df.columns)
        return {
            "tool": "Python_Pandas",
            "file_path": os.path.basename(file_path), # Consistent with R script
            "encoding_tested": encoding_to_test,
            "status": "Success",
            "rows": rows,
            "cols": cols,
            "cells": rows * cols,
            "error_message": None
        }
    except FileNotFoundError:
        return {
            "tool": "Python_Pandas",
            "file_path": os.path.basename(file_path),
            "encoding_tested": encoding_to_test,
            "status": "Failure",
            "rows": None,
            "cols": None,
            "cells": None,
            "error_message": f"File not found: {file_path}"
        }
    except UnicodeDecodeError as e:
        return {
            "tool": "Python_Pandas",
            "file_path": os.path.basename(file_path),
            "encoding_tested": encoding_to_test,
            "status": "Failure",
            "rows": None,
            "cols": None,
            "cells": None,
            "error_message": f"UnicodeDecodeError: {str(e)}"
        }
    except Exception as e:
        return {
            "tool": "Python_Pandas",
            "file_path": os.path.basename(file_path),
            "encoding_tested": encoding_to_test,
            "status": "Failure",
            "rows": None,
            "cols": None,
            "cells": None,
            "error_message": str(e)
        }

def get_csv_dimensions_duckdb(file_path, encoding_to_test):
    """
    Attempts to read a CSV file with DuckDB using a specific encoding
    and returns its dimensions and status.
    """
    # Ensure the input file_path is properly escaped for SQL if it contains special characters.
    # However, DuckDB's read_csv function usually handles paths well.
    # Using f-string for table name based on file_path might be problematic if file_path has special chars.
    # It's safer to let read_csv handle the file directly.

    # DuckDB connection
    try:
        con = duckdb.connect(database=':memory:', read_only=False)
        # DuckDB's read_csv has an ENCODING parameter
        # The direct query approach:
        # query = f"SELECT COUNT(*), COUNT(COLUMN_NAMES(*)[1]) FROM read_csv_auto('{file_path}', ENCODING='{encoding_to_test}');"
        # However, getting number of columns this way can be tricky or version dependent.
        # A more robust way is to load it into a DataFrame-like structure or use describe.

        # Let's try to read it and get shape, similar to pandas
        # DuckDB's Python API allows reading CSV directly into a relation, then to Pandas DF or query
        relation = con.read_csv(file_path, encoding=encoding_to_test)
        df = relation.df() # Convert to Pandas DataFrame to easily get shape

        rows = len(df)
        cols = len(df.columns)

        con.close()
        return {
            "tool": "Python_DuckDB",
            "file_path": os.path.basename(file_path),
            "encoding_tested": encoding_to_test,
            "status": "Success",
            "rows": rows,
            "cols": cols,
            "cells": rows * cols,
            "error_message": None
        }
    except duckdb.FileNotFoundException if hasattr(duckdb, 'FileNotFoundException') else FileNotFoundError as e: # DuckDB might raise its own or standard
        return {
            "tool": "Python_DuckDB",
            "file_path": os.path.basename(file_path),
            "encoding_tested": encoding_to_test,
            "status": "Failure",
            "rows": None,
            "cols": None,
            "cells": None,
            "error_message": f"File not found: {file_path} (DuckDB)"
        }
    except duckdb.IOException as e: # DuckDB often wraps encoding errors in IOException
        # Check if it's an encoding error by inspecting the message
        error_str = str(e).lower()
        if "invalid unicode" in error_str or "encoding" in error_str or "codec" in error_str:
            status_msg = f"UnicodeDecodeError (likely): {str(e)}"
        else:
            status_msg = str(e)
        return {
            "tool": "Python_DuckDB",
            "file_path": os.path.basename(file_path),
            "encoding_tested": encoding_to_test,
            "status": "Failure",
            "rows": None,
            "cols": None,
            "cells": None,
            "error_message": status_msg
        }
    except Exception as e: # Catch other DuckDB errors or general errors
        return {
            "tool": "Python_DuckDB",
            "file_path": os.path.basename(file_path),
            "encoding_tested": encoding_to_test,
            "status": "Failure",
            "rows": None,
            "cols": None,
            "cells": None,
            "error_message": str(e)
        }
    finally:
        if 'con' in locals() and con:
            try:
                con.close()
            except: #pylint: disable=bare-except
                pass


def main():
    parser = argparse.ArgumentParser(description="Check CSV dimensions using Pandas and DuckDB for a given encoding.")
    parser.add_argument("csv_file_path", help="Path to the CSV file to inspect.")
    parser.add_argument("output_csv_path", help="Path to write the results CSV to.")
    parser.add_argument("encoding_to_test", help="The encoding to test (e.g., UTF-8, BIG5).")

    if len(sys.argv) != 4: # Script name + 3 arguments
        parser.print_help()
        sys.exit("Usage: python check_csv_dimensions_py.py <csv_file_path> <output_csv_path> <encoding_to_test>")

    args = parser.parse_args()

    csv_file = args.csv_file_path
    output_file = args.output_csv_path
    encoding = args.encoding_to_test

    print(f"Python: Testing {csv_file} with encoding: {encoding}")

    results = []

    # Test with Pandas
    print(f"  Python: Running Pandas check for encoding: {encoding}...")
    pandas_result = get_csv_dimensions_pandas(csv_file, encoding)
    results.append(pandas_result)
    if pandas_result["status"] == "Success":
        print(f"    Pandas: Success (Rows: {pandas_result['rows']}, Cols: {pandas_result['cols']})")
    else:
        print(f"    Pandas: Failure - {pandas_result['error_message']}")


    # Test with DuckDB
    print(f"  Python: Running DuckDB check for encoding: {encoding}...")
    duckdb_result = get_csv_dimensions_duckdb(csv_file, encoding)
    results.append(duckdb_result)
    if duckdb_result["status"] == "Success":
        print(f"    DuckDB: Success (Rows: {duckdb_result['rows']}, Cols: {duckdb_result['cols']})")
    else:
        print(f"    DuckDB: Failure - {duckdb_result['error_message']}")


    # Define fieldnames for the output CSV, matching Rust's ScriptOutput struct
    fieldnames = ["tool", "file_path", "encoding_tested", "status", "rows", "cols", "cells", "error_message"]

    try:
        with open(output_file, 'w', newline='', encoding='utf-8') as csvfile:
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            for result_row in results:
                # Ensure None values are handled correctly by DictWriter (writes as empty string)
                writer.writerow(result_row)
        print(f"Python results for {encoding} saved to: {output_file}")
    except Exception as e:
        sys.exit(f"Python: Critical error writing output CSV {output_file}: {e}")

if __name__ == "__main__":
    main()