use eframe::egui::{self, ViewportCommand};

#[derive(Clone, Copy, Debug)]
pub struct MonitorGeometry {
    pub position: egui::Pos2,
    pub size: egui::Vec2,
}

pub fn position_window(
    ctx: &egui::Context,
    width: f32,
    height: f32,
    bottom_align: bool,
    top_align: bool,
    monitor: Option<&MonitorGeometry>,
) {
    let monitor_geometry = monitor.copied().or_else(|| {
        ctx.input(|i| {
            i.viewport().monitor_size.map(|size| MonitorGeometry {
                position: egui::Pos2::ZERO,
                size,
            })
        })
    });
    let Some(monitor_geometry) = monitor_geometry else {
        return;
    };

    let position = window_position_for_monitor(
        Some(&monitor_geometry),
        width,
        height,
        bottom_align,
        top_align,
    );
    if let Some(position) = position {
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(position));
    }
}

pub fn window_position_for_monitor(
    monitor: Option<&MonitorGeometry>,
    width: f32,
    height: f32,
    bottom_align: bool,
    top_align: bool,
) -> Option<egui::Pos2> {
    let monitor = monitor?;
    let centered_x = monitor.position.x + ((monitor.size.x - width) * 0.5).max(0.0);
    let relative_y = if top_align {
        0.0
    } else if bottom_align {
        (monitor.size.y - height).max(0.0)
    } else {
        ((monitor.size.y - height) * 0.5).max(0.0)
    };
    Some(egui::pos2(centered_x, monitor.position.y + relative_y))
}

#[cfg(test)]
mod tests {
    use super::{window_position_for_monitor, MonitorGeometry};
    use eframe::egui::{pos2, vec2};

    fn monitor() -> MonitorGeometry {
        MonitorGeometry {
            position: pos2(100.0, 50.0),
            size: vec2(1000.0, 600.0),
        }
    }

    #[test]
    fn window_position_for_monitor_aligns_top() {
        let pos = window_position_for_monitor(Some(&monitor()), 400.0, 100.0, false, true)
            .expect("position expected");
        assert_eq!(pos, pos2(400.0, 50.0));
    }

    #[test]
    fn window_position_for_monitor_aligns_center_by_default() {
        let pos = window_position_for_monitor(Some(&monitor()), 400.0, 100.0, false, false)
            .expect("position expected");
        assert_eq!(pos, pos2(400.0, 300.0));
    }

    #[test]
    fn window_position_for_monitor_aligns_bottom() {
        let pos = window_position_for_monitor(Some(&monitor()), 400.0, 100.0, true, false)
            .expect("position expected");
        assert_eq!(pos, pos2(400.0, 550.0));
    }
}

pub fn detect_monitor(monitor_index: usize) -> Option<MonitorGeometry> {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
        use windows::Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
        };

        extern "system" fn enum_proc(
            hmonitor: HMONITOR,
            _hdc: HDC,
            _rect: *mut RECT,
            lparam: LPARAM,
        ) -> BOOL {
            let monitors = unsafe { &mut *(lparam.0 as *mut Vec<HMONITOR>) };
            monitors.push(hmonitor);
            BOOL(1)
        }

        let mut monitors: Vec<HMONITOR> = Vec::new();
        unsafe {
            let _ = EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(enum_proc),
                LPARAM(&mut monitors as *mut Vec<HMONITOR> as isize),
            );
        }

        let hmonitor = monitors.get(monitor_index)?;

        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let ok = unsafe { GetMonitorInfoW(*hmonitor, &mut info) };
        if !ok.as_bool() {
            return None;
        }

        let rect = info.rcMonitor;
        let width = (rect.right - rect.left) as f32;
        let height = (rect.bottom - rect.top) as f32;
        let scale = monitor_scale_factor(*hmonitor);

        Some(MonitorGeometry {
            position: egui::pos2(rect.left as f32 / scale, rect.top as f32 / scale),
            size: egui::vec2(width / scale, height / scale),
        })
    }

    #[cfg(not(windows))]
    {
        let _ = monitor_index;
        None
    }
}

#[cfg(windows)]
fn monitor_scale_factor(hmonitor: windows::Win32::Graphics::Gdi::HMONITOR) -> f32 {
    use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

    let mut dpi_x: u32 = 96;
    let mut dpi_y: u32 = 96;
    unsafe {
        let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
    }
    dpi_x as f32 / 96.0
}
