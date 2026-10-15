# LimeRS v0.5.2 — Shift+, 常用標點符號選單

此版本是 **免註冊 Windows 綠色版** 的原始碼，基於 v0.5.1.3 已修正的 Rust 核心與 v0.5.1 介面；保留目前已匯入的新酷音詞庫使用方式。

## 新增功能

- **左或右 Shift + 逗號鍵 `,`**：開啟／關閉直式常用標點符號表；攔截 `>`，避免它被錯誤輸入。
- **↑ / ↓**：移動標點候選；**Enter**：送出反白標點；**Esc**：關閉、不送出。
- **PageUp / PageDown**：每次翻 9 個選項；**Home / End**：到第一／最後選項。
- **右 Shift + 1～9**：選目前頁面的標點（沿用快捷選字）。也可用滑鼠直接點。
- 直接打字：關閉標點選單，繼續正常輸入；再次按 `Shift + ,` 也可關閉選單。
- 若已有未完成的注音組字，**開啟再按 Esc 不會遺失組字**。選定標點則先提交現有的 **第 1 候選整句**，再送出標點，例如「你好」加「，」。
- 程式仍然 **雙擊即啟用**，Ctrl+Alt+F9 暫停／恢復。保留中文候選、右 Shift 選字與 SQLite 學習。

## 更新方式（安全）

1. **先在系統匣退出 LimeRS**。
2. 備份目前 `portable/data/custom.tsv`、`portable/data/learning.sqlite3`；**不要用本 ZIP 的 `portable/data/custom.tsv` 覆蓋已匯入新酷音詞庫的版本**。
3. 只需覆蓋 `app/src/main.rs`，新增 `app/src/punctuation.rs`。本版功能沒有修改核心 API；若核心尚未修好 `navigate_edge()`，完整專案已含已修正核心。
4. 執行 `scripts\build-portable.cmd`，再雙擊 `portable\LimeRS.exe`。不需要註冊 Windows TSF。

```powershell
cd D:\AI\rust\5jup
.\scripts\build-portable.cmd
```

## 測試順序

1. 打開記事本，直接按 `Shift+,`，第一個選項應是「，」。按 `↓` 一次、Enter 應輸出「。」。
2. 再次按 `Shift+,`，按 Esc 應無任何字元輸出。
3. 先打 `su3cl3`（你好），再按 `Shift+,`、Enter，應提交「你好，」。
4. 打 `su3cl3`，開標點選單並按 Esc，原本的中文組字應仍在。
5. 原本的 `↑/↓` 選中文、右 Shift + 數字、Enter 與英文原始鍵碼選擇仍需正常。
6. 驗證匯入的 Chewing `custom.tsv` 仍正常載入。

## 驗證與限制

此環境未提供 Windows Rust/Cargo 編譯工具，故完成原始碼與 ZIP 靜態檢查，但 Windows EXE 仍需在你的電腦編譯測試。
輸入工具使用鍵盤 Hook 與 `SendInput`，請先用記事本測試；部分權限較高的程式、遊戲、安全輸入欄可能不相容。在輸入密碼時請暫停 LimeRS。
