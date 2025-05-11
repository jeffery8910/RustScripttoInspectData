# CSV 編碼與維度協調器 (egui 版本)

[![Rust CI](https://github.com/jeffery8910/csv-encoding-orchestrator-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/jeffery8910/csv-encoding-orchestrator-rust/actions/workflows/rust.yml) 

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT) <!-- 假設您使用 MIT/Apache 雙許可 -->

一個使用 Rust 和 egui 圖形化使用者介面 (GUI) 的工具，用於協調 R 和 Python (Pandas, DuckDB) 對 CSV 檔案在不同編碼下的維度（行數、列數）進行檢查和比較。

## 功能

*   **圖形化使用者介面：** 使用 [egui](https://github.com/emilk/egui) 構建，易於使用。
*   **依賴項檢查：** 啟動時自動檢測 R、Python 及其所需套件 (readr, dplyr, pandas, duckdb) 是否已正確安裝和配置。
*   **檔案選擇：** 允許使用者通過原生檔案對話框選擇要分析的 CSV 檔案。
*   **多編碼測試：** 使用者可以輸入一個或多個編碼 (例如 `UTF-8`, `BIG5`, `GBK`) 進行測試。
*   **協調執行：** Rust 主程式會為每個指定的編碼調用外部的 R 和 Python 腳本。
*   **結果彙總與比較：** 捕獲 R 和 Python 腳本的輸出，並在 GUI 中以表格形式清晰展示和比較結果。
*   **進度顯示：** 在執行過程中顯示詳細的進度日誌。

## 專案結構

```
csv-encoding-orchestrator-rust/
├── .git/
├── .gitignore
├── Cargo.toml               # Rust 專案配置文件
├── README.md                # 本檔案
├── LICENSE-MIT              # MIT 許可證檔案 (或您選擇的許可證)
├── LICENSE-APACHE           # Apache 2.0 許可證檔案 (或您選擇的許可證)
├── src/
│   ├── main.rs              # 主程式入口，初始化 egui 應用
│   ├── app.rs               # egui 應用的核心邏輯和 UI
│   └── dep_check.rs         # 依賴項檢查邏輯
├── scripts/                 # R 和 Python 輔助腳本
│   ├── R/
│   │   └── check_csv_dimensions_r.R
│   └── python/
│       └── check_csv_dimensions_py.py
├── data/                    # (可選) 存放範例 CSV 檔案
│   ├── sample_utf8.csv
│   └── sample_big5.csv
├── assets/                  # (可選) 存放應用程式圖示等資源
│   └── icon.png
└── target/                  # 編譯輸出目錄 (由 cargo 生成)
```

## 背景

在處理來自不同來源的 CSV 檔案時，由於編碼問題導致的資料讀取錯誤或維度不一致是一個常見的痛點。此工具旨在：

1.  提供一個統一的平台來測試不同編碼對 CSV 檔案解析的影響。
2.  利用 R 和 Python 各自強大的 CSV 解析能力和生態系統。
3.  通過 Rust 作為主控和報告工具，提供一個可靠且高效的解決方案。
4.  以圖形化介面簡化操作流程。

## 先決條件 (環境設置)

在使用本工具前，請確保您的系統已安裝並正確配置了以下軟體。本應用程式啟動時會檢查這些依賴項：

1.  **Rust 工具鏈：**
    *   從 [rust-lang.org](https://www.rust-lang.org/) 安裝 Rust (包含 `cargo`)。

2.  **R 環境：**
    *   **安裝 R：** 從 [CRAN](https://cran.r-project.org/) 下載並安裝 R。
    *   **`Rscript` 在 PATH 中：** 確保 `Rscript` 命令可以在您的終端/命令提示符中直接執行。通常 R 安裝程式會自動將其添加到系統 PATH。
    *   **安裝 R 套件：** 在 R 控制台中執行以下命令安裝必要的套件：
        ```R
        install.packages(c("readr", "dplyr"))
        ```

3.  **Python 環境：**
    *   **安裝 Python：** 從 [python.org](https://www.python.org/) 下載並安裝 Python 3。
    *   **`python` 或 `python3` 在 PATH 中：** 確保 `python` (或 `python3`) 命令可以在您的終端/命令提示符中直接執行。Python 安裝程式通常會提供添加到 PATH 的選項。
    *   **安裝 Python 套件：** 使用 pip 安裝必要的套件：
        ```bash
        pip install pandas duckdb
        # 或者 pip3 install pandas duckdb
        ```

## 如何編譯和執行

1.  **克隆倉庫 (如果尚未克隆)：**
    ```bash
    git clone https://github.com/YOUR_USERNAME/csv-encoding-orchestrator-rust.git # 替換 YOUR_USERNAME
    cd csv-encoding-orchestrator-rust
    ```

2.  **編譯：**
    *   開發模式 (較快編譯，用於測試)：
        ```bash
        cargo build
        ```
    *   發行模式 (優化，用於分發)：
        ```bash
        cargo build --release
        ```

3.  **執行：**
    *   編譯後的可執行檔案位於 `target/debug/csv_encoding_orchestrator_egui` (或 `target/release/...`，Windows 上為 `.exe` 後綴)。
    *   直接運行該可執行檔案即可啟動 GUI 應用程式。
        *   Windows (Release): `.\target\release\csv_encoding_orchestrator_egui.exe`
        *   Linux/macOS (Release): `./target/release/csv_encoding_orchestrator_egui`

    *   或者，您可以通過 `cargo run` 直接編譯並運行：
        ```bash
        cargo run
        # 或 cargo run --release
        ```

## 如何使用應用程式

1.  **啟動應用程式。**
2.  **檢查「System Dependencies」部分：** 確保所有必要的依賴項都顯示為綠色勾號 (✓)。如果存在問題 (紅色 ✗ 或黃色 ⚠️)，請根據提示解決環境配置問題。
3.  **點擊「Select CSV File」按鈕** 並選擇您要分析的 CSV 檔案。
4.  **在「Encodings (comma-separated)」文本框中輸入您要測試的編碼列表** (例如：`UTF-8,BIG5,GB18030`)。
5.  **點擊「Run Checks」按鈕。**
6.  **在「Progress Log」區域查看實時的執行進度和詳細信息。**
7.  **分析完成後，在「Results」表格中查看 R 和 Python (Pandas, DuckDB) 對於每種編碼的分析結果。**

## 輔助腳本

本工具依賴於 `scripts/` 目錄下的 R 和 Python 腳本來執行實際的 CSV 解析：

*   **`scripts/R/check_csv_dimensions_r.R`:** 使用 `readr` 套件讀取 CSV 並報告維度。
*   **`scripts/python/check_csv_dimensions_py.py`:** 分別使用 `pandas` 和 `duckdb` 讀取 CSV 並報告維度。

Rust 主程式負責調用這些腳本，傳遞 CSV 檔案路徑、待測試的編碼以及一個臨時輸出檔案路徑。腳本將其結果以 CSV 格式寫入該臨時檔案，然後由 Rust 主程式解析並展示。

**重要：** 這些腳本必須與編譯後的可執行檔案保持相對路徑的正確性 (即，可執行檔案旁邊需要有一個 `scripts` 目錄，其中包含 `R` 和 `python` 子目錄及對應的腳本)。

## 如何貢獻

歡迎各種形式的貢獻！如果您有任何建議、發現錯誤或想要添加新功能，請隨時：

1.  創建一個 [Issue](https://github.com/YOUR_USERNAME/csv-encoding-orchestrator-rust/issues)。 <!-- 替換 YOUR_USERNAME -->
2.  Fork 本倉庫，創建您的功能分支 (`git checkout -b feature/AmazingFeature`)。
3.  提交您的更改 (`git commit -m 'Add some AmazingFeature'`)。
4.  將分支推送到遠程倉庫 (`git push origin feature/AmazingFeature`)。
5.  開啟一個 Pull Request。

## 許可證

本項目採用雙重許可：MIT 和 Apache License 2.0。

*   [MIT License](LICENSE-MIT)
*   [Apache License 2.0](LICENSE-APACHE)
