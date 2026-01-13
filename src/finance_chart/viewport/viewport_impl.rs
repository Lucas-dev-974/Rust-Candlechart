use super::super::scale::{PriceScale, TimeScale};
use super::super::core::TimeSeries;

/// Limites de zoom temporel pour éviter les comportements extrêmes
const MIN_TIME_RANGE: i64 = 60;        // Minimum 1 minute visible
const MAX_TIME_RANGE: i64 = 10 * 365 * 24 * 3600; // Maximum 10 ans visible (pour les séries 1M)

/// Limites de zoom de prix pour éviter les comportements extrêmes
const MIN_PRICE_RANGE: f64 = 0.01;     // Minimum 0.01 de plage de prix
const MAX_PRICE_RANGE: f64 = 1_000_000.0; // Maximum 1M de plage de prix

/// Viewport gère la vue visible du graphique
/// 
/// Combine les échelles de prix et de temps pour définir
/// quelle portion des données est visible à l'écran.
#[derive(Debug, Clone)]
pub struct Viewport {
    price_scale: PriceScale,
    time_scale: TimeScale,
    /// Largeur totale du viewport en pixels
    width: f32,
    /// Hauteur totale du viewport en pixels
    height: f32,
}

impl Viewport {
    /// Crée un nouveau viewport
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            price_scale: PriceScale::new(0.0, 100.0, height),
            time_scale: TimeScale::new(0, 1000, width),
            width,
            height,
        }
    }

    /// Initialise le viewport sur les dernières bougies (focus sur les données récentes)
    /// 
    /// # Arguments
    /// * `data` - La série temporelle
    /// * `visible_candles` - Nombre de bougies à afficher initialement
    pub fn focus_on_recent(&mut self, data: &TimeSeries, visible_candles: usize) {
        let max_time = match data.max_timestamp() {
            Some(t) => t,
            None => return,
        };

        let min_time = match data.min_timestamp() {
            Some(t) => t,
            None => return,
        };

        // Calculer l'intervalle entre bougies (supposé constant)
        let total_candles = data.len();
        if total_candles == 0 {
            return;
        }
        
        if total_candles == 1 {
            // Cas spécial : une seule bougie, afficher avec une petite marge temporelle
            if let Some((min_price, max_price)) = data.price_range() {
                // Ajouter une marge de 10% pour la visibilité
                let price_margin = (max_price - min_price) * 0.1;
                self.price_scale.set_price_range(
                    min_price - price_margin,
                    max_price + price_margin
                );
            }
            // Pour le temps, créer une petite plage autour de la bougie (1 jour de chaque côté)
            let one_day = 86400;
            self.time_scale.set_time_range(min_time - one_day, max_time + one_day);
            return;
        }

        // Limiter le nombre de bougies visibles au nombre réel de bougies disponibles
        let actual_visible_candles = visible_candles.min(total_candles);
        
        let total_time_range = max_time - min_time;
        
        // Si on veut afficher toutes les bougies ou plus, afficher toutes les données
        // Pour les séries avec peu de bougies (comme 1M), toujours afficher toutes les données
        if actual_visible_candles >= total_candles || total_candles <= 50 {
            // Pour les séries avec peu de bougies, ajouter un petit padding temporel
            // pour s'assurer que toutes les bougies sont visibles
            let time_padding = if total_candles <= 50 && total_time_range > 0 {
                total_time_range / 20 // 5% de padding de chaque côté
            } else {
                0
            };
            
            self.time_scale.set_time_range(
                min_time.saturating_sub(time_padding),
                max_time + time_padding
            );
            
            if let Some((min_price, max_price)) = data.price_range() {
                // Ajouter une petite marge pour la visibilité
                let price_margin = (max_price - min_price) * 0.05;
                self.price_scale.set_price_range(
                    min_price - price_margin,
                    max_price + price_margin
                );
            }
            return;
        }
        
        // Calculer l'intervalle moyen entre bougies
        let candle_interval = if total_candles > 1 {
            total_time_range / (total_candles as i64 - 1)
        } else {
            total_time_range
        };

        // Calculer la plage de temps pour les N dernières bougies
        let visible_time_range = candle_interval * actual_visible_candles as i64;
        let start_time = (max_time - visible_time_range).max(min_time); // S'assurer qu'on ne dépasse pas min_time

        self.time_scale.set_time_range(start_time, max_time);

        // Calculer la plage de prix uniquement pour les bougies visibles
        // Utiliser la méthode avec cache si disponible
        let price_range = if let Some(range) = data.price_range_for_time_range(start_time..max_time) {
            range
        } else {
            // Fallback: calculer manuellement
            let visible_data = data.visible_candles(start_time..max_time);
            Self::price_range_for_candles(visible_data).unwrap_or_else(|| {
                // Dernier fallback: utiliser la plage globale
                data.price_range().unwrap_or((0.0, 100.0))
            })
        };
        
        // Ajouter une petite marge pour la visibilité
        let price_margin = (price_range.1 - price_range.0) * 0.05;
        self.price_scale.set_price_range(
            price_range.0 - price_margin,
            price_range.1 + price_margin
        );
    }

    /// Calcule la plage de prix pour un slice de bougies
    fn price_range_for_candles(candles: &[super::super::core::Candle]) -> Option<(f64, f64)> {
        candles.iter().fold(None, |acc, candle| {
            Some(match acc {
                None => (candle.low, candle.high),
                Some((min, max)) => (min.min(candle.low), max.max(candle.high)),
            })
        })
    }

    /// Met à jour la taille du viewport
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.price_scale.set_height(height);
        self.time_scale.set_width(width);
    }

    /// Retourne une référence à l'échelle de prix
    pub fn price_scale(&self) -> &PriceScale {
        &self.price_scale
    }

    /// Retourne une référence à l'échelle temporelle
    pub fn time_scale(&self) -> &TimeScale {
        &self.time_scale
    }

    /// Retourne la largeur du viewport
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Retourne la hauteur du viewport
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Zoom progressif centré au milieu du graphique (axe X uniquement)
    /// 
    /// `factor` > 1.0 = zoom out (plage plus grande), < 1.0 = zoom in (plage plus petite)
    pub fn zoom(&mut self, factor: f64) {
        self.zoom_horizontal(factor);
    }

    /// Zoom horizontal (axe X / temps)
    fn zoom_horizontal(&mut self, factor: f64) {
        let (min_time, max_time) = self.time_scale.time_range();
        let time_range = max_time - min_time;
        let center_time = min_time + time_range / 2;
        
        let new_time_range = (time_range as f64 * factor) as i64;
        let new_time_range = new_time_range.clamp(MIN_TIME_RANGE, MAX_TIME_RANGE);
        
        let new_min = center_time - new_time_range / 2;
        let new_max = center_time + new_time_range / 2;
        self.time_scale.set_time_range(new_min, new_max);
    }

    /// Zoom vertical (axe Y / prix) - ALT + molette
    /// 
    /// `factor` > 1.0 = zoom out, < 1.0 = zoom in
    pub fn zoom_vertical(&mut self, factor: f64) {
        let (min_price, max_price) = self.price_scale.price_range();
        let price_range = max_price - min_price;
        
        // Éviter les divisions par zéro et les plages invalides
        if price_range <= 0.0 || !price_range.is_finite() || !min_price.is_finite() || !max_price.is_finite() {
            return;
        }
        
        // Calculer le centre de la plage actuelle
        let center_price = min_price + price_range / 2.0;
        
        // Calculer la nouvelle plage en multipliant par le facteur
        let new_price_range = price_range * factor;
        
        // Vérifier que la nouvelle plage est valide (positive et finie)
        if new_price_range <= 0.0 || !new_price_range.is_finite() {
            return;
        }
        
        // Utiliser des limites relatives plutôt qu'absolues pour éviter les problèmes
        // Limiter à 0.1% de la plage actuelle minimum (pour zoom in)
        // et à 1000x la plage actuelle maximum (pour zoom out)
        let min_allowed_range = price_range * 0.001;  // 0.1% de la plage actuelle
        let max_allowed_range = price_range * 1000.0; // 1000x la plage actuelle
        
        // Appliquer les limites relatives
        let clamped_range = new_price_range.clamp(min_allowed_range, max_allowed_range);
        
        // Si le clamp a modifié la valeur, cela signifie qu'on a atteint une limite
        // Dans ce cas, ne pas appliquer le zoom pour éviter les sauts
        if (clamped_range - new_price_range).abs() > 0.0001 {
            return;
        }
        
        // Calculer les nouvelles limites centrées sur le centre actuel
        let new_min = center_price - clamped_range / 2.0;
        let new_max = center_price + clamped_range / 2.0;
        
        // Vérifications finales
        if !new_min.is_finite() || !new_max.is_finite() || new_min >= new_max {
            return;
        }
        
        self.price_scale.set_price_range(new_min, new_max);
    }

    /// Zoom sur les deux axes (X et Y) - CTRL + molette
    /// 
    /// `factor` > 1.0 = zoom out, < 1.0 = zoom in
    pub fn zoom_both(&mut self, factor: f64) {
        self.zoom_horizontal(factor);
        self.zoom_vertical(factor);
    }

    /// Pan horizontal basé sur un delta en pixels
    pub fn pan_horizontal(&mut self, delta_x: f32) {
        let (min_time, max_time) = self.time_scale.time_range();
        let time_range = max_time - min_time;
        let seconds_per_pixel = time_range as f64 / self.width as f64;
        let delta_seconds = (delta_x as f64 * seconds_per_pixel) as i64;
        self.time_scale.set_time_range(min_time + delta_seconds, max_time + delta_seconds);
    }

    /// Pan vertical basé sur un delta en pixels
    pub fn pan_vertical(&mut self, delta_y: f32) {
        let (min_price, max_price) = self.price_scale.price_range();
        let price_range = max_price - min_price;
        let price_per_pixel = price_range / self.height as f64;
        let delta_price = delta_y as f64 * price_per_pixel;
        self.price_scale.set_price_range(min_price + delta_price, max_price + delta_price);
    }
}

