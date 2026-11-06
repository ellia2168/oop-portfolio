//! High-performance Rust & WebAssembly simulation core for Radiomyces electrovorus.
//!
//! Provides real-time FDTD electrodynamics coupled with biological rectification
//! kinetics and ATP metabolic flux models.

pub mod fdtd;
pub mod mycelium;

use fdtd::Fdtd2D;
use mycelium::MyceliumNetwork;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct EmFungusCore {
    fdtd: Fdtd2D,
    network: MyceliumNetwork,
    absorbed_power: f32,
}

#[wasm_bindgen]
impl EmFungusCore {
    /// Instantiate a new simulation core with specified FDTD grid dimensions.
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Self {
        // Enforce sensible boundary ranges
        let w = width.clamp(40, 256);
        let h = height.clamp(40, 256);

        let fdtd = Fdtd2D::new(w, h);
        let network = MyceliumNetwork::new(w, h);

        Self {
            fdtd,
            network,
            absorbed_power: 0.0,
        }
    }

    /// Advance physics and bio-metabolism by multiple sub-steps per animation frame
    pub fn step(&mut self, sub_steps: usize) {
        let steps = sub_steps.clamp(1, 16);
        for _ in 0..steps {
            self.fdtd.update();
            self.network.step(&mut self.fdtd);
        }
        self.absorbed_power = self.fdtd.compute_absorbed_power();
    }

    /// Adjust ambient carrier frequency and RF power flux
    pub fn set_source(&mut self, freq_ghz: f32, power_dbm: f32) {
        self.fdtd.freq_hz = (freq_ghz * 1e9).clamp(0.1e9, 20.0e9);

        // Convert dBm to normalized field amplitude: P_mw = 10^(dBm/10)
        let p_mw = 10.0f32.powf(power_dbm / 10.0);
        let amplitude = (p_mw * 0.02).sqrt().clamp(0.05, 5.0);
        self.fdtd.source_amplitude = amplitude;
    }

    /// Inject a transient high-energy microwave/radar pulse
    pub fn trigger_pulse(&mut self, magnitude: f32) {
        self.fdtd.pulse_active = magnitude.clamp(0.5, 8.0);
    }

    /// Direct pointer to Ez field array for zero-copy WebGL/Canvas rendering
    pub fn get_ez_ptr(&self) -> *const f32 {
        self.fdtd.ez.as_ptr()
    }

    pub fn get_ez_len(&self) -> usize {
        self.fdtd.ez.len()
    }

    pub fn get_width(&self) -> usize {
        self.fdtd.width
    }

    pub fn get_height(&self) -> usize {
        self.fdtd.height
    }

    pub fn get_bio_energy(&self) -> f32 {
        self.network.bio_energy_joules
    }

    pub fn get_cell_potential_mv(&self) -> f32 {
        self.network.cell_potential_mv
    }

    pub fn get_absorbed_power(&self) -> f32 {
        self.absorbed_power
    }

    pub fn get_atp_rate(&self) -> f32 {
        self.network.atp_generation_rate
    }

    pub fn get_corona_events(&self) -> u32 {
        self.network.total_corona_events
    }

    /// Flat array of active nodes: [x, y, voltage_mv, is_corona (1.0/0.0), ...]
    pub fn get_nodes_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.network.nodes.len() * 4);
        for n in &self.network.nodes {
            data.push(n.x as f32);
            data.push(n.y as f32);
            data.push(n.local_voltage_mv);
            data.push(if n.corona_active { 1.0 } else { 0.0 });
        }
        data
    }
}
