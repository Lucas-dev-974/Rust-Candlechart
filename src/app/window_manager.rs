//! Gestionnaire de fenêtres pour l'application

use iced::window;

/// Type de fenêtre
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Main,
    Settings,
    ProviderConfig,
    Downloads,
}

/// Gestionnaire de fenêtres simplifié
#[derive(Debug, Clone)]
pub struct WindowManager {
    main_window_id: Option<window::Id>,
    settings_window_id: Option<window::Id>,
    provider_config_window_id: Option<window::Id>,  
    downloads_window_id: Option<window::Id>,
}

impl WindowManager {
    pub fn new(main_id: window::Id) -> Self {
        Self {
            main_window_id: Some(main_id),
            settings_window_id: None,
            provider_config_window_id: None,
            downloads_window_id: None,
        }
    }
    
    pub fn get_id(&self, window_type: WindowType) -> Option<window::Id> {
        match window_type {
            WindowType::Main => self.main_window_id,
            WindowType::Settings => self.settings_window_id,
            WindowType::ProviderConfig => self.provider_config_window_id,
            WindowType::Downloads => self.downloads_window_id,
        }
    }
    
    pub fn set_id(&mut self, window_type: WindowType, id: window::Id) {
        match window_type {
            WindowType::Main => self.main_window_id = Some(id),
            WindowType::Settings => self.settings_window_id = Some(id),
            WindowType::ProviderConfig => self.provider_config_window_id = Some(id),
            WindowType::Downloads => self.downloads_window_id = Some(id),
        }
    }
    
    pub fn remove_id(&mut self, window_type: WindowType) {
        match window_type {
            WindowType::Main => self.main_window_id = None,
            WindowType::Settings => self.settings_window_id = None,
            WindowType::ProviderConfig => self.provider_config_window_id = None,
            WindowType::Downloads => self.downloads_window_id = None,
        }
    }
    
    pub fn is_open(&self, window_type: WindowType) -> bool {
        self.get_id(window_type).is_some()
    }
    
    pub fn get_window_type(&self, id: window::Id) -> Option<WindowType> {
        if self.main_window_id == Some(id) {
            Some(WindowType::Main)
        } else if self.settings_window_id == Some(id) {
            Some(WindowType::Settings)
        } else if self.provider_config_window_id == Some(id) {
            Some(WindowType::ProviderConfig)
        } else if self.downloads_window_id == Some(id) {
            Some(WindowType::Downloads)
        } else {
            None
        }
    }
}

