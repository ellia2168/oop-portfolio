//! Biological mycelium network dynamics, rectification kinetics, and bio-capacitance energy model.

use crate::fdtd::{Fdtd2D, MaterialType};

#[derive(Clone, Copy, Debug)]
pub enum NodeType {
    SpireTip,
    DishReflector,
    StemConduit,
    HyphaeTip,
}

#[derive(Clone, Debug)]
pub struct BioNode {
    pub x: usize,
    pub y: usize,
    pub node_type: NodeType,
    pub local_voltage_mv: f32,
    pub corona_active: bool,
}

pub struct MyceliumNetwork {
    pub nodes: Vec<BioNode>,
    pub bio_energy_joules: f32,
    pub cell_potential_mv: f32,
    pub total_corona_events: u32,
    pub atp_generation_rate: f32,
    pub growth_counter: u32,
}

impl MyceliumNetwork {
    pub fn new(grid_w: usize, grid_h: usize) -> Self {
        let cx = grid_w / 2;
        let cy = grid_h / 2 + 10;

        let mut nodes = Vec::new();

        // 1. Spire Tip (Monopole Receiver Apex)
        nodes.push(BioNode {
            x: cx,
            y: cy.saturating_sub(26),
            node_type: NodeType::SpireTip,
            local_voltage_mv: 0.0,
            corona_active: false,
        });

        // 2. Parabolic Concentrator Nodes
        for offset in [-18, -12, -6, 6, 12, 18] {
            let nx = (cx as i32 + offset) as usize;
            let ny = (cy as i32 + ((0.055 * (offset * offset) as f32) as i32)) as usize;
            nodes.push(BioNode {
                x: nx,
                y: ny,
                node_type: NodeType::DishReflector,
                local_voltage_mv: 0.0,
                corona_active: false,
            });
        }

        // 3. Stem Conduit Nodes
        for step in 1..4 {
            nodes.push(BioNode {
                x: cx,
                y: cy + step * 4,
                node_type: NodeType::StemConduit,
                local_voltage_mv: 0.0,
                corona_active: false,
            });
        }

        // 4. Initial Hyphae Branches on Substrate
        let sub_y = grid_h.saturating_sub(11);
        for &hx in &[cx.saturating_sub(15), cx.saturating_sub(8), cx + 8, cx + 15] {
            nodes.push(BioNode {
                x: hx,
                y: sub_y,
                node_type: NodeType::HyphaeTip,
                local_voltage_mv: 0.0,
                corona_active: false,
            });
        }

        Self {
            nodes,
            bio_energy_joules: 12.5,
            cell_potential_mv: 120.0,
            total_corona_events: 0,
            atp_generation_rate: 0.0,
            growth_counter: 0,
        }
    }

    /// Update bio-chemical metabolism from FDTD electric fields
    pub fn step(&mut self, fdtd: &mut Fdtd2D) {
        let mut total_rectified_current = 0.0;
        let w = fdtd.width;

        for node in &mut self.nodes {
            let idx = node.y * w + node.x;
            if idx >= fdtd.ez.len() {
                continue;
            }

            let field_ez = fdtd.ez[idx].abs();

            // Asymmetric biological diode rectification (Schottky-like membrane model)
            // Induced RF potential converted to rectified DC proton gradient
            let rectification_efficiency = match node.node_type {
                NodeType::SpireTip => 4.2,      // Highest field concentration
                NodeType::DishReflector => 2.5, // Parabolic focusing
                NodeType::HyphaeTip => 1.8,     // Fractal array collector
                NodeType::StemConduit => 0.8,   // Waveguide conduit
            };

            let rectified = field_ez * rectification_efficiency;
            node.local_voltage_mv = node.local_voltage_mv * 0.92 + rectified * 85.0;
            total_rectified_current += rectified;

            // Corona discharge check (Dielectric breakdown of adjacent air)
            let breakdown_threshold = match node.node_type {
                NodeType::SpireTip => 1.85,
                NodeType::HyphaeTip => 2.40,
                _ => 3.50,
            };

            if field_ez > breakdown_threshold || node.local_voltage_mv > 650.0 {
                node.corona_active = true;
                self.total_corona_events += 1;
                // Dissipate excess potential into ionization spark
                node.local_voltage_mv *= 0.5;
            } else {
                node.corona_active = false;
            }
        }

        // Biological Capacitor charge dynamics: dV/dt = (I_rect - I_leak - I_metabolism) / C
        let metabolic_burn = 0.045; // Base living cost
        let leak_rate = self.cell_potential_mv * 0.0008;
        let net_current = total_rectified_current * 0.15 - metabolic_burn - leak_rate;

        self.cell_potential_mv = (self.cell_potential_mv + net_current * 8.0).clamp(10.0, 950.0);
        self.bio_energy_joules = (self.bio_energy_joules + net_current * 0.02).max(0.0);
        self.atp_generation_rate = (total_rectified_current * 14.5).max(0.0);

        // Electrotropism: If bio-energy is high, grow new hyphae towards highest field gradient
        self.growth_counter += 1;
        if self.growth_counter % 80 == 0 && self.bio_energy_joules > 18.0 && self.nodes.len() < 48 {
            self.grow_hyphae(fdtd);
        }
    }

    /// Sprout new hyphal branch towards ambient field gradient
    fn grow_hyphae(&mut self, fdtd: &mut Fdtd2D) {
        let w = fdtd.width;
        let h = fdtd.height;
        let sub_y = h.saturating_sub(11);

        // Find current furthest left and right tips
        let mut min_x = w;
        let mut max_x = 0;
        for node in &self.nodes {
            if matches!(node.node_type, NodeType::HyphaeTip) {
                if node.x < min_x { min_x = node.x; }
                if node.x > max_x { max_x = node.x; }
            }
        }

        // Branch left or right along conductive substrate
        if min_x > 10 {
            let new_x = min_x.saturating_sub(4);
            self.nodes.push(BioNode {
                x: new_x,
                y: sub_y,
                node_type: NodeType::HyphaeTip,
                local_voltage_mv: 0.0,
                corona_active: false,
            });
            // Update material conductivity in FDTD grid
            let idx = sub_y * w + new_x;
            if idx < fdtd.material.len() {
                fdtd.material[idx] = MaterialType::MelaninBioTissues;
            }
        } else if max_x + 10 < w {
            let new_x = max_x + 4;
            self.nodes.push(BioNode {
                x: new_x,
                y: sub_y,
                node_type: NodeType::HyphaeTip,
                local_voltage_mv: 0.0,
                corona_active: false,
            });
            let idx = sub_y * w + new_x;
            if idx < fdtd.material.len() {
                fdtd.material[idx] = MaterialType::MelaninBioTissues;
            }
        }
    }
}
