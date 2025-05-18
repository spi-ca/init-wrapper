use static_alloc::Bump;
#[global_allocator]
static GLOBAL_ALLOCATOR: Bump<[u8; 1 << 16]> = Bump::uninit();
