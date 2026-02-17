//! Transfer functions from audio space to biophysical space.
//! These are placeholder linear models; in practice they'd be calibrated from your data.

use crate::audio_nanopolytope::AudioState;

/// A simple linear transfer: maps audio amplitude to Energy, carrier to theta proxy, etc.
pub fn default_transfer(audio: &AudioState) -> [f64; 5] {
    // This is an example – replace with your calibrated values.
    let e = 0.2 * audio.amplitude + 0.1 * (audio.session_duration_sec / 600.0).min(1.0);
    let m_prot = 0.0; // audio shouldn't affect protein burden
    let s_bio = 0.3 * audio.beat_hz / 20.0; // beat frequency influences stress
    let theta = 0.5 * (audio.carrier_hz / 500.0); // carrier affects brain state proxy
    let t = 0.0; // no temperature change
    [e, m_prot, s_bio, theta, t]
}
