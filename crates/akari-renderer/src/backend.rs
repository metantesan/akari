/// Uses a native backend set per target. `WGPU_BACKEND` remains an explicit
/// override for diagnosing driver problems.
pub fn instance_descriptor() -> wgpu::InstanceDescriptor {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle_from_env();
    if std::env::var_os("WGPU_BACKEND").is_none() {
        descriptor.backends = platform_backends();
    }
    descriptor
}

pub(super) fn platform_backends() -> wgpu::Backends {
    #[cfg(target_os = "linux")]
    {
        return wgpu::Backends::VULKAN | wgpu::Backends::GL;
    }
    #[cfg(target_os = "windows")]
    {
        return wgpu::Backends::DX12 | wgpu::Backends::VULKAN;
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        return wgpu::Backends::METAL;
    }
    #[cfg(target_os = "android")]
    {
        return wgpu::Backends::VULKAN | wgpu::Backends::GL;
    }
    #[allow(unreachable_code)]
    wgpu::Backends::all()
}

pub(super) fn backend_priority(backend: wgpu::Backend) -> u8 {
    #[cfg(target_os = "linux")]
    {
        return match backend {
            wgpu::Backend::Vulkan => 0,
            wgpu::Backend::Gl => 1,
            _ => 2,
        };
    }
    #[cfg(target_os = "windows")]
    {
        return match backend {
            wgpu::Backend::Dx12 => 0,
            wgpu::Backend::Vulkan => 1,
            _ => 2,
        };
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        return match backend {
            wgpu::Backend::Metal => 0,
            _ => 1,
        };
    }
    #[cfg(target_os = "android")]
    {
        return match backend {
            wgpu::Backend::Vulkan => 0,
            wgpu::Backend::Gl => 1,
            _ => 2,
        };
    }
    #[allow(unreachable_code)]
    0
}

pub(super) fn device_priority(device_type: wgpu::DeviceType) -> u8 {
    match device_type {
        wgpu::DeviceType::DiscreteGpu => 0,
        wgpu::DeviceType::IntegratedGpu => 1,
        wgpu::DeviceType::VirtualGpu => 2,
        wgpu::DeviceType::Cpu => 3,
        wgpu::DeviceType::Other => 4,
    }
}
