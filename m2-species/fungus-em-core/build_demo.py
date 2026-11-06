import base64

with open('/Users/linyunjia/fungus-em-core/pkg/fungus_em_core_bg.wasm', 'rb') as f:
    wasm_b64 = base64.b64encode(f.read()).decode('utf-8')

template = """<!DOCTYPE html>
<html lang="zh-TW">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Radiomyces electrovorus - Rust Wasm 運算核心即時渲染</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      background: #020617;
      color: #e2e8f0;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      overflow: hidden;
      display: flex;
      flex-direction: column;
      height: 100vh;
    }
    header {
      background: #0f172a;
      padding: 12px 24px;
      border-bottom: 1px solid #1e293b;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .title-box h1 {
      font-size: 18px;
      background: linear-gradient(90deg, #38bdf8, #818cf8, #c084fc);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .badge {
      font-size: 11px;
      background: #1e293b;
      color: #38bdf8;
      padding: 3px 8px;
      border-radius: 4px;
      border: 1px solid #38bdf844;
      font-family: monospace;
    }
    #main-container {
      flex: 1;
      display: flex;
      position: relative;
    }
    #canvas-wrap {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      background: radial-gradient(circle at center, #090e1a 0%, #020617 100%);
      position: relative;
    }
    canvas {
      border: 1px solid #334155;
      border-radius: 8px;
      box-shadow: 0 0 35px rgba(56, 189, 248, 0.2);
      image-rendering: pixelated;
    }
    #sidebar {
      width: 340px;
      background: #0b1329;
      border-left: 1px solid #1e293b;
      padding: 20px;
      display: flex;
      flex-direction: column;
      gap: 16px;
      overflow-y: auto;
    }
    .section-title {
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 1.5px;
      color: #64748b;
      font-weight: 700;
    }
    .metric-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 10px;
    }
    .metric-card {
      background: #0f172a;
      border: 1px solid #1e293b;
      border-radius: 8px;
      padding: 12px;
    }
    .metric-title {
      font-size: 11px;
      color: #94a3b8;
    }
    .metric-value {
      font-size: 18px;
      font-weight: 700;
      color: #38bdf8;
      font-family: monospace;
      margin-top: 4px;
    }
    .control-item {
      display: flex;
      flex-direction: column;
      gap: 6px;
    }
    .control-label {
      font-size: 12px;
      display: flex;
      justify-content: space-between;
      color: #cbd5e1;
    }
    input[type=range] {
      accent-color: #38bdf8;
      background: #1e293b;
      height: 5px;
      border-radius: 3px;
    }
    .btn {
      background: linear-gradient(135deg, #0284c7, #6366f1);
      color: white;
      border: none;
      padding: 10px;
      border-radius: 6px;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.2s;
    }
    .btn:hover {
      filter: brightness(1.2);
      transform: translateY(-1px);
    }
    .legend {
      font-size: 11px;
      color: #94a3b8;
      line-height: 1.6;
      background: #0f172a;
      padding: 10px;
      border-radius: 6px;
      border-left: 3px solid #818cf8;
    }
  </style>
</head>
<body>
  <header>
    <div class="title-box">
      <h1>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/><path d="m10 15 5-3-5-3v6Z"/>
        </svg>
        電磁共生真菌 FDTD & 生物整流 Rust 運算核心
      </h1>
    </div>
    <div style="display:flex; gap:10px; align-items:center;">
      <span class="badge">WASM OPTIMIZED</span>
      <span class="badge" id="fps-counter">60 FPS</span>
    </div>
  </header>

  <div id="main-container">
    <div id="canvas-wrap">
      <canvas id="fdtd-canvas" width="128" height="128" style="width: 580px; height: 580px;"></canvas>
    </div>

    <div id="sidebar">
      <div class="section-title">Rust Core Realtime Telemetry</div>

      <div class="metric-grid">
        <div class="metric-card">
          <div class="metric-title">細胞跨膜電位</div>
          <div class="metric-value" id="m-volt">0 mV</div>
        </div>
        <div class="metric-card">
          <div class="metric-title">蓄積生物電能</div>
          <div class="metric-value" id="m-energy">0 J</div>
        </div>
        <div class="metric-card">
          <div class="metric-title">吸波功率耗散</div>
          <div class="metric-value" id="m-power">0.0 mW</div>
        </div>
        <div class="metric-card">
          <div class="metric-title">電暈放電次數</div>
          <div class="metric-value" id="m-corona" style="color:#f43f5e;">0</div>
        </div>
      </div>

      <div class="section-title" style="margin-top:10px;">電磁激勵源參數 (RF Source)</div>

      <div class="control-item">
        <div class="control-label">
          <span>載波頻率 (Carrier)</span>
          <span id="v-freq" style="font-family:monospace; color:#38bdf8;">2.45 GHz</span>
        </div>
        <input type="range" id="sl-freq" min="0.5" max="8.0" step="0.05" value="2.45">
      </div>

      <div class="control-item">
        <div class="control-label">
          <span>發射功率 (Power)</span>
          <span id="v-power" style="font-family:monospace; color:#38bdf8;">25 dBm</span>
        </div>
        <input type="range" id="sl-power" min="5" max="45" step="1" value="25">
      </div>

      <div class="control-item">
        <div class="control-label">
          <span>每幀 FDTD 疊代步數</span>
          <span id="v-steps" style="font-family:monospace; color:#38bdf8;">4 Steps</span>
        </div>
        <input type="range" id="sl-steps" min="1" max="12" step="1" value="4">
      </div>

      <button class="btn" id="btn-pulse">⚡ 激發雷達大功率脈衝 (Shock Pulse)</button>

      <div class="legend">
        <b>視圖物理原理解析</b>：<br>
        • 上方橫線為入射高頻電磁波源（FDTD TMz 模式波動）。<br>
        • 中央倒碗狀為<b>拋物面天線菌蓋</b>，將波前反射聚焦到焦點。<br>
        • 焦點上的<b>單極棘刺</b>與基底<b>分形菌絲</b>進行非線性生物二極管整流，將電磁場轉換為跨膜電位。<br>
        • 強電場超過空氣擊穿閾值時誘發<b>電暈放電（聖艾爾摩之火）</b>。
      </div>
    </div>
  </div>

  <script type="module">
    // Inline Base64 Wasm binary
    const wasmBase64 = "__WASM_BASE64__";
    const wasmBytes = Uint8Array.from(atob(wasmBase64), c => c.charCodeAt(0));

    let wasmMemory;
    const imports = {
      wbg: {
        __wbindgen_throw: function(msg, len) {
          console.error("Wasm bindgen throw:", msg, len);
        },
        __wbindgen_init_externref_table: function() {}
      }
    };

    const { instance } = await WebAssembly.instantiate(wasmBytes, imports);
    wasmMemory = instance.exports.memory;

    // Rust Wasm Exports wrapper
    class EmFungusCore {
      constructor(w, h) {
        this.ptr = instance.exports.emfunguscore_new(w, h);
      }
      step(n) { instance.exports.emfunguscore_step(this.ptr, n); }
      set_source(f, p) { instance.exports.emfunguscore_set_source(this.ptr, f, p); }
      trigger_pulse(m) { instance.exports.emfunguscore_trigger_pulse(this.ptr, m); }
      get_ez_ptr() { return instance.exports.emfunguscore_get_ez_ptr(this.ptr); }
      get_ez_len() { return instance.exports.emfunguscore_get_ez_len(this.ptr); }
      get_width() { return instance.exports.emfunguscore_get_width(this.ptr); }
      get_height() { return instance.exports.emfunguscore_get_height(this.ptr); }
      get_bio_energy() { return instance.exports.emfunguscore_get_bio_energy(this.ptr); }
      get_cell_potential_mv() { return instance.exports.emfunguscore_get_cell_potential_mv(this.ptr); }
      get_absorbed_power() { return instance.exports.emfunguscore_get_absorbed_power(this.ptr); }
      get_atp_rate() { return instance.exports.emfunguscore_get_atp_rate(this.ptr); }
      get_corona_events() { return instance.exports.emfunguscore_get_corona_events(this.ptr); }
    }

    const GRID_W = 128;
    const GRID_H = 128;
    const core = new EmFungusCore(GRID_W, GRID_H);
    core.set_source(2.45, 25.0);

    const canvas = document.getElementById('fdtd-canvas');
    const ctx = canvas.getContext('2d');
    const imgData = ctx.createImageData(GRID_W, GRID_H);

    // UI elements
    const mVolt = document.getElementById('m-volt');
    const mEnergy = document.getElementById('m-energy');
    const mPower = document.getElementById('m-power');
    const mCorona = document.getElementById('m-corona');
    const fpsCounter = document.getElementById('fps-counter');

    const slFreq = document.getElementById('sl-freq');
    const slPower = document.getElementById('sl-power');
    const slSteps = document.getElementById('sl-steps');
    const vFreq = document.getElementById('v-freq');
    const vPower = document.getElementById('v-power');
    const vSteps = document.getElementById('v-steps');
    const btnPulse = document.getElementById('btn-pulse');

    let subSteps = 4;
    slFreq.oninput = (e) => {
      vFreq.textContent = `${parseFloat(e.target.value).toFixed(2)} GHz`;
      core.set_source(parseFloat(e.target.value), parseFloat(slPower.value));
    };
    slPower.oninput = (e) => {
      vPower.textContent = `${e.target.value} dBm`;
      core.set_source(parseFloat(slFreq.value), parseFloat(e.target.value));
    };
    slSteps.oninput = (e) => {
      subSteps = parseInt(e.target.value);
      vSteps.textContent = `${subSteps} Steps`;
    };
    btnPulse.onclick = () => {
      core.trigger_pulse(4.5);
    };

    let lastTime = performance.now();
    let frameCount = 0;

    function renderLoop() {
      // 1. Advance Rust simulation core
      core.step(subSteps);

      // 2. Direct memory read from WebAssembly linear buffer (Zero Copy)
      const ezPtr = core.get_ez_ptr();
      const ezLen = core.get_ez_len();
      const ezView = new Float32Array(wasmMemory.buffer, ezPtr, ezLen);

      // 3. Color map Ez electric field
      const data = imgData.data;
      for (let i = 0; i < ezLen; i++) {
        const val = ezView[i];
        const pIdx = i * 4;

        if (val > 0) {
          // Positive E-field: Cyan to Electric Blue
          const norm = Math.min(255, Math.floor(val * 130));
          data[pIdx]     = 10;
          data[pIdx + 1] = Math.floor(norm * 0.85);
          data[pIdx + 2] = norm;
          data[pIdx + 3] = 255;
        } else {
          // Negative E-field: Deep Violet to Magenta
          const norm = Math.min(255, Math.floor(-val * 130));
          data[pIdx]     = Math.floor(norm * 0.95);
          data[pIdx + 1] = 15;
          data[pIdx + 2] = Math.floor(norm * 0.8);
          data[pIdx + 3] = 255;
        }
      }

      ctx.putImageData(imgData, 0, 0);

      // 4. Update HUD metrics
      mVolt.textContent = `${core.get_cell_potential_mv().toFixed(1)} mV`;
      mEnergy.textContent = `${core.get_bio_energy().toFixed(2)} J`;
      mPower.textContent = `${(core.get_absorbed_power() * 10).toFixed(1)} mW`;
      mCorona.textContent = core.get_corona_events();

      // FPS calculation
      frameCount++;
      const now = performance.now();
      if (now - lastTime >= 1000) {
        fpsCounter.textContent = `${frameCount} FPS`;
        frameCount = 0;
        lastTime = now;
      }

      requestAnimationFrame(renderLoop);
    }

    renderLoop();
  </script>
</body>
</html>
"""

final_html = template.replace("__WASM_BASE64__", wasm_b64)
with open('/Users/linyunjia/fungus-em-core/demo.html', 'w', encoding='utf-8') as f:
    f.write(final_html)

print("Demo successfully built at /Users/linyunjia/fungus-em-core/demo.html")
