//! FDTD (Finite-Difference Time-Domain) 2D TMz mode electromagnetic wave engine.
//!
//! Models wave propagation in free space and its interaction with the conductive/dielectric
//! biological tissues of Radiomyces electrovorus.

use std::f32::consts::PI;

pub const C_0: f32 = 299_792_458.0;      // Speed of light in vacuum (m/s)
pub const EPS_0: f32 = 8.854_187_817e-12; // Permittivity of free space (F/m)
pub const MU_0: f32 = 1.256_637_061e-6;  // Permeability of free space (H/m)

#[derive(Clone, Copy, PartialEq)]
pub enum MaterialType {
    Air,
    SubstrateConductive, // Ground plane or PCB trace
    MelaninBioTissues,   // Permittivity ~4.0, moderate conductivity
    MetallicSpire,       // High conductivity receptor
    ResonantDish,        // High dielectric parabolic concentrator
}

pub struct Fdtd2D {
    pub width: usize,
    pub height: usize,
    pub dx: f32,
    pub dy: f32,
    pub dt: f32,

    // Field components
    pub ez: Vec<f32>,
    pub hx: Vec<f32>,
    pub hy: Vec<f32>,

    // Material coefficient grids
    pub ca: Vec<f32>,
    pub cb: Vec<f32>,
    pub da: Vec<f32>,
    pub db: Vec<f32>,

    // Material tags
    pub material: Vec<MaterialType>,

    // Absorbing boundary buffers (Mur 1st order)
    ez_boundary_left_prev: Vec<f32>,
    ez_boundary_right_prev: Vec<f32>,
    ez_boundary_top_prev: Vec<f32>,
    ez_boundary_bottom_prev: Vec<f32>,

    // Source settings
    pub time: f32,
    pub freq_hz: f32,
    pub source_amplitude: f32,
    pub pulse_active: f32,
    pub source_x: usize,
    pub source_y: usize,
}

impl Fdtd2D {
    pub fn new(width: usize, height: usize) -> Self {
        let dx: f32 = 0.002; // 2mm spatial resolution
        let dy: f32 = 0.002;
        // Courant stability condition: dt <= 1 / (c * sqrt(1/dx^2 + 1/dy^2))
        let dt = 1.0f32 / (C_0 * (1.0f32 / (dx * dx) + 1.0f32 / (dy * dy)).sqrt()) * 0.95;

        let total_cells = width * height;

        let mut instance = Self {
            width,
            height,
            dx,
            dy,
            dt,
            ez: vec![0.0; total_cells],
            hx: vec![0.0; total_cells],
            hy: vec![0.0; total_cells],
            ca: vec![1.0; total_cells],
            cb: vec![dt / (EPS_0 * dx); total_cells],
            da: vec![1.0; total_cells],
            db: vec![dt / (MU_0 * dx); total_cells],
            material: vec![MaterialType::Air; total_cells],
            ez_boundary_left_prev: vec![0.0; height],
            ez_boundary_right_prev: vec![0.0; height],
            ez_boundary_top_prev: vec![0.0; width],
            ez_boundary_bottom_prev: vec![0.0; width],
            time: 0.0,
            freq_hz: 2.45e9, // 2.45 GHz default
            source_amplitude: 1.0,
            pulse_active: 0.0,
            source_x: width / 2,
            source_y: height / 6,
        };

        instance.init_materials();
        instance
    }

    /// Place biological structures and conductive substrate in the grid
    pub fn init_materials(&mut self) {
        let cx = (self.width / 2) as f32;
        let cy = (self.height / 2 + 10) as f32;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let fx = x as f32;
                let fy = y as f32;

                // 1. Ground metallic base
                if y >= self.height - 12 {
                    self.set_cell_material(idx, MaterialType::SubstrateConductive, 20.0, 50.0);
                    continue;
                }

                // 2. Parabolic reflector dish: (y - y0) = a * (x - x0)^2
                let dx = fx - cx;
                let dy = fy - cy;
                let parabola_y = 0.055 * (dx * dx);

                if dy >= parabola_y - 2.5 && dy <= parabola_y + 2.0 && dx.abs() < 28.0 {
                    self.set_cell_material(idx, MaterialType::ResonantDish, 4.5, 0.8);
                }
                // 3. Central Monopole Spire
                else if dx.abs() <= 1.5 && fy >= cy - 26.0 && fy <= cy + 5.0 {
                    self.set_cell_material(idx, MaterialType::MetallicSpire, 12.0, 15.0);
                }
                // 4. Stipe (stem) down to substrate
                else if dx.abs() <= 3.5 && fy >= cy && fy < (self.height - 12) as f32 {
                    self.set_cell_material(idx, MaterialType::MelaninBioTissues, 3.8, 1.2);
                }
            }
        }
    }

    fn set_cell_material(&mut self, idx: usize, mat: MaterialType, eps_r: f32, sigma: f32) {
        self.material[idx] = mat;
        let eps = eps_r * EPS_0;
        let sig = sigma;

        // FDTD lossy medium coefficients
        self.ca[idx] = (1.0 - (sig * self.dt) / (2.0 * eps)) / (1.0 + (sig * self.dt) / (2.0 * eps));
        self.cb[idx] = (self.dt / (eps * self.dx)) / (1.0 + (sig * self.dt) / (2.0 * eps));
    }

    /// Single FDTD time-step
    pub fn update(&mut self) {
        let w = self.width;
        let h = self.height;
        let dt_over_mu_dx = self.dt / (MU_0 * self.dx);
        let dt_over_mu_dy = self.dt / (MU_0 * self.dy);

        // 1. Update Hx and Hy from Ez
        for y in 0..(h - 1) {
            for x in 0..(w - 1) {
                let idx = y * w + x;
                let ez_curr = self.ez[idx];
                let ez_y_plus = self.ez[(y + 1) * w + x];
                let ez_x_plus = self.ez[y * w + (x + 1)];

                self.hx[idx] -= dt_over_mu_dy * (ez_y_plus - ez_curr);
                self.hy[idx] += dt_over_mu_dx * (ez_x_plus - ez_curr);
            }
        }

        // 2. Update Ez from Hx and Hy (excluding boundaries)
        for y in 1..(h - 1) {
            for x in 1..(w - 1) {
                let idx = y * w + x;
                let chx = self.hx[idx] - self.hx[(y - 1) * w + x];
                let chy = self.hy[idx] - self.hy[y * w + (x - 1)];

                self.ez[idx] = self.ca[idx] * self.ez[idx] + self.cb[idx] * (chy - chx);
            }
        }

        // 3. Inject Source
        self.time += self.dt;
        let src_idx = self.source_y * w + self.source_x;
        let omega = 2.0 * PI * self.freq_hz;
        let wave_val = (omega * self.time).sin() * self.source_amplitude;

        // Pulsed excitation component
        let pulse_val = if self.pulse_active > 0.001 {
            let p = self.pulse_active * 2.5 * (omega * self.time * 2.0).sin();
            self.pulse_active *= 0.985;
            p
        } else {
            0.0
        };

        // Add planar wave front across horizontal line around source_y
        for x in (w / 4)..(3 * w / 4) {
            let i = self.source_y * w + x;
            self.ez[i] += (wave_val + pulse_val) * 0.45;
        }
        self.ez[src_idx] += wave_val + pulse_val;

        // 4. Apply Mur 1st-order Absorbing Boundary Condition (ABC)
        self.apply_mur_abc();
    }

    fn apply_mur_abc(&mut self) {
        let w = self.width;
        let h = self.height;
        let coeff = (C_0 * self.dt - self.dx) / (C_0 * self.dt + self.dx);

        // Left and Right boundaries
        for y in 0..h {
            let left_idx = y * w;
            let left_inner = y * w + 1;
            let right_idx = y * w + (w - 1);
            let right_inner = y * w + (w - 2);

            let ez_left = self.ez[left_inner] + coeff * (self.ez[left_idx] - self.ez_boundary_left_prev[y]);
            self.ez_boundary_left_prev[y] = self.ez[left_inner];
            self.ez[left_idx] = ez_left;

            let ez_right = self.ez[right_inner] + coeff * (self.ez[right_idx] - self.ez_boundary_right_prev[y]);
            self.ez_boundary_right_prev[y] = self.ez[right_inner];
            self.ez[right_idx] = ez_right;
        }

        // Top and Bottom boundaries
        for x in 0..w {
            let top_idx = x;
            let top_inner = w + x;
            let bot_idx = (h - 1) * w + x;
            let bot_inner = (h - 2) * w + x;

            let ez_top = self.ez[top_inner] + coeff * (self.ez[top_idx] - self.ez_boundary_top_prev[x]);
            self.ez_boundary_top_prev[x] = self.ez[top_inner];
            self.ez[top_idx] = ez_top;

            let ez_bot = self.ez[bot_inner] + coeff * (self.ez[bot_idx] - self.ez_boundary_bottom_prev[x]);
            self.ez_boundary_bottom_prev[x] = self.ez[bot_inner];
            self.ez[bot_idx] = ez_bot;
        }
    }

    /// Calculate total power currently dissipated/absorbed in fungus tissues
    pub fn compute_absorbed_power(&self) -> f32 {
        let mut power = 0.0;
        for i in 0..self.ez.len() {
            if self.material[i] != MaterialType::Air {
                let e = self.ez[i];
                power += e * e;
            }
        }
        power * 0.015
    }
}
