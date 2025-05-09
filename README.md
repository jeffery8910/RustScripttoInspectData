# CSV 編碼協調檢閱器 (Rust) - CSV Encoding Orchestrator (Rust)

[![Build Status](https://img.shields.io/github/actions/workflow/status/YOUR_USERNAME/csv-encoding-orchestrator-rust/rust.yml?branch=main)](https://github.com/YOUR_USERNAME/csv-encoding-orchestrator-rust/actions)
[![Latest Release](https://img.shields.io/github/v/release/YOUR_USERNAME/csv-encoding-orchestrator-rust)](https://github.com/YOUR_USERNAME/csv-encoding-orchestrator-rust/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一個命令列工具，旨在幫助您審核和比較**同一 CSV 檔案**在使用 **R 語言**、**Python Pandas** 及 **Python DuckDB** 以**不同文字編碼**開啟時的資料維度（列數、欄位數、單元格總數）。

本工具使用 Rust 語言開發，作為一個**協調器 (Orchestrator)**，它會自動調用預設的或您指定的 R 和 Python 腳本來執行實際的 CSV 讀取和維度計算。最終，它會將所有工具的結果匯總並呈現給您，方便快速發現因編碼問題導致的讀取差異。

**專為需要驗證跨工具、跨編碼資料一致性的團隊設計，特別適合那些希望自動化此流程的成員。**

## ✨ 核心功能

*   **多工具比較：** 自動執行並比較以下工具的 CSV 讀取結果：
    *   R 語言 (透過 `readr` 套件)
    *   Python Pandas
    *   Python DuckDB
*   **多編碼測試：** 使用者可以指定多種常見編碼 (如 UTF-8, BIG5, GBK, Shift_JIS, ISO-8859-1 等) 來嘗試開啟同一個 CSV 檔案。
*   **維度檢查與匯總：** 對於每種工具和每種編碼的組合，程式會嘗試收集並報告：
    *   **列數 (Rows)**
    *   **欄位數 (Columns)**
    *   **總單元格數 (Cells)**
*   **狀態與錯誤報告：** 清晰標示每次嘗試的成功或失敗狀態，並提供相關的錯誤訊息。
*   **集中式報告：** 將 R、Pandas、DuckDB 的結果整合到一個統一的視圖中，方便比較。
*   **單一 Rust 執行檔：** Rust 工具本身是單一執行檔，但它**依賴於外部的 R 和 Python 環境及腳本**。
*   **腳本可配置：** 預設使用專案內提供的 R 和 Python 腳本，但使用者也可以指定自己的腳本路徑。

## 🤔 為何需要這個工具？

當您需要在不同分析工具 (如 R 和 Python 生態) 中處理同一個 CSV 檔案，或者當檔案來源的編碼不確定時，可能會遇到以下問題：

1.  **不一致的資料維度：** R 可能讀取到 1000 列，而 Python Pandas 因編碼錯誤只讀取到 950 列，或欄位數不同。
2.  **隱藏的讀取錯誤：** 某些工具可能在錯誤的編碼下「成功」讀取，但實際上資料已損壞或截斷，僅從維度上就可能發現端倪。
3.  **繁瑣的手動檢查：** 逐個工具、逐個編碼手動測試非常耗時且容易出錯。

本工具旨在自動化這個檢查流程，提供一個標準化的方式來審核和審查 CSV 檔案在不同環境和編碼下的基礎讀取情況。

## ⚠️ 重要：依賴性

本 Rust 工具是一個**協調器**，它本身不直接解析 CSV 檔案的多種編碼。它依賴於以下外部環境和套件：

1.  **R 環境：**
    *   已安裝 R。
    *   `Rscript` 命令在您的系統 PATH 中可用 (或透過 `--r-executable` 指定)。
    *   已安裝 R 套件：`readr`, `dplyr` (預設 R 腳本需要)。
2.  **Python 環境：**
    *   已安裝 Python (建議 Python 3.x)。
    *   `python` 或 `python3` 命令在您的系統 PATH 中可用 (或透過 `--python-executable` 指定)。
    *   已安裝 Python 套件：`pandas`, `duckdb` (預設 Python 腳本需要)。
3.  **R 和 Python 腳本：**
    *   本專案在 `scripts/` 目錄下提供了預設的 R 和 Python 腳本 (`check_csv_dimensions_r.R` 和 `check_csv_dimensions_py.py`)。
    *   這些腳本負責實際的 CSV 讀取和維度統計。您可以修改它們或使用 `--r-script-dir` 和 `--python-script-dir` 指定您自己的腳本目錄。

**在執行本工具前，請確保上述依賴已正確安裝和配置。**

## 🚀 快速開始

### 1. 下載 Rust 工具

前往本專案的 [Releases 頁面](https://github.com/YOUR_USERNAME/csv-encoding-orchestrator-rust/releases) 下載符合您作業系統的最新版本 `csv_encoding_orchestrator` (或 `csv_encoding_orchestrator.exe`)。

### 2. 準備 R 和 Python 腳本

*   確保本專案的 `scripts/` 目錄 (包含 R 和 Python 腳本) 與您的 `csv_encoding_orchestrator` 執行檔位於可訪問的路徑。最簡單的方式是將 `scripts/` 目錄與執行檔放在同一層級。
*   或者，在執行時使用 `--r-script-dir` 和 `--python-script-dir` 選項指定腳本所在目錄。

### 3. (可選) 加入環境變數 PATH

為了方便在任何路徑下執行 Rust 工具，您可以將其執行檔所在目錄加入到系統的環境變數 `PATH` 中。

### 4. 賦予執行權限 (Linux / macOS)

在 Linux 或 macOS 上，您可能需要先賦予 Rust 工具執行權限：
```bash
chmod +x ./csv_encoding_orchestrator
```

## ⚙️ 使用方法

基本指令格式：

```bash
./csv_encoding_orchestrator <CSV檔案路徑> --encodings <編碼列表> [其他選項]
```

**範例：**

1.  **基本檢查 (指定編碼)：**
    ```bash
    ./csv_encoding_orchestrator data/my_data.csv --encodings UTF-8,BIG5,GBK
    ```

2.  **指定 R 和 Python 腳本目錄 (如果不在預設位置)：**
    ```bash
    ./csv_encoding_orchestrator data/my_data.csv --encodings UTF-8 \
        --r-script-dir /path/to/my/r_scripts \
        --python-script-dir /path/to/my/python_scripts
    ```

3.  **指定 Rscript 和 Python 執行檔名稱/路徑 (如果不是預設的 `Rscript` 和 `python`)：**
    ```bash
    ./csv_encoding_orchestrator data/my_data.csv --encodings UTF-8 \
        --r-executable /usr/local/bin/Rscript \
        --python-executable python3
    ```

4.  **獲取幫助 (查看所有可用選項)：**
    ```bash
    ./csv_encoding_orchestrator --help
    ```

**常見的編碼選項 (`--encodings`):** `UTF-8`, `BIG5`, `GBK`, `GB18030`, `Shift_JIS`, `EUC-KR`, `ISO-8859-1`, `Windows-1252`, `UTF-16LE`, `UTF-16BE` 等。請確保 R 和 Python 腳本中使用的函式庫支援您指定的編碼。

## 📊 輸出範例

執行 `./csv_encoding_orchestrator data/sample.csv --encodings UTF-8,BIG5` 可能會看到類似以下的輸出：

```
Inspecting file: "data/sample.csv"
Using R script: "./scripts/R/check_csv_dimensions_r.R"
Using Python script: "./scripts/python/check_csv_dimensions_py.py"
Encodings to test: ["UTF-8", "BIG5"]
----------------------------------------------------

Testing with encoding: UTF-8
  Running R script for encoding: UTF-8...
    R script finished successfully for UTF-8.
  Running Python script for encoding: UTF-8...
    Python script finished successfully for UTF-8.

Testing with encoding: BIG5
  Running R script for encoding: BIG5...
    R script failed for BIG5 with status: exit status: 1. Output:
    Error: BIG5 encoding error at byte ...
    Execution halted
  Running Python script for encoding: BIG5...
    Python script finished successfully for BIG5. (Python Pandas might be more lenient or use a replacement char)


--- Final Comparison Report ---
Tool            | File       | Encoding        | Status           | Rows   | Cols   | Cells    | Error/Notes
-----------------------------------------------------------------------------------------------------------------------
R_readr         | sample.csv | UTF-8           | Success          | 1025   | 15     | 15375    |
Python_Pandas   | sample.csv | UTF-8           | Success          | 1025   | 15     | 15375    |
Python_DuckDB   | sample.csv | UTF-8           | Success          | 1025   | 15     | 15375    |
R_readr         | sample.csv | BIG5            | ExecutionFailure | N/A    | N/A    | N/A      | R script execution failed. Output: Error: BIG5 encoding error...
Python_Pandas   | sample.csv | BIG5            | Success          | 990    | 15     | 14850    | (Note: Pandas might have read with replacement chars)
Python_DuckDB   | sample.csv | BIG5            | Failure          | N/A    | N/A    | N/A      | Error with DuckDB (via Pandas helper): 'big5' codec can't decode byte...
```

**輸出結果說明：**

*   **Tool:** 執行的工具 (例如 `R_readr`, `Python_Pandas`, `Python_DuckDB`)。
*   **File:** 被檢查的 CSV 檔案名稱。
*   **Encoding:** 該次測試使用的編碼。
*   **Status:** `Success` (成功讀取並獲取維度), `Failure` (工具回報讀取失敗), `ExecutionFailure` (腳本本身執行出錯), `ExecutionError` (無法啟動腳本)。
*   **Rows, Cols, Cells:** 讀取到的維度數據，`N/A` 表示未能獲取。
*   **Error/Notes:** 相關的錯誤訊息或備註。

## 🛠️ 給好奇的你，這是我們的方法

這個 Rust 工具的運作流程如下：

1.  **接收輸入：** 解析使用者透過命令列傳入的 CSV 檔案路徑、要測試的編碼列表，以及 R/Python 腳本和執行檔的路徑等設定。
2.  **循環測試：** 對於使用者指定的每種編碼：
    a.  **執行 R 腳本：** Rust 工具會以子進程方式調用指定的 R 腳本 (`Rscript your_r_script.R <csv_file> <temp_output_file> <current_encoding>`)。R 腳本會嘗試以該編碼讀取 CSV，並將結果 (維度、狀態等) 寫入一個臨時的 CSV 檔案。
    b.  **收集 R 結果：** Rust 工具等待 R 腳本執行完畢，然後讀取並解析該臨時 CSV 檔案的內容。
    c.  **執行 Python 腳本：** 類似地，Rust 調用指定的 Python 腳本 (`python your_py_script.py <csv_file> <temp_output_file> <current_encoding>`)。Python 腳本會使用 Pandas 和 DuckDB 嘗試讀取，並將兩者的結果都寫入一個臨時的 CSV 檔案。
    d.  **收集 Python 結果：** Rust 工具讀取並解析 Python 腳本生成的臨時 CSV 檔案。
3.  **彙整與報告：** 在測試完所有指定的編碼後，Rust 工具會將收集到的所有結果 (來自 R, Pandas, DuckDB) 整理成一個統一的表格，並將其顯示在終端機或根據使用者要求輸出到檔案。

這個設計將繁重的 CSV 解析和特定語言的細節留給了 R 和 Python 腳本，而 Rust 則專注於流程控制、命令執行、結果收集和最終報告的生成。

## 👨‍💻 開發者資訊 (Rust 工具本身)

如果您想修改 Rust 工具的原始碼或參與開發：

### 環境需求 (Rust 部分)

*   [Rust 程式語言環境](https://www.rust-lang.org/tools/install) (建議使用 `rustup`)

### 建置專案

1.  Clone 此儲存庫：
    ```bash
    git clone https://github.com/YOUR_USERNAME/csv-encoding-orchestrator-rust.git
    cd csv-encoding-orchestrator-rust
    ```
2.  建置 (除錯模式)：
    ```bash
    cargo build
    ```
    執行檔會在 `target/debug/` 目錄下。
3.  建置 (發行模式，優化性能與大小)：
    ```bash
    cargo build --release
    ```
    執行檔會在 `target/release/` 目錄下。

### 主要使用的 Crates (Rust 函式庫)

*   `clap`: 用於解析命令列參數。
*   `duct`: 方便地執行外部命令 (Rscript, python) 並捕獲其輸出。
*   `csv`: 用於解析由 R/Python 腳本生成的臨時結果 CSV 檔案。
*   `serde`, `serde_json`: 用於資料結構與 CSV/JSON 之間的序列化/反序列化。
*   `tempfile`: 用於創建臨時檔案來接收 R/Python 腳本的輸出。
*   `prettytable-rs` (可選): 用於在終端機美化表格輸出。

## 📜 授權條款

本專案採用 [MIT License](LICENSE) 授權。

## 🤝 貢獻
本專案正在建置中！
歡迎任何形式的貢獻！您可以：
*   回報 Bug (請到 [Issues](https://github.com/YOUR_USERNAME/csv-encoding-orchestrator-rust/issues) 頁面)。
*   提出功能建議。
*   改進 R 或 Python 腳本。
*   改進 Rust 協調器。
*   提交 Pull Requests。
