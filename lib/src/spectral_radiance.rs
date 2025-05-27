/// Spectral radiance results, based on IASI L1C files
#[derive(Debug,Clone)]
pub struct SpectralRadiance {
    /// Starting wavenumber (cm^-1)
    pub nu1:f32,

    /// Wavenumber step (cm^-1)
    pub dnu:f32,

    /// Instrument channel corresponding to the starting
    /// wavenumber (for reference, as wavenumbers could
    /// change slightly due to calibration)
    pub ichan1:usize,

    /// The spectral radiance values W/m^2/sr/(cm^-1)
    pub radiance:Vec<f32>
}

impl SpectralRadiance {
    pub fn get_channel(&self,ichan:usize)->Option<f32> {
	if self.ichan1 <= ichan && ichan < self.ichan1 + self.radiance.len() {
	    Some(self.radiance[ichan - self.ichan1])
	} else {
	    None
	}
    }
}
