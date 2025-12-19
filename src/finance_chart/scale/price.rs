/// Échelle linéaire pour convertir les prix en coordonnées Y
/// 
/// Gère la conversion entre valeurs de prix et coordonnées écran,
/// avec support pour l'autoscaling et les marges.
#[derive(Debug, Clone)]
pub struct PriceScale {
    /// Prix minimum visible
    min_price: f64,
    /// Prix maximum visible
    max_price: f64,
    /// Hauteur disponible en pixels
    height: f32,
    /// Marge verticale en pourcentage (0.0 = pas de marge)
    margin_ratio: f32,
}

impl PriceScale {
    /// Crée une nouvelle échelle de prix
    pub fn new(min_price: f64, max_price: f64, height: f32) -> Self {
        Self {
            min_price,
            max_price,
            height,
            margin_ratio: 0.1, // 10% de marge par défaut
        }
    }

    /// Met à jour la hauteur disponible
    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    /// Met à jour la plage de prix (autoscaling)
    pub fn set_price_range(&mut self, min: f64, max: f64) {
        self.min_price = min;
        self.max_price = max;
    }

    /// Retourne la plage de prix actuelle
    pub fn price_range(&self) -> (f64, f64) {
        (self.min_price, self.max_price)
    }

    /// Convertit un prix en coordonnée Y (0 = haut de l'écran)
    pub fn price_to_y(&self, price: f64) -> f32 {
        let range = self.max_price - self.min_price;
        if range == 0.0 {
            return self.height / 2.0;
        }

        // Appliquer les marges
        let margin = range * self.margin_ratio as f64;
        let effective_min = self.min_price - margin;
        let effective_max = self.max_price + margin;
        let effective_range = effective_max - effective_min;

        // Inverser Y (0 = haut, height = bas)
        let normalized = (price - effective_min) / effective_range;
        self.height * (1.0 - normalized as f32)
    }

    /// Convertit une coordonnée Y en prix
    pub fn y_to_price(&self, y: f32) -> f64 {
        let range = self.max_price - self.min_price;
        if self.height == 0.0 {
            return self.min_price;
        }

        // Appliquer les marges (inverse de price_to_y)
        let margin = range * self.margin_ratio as f64;
        let effective_min = self.min_price - margin;
        let effective_max = self.max_price + margin;
        let effective_range = effective_max - effective_min;

        // Inverser Y (0 = haut = prix max, height = bas = prix min)
        let normalized = 1.0 - (y as f64 / self.height as f64);
        effective_min + (normalized * effective_range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_scale() {
        let scale = PriceScale::new(100.0, 200.0, 100.0);
        
        // Prix max devrait être en haut (Y proche de 0)
        let y_max = scale.price_to_y(200.0);
        assert!(y_max < 10.0);
        
        // Prix min devrait être en bas (Y proche de height)
        let y_min = scale.price_to_y(100.0);
        assert!(y_min > 90.0);
        
        // Prix médian devrait être au milieu
        let y_mid = scale.price_to_y(150.0);
        assert!((y_mid - 50.0).abs() < 5.0);
    }
}

