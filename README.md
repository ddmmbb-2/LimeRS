# LimeRS

**Windows 綠色版繁體中文注音輸入工具｜Rust 製作｜中英文混合候選**

LimeRS 是一款使用 Rust 開發、專為 Windows 設計的繁體中文注音輸入工具。

它的設計靈感來自 Android **LIME（萊姆輸入法）**：**不必反覆切換中英文模式，中文組字結果與原始英文鍵碼就能同時出現在候選清單中。**

LimeRS 採用 Windows Keyboard Hook 與 `SendInput`，是**免安裝、免註冊 TSF 的獨立綠色版工具**，並非 Windows 原生 TSF 輸入法，也不是 LIME 官方推出的 Windows 版本。

**目前版本：v0.5.2 Beta**

正在持續改善詞庫準確度、操作細節與跨程式相容性。

## 功能特色

- **中英混合候選**：第 1 項為整句中文預測，第 2 項保留原始英文鍵碼，第 3 項起提供逐字／逐詞候選。
- **繁體中文注音**：標準大千注音鍵位，支援一、二、三、四聲與輕聲。
- **連續組字**：支援多段注音組字，以及逐段選字修正。
- **選字學習**：使用 SQLite 儲存個人選字習慣。
- **詞庫擴充**：可透過 `custom.tsv` 加入自訂詞庫，搭配新酷音詞庫匯入工具使用。
- **常用標點符號**：按 `Shift + ,` 開啟直式標點符號表。
- **綠色免安裝**：雙擊 EXE 即可啟用，不需註冊 Windows 輸入法。
- **微軟正黑體**：候選視窗使用系統既有的中文字型。

## 候選字操作

例如輸入 `dj94xk4`（快樂）：

| 順序 | 候選 | 說明 |
|---|---|---|
| 1 | 快樂 | 預設整句，按 Enter 送出 |
| 2 | dj94xk4 | 保留英文原始鍵碼 |
| 3 | 快 | 逐段選字 |
| 4 起 | 其他同音字 | 依照詞庫及學習紀錄排序 |

實際的候選順序會依照詞庫及個人使用習慣調整。

## 快捷鍵

| 快捷鍵 | 功能 |
|---|---|
| Enter | 送出目前反白候選，預設為整句中文 |
| ↑ / ↓ | 上下移動候選 |
| PageUp / PageDown | 翻頁選字 |
| Home / End | 第一個／最後一個候選 |
| 右 Shift + 1～9 | 選擇目前頁面的對應候選 |
| 單按右 Shift | 切換英文大小寫狀態 |
| Shift + , | 開啟／關閉常用標點符號選單 |
| Esc | 取消組字或關閉標點選單 |
| Ctrl + Alt + F9 | 暫停／恢復 LimeRS |

在標點符號選單中，也支援上下鍵、Enter、Esc、翻頁及滑鼠選取。

正在組字時選擇標點，會先送出預設整句，再接上標點；按 Esc 則保留原本組字。

## 下載與使用

一般使用者可至本專案 **Releases** 下載：

`LimeRS-Windows-Portable-v0.5.2.zip`

發布後：

1. 解壓縮至任意資料夾。
2. 雙擊 `LimeRS.exe`。
3. 程式會立即啟用，並顯示通知區圖示。
4. 建議先使用 Windows 英文鍵盤配置，在記事本測試輸入。
5. 不需要安裝或註冊 Windows 輸入法。

可以透過 `Ctrl + Alt + F9` 暫停使用，或從通知區圖示選單完全退出。

## 從原始碼編譯

建議環境：

- Windows 10 / 11（x64）
- Rust stable（MSVC 工具鏈）
- Visual Studio C++ Build Tools

在專案根目錄執行：

```powershell
cargo test -p limers_core
.\scripts\build-portable.cmd
```

編譯完成後，執行檔位於：

`portable\LimeRS.exe`

目前綠色版不需要 CMake，也不需要編譯或註冊 TSF DLL。

## 專案結構

```text
LimeRS/
├── app/
│   └── src/
│       ├── main.rs
│       └── punctuation.rs
├── core/
│   ├── src/lib.rs
│   └── assets/
├── scripts/
│   └── build-portable.cmd
├── portable/
│   └── data/
├── third_party_licenses/
├── Cargo.toml
└── LICENSE
```

`app` 負責 Windows 鍵盤攔截、候選視窗與文字輸出；`core` 負責注音組字、候選排序與個人學習。

## 詞庫與個人資料

LimeRS 內建基本注音字庫，也支援外部詞庫擴充。

推薦使用 [新酷音詞庫（libchewing-data）](https://github.com/chewing/libchewing-data)，改善常用字詞、詞組及候選排序。

外部詞庫位置：

`portable/data/custom.tsv`

先前提供的 `Import-Chewing.ps1` 是獨立的詞庫轉換工具，並非本專案預設附帶的必要檔案。匯入詞庫後，需要完全退出並重新啟動 LimeRS。

個人學習資料通常位於：

`portable/data/learning.sqlite3`

更新程式時，請保留這兩個檔案，不要以空白資料覆蓋既有個人詞庫。

也請勿將個人 SQLite 資料庫或自行匯入的個人詞庫直接提交至 GitHub。

## 目前限制與開發方向

LimeRS 目前仍屬 Beta 階段。

- 部分字詞排序可能不夠準確，仍需手動選字。
- 整句預測依賴詞庫及既有排序機制。
- Keyboard Hook 與 `SendInput` 對部分應用程式可能存在相容性限制。
- 目前不是 Windows TSF 輸入法，因此不使用 Win + Space 切換 LimeRS。
- 將持續改善詞庫、組字準確率及候選視窗體驗。

歡迎透過 GitHub Issues 回報問題，最好能提供原始鍵碼、預期文字、實際候選及使用版本。

輸入密碼或其他敏感資訊時，建議先暫停 LimeRS。

## 授權與致謝

LimeRS 自行開發的程式碼採用 **MIT License**。

內建讀音資料及使用者自行匯入的詞庫可能具有不同授權條件，請參閱 `third_party_licenses/THIRD_PARTY_NOTICE.md`。

特別感謝：

- [LIME 輸入法](https://github.com/lime-ime/limeime)：中英文混合輸入設計靈感。
- [新酷音（Chewing）](https://github.com/chewing/libchewing-data)：繁體中文注音詞庫來源。
- [Rust](https://www.rust-lang.org/)：本專案使用的程式語言。

**LimeRS 為獨立開發的非官方專案，與 LIME／新酷音官方團隊沒有隸屬或合作關係。**
