/// Échelle linéaire pour convertir les volumes en coordonnées Y
/// 
/// Gère la conversion entre valeurs de volume et coordonnées écran,
/// avec support pour l'autoscaling et les marges.
#[derive(Debug, Clone)]
pub struct VolumeScale {
    /// Volume minimum visible
    min_volume: f64,
    /// Volume maximum visible
    max_volume: f64,
    /// Hauteur disponible en pixels
    height: f32,
    /// Marge verticale en pourcentage (0.0 = pas de marge)
    margin_ratio: f32,
}

impl VolumeScale {
    /// Crée une nouvelle échelle de volume
    pub fn new(min_volume: f64, max_volume: f64, height: f32) -> Self {
        Self {
            min_volume,
            max_volume,
            height,
            margin_ratio: 0.05, // 5% de marge par défaut (plus petit que pour les prix)
        }
    }

    /// Met à jour la hauteur disponible
    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    /// Retourne la plage de volume actuelle
    pub fn volume_range(&self) -> (f64, f64) {
        (self.min_volume, self.max_volume)
    }

    /// Convertit un volume en coordonnée Y (0 = haut de l'écran)
    /// Pour les volumes, on veut que le volume max soit en haut et le min en bas
    /// Retourne la position Y du HAUT de la barre pour ce volume
    pub fn volume_to_y(&self, volume: f64) -> f32 {
        let range = self.max_volume - self.min_volume;
        if range == 0.0 {
            return self.height; // Si pas de range, mettre en bas
        }

        // Appliquer les marges uniquement en haut (pour le max), pas en bas
        // Cela garantit que volume 0 = bas de l'écran (y = height)
        let margin = range * self.margin_ratio as f64;
        let effective_min = self.min_volume; // Pas de marge en bas
        let effective_max = self.max_volume + margin; // Marge uniquement en haut
        let effective_range = effective_max - effective_min;

        // Pour les volumes, on veut que le volume max soit en haut (Y proche de 0)
        // et le volume min en bas (Y proche de height)
        let normalized = (volume - effective_min) / effective_range;
        self.height * (1.0 - normalized as f32)
    }

    /// Convertit une coordonnée Y en volume (inverse de volume_to_y)
    pub fn y_to_volume(&self, y: f32) -> f64 {
        let range = self.max_volume - self.min_volume;
        if range == 0.0 {
            return self.min_volume;
        }

        let margin = range * self.margin_ratio as f64;
        let effective_min = self.min_volume;
        let effective_max = self.max_volume + margin;
        let effective_range = effective_max - effective_min;

        // Inverse de la formule dans volume_to_y
        let normalized = 1.0 - (y / self.height);
        effective_min + (normalized as f64 * effective_range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_scale() {
        let scale = VolumeScale::new(0.0, 1000.0, 100.0);
        
        // Volume max devrait être en haut (Y proche de 0)
        let y_max = scale.volume_to_y(1000.0);
        assert!(y_max < 10.0);
        
        // Volume min devrait être en bas (Y proche de height)
        let y_min = scale.volume_to_y(0.0);
        assert!(y_min > 90.0);
        
        // Volume médian devrait être au milieu
        let y_mid = scale.volume_to_y(500.0);
        assert!((y_mid - 50.0).abs() < 5.0);
    }
}

