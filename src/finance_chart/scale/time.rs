/// Échelle temporelle pour convertir les timestamps en coordonnées X
/// 
/// Gère la conversion entre timestamps Unix et coordonnées écran,
/// avec support pour le zoom et le pan.
#[derive(Debug, Clone)]
pub struct TimeScale {
    /// Timestamp minimum visible
    min_time: i64,
    /// Timestamp maximum visible
    max_time: i64,
    /// Largeur disponible en pixels
    width: f32,
}

impl TimeScale {
    /// Crée une nouvelle échelle temporelle
    pub fn new(min_time: i64, max_time: i64, width: f32) -> Self {
        Self {
            min_time,
            max_time,
            width,
        }
    }

    /// Met à jour la largeur disponible
    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    /// Met à jour la plage temporelle
    pub fn set_time_range(&mut self, min: i64, max: i64) {
        self.min_time = min;
        self.max_time = max;
    }

    /// Retourne la plage temporelle actuelle
    pub fn time_range(&self) -> (i64, i64) {
        (self.min_time, self.max_time)
    }

    /// Convertit un timestamp en coordonnée X
    pub fn time_to_x(&self, timestamp: i64) -> f32 {
        let range = self.max_time - self.min_time;
        if range == 0 {
            return self.width / 2.0;
        }

        let normalized = (timestamp - self.min_time) as f64 / range as f64;
        normalized as f32 * self.width
    }

    /// Convertit une coordonnée X en timestamp
    pub fn x_to_time(&self, x: f32) -> i64 {
        let range = self.max_time - self.min_time;
        if self.width == 0.0 {
            return self.min_time;
        }

        let normalized = x as f64 / self.width as f64;
        self.min_time + (normalized * range as f64) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_scale() {
        let scale = TimeScale::new(1000, 2000, 100.0);
        
        let x_min = scale.time_to_x(1000);
        assert_eq!(x_min, 0.0);
        
        let x_max = scale.time_to_x(2000);
        assert_eq!(x_max, 100.0);
        
        let x_mid = scale.time_to_x(1500);
        assert_eq!(x_mid, 50.0);
    }
}

