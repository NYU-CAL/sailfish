mod patch;
pub mod iso2d;

pub use patch::{host, ffi};

#[cfg(feature = "cuda")]
pub use patch::device;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Number of zones on the i-axis
    pub ni: i32,
    /// Number of zones on the j-axis
    pub nj: i32,
    /// Left coordinate edge of the domain
    pub x0: f64,
    /// Right coordinate edge of the domain
    pub y0: f64,
    /// Zone spacing on the i-axis
    pub dx: f64,
    /// Zone spacing on the j-axis
    pub dy: f64,
}

impl Mesh {
    /// Creates a square mesh that is centered on the origin, with the given
    /// number of zones on each side.
    pub fn centered_square(domain_radius: f64, resolution: u32) -> Self {
        Self {
            x0: -domain_radius,
            y0: -domain_radius,
            ni: resolution as i32,
            nj: resolution as i32,
            dx: 2.0 * domain_radius / resolution as f64,
            dy: 2.0 * domain_radius / resolution as f64,
        }
    }
    /// Returns the number of zones on the i-axis as a `u32`.
    pub fn ni(&self) -> u32 {
        self.ni as u32
    }
    /// Returns the number of zones on the j-axis as a `u32`.
    pub fn nj(&self) -> u32 {
        self.nj as u32
    }
    /// Returns the number of total zones (`ni * nj`) as a `usize`.
    pub fn num_total_zones(&self) -> usize {
        (self.ni * self.nj) as usize
    }
    /// Returns the number of zones in each direction
    pub fn shape(&self) -> [u32; 2] {
        [self.ni as u32, self.nj as u32]
    }
    /// Returns the row-major memory strides. Assumes 3 conserved
    /// quantities.
    pub fn strides(&self) -> [usize; 2] {
        [self.nj as usize * 3, 3]
    }
    /// Returns the cell-center [x, y] coordinate at a given index.
    /// Out-of-bounds indexes are allowed.
    pub fn cell_coordinates(&self, i: i32, j: i32) -> [f64; 2] {
        let x = self.x0 + (i as f64 + 0.5) * self.dx;
        let y = self.y0 + (j as f64 + 0.5) * self.dy;
        [x, y]
    }
}

#[derive(Clone, Copy)]
pub enum ExecutionMode {
    CPU,
    OMP,
    GPU,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum EquationOfState {
    Isothermal { sound_speed: f64 },
    LocallyIsothermal { mach_number: f64 },
    GammaLaw { gamma_law_index: f64 },
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PointMass {
    pub x: f64,
    pub y: f64,
    pub mass: f64,
    pub rate: f64,
    pub radius: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum BufferZone {
    None,
    Keplerian {
        surface_density: f64,
        central_mass: f64,
        driving_rate: f64,
        outer_radius: f64,
        onset_width: f64,
    }
}

pub trait Solve {
    fn primitive(&self) -> Vec<f64>;
    fn advance(&mut self, rk_order: usize, dt: f64);
}

pub mod cpu {
    use super::*;

    pub struct Solver {
        mesh: Mesh,
        primitive1: host::Patch,
        primitive2: host::Patch,
        conserved0: host::Patch,
    }

    impl Solver {
        pub fn new(mesh: super::Mesh, primitive: Vec<f64>) -> Self {
            let primitive1 = host::Patch::from_vec([-2, -2], [mesh.ni() + 4, mesh.nj() + 4], 3, &primitive);
            let primitive2 = host::Patch::zeros([-2, -2], [mesh.ni() + 4, mesh.nj() + 4], 3);
            let conserved0 = host::Patch::zeros([0, 0], mesh.shape(), 3);
            Self {
                mesh, primitive1, primitive2, conserved0,
            }
        }
    }

    impl Solve for Solver {
        fn primitive(&self) -> Vec<f64> {
            self.primitive1.to_vec()
        }
        fn advance(&mut self, rk_order: usize, dt: f64) {
            if rk_order != 1 {
                todo!()
            }
            iso2d::primitive_to_conserved_cpu(&self.primitive1, &mut self.conserved0);
            iso2d::advance_rk_cpu(&self.mesh, &self.conserved0, &self.primitive1, &mut self.primitive2, 0.0, dt);
            std::mem::swap(&mut self.primitive1, &mut self.primitive2);
        }
    }
}

pub mod omp {
    use super::*;

    pub struct Solver {
        mesh: Mesh,
        primitive1: host::Patch,
        primitive2: host::Patch,
        conserved0: host::Patch,
    }

    impl Solver {
        pub fn new(mesh: super::Mesh, primitive: Vec<f64>) -> Self {
            let primitive1 = host::Patch::from_vec([-2, -2], [mesh.ni() + 4, mesh.nj() + 4], 3, &primitive);
            let primitive2 = host::Patch::zeros([-2, -2], [mesh.ni() + 4, mesh.nj() + 4], 3);
            let conserved0 = host::Patch::zeros([0, 0], mesh.shape(), 3);
            Self {
                mesh, primitive1, primitive2, conserved0,
            }
        }
    }

    impl Solve for Solver {
        fn primitive(&self) -> Vec<f64> {
            self.primitive1.to_vec()
        }
        fn advance(&mut self, rk_order: usize, dt: f64) {
            if rk_order != 1 {
                todo!()
            }
            iso2d::primitive_to_conserved_omp(&self.primitive1, &mut self.conserved0);
            iso2d::advance_rk_omp(&self.mesh, &self.conserved0, &self.primitive1, &mut self.primitive2, 0.0, dt);
            std::mem::swap(&mut self.primitive1, &mut self.primitive2);
        }
    }
}
