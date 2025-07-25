use std::error::Error;
use wasapi::{DeviceCollection, Direction, Device, ShareMode, get_default_device};

#[derive(Debug, Clone)]
pub struct LoopbackDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: String,
    pub device_type: DeviceType,
    pub loopback_method: LoopbackMethod,
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    Render,
    Capture,
}

#[derive(Debug, Clone)]
pub enum LoopbackMethod {
    RenderLoopback,
    CaptureDevice,
    StereoMix,
}

pub struct WASAPILoopbackEnumerator {
    render_collection: DeviceCollection,
    capture_collection: DeviceCollection,
}

impl WASAPILoopbackEnumerator {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let render_collection = DeviceCollection::new(&Direction::Render)
            .map_err(|e| format!("Failed to create render device collection: {}", e))?;
        let capture_collection = DeviceCollection::new(&Direction::Capture)
            .map_err(|e| format!("Failed to create capture device collection: {}", e))?;
        
        Ok(Self { 
            render_collection,
            capture_collection,
        })
    }
    
    pub fn enumerate_loopback_devices(&self) -> Result<Vec<LoopbackDevice>, Box<dyn Error>> {
        println!("\n=== COMPREHENSIVE WASAPI LOOPBACK DEVICE SCAN ===");
        
        let mut loopback_devices = Vec::new();
        
        // Get default devices for comparison
        let default_render = get_default_device(&Direction::Render).ok();
        let default_capture = get_default_device(&Direction::Capture).ok();
        
        let default_render_id = default_render.as_ref().and_then(|d| d.get_id().ok()).unwrap_or_default();
        let default_capture_id = default_capture.as_ref().and_then(|d| d.get_id().ok()).unwrap_or_default();
        
        // Strategy 1: Try render devices with loopback
        println!("🔍 Strategy 1: Render devices with loopback capability");
        if let Ok(render_devices) = self.scan_render_devices(&default_render_id) {
            loopback_devices.extend(render_devices);
        }
        
        // Strategy 2: Try capture devices that might be loopback
        println!("🔍 Strategy 2: Capture devices with loopback names");
        if let Ok(capture_devices) = self.scan_capture_devices(&default_capture_id) {
            loopback_devices.extend(capture_devices);
        }
        
        // Strategy 3: Look for stereo mix and similar devices
        println!("🔍 Strategy 3: Stereo Mix and system audio devices");
        if let Ok(stereo_devices) = self.scan_stereo_mix_devices(&default_capture_id) {
            loopback_devices.extend(stereo_devices);
        }
        
        // Remove duplicates by ID
        loopback_devices.sort_by(|a, b| a.id.cmp(&b.id));
        loopback_devices.dedup_by(|a, b| a.id == b.id);
        
        if loopback_devices.is_empty() {
            println!("❌ No suitable loopback devices found across all strategies!");
        } else {
            println!("\n✅ Found {} potential loopback device(s):", loopback_devices.len());
            for (i, device) in loopback_devices.iter().enumerate() {
                let method_str = match device.loopback_method {
                    LoopbackMethod::RenderLoopback => "Render Loopback",
                    LoopbackMethod::CaptureDevice => "Capture Device", 
                    LoopbackMethod::StereoMix => "Stereo Mix",
                };
                let default_str = if device.is_default { " (Default)" } else { "" };
                println!("  {}. {} [{}]{}", i + 1, device.name, method_str, default_str);
            }
        }
        
        Ok(loopback_devices)
    }
    
    fn scan_render_devices(&self, default_id: &str) -> Result<Vec<LoopbackDevice>, Box<dyn Error>> {
        let device_count = self.render_collection.get_nbr_devices()?;
        let mut devices = Vec::new();
        
        for i in 0..device_count {
            if let Ok(device) = self.render_collection.get_device_at_index(i) {
                if let Ok(device_info) = self.create_render_device_info(&device, default_id) {
                    if self.test_render_loopback_capability(&device) {
                        println!("  ✓ Render loopback: {}", device_info.name);
                        devices.push(device_info);
                    }
                }
            }
        }
        
        Ok(devices)
    }
    
    fn scan_capture_devices(&self, default_id: &str) -> Result<Vec<LoopbackDevice>, Box<dyn Error>> {
        let device_count = self.capture_collection.get_nbr_devices()?;
        let mut devices = Vec::new();
        
        for i in 0..device_count {
            if let Ok(device) = self.capture_collection.get_device_at_index(i) {
                if let Ok(name) = device.get_friendlyname() {
                    // Look for devices that might be loopback-capable
                    if self.is_potential_loopback_capture_device(&name) {
                        if let Ok(device_info) = self.create_capture_device_info(&device, default_id) {
                            if self.test_capture_device_capability(&device) {
                                println!("  ✓ Capture loopback: {}", device_info.name);
                                devices.push(device_info);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(devices)
    }
    
    fn scan_stereo_mix_devices(&self, default_id: &str) -> Result<Vec<LoopbackDevice>, Box<dyn Error>> {
        let device_count = self.capture_collection.get_nbr_devices()?;
        let mut devices = Vec::new();
        
        for i in 0..device_count {
            if let Ok(device) = self.capture_collection.get_device_at_index(i) {
                if let Ok(name) = device.get_friendlyname() {
                    if self.is_stereo_mix_device(&name) {
                        if let Ok(mut device_info) = self.create_capture_device_info(&device, default_id) {
                            device_info.loopback_method = LoopbackMethod::StereoMix;
                            if self.test_capture_device_capability(&device) {
                                println!("  ✓ Stereo Mix: {}", device_info.name);
                                devices.push(device_info);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(devices)
    }
    
    fn is_potential_loopback_capture_device(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("loopback") || 
        name_lower.contains("what u hear") ||
        name_lower.contains("what you hear") ||
        name_lower.contains("wave") ||
        name_lower.contains("mix") ||
        (name_lower.contains("speakers") && name_lower.contains("capture"))
    }
    
    fn is_stereo_mix_device(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("stereo mix") ||
        name_lower.contains("stereomix") ||
        name_lower.contains("wave") ||
        name_lower.contains("what u hear") ||
        name_lower.contains("what you hear")
    }
    
    fn create_render_device_info(&self, device: &Device, default_id: &str) -> Result<LoopbackDevice, Box<dyn Error>> {
        let id = device.get_id()?;
        let name = device.get_friendlyname().unwrap_or_else(|_| "Unknown Render Device".to_string());
        let is_default = id == default_id;
        
        let (sample_rate, channels, format) = self.get_device_format(device)?;
        
        Ok(LoopbackDevice {
            id,
            name,
            is_default,
            sample_rate,
            channels,
            format,
            device_type: DeviceType::Render,
            loopback_method: LoopbackMethod::RenderLoopback,
        })
    }
    
    fn create_capture_device_info(&self, device: &Device, default_id: &str) -> Result<LoopbackDevice, Box<dyn Error>> {
        let id = device.get_id()?;
        let name = device.get_friendlyname().unwrap_or_else(|_| "Unknown Capture Device".to_string());
        let is_default = id == default_id;
        
        let (sample_rate, channels, format) = self.get_device_format(device)?;
        
        Ok(LoopbackDevice {
            id,
            name,
            is_default,
            sample_rate,
            channels,
            format,
            device_type: DeviceType::Capture,
            loopback_method: LoopbackMethod::CaptureDevice,
        })
    }
    
    fn get_device_format(&self, device: &Device) -> Result<(u32, u16, String), Box<dyn Error>> {
        let audio_client = device.get_iaudioclient()
            .map_err(|e| format!("Failed to get audio client: {}", e))?;
        let format = audio_client.get_mixformat()
            .map_err(|e| format!("Failed to get mix format: {}", e))?;
        
        let sample_rate = format.get_samplespersec();
        let channels = format.get_nchannels();
        let bits_per_sample = format.get_bitspersample();
        
        let format_str = if bits_per_sample == 32 {
            "IEEE Float 32bit".to_string()
        } else if bits_per_sample == 24 {
            "PCM 24bit".to_string()
        } else if bits_per_sample == 16 {
            "PCM 16bit".to_string()
        } else {
            format!("PCM {}bit", bits_per_sample)
        };
        
        Ok((sample_rate, channels, format_str))
    }
    
    fn test_render_loopback_capability(&self, device: &Device) -> bool {
        match self.try_initialize_render_loopback(device) {
            Ok(_) => {
                println!("    → Render loopback test: ✅ SUCCESS");
                true
            },
            Err(e) => {
                println!("    → Render loopback test: ❌ FAILED ({})", e);
                false
            }
        }
    }
    
    fn test_capture_device_capability(&self, device: &Device) -> bool {
        match self.try_initialize_capture_device(device) {
            Ok(_) => {
                println!("    → Capture device test: ✅ SUCCESS");
                true
            },
            Err(e) => {
                println!("    → Capture device test: ❌ FAILED ({})", e);
                false
            }
        }
    }
    
    fn try_initialize_render_loopback(&self, device: &Device) -> Result<(), Box<dyn Error>> {
        let mut audio_client = device.get_iaudioclient()
            .map_err(|e| format!("Failed to get audio client: {}", e))?;
        let mix_format = audio_client.get_mixformat()
            .map_err(|e| format!("Failed to get mix format: {}", e))?;
        
        // Try to initialize the audio client in loopback mode
        let result = audio_client.initialize_client(
            &mix_format,
            10_000_000, // 1 second buffer in 100ns units
            &Direction::Render,
            &ShareMode::Shared,
            true, // use_loopback
        );
        
        result.map_err(|e| {
            let error_msg = match format!("{:?}", e).as_str() {
                s if s.contains("0x88890003") => "AUDCLNT_E_WRONG_ENDPOINT_TYPE - Device doesn't support render loopback".to_string(),
                s if s.contains("0x80070057") => "E_INVALIDARG - Invalid parameters".to_string(),
                s if s.contains("0x88890001") => "AUDCLNT_E_NOT_INITIALIZED - Client not initialized".to_string(),
                _ => format!("WASAPI Error: {:?}", e)
            };
            error_msg.into()
        })
    }
    
    fn try_initialize_capture_device(&self, device: &Device) -> Result<(), Box<dyn Error>> {
        let mut audio_client = device.get_iaudioclient()
            .map_err(|e| format!("Failed to get audio client: {}", e))?;
        let mix_format = audio_client.get_mixformat()
            .map_err(|e| format!("Failed to get mix format: {}", e))?;
        
        // Try to initialize as regular capture device (no loopback flag)
        let result = audio_client.initialize_client(
            &mix_format,
            10_000_000, // 1 second buffer in 100ns units
            &Direction::Capture,
            &ShareMode::Shared,
            false, // no loopback for capture devices
        );
        
        result.map_err(|e| format!("Failed to initialize capture device: {:?}", e).into())
    }
    
    pub fn list_devices_detailed(&self) -> Result<(), Box<dyn Error>> {
        let devices = self.enumerate_loopback_devices()?;
        
        if devices.is_empty() {
            return Ok(());
        }
        
        println!("\n=== DETAILED DEVICE INFORMATION ===");
        
        for (index, device) in devices.iter().enumerate() {
            println!("\n{}. {}", index + 1, device.name);
            println!("   ID: {}", device.id);
            println!("   Type: {:?}", device.device_type);
            println!("   Method: {:?}", device.loopback_method);
            println!("   Sample Rate: {} Hz", device.sample_rate);
            println!("   Channels: {}", device.channels);
            println!("   Format: {}", device.format);
            println!("   Default: {}", if device.is_default { "Yes" } else { "No" });
            
            if self.verify_device_capability(device) {
                println!("   Status: ✓ Ready for loopback");
            } else {
                println!("   Status: ⚠ May have issues");
            }
        }
        
        Ok(())
    }
    
    fn verify_device_capability(&self, device_info: &LoopbackDevice) -> bool {
        // Re-verify the device capability based on its type and method
        match device_info.device_type {
            DeviceType::Render => {
                if let Ok(device_collection) = DeviceCollection::new(&Direction::Render) {
                    let device_count = device_collection.get_nbr_devices().unwrap_or(0);
                    for i in 0..device_count {
                        if let Ok(device) = device_collection.get_device_at_index(i) {
                            if let Ok(id) = device.get_id() {
                                if id == device_info.id {
                                    return self.test_render_loopback_capability(&device);
                                }
                            }
                        }
                    }
                }
            },
            DeviceType::Capture => {
                if let Ok(device_collection) = DeviceCollection::new(&Direction::Capture) {
                    let device_count = device_collection.get_nbr_devices().unwrap_or(0);
                    for i in 0..device_count {
                        if let Ok(device) = device_collection.get_device_at_index(i) {
                            if let Ok(id) = device.get_id() {
                                if id == device_info.id {
                                    return self.test_capture_device_capability(&device);
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
    
    pub fn auto_select_best_device(&self) -> Result<Option<LoopbackDevice>, Box<dyn Error>> {
        let devices = self.enumerate_loopback_devices()?;
        
        if devices.is_empty() {
            return Ok(None);
        }
        
        // Priority 1: Default render device with render loopback
        if let Some(default_render) = devices.iter().find(|d| 
            d.is_default && 
            matches!(d.device_type, DeviceType::Render) && 
            matches!(d.loopback_method, LoopbackMethod::RenderLoopback)
        ) {
            println!("\n✓ Auto-selected: Default render device with loopback: {}", default_render.name);
            return Ok(Some(default_render.clone()));
        }
        
        // Priority 2: Any render device with loopback
        if let Some(render_device) = devices.iter().find(|d| 
            matches!(d.device_type, DeviceType::Render) && 
            matches!(d.loopback_method, LoopbackMethod::RenderLoopback)
        ) {
            println!("\n✓ Auto-selected: Render device with loopback: {}", render_device.name);
            return Ok(Some(render_device.clone()));
        }
        
        // Priority 3: Stereo Mix device
        if let Some(stereo_mix) = devices.iter().find(|d| 
            matches!(d.loopback_method, LoopbackMethod::StereoMix)
        ) {
            println!("\n✓ Auto-selected: Stereo Mix device: {}", stereo_mix.name);
            return Ok(Some(stereo_mix.clone()));
        }
        
        // Priority 4: Any capture device that might work
        if let Some(capture_device) = devices.iter().find(|d| 
            matches!(d.device_type, DeviceType::Capture)
        ) {
            println!("\n✓ Auto-selected: Capture device: {}", capture_device.name);
            return Ok(Some(capture_device.clone()));
        }
        
        // Fallback: First available device
        println!("\n⚠ No optimal device found, using first available: {}", devices[0].name);
        Ok(Some(devices[0].clone()))
    }
}