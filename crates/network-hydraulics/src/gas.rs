//! Isothermal dry-gas pipe segment.
//!
//! Friction factor and Reynolds number come from `pipeline-physics`. The
//! pressure-squared integration is the compressible isothermal Darcy form:
//!
//! ```text
//! P_up² − P_down² = f · (L/D) · Z · R · T · G²
//! ```
//!
//! with mass flux `G = ṁ / A` and `f` iterated against the mean-pressure density.

use pipeline_physics::fluid_dynamics::STEEL_ROUGHNESS;
use pipeline_physics::{flow_velocity, friction_factor, reynolds_number};
use serde::{Deserialize, Serialize};

use crate::error::{HydraulicsError, Result};

/// Fluid properties held constant along the network for one study.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GasProperties {
    pub temperature_k: f64,
    pub z_factor: f64,
    /// Specific gas constant R_specific = R_univ / M, J/(kg·K).
    pub specific_gas_constant: f64,
    /// Dynamic viscosity μ, Pa·s (converted to ν = μ/ρ inside the iteration).
    pub dynamic_viscosity_pa_s: f64,
}

impl GasProperties {
    /// Dry methane-like gas at 15 °C, Z = 1.
    pub fn methane_standard() -> Self {
        Self {
            temperature_k: 288.15,
            z_factor: 1.0,
            specific_gas_constant: 518.3,
            dynamic_viscosity_pa_s: 1.10e-5,
        }
    }

    pub fn density_at(&self, pressure_pa: f64) -> f64 {
        pressure_pa / (self.z_factor * self.specific_gas_constant * self.temperature_k)
    }

    /// Convert mass flow (kg/s) to Mcf/d at 101.325 kPa and the gas temperature.
    pub fn kg_s_to_mcf_d(&self, mass_flow_kg_s: f64) -> f64 {
        let rho = self.density_at(101_325.0);
        let m3_s = mass_flow_kg_s / rho;
        m3_s / 28.316846592 * 86_400.0
    }
}

/// Circular pipe geometry for one edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipeGeometry {
    pub length_m: f64,
    pub diameter_m: f64,
    #[serde(default = "default_roughness")]
    pub roughness_m: f64,
}

fn default_roughness() -> f64 {
    STEEL_ROUGHNESS
}

impl PipeGeometry {
    pub fn new(length_m: f64, diameter_m: f64) -> Self {
        Self {
            length_m,
            diameter_m,
            roughness_m: STEEL_ROUGHNESS,
        }
    }
}

pub fn pipe_area(diameter_m: f64) -> f64 {
    std::f64::consts::PI * diameter_m * diameter_m / 4.0
}

/// Upstream pressure from known downstream pressure and mass flow (kg/s).
pub fn isothermal_upstream_pressure(
    p_down_pa: f64,
    mass_flow_kg_s: f64,
    pipe: &PipeGeometry,
    gas: &GasProperties,
) -> Result<f64> {
    if p_down_pa <= 0.0 {
        return Err(HydraulicsError::InvalidInput(
            "downstream pressure must be positive".into(),
        ));
    }
    if pipe.length_m <= 0.0 || pipe.diameter_m <= 0.0 {
        return Err(HydraulicsError::InvalidInput(
            "pipe length and diameter must be positive".into(),
        ));
    }
    if mass_flow_kg_s < 0.0 {
        return Err(HydraulicsError::InvalidInput(
            "mass flow must be non-negative".into(),
        ));
    }
    if mass_flow_kg_s == 0.0 {
        return Ok(p_down_pa);
    }

    let area = pipe_area(pipe.diameter_m);
    let g = mass_flow_kg_s / area;
    let mut p_up = p_down_pa * 1.05;

    for _ in 0..80 {
        let p_avg = 0.5 * (p_up + p_down_pa);
        let rho = gas.density_at(p_avg);
        if rho <= 0.0 {
            return Err(HydraulicsError::InvalidInput(
                "non-positive gas density in pipe iteration".into(),
            ));
        }
        let nu = gas.dynamic_viscosity_pa_s / rho;
        let velocity = flow_velocity(mass_flow_kg_s / rho, pipe.diameter_m);
        let re = reynolds_number(velocity, pipe.diameter_m, nu);
        let f = friction_factor(re, pipe.roughness_m, pipe.diameter_m);
        if !f.is_finite() || f <= 0.0 {
            return Err(HydraulicsError::InvalidInput(format!(
                "non-finite friction factor at Re={re}"
            )));
        }

        let drop = f
            * (pipe.length_m / pipe.diameter_m)
            * gas.z_factor
            * gas.specific_gas_constant
            * gas.temperature_k
            * g
            * g;
        let p_up_sq = p_down_pa * p_down_pa + drop;
        let p_next = p_up_sq.sqrt();
        if (p_next - p_up).abs() < 0.1 {
            return Ok(p_next);
        }
        p_up = p_next;
    }

    Ok(p_up)
}

/// Mass flow (kg/s) that produces the given upstream/downstream pressures.
///
/// Independent check path for a single pipe: iterates `f` the same way as the
/// upstream-pressure solve.
pub fn mass_flow_from_pressures(
    p_up_pa: f64,
    p_down_pa: f64,
    pipe: &PipeGeometry,
    gas: &GasProperties,
) -> Result<f64> {
    if p_up_pa < p_down_pa {
        return Err(HydraulicsError::InvalidInput(
            "upstream pressure must be >= downstream pressure for forward flow".into(),
        ));
    }
    if (p_up_pa - p_down_pa).abs() < 1e-9 {
        return Ok(0.0);
    }

    let area = pipe_area(pipe.diameter_m);
    let mut m = 1.0;
    for _ in 0..80 {
        let p_avg = 0.5 * (p_up_pa + p_down_pa);
        let rho = gas.density_at(p_avg);
        let nu = gas.dynamic_viscosity_pa_s / rho;
        let velocity = flow_velocity(m / rho, pipe.diameter_m);
        let re = reynolds_number(velocity, pipe.diameter_m, nu);
        let f = friction_factor(re, pipe.roughness_m, pipe.diameter_m);
        let coeff = f
            * (pipe.length_m / pipe.diameter_m)
            * gas.z_factor
            * gas.specific_gas_constant
            * gas.temperature_k
            / (area * area);
        let m_next = ((p_up_pa * p_up_pa - p_down_pa * p_down_pa) / coeff).sqrt();
        if (m_next - m).abs() < 1e-9 {
            return Ok(m_next);
        }
        m = m_next;
    }
    Ok(m)
}
