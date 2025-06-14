use libc::{c_char, c_double, c_int, c_void, size_t, ssize_t};

// Opaque pointer type for MPS allocator
pub type MPSAllocator = *mut c_void;

unsafe extern "C" {
    // Get MPS allocator instance
    pub fn mps_get_allocator(shared: c_int) -> MPSAllocator;

    // Memory management functions
    pub fn mps_empty_cache(allocator: MPSAllocator);
    pub fn mps_free_inactive_buffers(allocator: MPSAllocator);

    // Buffer information functions
    pub fn mps_get_unaligned_buffer_size(allocator: MPSAllocator, ptr: *const c_void) -> ssize_t;
    pub fn mps_get_buffer_id(allocator: MPSAllocator, ptr: *const c_void) -> i64;
    pub fn mps_is_shared_buffer(allocator: MPSAllocator, ptr: *const c_void) -> c_int;
    pub fn mps_is_shared_storage_supported(allocator: MPSAllocator) -> c_int;

    // Utility functions
    pub fn mps_format_size(allocator: MPSAllocator, size: size_t) -> *mut c_char;

    // Watermark functions
    pub fn mps_set_low_watermark_ratio(allocator: MPSAllocator, ratio: c_double);
    pub fn mps_set_high_watermark_ratio(allocator: MPSAllocator, ratio: c_double);
    pub fn mps_get_low_watermark_value(allocator: MPSAllocator) -> ssize_t;
    pub fn mps_get_low_watermark_limit(allocator: MPSAllocator) -> size_t;
    pub fn mps_get_high_watermark_limit(allocator: MPSAllocator) -> size_t;

    // Memory statistics functions
    pub fn mps_get_total_allocated_memory(allocator: MPSAllocator) -> size_t;
    pub fn mps_get_current_allocated_memory(allocator: MPSAllocator) -> size_t;
    pub fn mps_get_driver_allocated_memory(allocator: MPSAllocator) -> size_t;
    pub fn mps_get_recommended_max_memory(allocator: MPSAllocator) -> size_t;

    // Cleanup function
    pub fn mps_free_allocator(allocator: MPSAllocator);
}

// Safe wrapper functions for easier usage
impl MPSAllocatorWrapper {
    pub fn new(shared: bool) -> Option<Self> {
        let allocator = unsafe { mps_get_allocator(if shared { 1 } else { 0 }) };
        if allocator.is_null() { None } else { Some(MPSAllocatorWrapper { allocator }) }
    }

    pub fn empty_cache(&self) {
        unsafe { mps_empty_cache(self.allocator) }
    }

    pub fn free_inactive_buffers(&self) {
        unsafe { mps_free_inactive_buffers(self.allocator) }
    }

    pub fn get_unaligned_buffer_size(&self, ptr: *const c_void) -> Option<isize> {
        let size = unsafe { mps_get_unaligned_buffer_size(self.allocator, ptr) };
        if size < 0 { None } else { Some(size) }
    }

    pub fn get_buffer_id(&self, ptr: *const c_void) -> Option<i64> {
        let id = unsafe { mps_get_buffer_id(self.allocator, ptr) };
        if id < 0 { None } else { Some(id) }
    }

    pub fn is_shared_buffer(&self, ptr: *const c_void) -> bool {
        unsafe { mps_is_shared_buffer(self.allocator, ptr) != 0 }
    }

    pub fn is_shared_storage_supported(&self) -> bool {
        unsafe { mps_is_shared_storage_supported(self.allocator) != 0 }
    }

    pub fn format_size(&self, size: usize) -> Option<String> {
        let c_str = unsafe { mps_format_size(self.allocator, size) };
        if c_str.is_null() {
            None
        } else {
            let rust_str =
                unsafe { std::ffi::CStr::from_ptr(c_str).to_string_lossy().into_owned() };
            unsafe { libc::free(c_str as *mut c_void) };
            Some(rust_str)
        }
    }

    pub fn set_low_watermark_ratio(&self, ratio: f64) {
        unsafe { mps_set_low_watermark_ratio(self.allocator, ratio) }
    }

    pub fn set_high_watermark_ratio(&self, ratio: f64) {
        unsafe { mps_set_high_watermark_ratio(self.allocator, ratio) }
    }

    pub fn get_low_watermark_value(&self) -> Option<isize> {
        let value = unsafe { mps_get_low_watermark_value(self.allocator) };
        if value < 0 { None } else { Some(value) }
    }

    pub fn get_low_watermark_limit(&self) -> usize {
        unsafe { mps_get_low_watermark_limit(self.allocator) }
    }

    pub fn get_high_watermark_limit(&self) -> usize {
        unsafe { mps_get_high_watermark_limit(self.allocator) }
    }

    pub fn get_total_allocated_memory(&self) -> usize {
        unsafe { mps_get_total_allocated_memory(self.allocator) }
    }

    pub fn get_current_allocated_memory(&self) -> usize {
        unsafe { mps_get_current_allocated_memory(self.allocator) }
    }

    pub fn get_driver_allocated_memory(&self) -> usize {
        unsafe { mps_get_driver_allocated_memory(self.allocator) }
    }

    pub fn get_recommended_max_memory(&self) -> usize {
        unsafe { mps_get_recommended_max_memory(self.allocator) }
    }
}

pub struct MPSAllocatorWrapper {
    allocator: MPSAllocator,
}

impl Drop for MPSAllocatorWrapper {
    fn drop(&mut self) {
        unsafe { mps_free_allocator(self.allocator) }
    }
}

// Convenience functions for direct usage
pub fn get_mps_allocator(shared: bool) -> Option<MPSAllocatorWrapper> {
    MPSAllocatorWrapper::new(shared)
}

pub fn empty_mps_cache() {
    if let Some(allocator) = get_mps_allocator(false) {
        allocator.empty_cache();
    }
}

pub fn mps_memory_stats() -> Option<MPSMemoryStats> {
    let allocator = get_mps_allocator(false)?;
    Some(MPSMemoryStats {
        total_allocated: allocator.get_total_allocated_memory(),
        current_allocated: allocator.get_current_allocated_memory(),
        driver_allocated: allocator.get_driver_allocated_memory(),
        recommended_max: allocator.get_recommended_max_memory(),
        low_watermark_limit: allocator.get_low_watermark_limit(),
        high_watermark_limit: allocator.get_high_watermark_limit(),
    })
}

#[derive(Debug, Clone)]
pub struct MPSMemoryStats {
    pub total_allocated: usize,
    pub current_allocated: usize,
    pub driver_allocated: usize,
    pub recommended_max: usize,
    pub low_watermark_limit: usize,
    pub high_watermark_limit: usize,
}

impl std::fmt::Display for MPSMemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MPS Memory Stats:\n\
             Total Allocated: {} bytes\n\
             Current Allocated: {} bytes\n\
             Driver Allocated: {} bytes\n\
             Recommended Max: {} bytes\n\
             Low Watermark Limit: {} bytes\n\
             High Watermark Limit: {} bytes",
            self.total_allocated,
            self.current_allocated,
            self.driver_allocated,
            self.recommended_max,
            self.low_watermark_limit,
            self.high_watermark_limit
        )
    }
}

// Platform-specific availability check
#[cfg(target_os = "macos")]
pub fn is_mps_available() -> bool {
    get_mps_allocator(false).is_some()
}

#[cfg(not(target_os = "macos"))]
pub fn is_mps_available() -> bool {
    false
}
