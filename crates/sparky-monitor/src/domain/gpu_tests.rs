use super::gpu::{DrmClient, GpuUse, combine, parse_fdinfo};

const AMDGPU: &str = "pos:\t0\nflags:\t02104002\ndrm-driver:\tamdgpu\ndrm-client-id:\t14\ndrm-total-vram:\t114504 KiB\ndrm-resident-vram:\t104772 KiB\ndrm-memory-vram:\t104772 KiB\ndrm-memory-gtt: \t23616 KiB\ndrm-engine-gfx:\t10110769228 ns\ndrm-engine-compute:\t4035788 ns\n";

#[test]
fn le_tempo_de_gpu_e_vram_do_cliente_drm() {
    assert_eq!(
        parse_fdinfo(AMDGPU),
        Some(DrmClient {
            id: 14,
            engine_ns: 10_110_769_228 + 4_035_788,
            vram: 104_772 * 1024
        })
    );
    assert_eq!(parse_fdinfo("pos:\t0\nflags:\t02\n"), None);
}

#[test]
fn intel_ignora_capacidade_dos_motores() {
    let i915 = "drm-client-id:\t7\ndrm-engine-render:\t500 ns\ndrm-engine-capacity-video:\t2\ndrm-resident-local0:\t2 MiB\n";
    assert_eq!(
        parse_fdinfo(i915),
        Some(DrmClient {
            id: 7,
            engine_ns: 500,
            vram: 2 * 1024 * 1024
        })
    );
}

#[test]
fn descritores_do_mesmo_cliente_contam_uma_vez() {
    let client = |id, engine_ns, vram| DrmClient {
        id,
        engine_ns,
        vram,
    };
    let total = combine([client(14, 100, 10), client(14, 100, 10), client(15, 50, 5)]);
    assert_eq!(
        total,
        Some(GpuUse {
            engine_ns: 150,
            vram: 15
        })
    );
    assert_eq!(combine([]), None);
}

#[test]
fn fabricante_pelo_id_pci() {
    assert_eq!(super::gpu::vendor("0x1002\n"), "AMD");
    assert_eq!(super::gpu::vendor("0x8086"), "Intel");
    assert_eq!(super::gpu::vendor("0x1234"), "GPU");
}
