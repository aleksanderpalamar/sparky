use super::gpu::{DrmClient, activity, clients, parse_fdinfo, vram};

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
    let unicos = clients([client(15, 50, 5), client(14, 100, 10), client(14, 100, 10)]);
    assert_eq!(unicos, [client(14, 100, 10), client(15, 50, 5)]);
    assert_eq!(vram(&unicos), 15);
    assert!(clients([]).is_empty());
}

#[test]
fn atividade_e_calculada_por_cliente_antes_de_somar() {
    let client = |id, engine_ns| DrmClient {
        id,
        engine_ns,
        vram: 0,
    };
    let antes = [client(1, 10_000_000_000), client(2, 1_000_000_000)];
    let depois = [client(2, 1_500_000_000)];
    assert_eq!(activity(&antes, &depois), 500_000_000);
    let novo = [client(2, 1_500_000_000), client(3, 200)];
    assert_eq!(activity(&antes, &novo), 500_000_200);
    assert_eq!(activity(&depois, &[client(2, 1_000_000_000)]), 0);
}

#[test]
fn fabricante_pelo_id_pci() {
    assert_eq!(super::gpu::vendor("0x1002\n"), "AMD");
    assert_eq!(super::gpu::vendor("0x8086"), "Intel");
    assert_eq!(super::gpu::vendor("0x1234"), "GPU");
}
