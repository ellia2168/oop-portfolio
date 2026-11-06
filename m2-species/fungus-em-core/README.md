# fungus-em-core: 電磁共生真菌（Radiomyces electrovorus）Rust 運算核心

高精度 FDTD（時域有限差分電磁場）與生物電磁整流代謝流的高性能 Rust / WebAssembly 運算引擎。

---

## 目錄架構

```
fungus-em-core/
├── Cargo.toml               # Rust 套件配置（cdylib + rlib，wasm-bindgen 依賴）
├── src/
│   ├── lib.rs              # Wasm 外觀層、零拷貝線性記憶體指標介面
│   ├── fdtd.rs             # 2D TMz 麥克斯韋時域有限差分引擎、Mur 一階吸波邊界、拋物天線反射體
│   └── mycelium.rs         # 生物肖特基整流動力學、細胞膜跨膜電位電容、聖艾爾摩放電判定
├── pkg/                    # wasm-pack 編譯產物（.wasm, .js, .d.ts）
├── demo.html               # 內嵌 Base64 Wasm 的即時 Canvas 渲染示範介面（支援直接雙擊開啟）
└── build_demo.py           # 自動化打包 Wasm 至 demo.html 的組裝腳本
```

---

## 核心物理與生物數學模型

### 1. FDTD 2D TMz 麥克斯韋旋度方程
在二維空間中，電磁場解耦為 $E_z, H_x, H_y$ 分量：
$$\frac{\partial H_x}{\partial t} = -\frac{1}{\mu_0} \frac{\partial E_z}{\partial y}$$
$$\frac{\partial H_y}{\partial t} = \frac{1}{\mu_0} \frac{\partial E_z}{\partial x}$$
$$\frac{\partial E_z}{\partial t} = \frac{1}{\epsilon(x,y)} \left( \frac{\partial H_y}{\partial x} - \frac{\partial H_x}{\partial y} - \sigma(x,y) E_z \right)$$

* **拋物面菌傘**：設定高介電常數 $\epsilon_r = 4.5$，幾何反射聚焦。
* **頂生單極棘刺**：高電導率 $\sigma = 15.0 \text{ S/m}$，集中感應電荷。
* **邊界條件**：Mur 一階無反射吸波邊界條件（Absorbing Boundary Conditions），消除邊緣人工反射干擾。

### 2. 生物整流與膜電位蓄能
$$\frac{dQ}{dt} = I_{\text{rect}} - I_{\text{leak}} - I_{\text{metabolic}}$$
$$V_{\text{membrane}} = \frac{Q}{C_{\text{bio}}}$$

* **非線性整流**：感應交流場經由不對稱質子通道整流為直流跨膜電位。
* **電暈放電（聖艾爾摩之火）**：當局部電場強度或端點電位超過空氣擊穿閾值時，觸發冷等離子體電離放電釋放多餘電荷。

---

## 編譯與建構指令

### 編譯為 WebAssembly (網頁前端整合)
```bash
# 需先安裝 wasm-pack
wasm-pack build --target web --release
```

### 原生本機高效能二進位程式
```bash
cargo build --release
```

### 零拷貝（Zero-Copy）前端渲染介面
在 JavaScript 中無需序列化大陣列，直接讀取 Rust Wasm 線性記憶體：
```javascript
const ezPtr = core.get_ez_ptr();
const ezLen = core.get_ez_len();
// 直接建立 TypedArray 視圖，零拷貝達到 60+ FPS
const ezView = new Float32Array(wasmMemory.buffer, ezPtr, ezLen);
```
