/// Matches the original Shell bitmap conversion: straight alpha is not divided twice.
pub fn shell_bgra_to_rgba(pixels: &mut [u8]) -> Result<(), String> {
    if pixels.len() % 4 != 0 { return Err("Invalid BGRA buffer length".into()); }
    let has_alpha = pixels.chunks_exact(4).any(|p| p[3] != 0);
    let premultiplied = has_alpha && pixels.chunks_exact(4).all(|p| p[..3].iter().all(|&c| c <= p[3]));
    for p in pixels.chunks_exact_mut(4) {
        p.swap(0, 2);
        if !has_alpha { p[3] = 255; }
        else if premultiplied && p[3] > 0 && p[3] < 255 {
            let alpha = p[3] as u32;
            for c in &mut p[..3] { *c = ((*c as u32 * 255 + alpha / 2) / alpha).min(255) as u8; }
        }
    }
    Ok(())
}
